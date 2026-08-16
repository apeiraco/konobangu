import { randomBytes } from "node:crypto";
import { createServer } from "node:http";
import type { AddressInfo } from "node:net";
import { exportJWK, generateKeyPair } from "jose";
import Provider from "oidc-provider";

/** Test accounts and consent only; OIDC protocol handling remains in the provider. */
export async function startIdp({
  redirectUris,
  clientSecret,
}: {
  redirectUris: string[];
  clientSecret: string;
}) {
  const { privateKey } = await generateKeyPair("RS256", { extractable: true });
  const key = await exportJWK(privateKey);
  Object.assign(key, { kid: "test-rsa", alg: "RS256", use: "sig" });
  let provider: Provider;
  await using ownership = new AsyncDisposableStack();
  const server = ownership.use(
    createServer(async (request, response) => {
      try {
        const uri = new URL(request.url ?? "/", "http://127.0.0.1");
        if (uri.pathname === "/health") {
          response.end("ready");
          return;
        }
        if (uri.pathname.startsWith("/interaction/")) {
          const interaction = await provider.interactionDetails(
            request,
            response,
          );
          if (request.method === "GET") {
            response.setHeader("content-type", "text/html; charset=utf-8");
            response.end(
              `<form method="post"><button name="account" value="A">Sign in as A</button><button name="account" value="B">Sign in as B</button></form>`,
            );
            return;
          }
          if (request.method !== "POST") {
            response.writeHead(405).end();
            return;
          }
          const parts: Buffer[] = [];
          for await (const part of request) parts.push(part);
          const account = new URLSearchParams(
            Buffer.concat(parts).toString(),
          ).get("account");
          if (account !== "A" && account !== "B") {
            response.writeHead(400).end();
            return;
          }
          if (interaction.prompt.name === "login") {
            await provider.interactionFinished(
              request,
              response,
              { login: { accountId: account } },
              { mergeWithLastSubmission: false },
            );
            return;
          }
          await consent(provider, request, response, interaction);
          return;
        }
        provider.callback()(request, response);
      } catch {
        response
          .writeHead(500)
          .end("Test identity provider interaction failed");
      }
    }),
  );
  await new Promise<void>((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", resolve);
  });
  const issuer = `http://127.0.0.1:${(server.address() as AddressInfo).port}`;
  provider = new Provider(issuer, {
    clients: [
      {
        client_id: "konobangu-test",
        client_secret: clientSecret,
        redirect_uris: redirectUris,
        response_types: ["code"],
        grant_types: ["authorization_code"],
        token_endpoint_auth_method: "client_secret_basic",
        id_token_signed_response_alg: "RS256",
      },
    ],
    jwks: { keys: [key] },
    cookies: { keys: [randomBytes(32).toString("hex")] },
    features: { devInteractions: { enabled: false } },
    interactions: {
      url: (_context, interaction) => `/interaction/${interaction.uid}`,
    },
    pkce: { required: () => true },
    claims: { openid: ["sub"], profile: ["name"], email: ["email"] },
    findAccount: async (_context, accountId) => ({
      accountId,
      claims: async () => ({
        sub: accountId,
        name: `User ${accountId}`,
        email: `${accountId.toLowerCase()}@example.test`,
      }),
    }),
  });
  const lifetime = ownership.move();
  const close = () => lifetime.disposeAsync();
  return {
    [Symbol.asyncDispose]: close,
    issuer,
    clientSecret,
    close,
  };
}

async function consent(
  provider: Provider,
  request: import("node:http").IncomingMessage,
  response: import("node:http").ServerResponse,
  interaction: Awaited<ReturnType<Provider["interactionDetails"]>>,
) {
  const clientId = interaction.params.client_id;
  if (!interaction.session || typeof clientId !== "string")
    throw new Error("Consent requires an authenticated client");
  const grant = interaction.grantId
    ? await provider.Grant.find(interaction.grantId)
    : new provider.Grant({
        accountId: interaction.session.accountId,
        clientId,
      });
  if (!grant) throw new Error("Consent grant was not found");
  const details = interaction.prompt.details;
  if (Array.isArray(details.missingOIDCScope))
    grant.addOIDCScope(details.missingOIDCScope.join(" "));
  if (Array.isArray(details.missingOIDCClaims))
    grant.addOIDCClaims(details.missingOIDCClaims);
  for (const [resource, scopes] of Object.entries(
    details.missingResourceScopes ?? {},
  ))
    grant.addResourceScope(resource, scopes.join(" "));
  const grantId = await grant.save();
  await provider.interactionFinished(
    request,
    response,
    { consent: { grantId } },
    { mergeWithLastSubmission: true },
  );
}

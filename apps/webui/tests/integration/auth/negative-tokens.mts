import { exportJWK, generateKeyPair, SignJWT } from "jose";

export async function generateNegativeTokens(issuer: string) {
  const { privateKey, publicKey } = await generateKeyPair("RS256", {
    extractable: true,
  });
  const foreign = await generateKeyPair("RS256");
  const jwk = await exportJWK(publicKey);
  Object.assign(jwk, { kid: "negative-fixture", alg: "RS256", use: "sig" });
  const tokens: Record<string, string> = {};
  const now = Math.floor(Date.now() / 1000);
  for (const name of [
    "nonce",
    "issuer",
    "audience",
    "expired",
    "signature",
    "scope",
  ]) {
    const payload = {
      iss: issuer,
      sub: "negative-account",
      aud: "konobangu-test",
      iat: now,
      exp: now + 300,
      nonce: "bound-nonce",
    };
    if (name === "nonce") payload.nonce = "foreign-nonce";
    if (name === "issuer") payload.iss = "https://foreign-issuer.example";
    if (name === "audience") payload.aud = "foreign-client";
    if (name === "expired") payload.exp = now - 300;
    tokens[name] = await new SignJWT(payload)
      .setProtectedHeader({ alg: "RS256", kid: jwk.kid })
      .sign(name === "signature" ? foreign.privateKey : privateKey);
  }
  return { jwks: { keys: [jwk] }, tokens };
}

if (import.meta.main) {
  const issuer = process.argv[2];
  if (!issuer) throw new Error("A loopback negative-test issuer is required");
  process.stdout.write(JSON.stringify(await generateNegativeTokens(issuer)));
}

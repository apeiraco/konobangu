import { randomBytes } from "node:crypto";
import { createWriteStream } from "node:fs";
import {
  mkdir,
  mkdtemp,
  readdir,
  readFile,
  rm,
  writeFile,
} from "node:fs/promises";
import { createServer } from "node:net";
import { join, relative, resolve } from "node:path";
import { finished } from "node:stream/promises";
import spawn from "cross-spawn";
import { startIdp } from "./idp.mts";

const root = resolve(import.meta.dirname, "../../../../..");
export async function startAuthFixture({ codegenOnly = false } = {}) {
  const evidence = join(root, "temp/verification/browser");
  await mkdir(evidence, { recursive: true });
  const directory = await mkdtemp(join(evidence, "run-"));
  const fixtureWebui = join(directory, "webui");
  await mkdir(fixtureWebui);
  let idp: Awaited<ReturnType<typeof startIdp>> | undefined;
  let server: ReturnType<typeof spawn> | undefined;
  let descriptor: {
    baseUrl: string;
    instanceUrl: string;
    basicUrl: string;
    secureUrl: string;
  };
  const cancellation = new AbortController();
  const cancellationHandlers: [NodeJS.Signals, () => void][] = [];
  for (const signal of ["SIGINT", "SIGTERM"] as const) {
    const interrupt = () =>
      cancellation.abort(new Error(`Fixture runner interrupted (${signal})`));
    process.once(signal, interrupt);
    // Fixture disposal removes handlers so other suites can own their lifetime.
    // Registration happens below after the stack is constructed.
    cancellationHandlers.push([signal, interrupt]);
  }
  const sleep = (ms: number) =>
    new Promise((resolve) => setTimeout(resolve, ms));
  async function run(
    command: string,
    args: string[],
    {
      capture = false,
      ...options
    }: import("node:child_process").SpawnOptions & { capture?: boolean } = {},
  ) {
    const child = spawn(command, args, {
      cwd: root,
      stdio: capture ? ["ignore", "pipe", "inherit"] : "inherit",
      signal: cancellation.signal,
      ...options,
    });
    let stdout = "";
    if (capture)
      child.stdout!.on("data", (data) => {
        stdout += data;
      });
    await new Promise<void>((resolve, reject) => {
      child.once("error", reject);
      child.once("close", (code, signal) =>
        code === 0
          ? resolve()
          : reject(new Error(`${command} failed (${code ?? signal})`)),
      );
    });
    return stdout;
  }
  async function port() {
    await using reservation = createServer();
    await new Promise<void>((resolve, reject) => {
      reservation.once("error", reject);
      reservation.listen(0, "127.0.0.1", resolve);
    });
    const number = (reservation.address() as import("node:net").AddressInfo)
      .port;
    return number;
  }
  await using ownership = new AsyncDisposableStack();
  const cleanup = ownership;
  cleanup.defer(() => {
    for (const [signal, handler] of cancellationHandlers)
      process.removeListener(signal, handler);
    delete process.env.KONOBANGU_AUTH_DESCRIPTOR;
    delete process.env.KONOBANGU_LEGACY_CHROMIUM;
  });
  cleanup.defer(() => rm(join(directory, "config.json"), { force: true }));
  cleanup.defer(async () => {
    await idp?.[Symbol.asyncDispose]();
  });

  {
    const ports = await Promise.all(Array.from({ length: 5 }, port));
    idp = await startIdp({
      redirectUris: ports.map(
        (value, index) =>
          `${index === 3 ? "https" : "http"}://127.0.0.1:${value}/api/auth/session/callback`,
      ),
      clientSecret: `fixture-${randomBytes(32).toString("hex")}`,
    });
    // The secret is fixture-owned and never injected into the browser build.
    await writeFile(
      join(directory, "config.json"),
      JSON.stringify({
        issuer: idp.issuer,
        client_secret: idp.clientSecret,
        ports,
        directory,
      }),
      { mode: 0o600 },
    );
    const artifacts: string[] = [];
    const build = spawn(
      "cargo",
      [
        "build",
        "-p",
        "recorder",
        "--locked",
        "--features",
        "testcontainers,test-utils",
        "--bin",
        "auth-test-server",
        "--message-format=json",
      ],
      {
        cwd: root,
        stdio: ["ignore", "pipe", "inherit"],
        signal: cancellation.signal,
      },
    );
    let output = "";
    build.stdout!.on("data", (data) => {
      output += data;
    });
    const buildCode = await new Promise((resolve, reject) => {
      build.once("error", reject);
      build.once("exit", resolve);
    });
    for (const line of output.split("\n").filter(Boolean)) {
      const event = JSON.parse(line);
      if (event.reason === "compiler-message")
        process.stderr.write(
          `${event.message.rendered ?? event.message.message}\n`,
        );
      if (
        event.reason === "compiler-artifact" &&
        event.target.name === "auth-test-server" &&
        event.executable
      )
        artifacts.push(event.executable);
    }
    if (buildCode !== 0)
      throw new Error(`Support server build failed (${buildCode})`);
    if (artifacts.length !== 1)
      throw new Error("Support server executable was not reported by Cargo");
    await rm(join(directory, "server.json"), { force: true });
    const log = cleanup.adopt(
      createWriteStream(join(directory, "server.log")),
      async (stream) => {
        stream.end();
        await finished(stream);
      },
    );
    cleanup.defer(async () => {
      if (server && server.exitCode === null && server.signalCode === null) {
        const running = server;
        running.kill("SIGTERM");
        using _timeout = setTimeout(() => running.kill("SIGKILL"), 15000);
        await new Promise<void>((resolve) =>
          running.once("close", () => resolve()),
        );
      }
    });
    server = spawn(artifacts[0], [], {
      cwd: root,
      env: {
        ...process.env,
        KONOBANGU_AUTH_TEST_CONFIG: join(directory, "config.json"),
      },
      stdio: ["ignore", "pipe", "pipe"],
    });
    server.stdout!.pipe(log, { end: false });
    server.stderr!.pipe(log, { end: false });
    server.once("error", (error) => {
      log.write(String(error));
    });
    let ready = false;
    for (let attempt = 0; attempt < 600; attempt++) {
      cancellation.signal.throwIfAborted();
      if (server.exitCode !== null || server.signalCode !== null)
        throw new Error(
          `Support server failed; inspect ${join(directory, "server.log")}`,
        );
      try {
        await readFile(join(directory, "server.json"));
        ready = true;
        break;
      } catch {
        await sleep(100);
      }
    }
    if (!ready) throw new Error("Support server readiness timed out");
    descriptor = JSON.parse(
      await readFile(join(directory, "server.json"), "utf8"),
    );
    for (const url of [
      descriptor.baseUrl,
      descriptor.instanceUrl,
      descriptor.basicUrl,
      descriptor.secureUrl,
    ]) {
      const response = await fetch(`${url}/api/auth/session/user-info`);
      if (response.status !== 401)
        throw new Error(
          `Unexpected unauthenticated readiness status ${response.status}`,
        );
    }

    const codegenEnv = {
      ...process.env,
      KONOBANGU_GRAPHQL_SCHEMA: `${descriptor.basicUrl}/api/graphql/introspection`,
      KONOBANGU_CODEGEN_AUTHORIZATION: `Basic ${Buffer.from("fixture-admin:fixture-admin-password").toString("base64")}`,
    };
    await run("just", ["dev-codegen"], { env: codegenEnv });
    const generatedDirectory = join(root, "apps/webui/src/infra/graphql/gql");
    async function generatedSnapshot() {
      const files = (await readdir(generatedDirectory))
        .filter((file) => file.endsWith(".ts"))
        .sort();
      return JSON.stringify(
        await Promise.all(
          files.map(async (file) => [
            file,
            await readFile(join(generatedDirectory, file), "utf8"),
          ]),
        ),
      );
    }
    const firstGenerated = await generatedSnapshot();
    await run("just", ["dev-codegen"], { env: codegenEnv });
    if (firstGenerated !== (await generatedSnapshot()))
      throw new Error("GraphQL code generation is not stable");
    if (!codegenOnly) {
      const fixtureSecret = idp.clientSecret;
      // Browser fixtures compile OIDC independently from the maintainer's release provider.
      await run(
        "pnpm",
        [
          "--filter",
          "webui",
          "exec",
          "vite",
          "build",
          "--outDir",
          relative(join(root, "apps/webui"), fixtureWebui),
          "--emptyOutDir",
        ],
        {
          env: { ...process.env, AUTH__PROVIDER__TYPE: "oidc" },
        },
      );
      await run("pnpm", [
        "--filter",
        "webui",
        "exec",
        "biome",
        "check",
        "--write",
        "src/presentation/routeTree.gen.ts",
      ]);
      async function verifyBundle(directory: string) {
        for (const entry of await readdir(directory, { withFileTypes: true })) {
          const path = join(directory, entry.name);
          if (entry.isDirectory()) await verifyBundle(path);
          else if (/\.(js|html|css|map)$/.test(entry.name)) {
            const content = await readFile(path, "utf8");
            for (const sentinel of [
              fixtureSecret,
              "fixture-admin-password",
              "Sign in as A",
              "Sign in as B",
            ])
              if (content.includes(sentinel))
                throw new Error(
                  "Test credential or interaction leaked into the browser bundle",
                );
          }
        }
      }
      await verifyBundle(fixtureWebui);
      // A fixed older official browser provides genuine missing Temporal capability.
      // Never delete or replace a native Temporal object to exercise this branch.
      await run("mise", [
        "exec",
        "node@24.15.0",
        "--",
        "pnpm",
        "dlx",
        "playwright@1.51.1",
        "install",
        "chromium",
        "--only-shell",
      ]);
      const stdout = await run(
        "pnpm",
        [
          "dlx",
          "playwright@1.51.1",
          "install",
          "--dry-run",
          "chromium",
          "--only-shell",
        ],
        { capture: true },
      );
      const location =
        /browser: chromium-headless-shell[^\n]*\n\s*Install location:\s*([^\n]+)/
          .exec(stdout)?.[1]
          .trim();
      if (!location) throw new Error("Missing legacy browser install location");
      const legacyExecutable = join(
        location,
        process.platform === "darwin"
          ? "chrome-mac/headless_shell"
          : process.platform === "win32"
            ? "chrome-win/headless_shell.exe"
            : "chrome-linux/headless_shell",
      );
      process.env.KONOBANGU_AUTH_DESCRIPTOR = join(directory, "server.json");
      process.env.KONOBANGU_LEGACY_CHROMIUM = legacyExecutable;
    }
  }

  const lifetime = ownership.move();
  return {
    descriptor,
    [Symbol.asyncDispose]: () => lifetime.disposeAsync(),
  };
}

if (import.meta.main) {
  if (
    !process.argv.includes("--codegen-only") &&
    !process.argv.includes("--manual-media")
  )
    throw new Error(
      "Use Playwright or just test browser; fixture CLI accepts --codegen-only or --manual-media",
    );
  await using fixture = await startAuthFixture({
    codegenOnly: process.argv.includes("--codegen-only"),
  });
  if (process.argv.includes("--manual-media")) {
    console.log(
      `Native browser fixture ready: ${fixture.descriptor.basicUrl}/subscriptions/detail/5000`,
    );
    await new Promise<void>((resolve) => process.once("SIGINT", resolve));
  }
}

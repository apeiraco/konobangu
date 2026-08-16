import assert from "node:assert/strict";
import {
  existsSync,
  mkdirSync,
  mkdtempDisposableSync,
  readFileSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { bindings } from "../commands/bindings.mts";
import { cleanWorkspace } from "../commands/clean.mts";
import { cleanTypes } from "../commands/types.mts";
import { assertStaticElf } from "../lib/artifacts.mts";
import { CommandError, root, run } from "../lib/process.mts";

mkdirSync(join(root, "temp"), { recursive: true });

test("cleanup removes declared outputs without deleting secrets, data or dependencies", () => {
  using temporary = mkdtempDisposableSync(join(root, "temp/cleanup-"));
  const base = temporary.path;
  writeFileSync(
    join(base, "tsconfig.json"),
    readFileSync(join(root, "tsconfig.json")),
  );
  writeFileSync(
    join(base, "konobangu-metadata.toml"),
    readFileSync(join(root, "konobangu-metadata.toml")),
  );
  const generated = [
    "apps/webui/dist",
    "apps/webui/dist-tsc",
    "apps/webui/dist-tsc/app.tsbuildinfo",
    "apps/email-playground/out",
  ];
  const preserved = ["apps/webui/node_modules", "apps/recorder/data"];
  for (const directory of [...generated, ...preserved]) {
    mkdirSync(join(base, directory), { recursive: true });
    writeFileSync(
      join(base, directory, "sentinel"),
      "keep unless owned output",
    );
  }
  const secret = join(base, "apps/recorder/.env");
  writeFileSync(secret, "SECRET=private");
  cleanWorkspace("outputs", base);
  for (const directory of generated)
    assert.equal(existsSync(join(base, directory)), false);
  for (const directory of preserved)
    assert.equal(existsSync(join(base, directory, "sentinel")), true);
  assert.equal(readFileSync(secret, "utf8"), "SECRET=private");
  cleanWorkspace("dependencies", base);
  assert.equal(existsSync(join(base, "apps/webui/node_modules")), false);
  assert.equal(existsSync(join(base, "apps/recorder/data/sentinel")), true);
  assert.equal(readFileSync(secret, "utf8"), "SECRET=private");
  assert.throws(() => cleanWorkspace("unknown", base), /Use clean/);
});

test("type cleanup follows reference membership and rejects escaping or empty selections", () => {
  using temporary = mkdtempDisposableSync(join(root, "temp/references-"));
  const base = temporary.path;
  writeFileSync(
    join(base, "konobangu-metadata.toml"),
    readFileSync(join(root, "konobangu-metadata.toml")),
  );
  const config = join(base, "tsconfig.json");
  const output = join(base, "custom/consumer/dist-tsc");
  mkdirSync(output, { recursive: true });
  writeFileSync(
    config,
    JSON.stringify({ references: [{ path: "./custom/consumer" }] }),
  );
  assert.throws(() => cleanTypes(["--project="], base), /Unknown reference/);
  assert(existsSync(output));
  cleanTypes([], base);
  assert(!existsSync(output));
  writeFileSync(
    config,
    JSON.stringify({ references: [{ path: "../outside" }] }),
  );
  assert.throws(() => cleanTypes([], base), /repository-relative/);
});
test("process arguments preserve spaces, Unicode and shell metacharacters", () => {
  const values = ["a b", "中文", "$(echo unsafe)", "a&b", 'quote"literal'];
  const result = run(
    process.execPath,
    [
      "-e",
      "process.stdout.write(JSON.stringify(process.argv.slice(1)))",
      ...values,
    ],
    { capture: true },
  );
  assert.deepEqual(JSON.parse(result.stdout), values);
});
test("captured workspace metadata can exceed one MiB", () => {
  const result = run(
    process.execPath,
    ["-e", "process.stdout.write('x'.repeat(2 * 1024 * 1024))"],
    { capture: true },
  );
  assert.equal(result.stdout.length, 2 * 1024 * 1024);
});
test("child failures preserve the original exit code", () => {
  assert.throws(
    () => run(process.execPath, ["-e", "process.exit(7)"], { capture: true }),
    (error: unknown) => error instanceof CommandError && error.code === 7,
  );
});
test("binding verification rejects missing and added entries", () => {
  using temporary = mkdtempDisposableSync(join(root, "temp/tooling-test-"));
  const directory = temporary.path;

  const file = join(directory, "bindings.json");
  bindings("snapshot", file);
  bindings("verify", file);
  const expected = JSON.parse(readFileSync(file, "utf8"));
  expected["obsolete.ts"] = "fake";
  writeFileSync(file, JSON.stringify(expected));
  assert.throws(() => bindings("verify", file), /obsolete.ts/);
});
test("ELF validation rejects dynamic loaders and libraries", () => {
  const data = Buffer.alloc(192);
  data.set([127, 69, 76, 70, 2, 1]);
  data.writeUInt16LE(62, 18);
  data.writeBigUInt64LE(64n, 32);
  data.writeUInt16LE(56, 54);
  data.writeUInt16LE(1, 56);
  assert.doesNotThrow(() => assertStaticElf(data));
  data.writeUInt32LE(3, 64);
  assert.throws(() => assertStaticElf(data), /interpreter/);
  data.writeUInt32LE(2, 64);
  data.writeBigUInt64LE(128n, 72);
  data.writeBigUInt64LE(32n, 96);
  data.writeBigInt64LE(1n, 128);
  assert.throws(() => assertStaticElf(data), /shared library/);
  assert.throws(() => assertStaticElf(Buffer.from("not ELF")), /ELF64/);
});
test("working directory supports spaces and Unicode on repeat runs", () => {
  using temporary = mkdtempDisposableSync(join(root, "temp/工具 path-"));
  const directory = temporary.path;

  for (let i = 0; i < 2; i++) {
    const result = run(
      process.execPath,
      ["-e", "process.stdout.write(process.cwd())"],
      { cwd: directory, capture: true },
    );
    assert.equal(result.stdout, directory);
  }
});
test("missing tools and captured output overflow fail explicitly", () => {
  assert.throws(
    () => run("konobangu-missing-tool-iteration7", [], { capture: true }),
    /konobangu-missing-tool-iteration7.*ENOENT/,
  );
  assert.throws(
    () =>
      run(
        process.execPath,
        ["-e", "process.stdout.write('x'.repeat(34*1024*1024))"],
        { capture: true },
      ),
    /ENOBUFS/,
  );
});
test("terminated child preserves a signal failure", {
  skip: process.platform === "win32",
}, () => {
  assert.throws(
    () =>
      run(process.execPath, ["-e", "process.kill(process.pid,'SIGTERM')"], {
        capture: true,
      }),
    (error: unknown) => error instanceof CommandError && error.code === 143,
  );
});
test("Windows command shims preserve argument arrays", {
  skip: process.platform !== "win32",
}, () => {
  using temporary = mkdtempDisposableSync(join(root, "temp/命令 shim-"));
  const directory = temporary.path;

  writeFileSync(
    join(directory, "args.cjs"),
    "process.stdout.write(JSON.stringify(process.argv.slice(2)))",
  );
  const command = join(directory, "args.cmd");
  writeFileSync(command, `@"${process.execPath}" "%~dp0args.cjs" %*\r\n`);
  const values = ["space value", "中文", "a&b", 'quote"literal'];
  assert.deepEqual(
    JSON.parse(run(command, values, { capture: true }).stdout),
    values,
  );
});

test("importing the executable CLI does not parse argv or perform actions", () => {
  const result = run(
    process.execPath,
    [
      "--input-type=module",
      "--eval",
      `process.argv.push("unknown-action");
     const { createCli } = await import("./scripts/dev-cli.mts");
     if (createCli().name() !== "dev-cli") throw new Error("CLI factory missing");
     process.stdout.write("imported");`,
    ],
    { capture: true },
  );
  assert.equal(result.stdout, "imported");
  assert.equal(result.stderr, "");
});

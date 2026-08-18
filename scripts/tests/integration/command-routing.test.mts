import assert from "node:assert/strict";
import {
  chmodSync,
  copyFileSync,
  existsSync,
  mkdirSync,
  mkdtempDisposableSync,
  readFileSync,
  writeFileSync,
} from "node:fs";
import { delimiter, join } from "node:path";
import { test } from "node:test";
import { releaseMetadataSnapshot } from "../../commands/release.mts";
import { CommandError, root, run } from "../../lib/process.mts";

// Exercise the real Just -> CLI -> runner boundary without invoking builds.
function runnerFixture(commands = ["cargo", "pnpm", "mise"]) {
  mkdirSync(join(root, "temp"), { recursive: true });
  using ownership = new DisposableStack();
  const temporary = ownership.use(
    mkdtempDisposableSync(join(root, "temp/命令 routing-")),
  );
  const directory = temporary.path;
  writeFileSync(
    join(directory, "package.json"),
    JSON.stringify({ type: "commonjs" }),
  );
  const log = join(directory, "calls.jsonl");
  const script = join(directory, "runner.cjs");
  writeFileSync(
    script,
    `const fs = require("node:fs");
    const args = process.argv.slice(2);
    fs.appendFileSync(process.env.ROUTING_LOG, JSON.stringify(args) + "\\n");
    fs.appendFileSync(process.env.ROUTING_LOG + ".cwd", JSON.stringify(process.cwd()) + "\\n");
    if (args[0] === "cargo") process.stderr.write(process.env.ROUTING_CARGO_STDERR || "");
    process.exit(Number(process.env["ROUTING_" + args[0].toUpperCase() + "_EXIT"] || 0));`,
  );
  for (const command of commands) {
    const path = join(
      directory,
      process.platform === "win32" ? `${command}.cmd` : command,
    );
    writeFileSync(
      path,
      process.platform === "win32"
        ? `@"${process.execPath}" "%~dp0runner.cjs" ${command} %*\r\n`
        : `#!${process.execPath}\nrequire(${JSON.stringify(script)});\n`,
    );
    if (process.platform !== "win32") {
      // Executable stubs carry their name in argv, matching the Windows shim.
      writeFileSync(
        path,
        `#!${process.execPath}\nprocess.argv.splice(2, 0, ${JSON.stringify(command)}); require(${JSON.stringify(script)});\n`,
      );
      chmodSync(path, 0o755);
    }
  }
  const lifetime = ownership.move();
  return {
    [Symbol.dispose]() {
      lifetime.dispose();
    },
    directory,
    env: {
      PATH: `${directory}${delimiter}${process.env.PATH}`,
      ROUTING_LOG: log,
    },
    calls: () =>
      readFileSync(log, "utf8")
        .trim()
        .split("\n")
        .map((line) => JSON.parse(line)),
    cwdCalls: () =>
      readFileSync(`${log}.cwd`, "utf8")
        .trim()
        .split("\n")
        .map((line) => JSON.parse(line)),
  };
}

test("Just forwards runner filters literally on every platform", () => {
  using fixture = runnerFixture();

  const filters = [
    '中文 with spaces $(echo unsafe) & "literal"',
    "apostrophe's",
    "",
  ];
  const result = run("just", ["test", "webui", ...filters], {
    capture: true,
    env: fixture.env,
    check: false,
  });
  assert.equal(result.code, 0, result.stderr);
  assert.equal(fixture.calls().length, 1);
  assert.deepEqual(fixture.calls()[0].slice(-filters.length), filters);
});
test("filtered Cargo failure preserves its status without postprocessing generated bindings", () => {
  using fixture = runnerFixture();

  const args = [
    "-p",
    "recorder",
    "--features",
    "test-utils",
    "--test",
    "media_http",
    "filter with spaces",
  ];
  const result = run("just", ["test", "rust", ...args], {
    capture: true,
    check: false,
    env: {
      ...fixture.env,
      ROUTING_CARGO_EXIT: "17",
    },
  });
  assert.equal(result.code, 17, result.stderr);
  assert.deepEqual(fixture.calls(), [["cargo", "test", "--locked", ...args]]);
});
test("native quality failures stop the group and preserve the original status", () => {
  using fixture = runnerFixture();
  const result = run("just", ["check"], {
    capture: true,
    check: false,
    env: { ...fixture.env, ROUTING_CARGO_EXIT: "17" },
  });
  assert.equal(result.code, 17, result.stderr);
  assert.equal(fixture.calls().length, 1);
  assert.equal(fixture.calls()[0][0], "cargo");
});
test("workspace failure stops before doctests", () => {
  using fixture = runnerFixture();
  const result = run("just", ["test", "rust"], {
    capture: true,
    check: false,
    env: { ...fixture.env, ROUTING_CARGO_EXIT: "17" },
  });
  assert.equal(result.code, 17, result.stderr);
  const calls = fixture.calls();
  assert.equal(calls.length, 1);
  assert(calls[0].includes("--all-targets"));
});
test("media backfill forwards recorder arguments directly without a CLI wrapper", () => {
  using fixture = runnerFixture();
  if (process.platform === "win32") {
    // PowerShell uses legacy escaping for .cmd; Cargo is a native .exe.
    // A renamed Node executable with a preload captures native argv without a compiler.
    copyFileSync(process.execPath, join(fixture.directory, "cargo.exe"));
    const preload = join(fixture.directory, "native.cjs");
    writeFileSync(
      preload,
      `if (process.execPath.endsWith('cargo.exe')) {
      const path = require('node:path');
      const fs = require('node:fs');
      fs.appendFileSync(process.env.ROUTING_LOG, JSON.stringify(['cargo', path.basename(process.argv[1]), ...process.argv.slice(2)]) + '\\n');
      process.exit(0);
    }`,
    );
    Object.assign(fixture.env, {
      NODE_OPTIONS: `--require "${preload.split("\\").join("/")}"`,
    });
  }
  const path = '中文 space $(echo unsafe) & "literal".toml';
  const result = run(
    "just",
    ["media-backfill", "--config-file", path, "--limit", "7"],
    {
      capture: true,
      check: false,
      env: fixture.env,
    },
  );
  assert.equal(result.code, 0, result.stderr);
  assert.deepEqual(fixture.calls(), [
    [
      "cargo",
      "run",
      "-p",
      "recorder",
      "--locked",
      "--bin",
      "media-backfill",
      "--",
      "--config-file",
      path,
      "--limit",
      "7",
    ],
  ]);
});
test("native command modes reject unknown selections before running tools", () => {
  using fixture = runnerFixture();
  for (const recipe of ["dev-codegen", "build-email"]) {
    const result = run("just", [recipe, "unknown"], {
      capture: true,
      env: fixture.env,
      check: false,
    });
    assert.notEqual(result.code, 0);
    assert.match(result.stderr, /Use/);
  }
});
test("invalid selections cannot silently pass or filter the complete gate", () => {
  for (const args of [
    ["test", "unknown"],
    ["test", "media", "unexpected-filter"],
    ["verify", "unknown"],
    ["check", "unknown"],
    ["lint", "unknown"],
  ]) {
    const result = run("just", args, { capture: true, check: false });
    assert.notEqual(result.code, 0, args.join(" "));
    assert.match(
      result.stderr,
      /Unknown|does not accept|too many arguments|does not contain recipe/,
    );
  }
});

test("media matrix failures stop before building or running smoke", () => {
  using fixture = runnerFixture();
  const result = run("just", ["test", "media"], {
    capture: true,
    check: false,
    env: { ...fixture.env, ROUTING_CARGO_EXIT: "17" },
  });
  assert.equal(result.code, 17, result.stderr);
  assert.equal(fixture.calls().length, 1);
  assert.equal(fixture.calls()[0][0], "cargo");
});
test("build, platform and prepare options reject unknown values before native work", () => {
  using fixture = runnerFixture();
  for (const args of [
    ["build-release", "--target", "not-a-target"],
    ["build-release", "--backend", "not-a-backend"],
    ["platform-check", "--target", "not-a-target"],
    ["release", "prepare", "--target", "not-a-target"],
    ["release", "prepare", "--backend", "serial"],
  ]) {
    const result = run("just", args, {
      capture: true,
      check: false,
      env: fixture.env,
    });
    assert.notEqual(result.code, 0, args.join(" "));
    assert.match(result.stderr, /invalid|unknown/);
  }
});

test("media smoke honors an absolute Cargo target directory on every platform", () => {
  using fixture = runnerFixture();
  const result = run(
    process.execPath,
    ["scripts/dev-cli.mts", "media", "smoke", "probe"],
    {
      capture: true,
      check: false,
      env: { CARGO_TARGET_DIR: fixture.directory },
    },
  );
  assert.notEqual(result.code, 0);
  assert(
    result.stderr.includes(
      join(
        fixture.directory,
        "debug",
        process.platform === "win32" ? "recorder-cli.exe" : "recorder-cli",
      ),
    ),
    result.stderr,
  );
  assert.match(result.stderr, /ENOENT/);
});
test("media conflict validation rejects unrelated build failures and missing conflicts", () => {
  using fixture = runnerFixture();
  for (const [exit, stderr, expected] of [
    ["0", "", 1],
    ["17", "unrelated compiler failure", 1],
    ["17", "media backends are mutually exclusive", 0],
  ] as const) {
    const result = run(
      process.execPath,
      ["scripts/dev-cli.mts", "media", "check-conflict"],
      {
        capture: true,
        check: false,
        env: {
          ...fixture.env,
          ROUTING_CARGO_EXIT: exit,
          ROUTING_CARGO_STDERR: stderr,
        },
      },
    );
    assert.equal(result.code, expected, result.stderr);
  }
});

test("release preparation stops at a failed Just gate before platform checks or staging", () => {
  // Resolve the real executor before installing a shim for the nested acceptance invocation.
  const actualJust = process.env.PATH?.split(delimiter)
    .map((directory) =>
      join(
        directory.replace(/^"|"$/g, ""),
        process.platform === "win32" ? "just.exe" : "just",
      ),
    )
    .find((filename) => existsSync(filename));
  assert(actualJust, "Just must be on PATH");
  using fixture = runnerFixture(["just"]);
  const result = run(actualJust, ["release", "prepare", "--target", "native"], {
    capture: true,
    check: false,
    env: { ...fixture.env, ROUTING_JUST_EXIT: "17" },
  });
  assert.equal(result.code, 17, result.stderr);
  assert.deepEqual(fixture.calls(), [["just", "verify"]]);
});
test("native feature failure prevents platform artifact construction", () => {
  using fixture = runnerFixture(["rustc", "just", "cargo"]);
  const result = run(
    process.execPath,
    ["scripts/dev-cli.mts", "platform-check"],
    {
      capture: true,
      check: false,
      env: { ...fixture.env, ROUTING_JUST_EXIT: "17" },
    },
  );
  assert.equal(result.code, 17, result.stderr);
  assert(fixture.calls().some((call) => call[0] === "just"));
  assert(!fixture.calls().some((call) => call[0] === "cargo"));
});
test("container artifact construction does not repeat host feature tests", () => {
  using fixture = runnerFixture(["rustc", "just", "docker"]);
  const result = run(
    process.execPath,
    [
      "scripts/dev-cli.mts",
      "platform-check",
      "--target",
      "x86_64-unknown-linux-musl",
    ],
    {
      capture: true,
      check: false,
      env: { ...fixture.env, ROUTING_DOCKER_EXIT: "17" },
    },
  );
  assert.equal(result.code, 17, result.stderr);
  assert(fixture.calls().some((call) => call[0] === "docker"));
  assert(!fixture.calls().some((call) => call[0] === "just"));
});
test("native Just forwarding preserves release paths without evaluating shell content", () => {
  const path = `outside/中文 space $(echo unsafe) & "literal" apostrophe's`;
  const result = run("just", ["release", "publish", path], {
    capture: true,
    check: false,
  });
  assert.equal(result.code, 1, result.stderr);
  assert.match(result.stderr, /Select a platform bundle/);
  assert.doesNotMatch(
    result.stderr,
    /not recognized|command not found|unexpected token/i,
  );
});

test("release plan exports the source selection and metadata hash to GitHub", () => {
  using temporary = mkdtempDisposableSync(join(root, "temp/plan-output-"));
  const output = join(temporary.path, "github-output");
  writeFileSync(output, "existing=value\n");
  run("just", ["release", "plan", "--format", "github-output"], {
    capture: true,
    env: { GITHUB_OUTPUT: output },
  });
  const values = Object.fromEntries(
    readFileSync(output, "utf8")
      .trim()
      .split(/\r?\n/)
      .map((line) => {
        const separator = line.indexOf("=");
        return [line.slice(0, separator), line.slice(separator + 1)];
      }),
  );
  assert.equal(values.existing, "value");
  const plan = JSON.parse(values.release_plan);
  const snapshot = releaseMetadataSnapshot();
  assert.deepEqual(plan.artifacts, snapshot.artifacts);
  assert.equal(plan.metadata_sha256, snapshot.sha256);
  for (const [key, selected] of Object.entries(snapshot.artifacts))
    assert.equal(values[key], String(selected));
  assert.equal(
    plan.source_sha,
    run("git", ["rev-parse", "HEAD"], { capture: true }).stdout.trim(),
  );
});

test("CLI help works outside the repository and rejects unknown commands", () => {
  const cli = join(root, "scripts/dev-cli.mts");
  assert.match(
    run(process.execPath, [cli, "--help"], {
      capture: true,
      cwd: join(root, "temp"),
    }).stdout,
    /build-release/,
  );
  assert.throws(
    () => run(process.execPath, [cli, "not-a-command"], { capture: true }),
    CommandError,
  );
});

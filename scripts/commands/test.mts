import { readdirSync } from "node:fs";
import { join } from "node:path";
import { root, run } from "../lib/process.mts";

// Only unrestricted runner arguments need an adapter; fixed sequences belong to Just.
export function testSuite(suite = "all", args: string[] = []) {
  if (!["rust", "webui", "tooling", "browser"].includes(suite) && args.length)
    throw new Error(`${suite} does not accept these arguments`);
  switch (suite) {
    case "rust":
      if (args.length) run("cargo", ["test", "--locked", ...args]);
      else run("just", ["_test-rust"]);
      break;
    case "webui":
      run("pnpm", ["--filter", "webui", "exec", "vitest", "run", ...args]);
      break;
    case "tooling":
      run(process.execPath, [
        "--test",
        ...args,
        ...readdirSync(join(root, "scripts/tests"), {
          recursive: true,
          encoding: "utf8",
        })
          .filter((file) => /\.(test|spec)\.[cm]?[jt]s$/.test(file))
          .sort()
          .map((file) => join(root, "scripts/tests", file)),
      ]);
      break;
    case "browser": {
      // The mature IdP supports LTS runtimes; Node 24 executes these TS fixtures directly.
      const env = { ...process.env };
      env.NO_COLOR = undefined;
      run(
        "mise",
        [
          "exec",
          "node@24.15.0",
          "--",
          "pnpm",
          "--filter",
          "webui",
          "exec",
          "playwright",
          "test",
          ...args,
        ],
        { env },
      );
      break;
    }
    case "media":
      run("just", ["_test-media-matrix"]);
      break;
    case "all":
      run("just", ["_test-all"]);
      break;
    default:
      throw new Error(`Unknown test suite: ${suite}`);
  }
}

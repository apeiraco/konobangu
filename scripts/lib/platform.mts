import { readFileSync } from "node:fs";
import { type BuildTarget, containerTarget } from "../commands/build.mts";
import { assertStaticElf } from "./artifacts.mts";
import { run } from "./process.mts";
export function inspectDependencies(artifact: string, target: BuildTarget) {
  if (containerTarget(target)) {
    assertStaticElf(readFileSync(artifact));
    return [{ command: "ELF dependency audit", static: true }];
  }
  const commands =
    process.platform === "win32"
      ? [["dumpbin", "/DEPENDENTS", artifact]]
      : process.platform === "darwin"
        ? [["otool", "-L", artifact]]
        : [
            ["readelf", "-d", artifact],
            ["readelf", "--version-info", artifact],
          ];
  const dependencies = commands.map(([command, ...args]) => ({
    command,
    args,
    ...run(command, args, { capture: true }),
  }));
  const text = dependencies.map((r) => r.stdout).join("\n");
  if (
    process.platform === "win32" &&
    /(?:vcruntime|msvcp|concrt|libcrypto|libssl|webp|jxl|aws.?lc)[\w.-]*\.dll/i.test(
      text,
    )
  )
    throw new Error("Unexpected codec/crypto or VC runtime DLL dependency");
  if (
    process.platform === "darwin" &&
    text
      .split("\n")
      .slice(1)
      .some(
        (line) =>
          line.trim() && !/^\s*\/(?:usr\/lib|System\/Library)\//.test(line),
      )
  )
    throw new Error("Unexpected non-system dylib dependency");
  if (process.platform === "linux") {
    for (const match of text.matchAll(/Shared library: \[([^\]]+)\]/g))
      if (
        !/^(?:lib(?:c|m|dl|pthread|rt|gcc_s|stdc\+\+)\.so[.\d]*|ld-linux[^/]*\.so[.\d]*)$/.test(
          match[1],
        )
      )
        throw new Error(`Unexpected shared library ${match[1]}`);
  }
  return dependencies;
}

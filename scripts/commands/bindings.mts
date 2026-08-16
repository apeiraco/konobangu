import { createHash } from "node:crypto";
import { mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { root } from "../lib/process.mts";
export function bindings(mode: "snapshot" | "verify", filename: string) {
  const directory = "apps/recorder/bindings";
  const hashes = Object.fromEntries(
    readdirSync(join(root, directory))
      .filter((f) => f.endsWith(".ts"))
      .sort()
      .map((f) => [
        `${directory}/${f}`,
        createHash("sha256")
          .update(readFileSync(join(root, directory, f)))
          .digest("hex"),
      ]),
  );
  const path = resolve(root, filename);
  if (mode === "snapshot") {
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, `${JSON.stringify(hashes, null, 2)}\n`);
    return;
  }
  const expected: Record<string, string> = JSON.parse(
    readFileSync(path, "utf8"),
  );
  const changed = [
    ...new Set([...Object.keys(hashes), ...Object.keys(expected)]),
  ].filter((k) => hashes[k] !== expected[k]);
  if (changed.length)
    throw new Error(`Generated bindings changed: ${changed.join(", ")}`);
  console.log(`Stable generated bindings: ${Object.keys(hashes).length} files`);
}

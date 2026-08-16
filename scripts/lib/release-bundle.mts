import {
  cpSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { basename, dirname, join } from "node:path";
import { sha256 } from "./artifacts.mts";

export function bundleHashes(directory: string): Record<string, string> {
  const hashes: Record<string, string> = {};
  function walk(path: string, prefix: string) {
    for (const entry of readdirSync(path, { withFileTypes: true }).sort(
      (a, b) => (a.name < b.name ? -1 : a.name > b.name ? 1 : 0),
    )) {
      const name = prefix ? `${prefix}/${entry.name}` : entry.name;
      if (name === "release.json") continue;
      if (entry.isSymbolicLink())
        throw new Error(`Release bundles cannot contain symlinks: ${name}`);
      if (entry.isDirectory()) walk(join(path, entry.name), name);
      else if (entry.isFile()) hashes[name] = sha256(join(path, entry.name));
    }
  }
  walk(directory, "");
  return hashes;
}
export function stageBundle(options: {
  output: string;
  artifact: string;
  webui: string;
  license: string;
  changelog: string;
  version: string;
  revision: string;
  dirty: boolean;
  target: string;
  verified?: boolean;
}) {
  rmSync(options.output, { recursive: true, force: true });
  mkdirSync(options.output, { recursive: true });
  cpSync(options.artifact, join(options.output, basename(options.artifact)));
  cpSync(
    join(dirname(options.artifact), "licenses"),
    join(options.output, "licenses"),
    { recursive: true },
  );
  cpSync(options.webui, join(options.output, "webui"), { recursive: true });
  cpSync(options.license, join(options.output, "LICENSE"));
  cpSync(options.changelog, join(options.output, "CHANGELOG.md"));
  const receipt = {
    version: options.version,
    target: options.target,
    revision: options.revision,
    dirty: options.dirty,
    artifact_name: basename(options.artifact),
    artifact_sha256: sha256(options.artifact),
    files: bundleHashes(options.output),
    // File staging alone does not attest that acceptance gates ran.
    verified: options.verified ?? false,
  };
  writeFileSync(
    join(options.output, "release.json"),
    `${JSON.stringify(receipt, null, 2)}\n`,
  );
  return receipt;
}
export function verifyBundleFiles(directory: string) {
  const receipt = JSON.parse(
    readFileSync(join(directory, "release.json"), "utf8"),
  );
  const actual = bundleHashes(directory);
  const expected = receipt.files as Record<string, string>;
  if (
    !expected ||
    JSON.stringify(Object.keys(actual).sort()) !==
      JSON.stringify(Object.keys(expected).sort()) ||
    Object.keys(actual).some((name) => actual[name] !== expected[name])
  )
    throw new Error("Release bundle changed after verification");
  return receipt;
}

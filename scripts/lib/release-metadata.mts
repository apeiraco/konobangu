import { readFileSync } from "node:fs";
import { isAbsolute, relative, resolve } from "node:path";
import semver from "semver";
import { parse } from "smol-toml";
import { root } from "./process.mts";

export type ManagedPackage = {
  name: string;
  manifest: string;
  publish: false;
  versioned?: boolean;
};
export type ReleaseArtifacts = {
  bundles: boolean;
  runtime_image: boolean;
  testing_torrents_image: boolean;
};
export type ReleaseMetadataSnapshot = {
  sha256: string;
  artifacts: ReleaseArtifacts;
};
export type Metadata = {
  tooling: { typescript: { config: string } };
  project: {
    version: string;
    display_name: string;
    description: string;
    license: string;
    repository_url: string;
  };
  release: {
    tag_prefix: string;
    output_directory: string;
    binary: string;
    cargo_package: string;
    changelog: string;
    artifacts: ReleaseArtifacts;
  };
  node_package: ManagedPackage[];
  rust_package: ManagedPackage[];
  python_package: ManagedPackage[];
};
export function releaseVersion(value: string): string {
  if (semver.valid(value) !== value)
    throw new Error(`Expected an exact SemVer version: ${value}`);
  return value;
}
export function projectPath(base: string, path: string): string {
  const target = resolve(base, path);
  const rel = relative(base, target);
  if (isAbsolute(path) || rel.startsWith("..") || !rel)
    throw new Error(`Expected a repository-relative path: ${path}`);
  return target;
}
export function loadMetadata(base = root): Metadata {
  return parseMetadata(
    readFileSync(resolve(base, "konobangu-metadata.toml"), "utf8"),
    base,
  );
}
export function parseMetadata(source: string, base = root): Metadata {
  const metadata = parse(source) as unknown as Metadata;
  releaseVersion(metadata.project.version);
  projectPath(base, metadata.tooling.typescript.config);
  for (const key of [
    "display_name",
    "description",
    "license",
    "repository_url",
  ] as const)
    if (!metadata.project[key]) throw new Error(`Missing project.${key}`);
  const seen = new Set<string>();
  for (const entry of [
    ...metadata.node_package,
    ...metadata.rust_package,
    ...metadata.python_package,
  ]) {
    projectPath(base, entry.manifest);
    if (!entry.name || entry.publish !== false || seen.has(entry.manifest))
      throw new Error(`Invalid package entry: ${entry.manifest}`);
    seen.add(entry.manifest);
  }
  if (!/^v?$/.test(metadata.release.tag_prefix))
    throw new Error("Unsupported tag prefix");
  projectPath(base, metadata.release.output_directory);
  if (!metadata.release.output_directory.startsWith("temp/"))
    throw new Error("Release output must be under temp/");
  projectPath(base, metadata.release.changelog);
  if (!/^[\w-]+$/.test(metadata.release.binary))
    throw new Error("Invalid release binary");
  const artifacts = metadata.release.artifacts;
  const artifactKeys = ["bundles", "runtime_image", "testing_torrents_image"];
  if (
    !artifacts ||
    Array.isArray(artifacts) ||
    Object.keys(artifacts).length !== artifactKeys.length ||
    artifactKeys.some(
      (key) => typeof artifacts[key as keyof ReleaseArtifacts] !== "boolean",
    )
  )
    throw new Error(
      "release.artifacts requires boolean bundles, runtime_image and testing_torrents_image fields",
    );
  // Keep metadata snapshots comparable after JSON serialization of receipts.
  metadata.release.artifacts = {
    bundles: artifacts.bundles,
    runtime_image: artifacts.runtime_image,
    testing_torrents_image: artifacts.testing_torrents_image,
  };
  return metadata;
}
export function packageVersion(source: string, section: string): string {
  const data = parse(source) as Record<string, Record<string, unknown>>;
  const version = data[section]?.version;
  if (typeof version !== "string")
    throw new Error(`Missing ${section}.version`);
  return version;
}
// Git may check out release manifests with CRLF on Windows.
function newline(source: string): string {
  return source.includes("\r\n") ? "\r\n" : "\n";
}
export function replaceTomlVersion(
  source: string,
  section: string,
  version: string,
): string {
  let replaced = false;
  let active = false;
  const result = source
    .split(/\r?\n/)
    .map((line) => {
      if (/^\s*\[/.test(line)) active = line.trim() === `[${section}]`;
      if (active && /^version\s*=/.test(line)) {
        replaced = true;
        return `version = "${version}"`;
      }
      return line;
    })
    .join(newline(source));
  if (!replaced) throw new Error(`Missing ${section}.version`);
  return result;
}
export function versionEdits(base: string, version: string) {
  releaseVersion(version);
  const metadata = loadMetadata(base);
  const edits: { path: string; source: string; next: string }[] = [];
  for (const entry of metadata.node_package) {
    const path = projectPath(base, entry.manifest);
    const source = readFileSync(path, "utf8");
    const data = JSON.parse(source);
    if (data.name !== entry.name)
      throw new Error(`Package name mismatch: ${entry.manifest}`);
    if (entry.versioned) data.version = version;
    if (entry.manifest === "package.json") {
      data.description = metadata.project.description;
      data.license = metadata.project.license;
      data.repository = {
        type: "git",
        url: `${metadata.project.repository_url}.git`,
      };
    }
    data.private = true;
    edits.push({
      path,
      source,
      next: `${JSON.stringify(data, null, 2)}\n`.replaceAll(
        "\n",
        newline(source),
      ),
    });
  }
  for (const [entries, section] of [
    [metadata.rust_package, "package"],
    [metadata.python_package, "project"],
  ] as const) {
    for (const entry of entries) {
      const path = projectPath(base, entry.manifest);
      const source = readFileSync(path, "utf8");
      const data = parse(source) as Record<string, Record<string, unknown>>;
      if (data[section]?.name !== entry.name)
        throw new Error(`Package name mismatch: ${entry.manifest}`);
      edits.push({
        path,
        source,
        next:
          section === "package"
            ? syncRustMetadata(
                replaceTomlVersion(source, section, version),
                metadata,
              )
            : replaceTomlVersion(source, section, version),
      });
    }
  }
  const path = resolve(base, "konobangu-metadata.toml");
  const source = readFileSync(path, "utf8");
  edits.push({
    path,
    source,
    next: replaceTomlVersion(source, "project", version),
  });
  return edits;
}

function syncRustMetadata(source: string, metadata: Metadata): string {
  let active = false;
  const fields: Record<string, string> = {
    license: JSON.stringify(metadata.project.license),
    repository: JSON.stringify(metadata.project.repository_url),
    publish: "false",
  };
  const seen = new Set<string>();
  const lines = source.split(/\r?\n/);
  const result: string[] = [];
  for (const line of lines) {
    if (/^\s*\[/.test(line)) {
      if (active)
        for (const [key, value] of Object.entries(fields))
          if (!seen.has(key)) result.push(`${key} = ${value}`);
      active = line.trim() === "[package]";
    }
    const key = active
      ? line.match(/^(license|repository|publish)\s*=/)?.[1]
      : undefined;
    if (key) {
      seen.add(key);
      result.push(`${key} = ${fields[key]}`);
    } else result.push(line);
  }
  if (active)
    for (const [key, value] of Object.entries(fields))
      if (!seen.has(key)) result.push(`${key} = ${value}`);
  return result.join(newline(source));
}

export function releaseNotes(changelog: string, version: string): string {
  releaseVersion(version);
  const lines = changelog.split(/\r?\n/);
  const start = lines.findIndex(
    (line) => line === `## ${version}` || line.startsWith(`## ${version} - `),
  );
  if (start < 0)
    throw new Error("Finalize the exact versioned changelog before releasing");
  const end = lines.findIndex(
    (line, index) => index > start && line.startsWith("## "),
  );
  return lines
    .slice(start + 1, end < 0 ? undefined : end)
    .join("\n")
    .trim();
}

import {
  appendFileSync,
  existsSync,
  readFileSync,
  writeFileSync,
} from "node:fs";
import { basename, dirname, join, relative, resolve } from "node:path";
import { sha256 } from "../lib/artifacts.mts";
import { root, run } from "../lib/process.mts";
import { stageBundle, verifyBundleFiles } from "../lib/release-bundle.mts";
import {
  loadMetadata,
  type Metadata,
  parseMetadata,
  type ReleaseMetadataSnapshot,
  releaseNotes,
  versionEdits,
} from "../lib/release-metadata.mts";
import { artifactPath, type BuildTarget, containerTarget } from "./build.mts";

export function checkReleaseVersion() {
  const metadata = loadMetadata();
  const edits = versionEdits(root, metadata.project.version);
  const changed = edits.filter((entry) => entry.source !== entry.next);
  if (changed.length)
    throw new Error(
      `Run release metadata sync; mismatched manifests: ${changed.map((e) => e.path).join(", ")}`,
    );
  console.log(`Release-managed packages match ${metadata.project.version}`);
}
export function setReleaseVersion(version: string) {
  const edits = versionEdits(root, version);
  const lockfiles = ["Cargo.lock", "pnpm-lock.yaml", "uv.lock"].map((path) => ({
    path: join(root, path),
    source: readFileSync(join(root, path), "utf8"),
  }));
  // Validate every manifest before changing any of them; restore on command failure.
  try {
    for (const edit of edits)
      if (edit.source !== edit.next) writeFileSync(edit.path, edit.next);
    run("cargo", ["update", "--workspace", "--offline"]);
    run("pnpm", ["install", "--lockfile-only", "--ignore-scripts"]);
    run("uv", ["lock", "--offline"]);
  } catch (error) {
    for (const edit of [...edits, ...lockfiles])
      writeFileSync(edit.path, edit.source);
    throw error;
  }
  checkReleaseVersion();
}
function releaseMetadataSource(base = root) {
  const source = readFileSync(join(base, "konobangu-metadata.toml"), "utf8");
  return {
    metadata: parseMetadata(source, base),
    // Git checkout line endings must not change cross-platform source identity.
    sha256: createHash("sha256")
      .update(source.replaceAll("\r\n", "\n"))
      .digest("hex"),
  };
}
export function releaseMetadataSnapshot(base = root): ReleaseMetadataSnapshot {
  const { metadata, sha256: metadataHash } = releaseMetadataSource(base);
  return {
    sha256: metadataHash,
    artifacts: metadata.release.artifacts,
  };
}
export function releasePlanData(metadata: Metadata, sourceSha: string) {
  const repository = metadata.project.repository_url.match(
    /^https:\/\/github\.com\/([\w.-]+\/[\w.-]+)$/,
  )?.[1];
  if (!repository)
    throw new Error("Expected a canonical GitHub repository URL");
  const artifacts = metadata.release.artifacts;
  const version = metadata.project.version;
  const images = Object.fromEntries(
    [
      ["runtime_image", `ghcr.io/${repository.toLowerCase()}`],
      [
        "testing_torrents_image",
        `ghcr.io/${repository.toLowerCase()}-testing-torrents`,
      ],
    ]
      .filter(([key]) => artifacts[key as keyof typeof artifacts])
      .map(([key, image]) => [
        key,
        {
          image,
          tags: [`${image}:${version}`, `${image}:sha-${sourceSha}`],
          latest_tag: version.includes("-") ? null : `${image}:latest`,
        },
      ]),
  );
  return {
    version,
    tag: `${metadata.release.tag_prefix}${version}`,
    source_sha: sourceSha,
    repository: metadata.project.repository_url,
    output: metadata.release.output_directory,
    package_registry_publish: false,
    artifacts,
    has_artifacts: Object.values(artifacts).some(Boolean),
    images,
  };
}
export function releasePlan(format: "json" | "github-output" = "json") {
  const { metadata, sha256: metadataHash } = releaseMetadataSource();
  const plan = {
    ...releasePlanData(
      metadata,
      run("git", ["rev-parse", "HEAD"], { capture: true }).stdout.trim(),
    ),
    metadata_sha256: metadataHash,
    dirty: Boolean(
      run("git", ["status", "--porcelain"], { capture: true }).stdout.trim(),
    ),
  };
  if (format === "json") console.log(JSON.stringify(plan, null, 2));
  else {
    const output = process.env.GITHUB_OUTPUT;
    if (!output) throw new Error("github-output requires GITHUB_OUTPUT");
    const values = {
      version: plan.version,
      source_sha: plan.source_sha,
      metadata_sha256: plan.metadata_sha256,
      has_artifacts: plan.has_artifacts,
      release_plan: JSON.stringify(plan),
      ...plan.artifacts,
      ...Object.fromEntries(
        Object.entries(plan.images).flatMap(([key, image]) => [
          [`${key}_tags`, image.tags.join(",")],
          [`${key}_latest_tag`, image.latest_tag ?? ""],
        ]),
      ),
    };
    appendFileSync(
      output,
      `${Object.entries(values)
        .map(([key, value]) => `${key}=${value}`)
        .join("\n")}\n`,
    );
  }
}
// Metadata and verified artifact staging form one release operation; Just owns the actual gates.
export function prepareRelease(target: BuildTarget) {
  checkReleaseVersion();
  run("just", ["verify"]);
  run("just", ["platform-check", "--target", target]);
  bundleRelease(target, true);
}
function bundleRelease(target: BuildTarget, verified = false) {
  checkReleaseVersion();
  const metadata = loadMetadata();
  const artifact = artifactPath(target);
  const output = join(
    root,
    metadata.release.output_directory,
    metadata.project.version,
    containerTarget(target)
      ? "linux-x64-musl"
      : `${process.platform}-${process.arch}`,
  );
  const webui = join(root, "apps/webui/dist");
  if (!existsSync(join(webui, "index.html")))
    throw new Error("Missing production WebUI");
  stageBundle({
    output,
    artifact,
    webui,
    license: join(root, "LICENSE"),
    changelog: join(root, metadata.release.changelog),
    version: metadata.project.version,
    target,
    verified,
    release_metadata: releaseMetadataSnapshot(),
    revision: run("git", ["rev-parse", "HEAD"], {
      capture: true,
    }).stdout.trim(),
    dirty:
      run("git", ["status", "--porcelain"], { capture: true }).stdout.trim()
        .length > 0,
  });
  console.log(`Prepared release: ${output}`);
}
export function tagRelease(execute: boolean) {
  checkReleaseVersion();
  const metadata = loadMetadata();
  if (!Object.values(metadata.release.artifacts).some(Boolean))
    throw new Error("No release artifacts are enabled in metadata");
  const tag = `${metadata.release.tag_prefix}${metadata.project.version}`;
  releaseNotes(
    readFileSync(join(root, metadata.release.changelog), "utf8"),
    metadata.project.version,
  );
  if (
    run("git", ["rev-parse", "--verify", `refs/tags/${tag}`], {
      capture: true,
      check: false,
    }).code === 0
  )
    throw new Error(`Tag already exists: ${tag}`);
  if (execute)
    run("git", [
      "tag",
      "-a",
      tag,
      "-m",
      `Konobangu ${metadata.project.version}`,
    ]);
  else
    console.log(
      `Dry run: git tag -a ${tag}; use --execute to create the local tag`,
    );
}

export function publishRelease(directory: string, execute: boolean) {
  checkReleaseVersion();
  const metadata = loadMetadata();
  if (!metadata.release.artifacts.bundles)
    throw new Error("Bundle publication is disabled in release.artifacts");
  const output = resolve(root, directory);
  const expected = join(
    root,
    metadata.release.output_directory,
    metadata.project.version,
  );
  const inside = relative(expected, output);
  if (!inside || inside.startsWith(".."))
    throw new Error(
      "Select a platform bundle beneath the current version release directory",
    );
  const receipt = verifyBundleFiles(output, releaseMetadataSnapshot());
  const revision = run("git", ["rev-parse", "HEAD"], {
    capture: true,
  }).stdout.trim();
  if (
    receipt.version !== metadata.project.version ||
    receipt.verified !== true ||
    receipt.dirty ||
    receipt.revision !== revision
  )
    throw new Error(
      "Prepare the committed current release candidate before publishing",
    );
  if (
    receipt.artifact_name !== metadata.release.binary &&
    receipt.artifact_name !== `${metadata.release.binary}.exe`
  )
    throw new Error("Invalid release artifact name");
  if (sha256(join(output, receipt.artifact_name)) !== receipt.artifact_sha256)
    throw new Error("Release executable differs from its receipt");
  const tag = `${metadata.release.tag_prefix}${metadata.project.version}`;
  const notes = releaseNotes(
    readFileSync(join(root, metadata.release.changelog), "utf8"),
    metadata.project.version,
  );
  const archive = join(
    dirname(output),
    `konobangu-${metadata.project.version}-${basename(output)}.tar.gz`,
  );
  if (!execute) {
    console.log(
      JSON.stringify({ tag, archive, output, execute: false }, null, 2),
    );
    return;
  }
  if (run("git", ["status", "--porcelain"], { capture: true }).stdout.trim())
    throw new Error("Commit the release candidate before publishing");
  const localTag = run("git", ["rev-parse", `${tag}^{commit}`], {
    capture: true,
  }).stdout.trim();
  const remoteTags = run(
    "git",
    [
      "ls-remote",
      "--tags",
      metadata.project.repository_url,
      `refs/tags/${tag}`,
      `refs/tags/${tag}^{}`,
    ],
    { capture: true },
  )
    .stdout.trim()
    .split("\n");
  const remoteRef = (ref: string) =>
    remoteTags.find((line) => line.split("\t")[1] === ref)?.split("\t")[0];
  const remoteTag =
    remoteRef(`refs/tags/${tag}^{}`) ?? remoteRef(`refs/tags/${tag}`);
  if (localTag !== revision || remoteTag !== revision)
    throw new Error(
      "Local and remote release tags must identify the prepared commit",
    );
  run("tar", ["-czf", archive, "-C", output, "."]);
  const notesFile = join(dirname(output), "release-notes.md");
  writeFileSync(notesFile, `${notes.trim()}\n`);
  run("gh", [
    "release",
    "create",
    tag,
    archive,
    "--verify-tag",
    "--repo",
    metadata.project.repository_url,
    "--title",
    `${metadata.project.display_name} ${metadata.project.version}`,
    "--notes-file",
    notesFile,
  ]);
}

import { createHash } from "node:crypto";

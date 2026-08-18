import assert from "node:assert/strict";
import {
  mkdirSync,
  mkdtempDisposableSync,
  readFileSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import {
  releaseMetadataSnapshot,
  releasePlanData,
} from "../commands/release.mts";
import { root } from "../lib/process.mts";
import { stageBundle, verifyBundleFiles } from "../lib/release-bundle.mts";
import {
  loadMetadata,
  projectPath,
  releaseNotes,
  releaseVersion,
  replaceTomlVersion,
  versionEdits,
} from "../lib/release-metadata.mts";

test("release versions reject ranges and preserve TOML dependency versions", () => {
  for (const value of ["v0.2.0", "^0.2.0", "0.2", "01.2.3", "not-a-version"])
    assert.throws(() => releaseVersion(value));
  assert.equal(releaseVersion("0.2.0-rc.1"), "0.2.0-rc.1");
  const source =
    '[package]\nname = "app"\nversion = "0.1.0"\n\n[dependencies.library]\nversion = "1.0"\n';
  assert.equal(
    replaceTomlVersion(source, "package", "0.2.0"),
    '[package]\nname = "app"\nversion = "0.2.0"\n\n[dependencies.library]\nversion = "1.0"\n',
  );
});
test("metadata plans validate all packages before modifying files", () => {
  using temporary = mkdtempDisposableSync(join(root, "temp/release-fixture-"));
  const directory = temporary.path;

  const metadata = loadMetadata();
  writeFileSync(
    join(directory, "konobangu-metadata.toml"),
    readFileSync(join(root, "konobangu-metadata.toml")),
  );
  for (const entry of [
    ...metadata.node_package,
    ...metadata.rust_package,
    ...metadata.python_package,
  ]) {
    const path = join(directory, entry.manifest);
    mkdirSync(join(path, ".."), { recursive: true });
    writeFileSync(path, readFileSync(join(root, entry.manifest)));
  }
  const edits = versionEdits(directory, "0.2.0");
  assert.equal(
    edits.length,
    metadata.node_package.length +
      metadata.rust_package.length +
      metadata.python_package.length +
      1,
  );
  assert.equal(
    loadMetadata(directory).project.version,
    metadata.project.version,
  );
  writeFileSync(
    join(directory, metadata.rust_package[0].manifest),
    '[package]\nname = "wrong"\nversion = "0.1.0"\n',
  );
  assert.throws(() => versionEdits(directory, "0.2.0"), /name mismatch/);
  assert.equal(
    loadMetadata(directory).project.version,
    metadata.project.version,
  );
  assert.throws(() => projectPath(directory, "../outside"));
});

test("release plans preserve checkout newlines and still detect metadata drift", () => {
  const metadata = loadMetadata();
  const manifests = [
    "konobangu-metadata.toml",
    ...[
      ...metadata.node_package,
      ...metadata.rust_package,
      ...metadata.python_package,
    ].map((entry) => entry.manifest),
  ];
  for (const newline of ["\n", "\r\n"]) {
    using temporary = mkdtempDisposableSync(
      join(root, "temp/newline-fixture-"),
    );
    for (const manifest of manifests) {
      const path = join(temporary.path, manifest);
      mkdirSync(join(path, ".."), { recursive: true });
      writeFileSync(
        path,
        readFileSync(join(root, manifest), "utf8")
          .replaceAll("\r\n", "\n")
          .replaceAll("\n", newline),
      );
    }
    for (const edit of versionEdits(temporary.path, metadata.project.version))
      assert.equal(edit.next, edit.source, edit.path);
    for (const edit of versionEdits(temporary.path, "0.2.0")) {
      assert.equal(
        edit.next,
        edit.next.replaceAll("\r\n", "\n").replaceAll("\n", newline),
        edit.path,
      );
      writeFileSync(edit.path, edit.next);
    }
    assert.equal(loadMetadata(temporary.path).project.version, "0.2.0");
    for (const edit of versionEdits(temporary.path, "0.2.0"))
      assert.equal(edit.next, edit.source, edit.path);

    const manifest = metadata.node_package.find((entry) => entry.versioned);
    assert(manifest);
    const path = join(temporary.path, manifest.manifest);
    const source = readFileSync(path, "utf8");
    writeFileSync(path, source.replace('"0.2.0"', '"0.1.0"'));
    assert.deepEqual(
      versionEdits(temporary.path, "0.2.0")
        .filter((edit) => edit.source !== edit.next)
        .map((edit) => edit.path),
      [path],
    );
  }
});

test("publication selections are independent and empty selections produce no images", () => {
  const metadata = loadMetadata();
  const source = "a".repeat(40);
  for (let mask = 0; mask < 8; mask++) {
    const artifacts = {
      bundles: Boolean(mask & 1),
      runtime_image: Boolean(mask & 2),
      testing_torrents_image: Boolean(mask & 4),
    };
    const plan = releasePlanData(
      { ...metadata, release: { ...metadata.release, artifacts } },
      source,
    );
    assert.deepEqual(plan.artifacts, artifacts);
    assert.equal(plan.has_artifacts, mask !== 0);
    for (const key of ["runtime_image", "testing_torrents_image"] as const)
      assert.equal(Boolean(plan.images[key]), artifacts[key]);
  }
  for (const version of ["0.1.0", "0.1.0-rc.1"]) {
    const plan = releasePlanData(
      {
        ...metadata,
        project: { ...metadata.project, version },
        release: {
          ...metadata.release,
          artifacts: {
            bundles: false,
            runtime_image: false,
            testing_torrents_image: true,
          },
        },
      },
      source,
    );
    const image = plan.images.testing_torrents_image;
    assert(image);
    assert(image.tags.includes(`${image.image}:${version}`));
    assert(image.tags.includes(`${image.image}:sha-${source}`));
    assert(!image.tags.includes(`${image.image}:latest`));
    assert.equal(
      image.latest_tag,
      version === "0.1.0" ? `${image.image}:latest` : null,
    );
  }
});

test("metadata rejects missing, mistyped and unknown publication selections", () => {
  using temporary = mkdtempDisposableSync(
    join(root, "temp/selection-fixture-"),
  );
  const path = join(temporary.path, "konobangu-metadata.toml");
  const source = readFileSync(join(root, "konobangu-metadata.toml"), "utf8");
  for (const invalid of [
    source.replace(/^bundles\s*=\s*(true|false)/m, 'bundles = "true"'),
    source.replace(/^bundles\s*=\s*(true|false)\r?\n/m, ""),
    source.replace(
      "[release.artifacts]",
      "[release.artifacts]\nbundle = false",
    ),
  ]) {
    writeFileSync(path, invalid);
    assert.throws(() => loadMetadata(temporary.path), /release\.artifacts/);
  }
});

test("metadata snapshots survive the receipt JSON round trip", () => {
  const snapshot = releaseMetadataSnapshot();
  assert.deepEqual(JSON.parse(JSON.stringify(snapshot)), snapshot);
});

test("checkout line endings do not change release metadata identity", () => {
  using temporary = mkdtempDisposableSync(join(root, "temp/snapshot-newline-"));
  const path = join(temporary.path, "konobangu-metadata.toml");
  const source = readFileSync(
    join(root, "konobangu-metadata.toml"),
    "utf8",
  ).replaceAll("\r\n", "\n");
  writeFileSync(path, source);
  const snapshot = releaseMetadataSnapshot(temporary.path);
  writeFileSync(path, source.replaceAll("\n", "\r\n"));
  assert.deepEqual(releaseMetadataSnapshot(temporary.path), snapshot);
});

test("release bundles record the platform binary and reject changed WebUI/notices", () => {
  using temporary = mkdtempDisposableSync(join(root, "temp/bundle-fixture-"));
  const directory = temporary.path;

  const artifact = join(directory, "native/recorder-cli.exe");
  mkdirSync(join(directory, "native/licenses"), { recursive: true });
  writeFileSync(artifact, "binary");
  writeFileSync(
    join(directory, "native/licenses/THIRD-PARTY-NOTICES.md"),
    "notices",
  );
  mkdirSync(join(directory, "dist"));
  writeFileSync(join(directory, "dist/index.html"), "webui");
  writeFileSync(join(directory, "LICENSE"), "MIT");
  writeFileSync(join(directory, "CHANGELOG.md"), "release");
  const output = join(directory, "bundle");
  mkdirSync(output);
  writeFileSync(join(output, "stale-worker"), "old");
  const receipt = stageBundle({
    output,
    artifact,
    webui: join(directory, "dist"),
    license: join(directory, "LICENSE"),
    changelog: join(directory, "CHANGELOG.md"),
    version: "0.2.0",
    revision: "commit",
    dirty: false,
    target: "native",
    release_metadata: {
      sha256: "a".repeat(64),
      artifacts: {
        bundles: true,
        runtime_image: false,
        testing_torrents_image: false,
      },
    },
  });
  assert.equal(
    receipt.verified,
    false,
    "File staging must not attest acceptance gates",
  );
  assert.equal(receipt.artifact_name, "recorder-cli.exe");
  assert.deepEqual(receipt.release_metadata.artifacts, {
    bundles: true,
    runtime_image: false,
    testing_torrents_image: false,
  });
  assert.equal(receipt.files["stale-worker"], undefined);
  assert.deepEqual(verifyBundleFiles(output), receipt);
  assert.deepEqual(
    verifyBundleFiles(output, receipt.release_metadata),
    receipt,
  );
  assert.throws(
    () =>
      verifyBundleFiles(output, {
        ...receipt.release_metadata,
        sha256: "b".repeat(64),
      }),
    /metadata differs/,
  );
  assert.throws(
    () =>
      verifyBundleFiles(output, {
        ...receipt.release_metadata,
        artifacts: {
          ...receipt.release_metadata.artifacts,
          testing_torrents_image: true,
        },
      }),
    /metadata differs/,
  );
  writeFileSync(join(output, "webui/index.html"), "changed");
  assert.throws(() => verifyBundleFiles(output), /changed after verification/);
});

test("release notes require the exact version rather than a prerelease prefix", () => {
  for (const newline of ["\n", "\r\n"]) {
    assert.throws(
      () => releaseNotes(["## 0.2.0-rc.1", "preview"].join(newline), "0.2.0"),
      /exact versioned/,
    );
    assert.equal(
      releaseNotes(["## 0.2.0", "released", "## 0.1.0"].join(newline), "0.2.0"),
      "released",
    );
    assert.equal(
      releaseNotes(
        [
          "## Unreleased",
          "next",
          "## 0.2.0 - 2026-10-05",
          "released",
          "## 0.1.0",
          "old",
        ].join(newline),
        "0.2.0",
      ),
      "released",
    );
  }
});

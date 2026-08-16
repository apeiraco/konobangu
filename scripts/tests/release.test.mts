import assert from "node:assert/strict";
import {
  mkdirSync,
  mkdtempDisposableSync,
  readFileSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
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
  });
  assert.equal(
    receipt.verified,
    false,
    "File staging must not attest acceptance gates",
  );
  assert.equal(receipt.artifact_name, "recorder-cli.exe");
  assert.equal(receipt.files["stale-worker"], undefined);
  assert.deepEqual(verifyBundleFiles(output), receipt);
  writeFileSync(join(output, "webui/index.html"), "changed");
  assert.throws(() => verifyBundleFiles(output), /changed after verification/);
});

test("release notes require the exact version rather than a prerelease prefix", () => {
  assert.throws(
    () => releaseNotes("## 0.2.0-rc.1\npreview", "0.2.0"),
    /exact versioned/,
  );
  assert.equal(
    releaseNotes(
      "## Unreleased\nnext\n## 0.2.0 - 2026-10-05\nreleased\n## 0.1.0\nold",
      "0.2.0",
    ),
    "released",
  );
});

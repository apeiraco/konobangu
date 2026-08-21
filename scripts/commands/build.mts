import { mkdirSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { assertStaticElf, sha256 } from "../lib/artifacts.mts";
import { collectLicenses } from "../lib/licenses.mts";
import { root, run } from "../lib/process.mts";

function buildContainer() {
  const outputDirectory = resolve(
    root,
    process.env.CARGO_TARGET_DIR ?? "target",
  );
  const artifact = artifactPath("x86_64-unknown-linux-musl");
  const cache = join(root, "temp/build/x86_64-unknown-linux-musl/cargo");
  mkdirSync(cache, { recursive: true });
  mkdirSync(dirname(artifact), { recursive: true });
  run("docker", [
    "build",
    "--platform=linux/amd64",
    "--tag",
    "konobangu-musl-builder:local",
    "build/musl",
  ]);
  run("docker", [
    "run",
    "--rm",
    "--platform=linux/amd64",
    "--mount",
    `type=bind,src=${root},dst=/work`,
    "--mount",
    `type=bind,src=${cache},dst=/cache/cargo`,
    "--mount",
    `type=bind,src=${outputDirectory},dst=/build-output`,
    "-e",
    "CARGO_TARGET_DIR=/build-output",
    "-e",
    "CARGO_HOME=/cache/cargo",
    ...(process.getuid
      ? ["--user", `${process.getuid()}:${process.getgid?.() ?? 0}`]
      : []),
    "-e",
    `CARGO_BUILD_JOBS=${process.env.CARGO_BUILD_JOBS ?? "2"}`,
    "konobangu-musl-builder:local",
    "cargo",
    "-Zhost-config",
    "-Ztarget-applies-to-host",
    "--config",
    "target-applies-to-host=false",
    "--config",
    'host.rustflags=["-C","target-feature=-crt-static"]',
    "build",
    "--release",
    "--target",
    "x86_64-unknown-linux-musl",
    "-p",
    "recorder",
    "--locked",
    "--bin",
    "recorder-cli",
  ]);
}
export const buildTargets = [
  "native",
  "x86_64-unknown-linux-gnu",
  "aarch64-unknown-linux-gnu",
  "x86_64-unknown-linux-musl",
  "x86_64-pc-windows-msvc",
  "aarch64-pc-windows-msvc",
  "x86_64-apple-darwin",
  "aarch64-apple-darwin",
] as const;
export type BuildTarget = (typeof buildTargets)[number];
export const containerTarget = (target: BuildTarget) =>
  target === "x86_64-unknown-linux-musl";
export function artifactPath(target: BuildTarget) {
  const base = process.env.CARGO_TARGET_DIR ?? join(root, "target");
  return join(
    base,
    ...(target === "native" ? [] : [target]),
    "release",
    target.includes("windows") ||
      (target === "native" && process.platform === "win32")
      ? "recorder-cli.exe"
      : "recorder-cli",
  );
}
// Keep notices and hashes tied to the exact selected target.
function collectBuildArtifact(target: BuildTarget) {
  const artifact = artifactPath(target);
  if (containerTarget(target)) assertStaticElf(readFileSync(artifact));
  collectLicenses(
    join(dirname(artifact), "licenses"),
    target === "native" ? undefined : target,
  );
  console.log(`Recorder SHA-256: ${sha256(artifact)}`);
}

// Building and collecting notices are one artifact operation across native/container targets.
export function buildRelease(target: BuildTarget = "native") {
  if (containerTarget(target)) buildContainer();
  else
    run("cargo", [
      "build",
      "--release",
      "--locked",
      ...(target === "native" ? [] : ["--target", target]),
      "-p",
      "recorder",
      "--bin",
      "recorder-cli",
    ]);
  collectBuildArtifact(target);
  return artifactPath(target);
}

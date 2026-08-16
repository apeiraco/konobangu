import { randomUUID } from "node:crypto";
import {
  chmodSync,
  cpSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { join, resolve } from "node:path";
import { sha256 } from "../lib/artifacts.mts";
import { inspectDependencies } from "../lib/platform.mts";
import { root, run } from "../lib/process.mts";
import { artifactPath, type BuildTarget, containerTarget } from "./build.mts";

function verifyEncodedOutputs(job: string) {
  const cases: object[] = [];
  const report = JSON.parse(
    readFileSync(join(job, "output/receipt.json"), "utf8"),
  );
  if (report.jxl !== true) throw new Error("Default release must retain JXL");
  for (const codec of ["jxl", "webp"]) {
    const path = join(job, `output/encoded.${codec}`);
    const data = readFileSync(path);
    if (!data.length || statSync(path).size > 64 * 1024 * 1024)
      throw new Error("Invalid encoded output size");
    const valid =
      codec === "jxl"
        ? data.subarray(0, 2).equals(Buffer.from([255, 10]))
        : data.toString("ascii", 0, 4) === "RIFF" &&
          data.toString("ascii", 8, 12) === "WEBP" &&
          data.readUInt32LE(4) === data.length - 8;
    if (!valid || report[`${codec}_sha256`] !== sha256(path))
      throw new Error(`Invalid ${codec} output`);
    cases.push({ codec, bytes: data.length, sha256: sha256(path) });
  }
  return cases;
}
export function verifyArtifact(
  target: BuildTarget,
  filename = artifactPath(target),
) {
  const artifact = resolve(root, filename),
    evidence = join(root, "temp/verification", target);
  mkdirSync(evidence, { recursive: true });
  const directory = mkdtempSync(join(evidence, "run-")),
    tag = `konobangu-artifact-verification:${randomUUID()}`;
  const receipt = {
    artifact,
    sha256: sha256(artifact),
    cases: [] as object[],
    target,
    dependencies: inspectDependencies(artifact, target),
    passed: false,
  };
  let imageCreated = false;
  {
    using cleanup = new DisposableStack();
    cleanup.defer(() =>
      rmSync(join(directory, "image"), { recursive: true, force: true }),
    );
    cleanup.defer(() => {
      if (imageCreated) cleanupDocker(["image", "rm", tag]);
    });
    cleanup.defer(() =>
      writeFileSync(
        join(directory, "receipt.json"),
        `${JSON.stringify(receipt, null, 2)}\n`,
      ),
    );
    const context = join(directory, "image");
    mkdirSync(context);
    const copiedArtifact = join(
      context,
      process.platform === "win32" && !containerTarget(target)
        ? "recorder.exe"
        : "recorder",
    );
    cpSync(artifact, copiedArtifact);
    chmodSync(copiedArtifact, 0o755);
    if (sha256(copiedArtifact) !== receipt.sha256)
      throw new Error("Verification input differs from artifact");
    if (containerTarget(target))
      writeFileSync(
        join(context, "Dockerfile"),
        'FROM scratch\nCOPY --chmod=755 recorder /recorder\nENTRYPOINT ["/recorder"]\n',
      );
    if (containerTarget(target))
      run(
        "docker",
        [
          "build",
          "--platform=linux/amd64",
          "--network=none",
          "--tag",
          tag,
          context,
        ],
        { timeout: 180000 },
      );
    imageCreated = containerTarget(target);
    for (const validInput of [true, false]) {
      const job = join(directory, validInput ? "codecs" : "invalid-input");
      mkdirSync(job, { mode: 0o700 });
      if (!validInput) writeFileSync(join(job, "input.png"), "invalid image");
      const name = `konobangu-artifact-check-${randomUUID()}`;
      const started = performance.now();
      let child: ReturnType<typeof run>;
      {
        using childCleanup = new DisposableStack();
        // Cleanup remains best effort and cannot overwrite the child failure.
        if (containerTarget(target))
          childCleanup.defer(() => cleanupDocker(["rm", "--force", name]));
        child = !containerTarget(target)
          ? run(
              copiedArtifact,
              [
                "media-smoke",
                ...(!validInput ? ["--input", join(job, "input.png")] : []),
                "--output",
                join(job, "output"),
              ],
              { capture: true, check: false, timeout: 60000 },
            )
          : run(
              "docker",
              [
                "run",
                "--rm",
                "--platform=linux/amd64",
                "--name",
                name,
                "--network=none",
                "--read-only",
                "--cap-drop=ALL",
                "--security-opt=no-new-privileges",
                "--user",
                `${process.getuid?.() ?? 0}:${process.getgid?.() ?? 0}`,
                "--mount",
                `type=bind,src=${job},dst=/job`,
                tag,
                "media-smoke",
                ...(!validInput ? ["--input", "/job/input.png"] : []),
                "--output",
                "/job/output",
              ],
              { capture: true, check: false, timeout: 60000 },
            );
      }
      writeFileSync(join(job, "stdout.txt"), child.stdout);
      writeFileSync(join(job, "stderr.txt"), child.stderr);
      receipt.cases.push({
        validInput,
        exit_code: child.code,
        elapsed_seconds: (performance.now() - started) / 1000,
      });
      if (!validInput) {
        if (
          child.code === 0 ||
          existsSync(join(job, "output/encoded.webp")) ||
          existsSync(join(job, "output/encoded.jxl"))
        )
          throw new Error("Invalid input must fail without encoded output");
        continue;
      }
      if (child.code !== 0)
        throw new Error(`Production media smoke failed: ${child.stderr}`);
      receipt.cases.push(...verifyEncodedOutputs(job));
    }
    if (
      sha256(artifact) !== receipt.sha256 ||
      sha256(copiedArtifact) !== receipt.sha256
    )
      throw new Error("Artifact changed during verification");
    receipt.passed = true;
  }
  console.log(JSON.stringify(receipt, null, 2));
  console.log(`Evidence: ${directory}`);
}

function cleanupDocker(args: string[]) {
  try {
    run("docker", args, { capture: true, check: false, timeout: 60000 });
  } catch (error) {
    console.error(`Docker cleanup: ${error}`);
  }
}

import { run } from "../lib/process.mts";
import { verifyArtifact } from "./artifact-smoke.mts";
import { type BuildTarget, buildRelease, containerTarget } from "./build.mts";

function checkPlatformHost(target: BuildTarget) {
  const host = run("rustc", ["-vV"], { capture: true }).stdout.match(
    /^host: (.+)$/m,
  )?.[1];
  if (target !== "native" && !containerTarget(target) && target !== host)
    throw new Error(
      "Run platform verification on the target host; only the Docker target executes through a container adapter",
    );
}
export function platformCheck(
  target: BuildTarget = "native",
  filename?: string,
) {
  checkPlatformHost(target);
  // Only toolchain, codec execution and artifact shape differ per target; the
  // feature matrix is a compilation concern that `just test media` owns once.
  if (!filename) buildRelease(target);
  verifyArtifact(target, filename);
}

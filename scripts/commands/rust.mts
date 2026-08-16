import { join } from "node:path";
import { root, run } from "../lib/process.mts";
// Artifact paths and executable suffixes are platform adaptations, not test scheduling.
export function mediaSmoke(output: string) {
  run(
    join(
      process.env.CARGO_TARGET_DIR ?? join(root, "target"),
      "debug",
      process.platform === "win32" ? "recorder-cli.exe" : "recorder-cli",
    ),
    ["media-smoke", "--output", join(root, "temp/media-backends", output)],
  );
}
export function checkMediaConflict() {
  const conflict = run(
    "cargo",
    ["check", "-p", "recorder", "--locked", "--features", "media-par-chili"],
    { capture: true, check: false },
  );
  if (
    conflict.code === 0 ||
    !/not both|mutually exclusive/.test(conflict.stderr)
  )
    throw new Error(
      "Both media backends must fail with an explicit diagnostic",
    );
}

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
    ["media-smoke", "--output", join(root, "temp/media-profiles", output)],
  );
}

import { rmSync } from "node:fs";
import { dirname, join } from "node:path";
import { root } from "../lib/process.mts";
import { loadMetadata } from "../lib/release-metadata.mts";
import { cleanTypes } from "./types.mts";

export function cleanWorkspace(scope: string, base = root) {
  if (!["outputs", "dependencies", "all"].includes(scope))
    throw new Error("Use clean outputs, dependencies or all");
  const projects = new Set(
    loadMetadata(base).node_package.map((entry) => dirname(entry.manifest)),
  );
  if (scope !== "dependencies") {
    cleanTypes([], base);
    for (const project of projects)
      rmSync(join(base, project, "dist"), { recursive: true, force: true });
    for (const output of [".react-email", "out"])
      rmSync(join(base, "apps/email-playground", output), {
        recursive: true,
        force: true,
      });
  }
  if (scope !== "outputs")
    for (const project of projects)
      rmSync(join(base, project, "node_modules"), {
        recursive: true,
        force: true,
      });
}

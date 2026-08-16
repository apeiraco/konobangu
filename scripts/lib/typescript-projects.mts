import { readFileSync } from "node:fs";
import { dirname, relative, resolve } from "node:path";
import { root } from "./process.mts";
import { loadMetadata, projectPath } from "./release-metadata.mts";

// The reference graph defines membership; metadata owns its entry configuration.
export function typescriptProjects(base = root): string[] {
  const config = projectPath(
    base,
    loadMetadata(base).tooling.typescript.config,
  );
  const references: { path: string }[] = JSON.parse(
    readFileSync(config, "utf8"),
  ).references;
  return [
    ...new Set(
      references.map(({ path }) => {
        const target = resolve(dirname(config), path);
        const project = relative(
          base,
          path.endsWith(".json") ? dirname(target) : target,
        );
        projectPath(base, project);
        return project.split("\\").join("/");
      }),
    ),
  ];
}

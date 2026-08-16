import { existsSync, readdirSync, readFileSync, rmSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { root, run } from "../lib/process.mts";
import { typescriptProjects } from "../lib/typescript-projects.mts";

export function cleanTypes(args: string[] = [], base = root) {
  const projects = typescriptProjects(base);
  const requested = args
    .find((argument) => argument.startsWith("--project="))
    ?.slice("--project=".length);
  if (requested !== undefined && !projects.includes(requested)) {
    throw new Error(`Unknown reference project: ${requested}`);
  }
  const selectedProjects = requested ? [requested] : projects;

  // Keep cleanup bounded to declared outputs, including when invoked elsewhere.
  for (const project of selectedProjects) {
    rmSync(join(base, project, "dist-tsc"), { recursive: true, force: true });
  }

  if (args.includes("--legacy")) {
    const excluded = new Set([
      "node_modules",
      "dist",
      "dist-tsc",
      ".git",
      ".react-email",
    ]);
    function removeLegacy(directory: string) {
      for (const entry of readdirSync(directory, { withFileTypes: true })) {
        const path = join(directory, entry.name);
        if (entry.isDirectory() && !excluded.has(entry.name)) {
          removeLegacy(path);
        } else if (entry.isFile() && entry.name.endsWith(".d.ts.map")) {
          const declaration = path.slice(0, -4);
          const map = JSON.parse(readFileSync(path, "utf8"));
          // Only paired declarations with a map pointing to an adjacent TS input
          // are old compiler outputs. Handwritten declarations have no such map.
          const stem = declaration.slice(0, -5);
          if (
            map.version === 3 &&
            map.sources?.length === 1 &&
            [`${stem}.ts`, `${stem}.tsx`].includes(
              resolve(dirname(path), map.sourceRoot ?? "", map.sources[0]),
            ) &&
            existsSync(resolve(dirname(path), map.sources[0])) &&
            existsSync(declaration) &&
            readFileSync(declaration, "utf8").includes(
              `//# sourceMappingURL=${entry.name}`,
            )
          ) {
            rmSync(declaration);
            rmSync(path);
          }
        }
      }
    }
    for (const project of selectedProjects) {
      removeLegacy(join(base, project));
      for (const info of [
        "tsconfig.tsbuildinfo",
        "tsconfig.scripts.tsbuildinfo",
      ]) {
        rmSync(join(base, project, info), { force: true });
      }
    }
    for (const output of [
      "tsconfig.tsbuildinfo",
      "tsconfig.base.tsbuildinfo",
    ]) {
      rmSync(join(base, output), { force: true });
    }
  }
}

export function rebuildTypes() {
  cleanTypes();
  try {
    run("pnpm", ["exec", "tsc", "-b", "--force", "--stopBuildOnErrors"]);
  } catch (error) {
    // Failed rebuilds must not leave partial declarations advertised as current.
    cleanTypes();
    throw error;
  }
}

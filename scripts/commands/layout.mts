import { readdirSync } from "node:fs";
import { join, relative } from "node:path";
import { root } from "../lib/process.mts";
import { typescriptProjects } from "../lib/typescript-projects.mts";

const excluded = new Set(["node_modules", "dist", "dist-tsc", ".react-email"]);
const handwritten = new Set([
  "apps/webui/src/env.d.ts",
  "apps/webui/src/infra/graphql/default-options.d.ts",
]);
export function checkTypesLayout() {
  const unexpected: string[] = [];

  function check(directory: string) {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      const path = join(directory, entry.name);
      if (entry.isDirectory() && !excluded.has(entry.name)) {
        check(path);
      } else if (
        entry.isFile() &&
        /\.(d\.[cm]?ts(?:\.map)?|tsbuildinfo)$/.test(entry.name) &&
        !handwritten.has(relative(root, path).split("\\").join("/"))
      ) {
        unexpected.push(relative(root, path).split("\\").join("/"));
      }
    }
  }

  for (const project of typescriptProjects()) {
    check(join(root, project));
  }
  for (const entry of readdirSync(root)) {
    if (/\.(d\.[cm]?ts(?:\.map)?|tsbuildinfo)$/.test(entry)) {
      unexpected.push(entry);
    }
  }
  if (unexpected.length) {
    console.error(
      `Unexpected source-side compiler outputs:\n${unexpected.join("\n")}`,
    );
    throw new Error("Source-side compiler outputs detected");
  } else {
    console.log(
      "No source-side declaration, declaration map or build metadata outputs.",
    );
  }
}

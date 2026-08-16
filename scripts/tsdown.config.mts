import { readdirSync } from "node:fs";
import { join } from "node:path";
import { defineConfig } from "tsdown";
import { root } from "./lib/process.mts";

// Preserve every declared package subpath without exposing tsc's reference outputs.
function entries(directory: string, extension: string, prefix: string) {
  return Object.fromEntries(
    readdirSync(join(root, directory))
      .filter((file) => file.endsWith(extension))
      .map((file) => [
        `${prefix}${file.slice(0, -extension.length)}`,
        join(root, directory, file),
      ]),
  );
}
export default defineConfig([
  {
    cwd: join(root, "apps/recorder"),
    entry: entries("apps/recorder/bindings", ".ts", "bindings/"),
    outDir: join(root, "apps/recorder/dist"),
    tsconfig: join(root, "apps/recorder/tsconfig.json"),
    format: "esm",
    dts: { oxc: {} },
    clean: true,
    outExtensions: () => ({ js: ".js", dts: ".d.ts" }),
  },
  {
    cwd: join(root, "packages/email"),
    entry: {
      index: join(root, "packages/email/index.tsx"),
      ...entries("packages/email/templates", ".tsx", "templates/"),
    },
    outDir: join(root, "packages/email/dist"),
    tsconfig: join(root, "packages/email/tsconfig.json"),
    format: "esm",
    dts: { oxc: {} },
    clean: true,
    outExtensions: () => ({ js: ".js", dts: ".d.ts" }),
  },
]);

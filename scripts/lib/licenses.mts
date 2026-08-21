import {
  cpSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { join, resolve } from "node:path";
import { root, run } from "./process.mts";

export function collectLicenses(destination: string, target?: string) {
  // Only this owned output directory is replaced, never source or dependency files.
  if (resolve(destination) === root || !destination.endsWith("licenses"))
    throw new Error("Expected a release licenses directory");
  rmSync(destination, { recursive: true, force: true });
  mkdirSync(destination, { recursive: true });
  cpSync(join(root, "LICENSE"), join(destination, "KONOBANGU-LICENSE"));
  const config = join(root, "about.toml");
  const effectiveTarget =
    target ??
    run("rustc", ["-vV"], { capture: true }).stdout.match(/^host: (.+)$/m)?.[1];
  if (!effectiveTarget) throw new Error("Cannot determine release target");
  run(process.env.CARGO_ABOUT ?? "cargo-about", [
    "generate",
    "--fail",
    "--locked",
    "--manifest-path",
    "apps/recorder/Cargo.toml",
    "--config",
    config,
    "--target",
    effectiveTarget,
    "--output-file",
    join(destination, "THIRD-PARTY-NOTICES.md"),
    "scripts/templates/third-party.hbs",
  ]);
  // Native code carried inside -sys archives has additional notices beyond SPDX metadata.
  const metadata = JSON.parse(
    run(
      "cargo",
      [
        "metadata",
        "--locked",
        "--format-version=1",
        "--filter-platform",
        effectiveTarget,
      ],
      { capture: true },
    ).stdout,
  ) as { packages: { name: string; manifest_path: string }[] };
  const native: Record<string, string[]> = {
    "libwebp-sys": ["vendor/COPYING", "vendor/PATENTS"],
    "aws-lc-sys": ["LICENSE", "aws-lc/third_party/fiat/LICENSE"],
    ring: [
      "LICENSE",
      "LICENSE-BoringSSL",
      "LICENSE-other-bits",
      "src/polyfill/once_cell/LICENSE-APACHE",
      "src/polyfill/once_cell/LICENSE-MIT",
    ],
    "zstd-sys": ["LICENSE", "zstd/LICENSE"],
  };
  const append: string[] = [];
  for (const [name, files] of Object.entries(native)) {
    const pkg = metadata.packages.find((p) => p.name === name);
    if (!pkg) continue;
    for (const file of files) {
      const path = join(pkg.manifest_path, "..", file);
      append.push(`\n## ${name}: ${file}\n\n${readFileSync(path, "utf8")}\n`);
    }
  }
  if (effectiveTarget.endsWith("-musl")) {
    for (const file of [
      "MUSL-COPYRIGHT",
      "GCC-COPYING3",
      "GCC-RUNTIME-EXCEPTION",
    ]) {
      append.push(
        `\n## Linux static runtime: ${file}\n\n${readFileSync(join(root, "deploy/licenses/runtime", file), "utf8")}\n`,
      );
    }
  }
  const path = join(destination, "THIRD-PARTY-NOTICES.md");
  writeFileSync(path, readFileSync(path, "utf8") + append.join("\n"));
}

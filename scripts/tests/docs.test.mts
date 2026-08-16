import assert from "node:assert/strict";
import {
  mkdirSync,
  mkdtempDisposableSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { checkDocs } from "../commands/docs.mts";
import { root } from "../lib/process.mts";

test("projected documentation resolves links from its canonical source", () => {
  using temporary = mkdtempDisposableSync(join(root, "temp/docs-"));
  const base = temporary.path;
  for (const directory of ["docs/en", "docs/zh", "assets"]) {
    mkdirSync(join(base, directory), { recursive: true });
  }
  writeFileSync(join(base, "assets/icon.txt"), "asset");
  writeFileSync(
    join(base, "README.md"),
    "[Icon](assets/icon.txt)\n[English](README.md) | [中文](docs/zh/README.md)\n",
  );
  symlinkSync("../../README.md", join(base, "docs/en/README.md"), "file");
  const chinese = join(base, "docs/zh/README.md");
  writeFileSync(chinese, "[English](../en/README.md) | [中文](README.md)\n");
  assert.doesNotThrow(() => checkDocs(base));
  writeFileSync(
    join(base, "README.md"),
    "[Missing](assets/missing.txt)\n[English](README.md) | [中文](docs/zh/README.md)\n",
  );
  assert.throws(
    () => checkDocs(base),
    /README\.md: missing assets\/missing\.txt/,
  );
});

import { existsSync, readdirSync, readFileSync, realpathSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { root } from "../lib/process.mts";

// AGENTS.md documents docs/{lang}/**/00x-TITLE.md (e.g. the roadmap subdirectory),
// so discovery must recurse rather than list only the language directory's direct children.
function markdownFiles(directory: string, base: string): string[] {
  return readdirSync(join(base, directory), { withFileTypes: true }).flatMap(
    (entry) => {
      const relative = `${directory}/${entry.name}`;
      if (entry.isDirectory()) return markdownFiles(relative, base);
      return entry.name.endsWith(".md") ? [relative] : [];
    },
  );
}

export function checkDocs(base = root) {
  // Projections resolve links relative to their canonical source, which is checked only once.
  const files = [
    ...new Set(
      [
        "README.md",
        ...["en", "zh"].flatMap((lang) => markdownFiles(`docs/${lang}`, base)),
      ].map((file) => realpathSync(join(base, file))),
    ),
  ];
  const errors: string[] = [];
  for (const source of files) {
    const file = relative(base, source);
    const content = readFileSync(source, "utf8");
    if (!(content.includes("[English](") && content.includes("[中文](")))
      errors.push(`${file}: missing language links`);
    for (const match of content.matchAll(/\]\(([^)]+)\)/g)) {
      const target = match[1].split("#")[0];
      if (!target || /^[a-z]+:/i.test(target)) continue;
      const path = resolve(dirname(source), target);
      if (!existsSync(path)) errors.push(`${file}: missing ${target}`);
    }
  }
  if (errors.length) throw new Error(errors.join("\n"));
  console.log(`Checked ${files.length} documentation files and local links.`);
}

#!/usr/bin/env node
import { Command, Option } from "commander";
import { bindings } from "./commands/bindings.mts";
import {
  type BuildTarget,
  buildRelease,
  buildTargets,
} from "./commands/build.mts";
import { cleanWorkspace } from "./commands/clean.mts";
import { copyWebui, waitRecorder } from "./commands/development.mts";
import { checkDocs } from "./commands/docs.mts";
import { checkTypesLayout } from "./commands/layout.mts";
import { platformCheck } from "./commands/platform.mts";
import {
  checkReleaseVersion,
  prepareRelease,
  publishRelease,
  releasePlan,
  setReleaseVersion,
  tagRelease,
} from "./commands/release.mts";
import { mediaSmoke } from "./commands/rust.mts";
import { testSuite } from "./commands/test.mts";
import { cleanTypes, rebuildTypes } from "./commands/types.mts";
import { CommandError } from "./lib/process.mts";
import { loadMetadata } from "./lib/release-metadata.mts";

export function createCli() {
  const cli = new Command()
    .enablePositionalOptions()
    .name("dev-cli")
    .description(
      "Custom repository filesystem, platform and release utilities",
    );
  const types = cli
    .command("types")
    .description("Manage TypeScript project outputs");
  types
    .command("clean")
    .option("--legacy", "Remove only proven legacy compiler outputs")
    .option("--project <path>")
    .action((options) =>
      cleanTypes([
        ...(options.legacy ? ["--legacy"] : []),
        ...(options.project ? [`--project=${options.project}`] : []),
      ]),
    );
  types.command("check").action(checkTypesLayout);
  types
    .command("rebuild")
    .description("Clean all reference outputs and rebuild with tsc")
    .action(rebuildTypes);
  cli
    .command("clean [scope]")
    .description("Bounded cleanup: outputs (default), dependencies, all")
    .action((scope: string = "outputs") => cleanWorkspace(scope));
  const generated = cli.command("bindings");
  for (const mode of ["snapshot", "verify"] as const)
    generated
      .command(`${mode} <file>`)
      .action((file: string) => bindings(mode, file));
  cli.command("check-docs").action(() => checkDocs());
  cli
    .command("test [suite] [args...]")
    .description(
      "Run all, rust, webui, tooling, browser or media; forward runner filters",
    )
    .allowUnknownOption()
    .passThroughOptions()
    .action(testSuite);
  const media = cli.command("media");
  media.command("smoke <output>").action(mediaSmoke);
  const targets: readonly string[] = buildTargets;
  function targetOption() {
    return new Option("--target <target>", "Build target")
      .choices([...buildTargets])
      .default("native");
  }
  function buildTarget(value: string): BuildTarget {
    if (!targets.includes(value)) throw new Error("Unknown build target");
    return value as BuildTarget;
  }
  cli
    .command("platform-check")
    .addOption(targetOption())
    .option(
      "--artifact <path>",
      "Verify an existing artifact without rebuilding",
    )
    .action((options) =>
      platformCheck(buildTarget(options.target), options.artifact),
    );
  cli
    .command("build-release")
    .addOption(targetOption())
    .action((options) => {
      buildRelease(buildTarget(options.target));
    });
  cli.command("copy-webui").action(copyWebui);
  cli.command("wait-recorder").action(waitRecorder);
  const release = cli
    .command("release")
    .description("Unified version and release management");
  release
    .command("plan")
    .addOption(
      new Option("--format <format>")
        .choices(["json", "github-output"])
        .default("json"),
    )
    .action((options) => releasePlan(options.format));
  const version = release.command("version");
  version.command("check").action(checkReleaseVersion);
  version.command("set <version>").action(setReleaseVersion);
  release
    .command("metadata-sync")
    .action(() => setReleaseVersion(loadMetadata().project.version));
  release
    .command("prepare")
    .addOption(targetOption())
    .action((options) => prepareRelease(buildTarget(options.target)));
  release
    .command("publish <directory>")
    .option(
      "--execute",
      "Publish an already verified bundle to an existing remote tag",
    )
    .action((directory: string, options: { execute: boolean }) =>
      publishRelease(directory, options.execute),
    );
  release
    .command("tag")
    .option("--execute", "Create the local annotated tag")
    .action((options: { execute: boolean }) => tagRelease(options.execute));

  return cli;
}

if (import.meta.main) {
  try {
    await createCli().parseAsync();
  } catch (error) {
    console.error(error instanceof Error ? error.message : error);
    process.exitCode = error instanceof CommandError ? error.code : 1;
  }
}

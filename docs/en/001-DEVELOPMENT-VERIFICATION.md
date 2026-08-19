# 001 — Development, builds and verification

## Tools and entry points

Use mise at the repository root to manage Node, pnpm, Rust, just, CMake and other tools; `rust-toolchain.toml` pins Rust. Just loads `.env` and coordinates pnpm, Cargo, uv and the sole custom utility `scripts/dev-cli.mts`. Recipe comments own the command reference: use `mise exec -- just --list` and `mise exec -- just dev-cli --help`.

Install native prerequisites and managed tools, then frozen Node workspace dependencies, before invoking dev-cli. `just setup` installs managed tools, dependencies and hooks; CI shares `.github/actions/setup` for its prerequisites. Verification does not modify or upgrade workspace dependencies; browser fixtures use fixed supported LTS Node and browser versions.

| Platform | Native build prerequisites |
| --- | --- |
| Windows 11 / Server 2022+ x64 | Visual Studio C++ Build Tools, Windows SDK, CMake, NASM, PowerShell 7.5+; run in the MSVC developer environment |
| macOS arm64/x64 | Xcode Command Line Tools, CMake, NASM |
| Linux x64 | C compiler, CMake, NASM, pkg-config, OpenSSL development package |
| Docker | Linux host or Windows/macOS Docker Desktop; container targets use the pinned linux/amd64 image |

Use pnpm for Node, Cargo for Rust and uv workspace for Python. Configure development dependencies and ordinary business, identity and scheduler database connections through [authentication](002-AUTHENTICATION-DECISION.md). Tests own isolated resources rather than sharing development databases.

## Execution model

| Layer | Responsibility |
| --- | --- |
| Native entry points | `lint/check/test/build-*` invoke ecosystem tools; runner arguments select individual tests |
| Gates | `verify` resource stages compose native entries; the checklist is defined only in `justfiles/test.just` |
| Platform artifacts | `platform-check` owns feature matrices, target builds, dependency checks and actual artifact smoke |
| Release | `release prepare` runs the complete gate and platform verification before bundling and recording receipts |

`verify static` checks metadata, tooling behavior, documentation links, formatting and diffs. `verify rust` uses Cargo/Docker for compilation, complete workspace tests and browser integration. `verify ts` checks declarations, bundler outputs, frontend unit tests, lint and output boundaries. `verify` runs static → rust → ts so Rust bindings and GraphQL generators precede TypeScript consumers. CI invokes the same stages in separate checkouts; the Rust stage rejects binding drift. Browser fixtures build OIDC into isolated directories without changing the release WebUI's authentication configuration.

Stages follow resources and must not overlap. New tests belong in existing Cargo, Vitest, Node or Playwright runners, without new recipes, CI jobs or gate steps. Only new toolchains, feature combinations or external dependencies justify additional steps; merge overlapping coverage. Do not write tests mirroring recipe literals, job counts or step order; test argument forwarding, exit codes, cleanup and artifact behavior.

`test rust` defaults to workspace libraries, binaries, integrations and playground examples, then runs doctests excluded by Cargo's all-targets mode. Explicit Cargo arguments replace the default selection and can target packages, features, test targets or name filters. Vitest/Node accept native filters. `test browser` uses real oidc-provider and Chromium to check authentication, polyfills and lifecycle behavior. Runner failures stop verification and preserve the original status; Just owns the default Rust sequence. Missing Docker or build prerequisites fail directly.

The qBittorrent lifecycle fixture pins a multi-platform image tag/digest in `packages/downloader/tests/integration/qbit-lifecycle.test.rs`. Upgrade that reference deliberately and run the downloader suite on Docker Desktop and native Linux. It covers authentication, repeated addition, sync, pause/resume, actual transfer, keep/delete files, timeout and shutdown. WebAPI 2.14 can report duplicate additions as HTTP 409; the adapter queries the server and accepts the conflict only when every hash in that submission exists. Mock tests cover file/magnet conflicts, stale local cache, partially missing batches and other API failures. [Upstream API changes](https://github.com/qbittorrent/qBittorrent/blob/master/WebAPI_Changelog.md#2140)

`lint rust` checks the workspace and every valid recorder backend/codec configuration with all-targets and -D warnings. Do not use --all-features because Rayon/Chili are mutually exclusive. `test media` checks codecs, lifecycle, HTTP and artifact smoke across compilation configurations. Native platform jobs own the host matrix; container jobs build and check target artifacts without repeating that matrix. Windows/macOS/Linux and Docker all require actual build and runtime evidence.

Lint exceptions are scoped to specific OIDC handlers and shared fixtures; other effective rustc/Clippy and Cargo manifest warnings fail. Inspect raw proc-macro/linker logs because -D warnings does not cover every diagnostic source. Preserve test IdP memory-adapter/TTL messages, Docker and license-tool diagnostics instead of filtering logs to claim zero warnings. Frontend bundle budgets and initial/polyfill/editor loading are checked through configuration and browser assertions.

### GitHub workflows

The default branch is `master`. Verification runs for pull requests targeting `master` or `release`, and pushes to those branches. A push to `dev` alone does not run verification; opening or updating its pull request to either target does. PR branch filters select the destination branch, as described in [GitHub's event reference](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#pull_request).

`Verification report` is the single required status check, aggregating the static, TypeScript, Rust and platform jobs. Failed, cancelled or skipped stages cannot pass it. New PR/mainline pushes supersede obsolete verification runs; release pushes retain their runs so publication can bind to an exact candidate.

Documentation deploys from `master` when its inputs change, with manual build support; deployment is restricted to `master` through `github-pages`. The testing-torrents publisher consumes the committed metadata opt-in flag and a pinned source, supports workflow_call and manual invocation on release, and exports its digest; it publishes a fixture image, not the application. Application releases currently use the local release tools below; automatic publication is specified in the [roadmap](roadmap/001-SHORT-TERM-ROADMAP.md#release-automation).

`aitiotekt-infra/meta/github-repositories.yml` owns the declarative settings for `apeiraco/konobangu`: read-only default Actions permissions, release branch protection requiring `Verification report`, the `stable-release` approval environment, and `github-pages` allowing `master` with `konobangu.apeiraco.com`. Stable approval allows only the `release` branch, allows the single maintainer's self-review and disables administrator bypass. Version tags are outputs of the dispatched release workflow, not deployment sources. Maintain/Admin direct pushes bypass the PR/check rules; separate safety rules still prohibit force-push and deletion. Workflows must validate the candidate even after a direct push. Apply infrastructure settings separately through its inspect/pull/plan/apply lifecycle; the manifest does not create branches or workflow files. Release versions stay in project metadata, not infrastructure environment variables.

## Ownership and organization

The root `justfile` owns global settings, argv adaptation and imports. `justfiles/` separates setup/dev/build/quality/test/release/tools. Just expresses native commands and fixed sequences, using its own if expressions for fixed scopes; complex operations belong in the typechecked and linted CLI. Pure forwarding uses `recipe_args`: native positional argv on Unix, PowerShell 7.5+ CommandWithArgs and argv arrays on Windows, preserving LASTEXITCODE explicitly. cross-spawn handles Windows command shims without reconstructing arguments as shell strings.

The CLI owns cohesive filesystem, network, metadata and artifact operations, including cleanup, TCP readiness, argument adaptation, Docker mounts and release transactions. Just continues to coordinate fixed gates and matrices. Avoid generic quality wrappers, public intermediate commands and manual verified switches. Keep pnpm scripts only for necessary ecosystem lifecycles; removing nonredundant commands requires working replacements.

`just dev-all` and `just dev-proxy` preserve Zellij pane/session controls; process fan-out cannot replace window management. Development codegen supports independent watch and backend-readiness watch. Shared tooling handles platform differences without Bash/GNU utilities for maintained orchestration. Do not add standalone one-off sh/py/mjs entries under root scripts; experiments belong in temp.

## TypeScript outputs and application lifecycle

| Directory | Purpose |
| --- | --- |
| `dist-tsc/{{project}}/` | tsc reference declarations and declaration maps, never package entry points |
| `dist/` | Vite applications and tsdown JS/public declarations |
| `dist-tsc/{{project}}.tsbuildinfo` | Incremental metadata |

recorder bindings retain ts-rs generator formatting and are excluded from Biome lint, formatting and import organization. File-set/content hashes check generation stability; tsc/tsdown still validate types and declarations. Do not format bindings after Rust tests or exports.

Metadata identifies the root TypeScript configuration; its references graph defines project membership, shared by cleanup/layout checks. The monorepo-tsc condition resolves workspace source directly for the editor, Vite, Vitest and React Email. External consumers read bundler outputs. Public exports provide explicit annotations for declaration generation.

Incremental builds can retain orphan outputs after source deletion. Release verification cleans declarations and metadata before a forced rebuild, cleaning partial outputs again on failure. Daily `check ts` is incremental; the complete gate does not repeat an empty incremental build after rebuilding. Cleanup removes declared directories only. Legacy cleanup requires map/source evidence and preserves handwritten declarations, secrets, data and receipts; never use git clean -xdf.

Business time uses Temporal/Intl. Bootstrap concurrently detects and loads missing capabilities before application modules; before that barrier it cannot implement Disposable or access resource primitives. Post-barrier owners use using/await using and standard disposable symbols; resource stacks compose cleanup, transferring long-lived ownership with move. Main owns manual HMR/pagehide; entry creates and returns the resource stack. BFCache persisted pages retain the application. Cancelled initialization never mounts; cleanup is LIFO and releases remaining resources even if one throws.

Node configurations, tests and utilities use native `import.meta.dirname` / `import.meta.filename` for local paths, without building and decoding file URLs. Browser application modules do not depend on these Node properties. [Node ESM](https://nodejs.org/api/esm.html#importmetadirname)

The shared tsdown configuration builds email package JS/declarations. The email playground uses Vite for development, static preview builds and HTML/plain-text exports. React Email components and its renderer retain email-specific markup and CSS inlining; the React Email CLI/Next preview application is not part of the toolchain. `dev-email` serves the template index on loopback:5003 with reloads; `build-email` writes the static preview to dist, and `build-email export` writes to out. Preview modules can attach PreviewProps to their default component. [React Email render](https://react.email/docs/utilities/render)

Node executable utilities and fixtures are erasable TypeScript, run directly by managed Node. Modules shared with tests export functions and guard executable side effects with import.meta.main.

### Test organization

Within each apps/packages project, use directories to identify the test layer:

| Layer | Location | Discovery |
| --- | --- | --- |
| JS/Node/TS/Python unit | Source-local `**/__test__/**` or project-root `tests/**`, excluding integration/e2e | `*.test.*` or `*.spec.*`, with the language's actual extension |
| Rust unit | Inline `#[cfg(test)]` modules or separate test modules | Native Cargo library/binary test discovery |
| Integration | `tests/integration/**` | `*.test.{ts,rs,py}` or `*.spec.{ts,rs,py}` and applicable JS/TS extensions |
| End-to-end | `tests/e2e/**` | The same filename convention, with complete product journeys |

JS/TS extensions include js/ts, jsx/tsx and the c/m module variants. Use glob braces rather than regex alternation for runner patterns. The directory declares the layer; using a browser alone does not make a test end-to-end. Unit fixtures can use controlled temporary files or subprocesses. Integration tests combine production components or external services; end-to-end tests exercise complete user journeys. Helpers, fixtures and snapshots stay alongside their owning layer and are not executable test entries. Shared fixtures may serve multiple suites.

Repository utility modules follow the same layout under scripts: unit contracts live in scripts/tests, real Just/CLI composition lives in scripts/tests/integration. The existing tooling runner collects both recursively. Cargo explicitly registers nested integration entries with stable target names; helper modules and e2e support binaries are not discovered as test targets. Vitest collects source-local __test__ and root tests while excluding integration/e2e. Playwright has only integration/e2e projects, each collecting its corresponding directory without overlap; feature directories such as auth do not create separate projects. The auth suite imports its automatic lifecycle fixture and declares in-file execution in order, sharing that worker's backend. Other suites remain parallel. Listing tests does not acquire services. Native filters continue to select cases through the existing runners, without additional recipes or CI gate steps.

The Python package has no tests yet. Its pytest configuration recognizes *.test.py/*.spec.py and uses importlib mode, allowing dotted filenames and duplicate basenames in different suites; no empty test gate or new dependency is introduced. [pytest discovery](https://docs.pytest.org/en/stable/example/pythoncollection.html#changing-naming-conventions), [pytest import modes](https://docs.pytest.org/en/stable/explanation/pythonpath.html#import-modes)

Before the first release after this refactor, edit existing migrations directly. Recreate isolated test databases when definitions change; intermediate implementations do not need compatibility migrations.

## Platform artifacts and release

`release.artifacts` selects bundles, the runtime image and testing-torrents independently. Use `just release plan` to inspect the source/config hash and selection; `--format github-output` exports the same data to CI. An empty selection runs verification without publication. These switches do not change test coverage or grant permissions. Bundle publication checks the recorded metadata hash/selection; image publication and final manifests follow the [release automation contract](roadmap/001-SHORT-TERM-ROADMAP.md#metadata-selection-and-publication-history).

JPXL, static WebP and aws-lc retain their actual native prerequisites. Codec source stays upstream with pinned Git commits, without source trees in this repository; do not set target-cpu=native. Windows uses static CRT; musl artifacts have no ELF interpreter/NEEDED; GNU records its glibc ABI boundary; macOS permits system dylibs/frameworks only. Targets share build-release/platform-check, and existing artifacts can be checked without rebuilding. See [015](004-MEDIA-AND-CONFIGURATION.md) for media and parallel execution semantics.

`konobangu-metadata.toml` owns versions, package inventory and release paths. Version updates validate all manifests before synchronizing Rust/Node/Python and lockfiles, restoring files on failure without committing, pushing or tagging. This project is an application: Node packages are private and Rust packages use publish=false.

```text
mise exec -- just release plan
mise exec -- just release version set 0.2.0
mise exec -- just release prepare --target native
# Commit the final tree and versioned changelog before creating the tag.
mise exec -- just release tag --execute
# Push the committed branch/tag yourself, then publish the verified bundle.
mise exec -- just release publish temp/release/0.2.0/PLATFORM --execute
```

tag/publish default to dry runs. Tag execution requires a clean tree and exact versioned changelog. Publish uploads only a committed, verified bundle to an existing remote tag with system tar and GitHub CLI; the maintainer controls pushing and remote tags. Prepare bundles the executable, WebUI, notices and revision/dirty/hash receipt only after all checks pass. Configuration, data and WebUI remain runtime inputs; the single-executable goal covers codecs/TLS without extra shared libraries.

cargo-about generates THIRD-PARTY-NOTICES from locked dependencies, supplemented by native attributions; failures stop packaging. Do not track versioned dependency license copies. musl runtime supplements accompany that target only. Evidence under temp/verification records platforms, source/dependency commits, commands and exit codes; old receipts do not validate new source. Actual execution evidence is separate from CI configuration. Unfinished-response progressive rendering and model evaluation remain on the [roadmap](roadmap/001-SHORT-TERM-ROADMAP.md).

## Documentation and contributor rules

### Documentation site

`apps/docs` uses stable VitePress and Vue. English convention documents (`README.md`, `CHANGELOG.md`, and CONTRIBUTING/SECURITY/PRIVACY when present) have suffixless repository-root sources; `docs/en` exposes relative file symlinks to them. Other languages keep regular source files with the same names under `docs/{lang}`. Numbered topic documents remain in their language directories. The site's `en` and `zh` directory symlinks project those directories; VitePress maps each README to the locale homepage and each CHANGELOG to its changelog route. Edit the canonical source, not a separate homepage. GitHub edit links resolve to the actual source, and `public/assets` projects repository assets.

Modern operating systems with working symlinks are required for development. On Windows, enable Developer Mode (or grant symlink creation privileges) and clone with `git -c core.symlinks=true clone ...`. Existing checkouts must enable `core.symlinks` and restore the tracked links before building. Do not replace them with copies or junctions. `check docs` deduplicates projections and resolves relative links from the canonical source; the site build also checks projection targets and rendered routes.

Just provides `dev-docs` (loopback:5004), `build-docs` (strict link validation, output in `apps/docs/dist`) and `docs-preview`. `verify ts` includes the site build. VitePress configuration participates in the root TypeScript reference graph; declarations go to `apps/docs/dist-tsc`, and temporary bundler data goes to `apps/docs/.cache`. English routes start at `/`, Chinese routes at `/zh/`; repository-relative links and configuration examples remain usable in both GitHub Markdown and the site.

The Documentation workflow builds and deploys relevant changes on `master`, or can be dispatched manually from `master`. In repository Settings → Pages, select **GitHub Actions**, set the custom domain to `konobangu.apeiraco.com`, and enable HTTPS after DNS verification. The subdomain's DNS CNAME points to `apeiraco.github.io`, without a repository path. `apps/docs/public/CNAME`, sitemap and canonical URLs use the same domain. Pull requests validate through the existing TypeScript verification stage; they do not deploy.

### Contributor rules

AGENTS.md owns core requirements and topic links. Current contracts belong in docs, future plans in the roadmap, history in CHANGELOG and necessary handoffs/receipts in temp. English/Chinese user-facing docs keep bidirectional language links and prefer same-language topics. YAML uses two-space indentation; code/comments use English. Follow AGENTS.md for GraphQL generated fields and skip keys, rerunning codegen and checking generated input types. See [015](004-MEDIA-AND-CONFIGURATION.md) for configuration naming, units and interpolation, and [014](003-TASK-DELIVERY-AND-MIGRATION.md) for migration/task delivery contracts.

[English](001-DEVELOPMENT-VERIFICATION.md) | [中文](../zh/001-DEVELOPMENT-VERIFICATION.md)

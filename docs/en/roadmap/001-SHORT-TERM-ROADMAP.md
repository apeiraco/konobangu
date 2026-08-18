# 001 — Short-term roadmap

This page owns future decisions and acceptance conditions. Current contracts live in [authentication](../002-AUTHENTICATION-DECISION.md), [tasks and upgrades](../003-TASK-DELIVERY-AND-MIGRATION.md), and [media/configuration](../004-MEDIA-AND-CONFIGURATION.md). Delivered changes belong in [CHANGELOG](../../../CHANGELOG.md). Iteration guides and receipts in `temp/` are working material, not deployment manuals.

## Publication gate

The local minor release candidate has passed independent review, including closure of the clean-checkout CLI dependency bootstrap issue. Automatic release publication remains an implementation gap; complete the delivery described below before claiming an automated release pipeline. Maintainers can use the existing local versioning, tagging and publication tools. Follow the [development verification contract](../001-DEVELOPMENT-VERIFICATION.md) and revalidate affected areas if production code or build dependencies change. Current architecture and operational details remain in the topic contracts.

The subsequent HMR review found a new development gate gap: vite-plugin-monaco-editor 1.1.0 uses recursive rmdirSync, removed in Node 26, preventing the full development server from starting. Before publication, repair the Monaco worker integration and verify the complete Vite configuration, component Fast Refresh and application root updates on the mise-pinned Node version. Passing the isolated HMR fixture does not close this item. The local review record is `temp/CONFIGURATION_BINARIES_RESOURCES_zh.md`.

## Remaining work

| Item | Direction | Acceptance |
| --- | --- | --- |
| R6 Model tooling | Exercise actual Python/Animeta model loading, inference, datasets and resources; see the [Animeta proposal](002-ANIMETA-MODEL.md) | Lockfile updates do not establish working inference; require reproducible samples, quality and resource budgets |
| R7 Awa replacement | Awa is the chosen candidate; wait for its runtime and official SeaORM adapter to support SQLx 0.9 and the project ORM together | A successful experiment must lead to an independently accepted production migration |
| R8 Progressive JXL | Retain static JXL defaults; demonstrate recognizable previews and subsequent refinements while the response is unfinished | Capture the actual production HTTP/page path; complete-file decoding is insufficient |
| Session storage | Adopt a mature SQLx 0.9 session store when available | Preserve durable revocation, restart/multi-instance behavior, cleanup and total connection budget |
| Stable Rust | Replace the pinned nightly once upstream quirks_path no longer requires it | Pass default, all valid feature combinations and trimmed branches on Windows/macOS/Linux |

## Release automation

Use Android-Credential-Provider-Fixer's tested-source dispatch model, shared with SecurityDept; Outposts' image-build/CD model does not cover this application's portable release bundles. Implement this as one delivery, without new feature-domain verification groups.

### Metadata selection and publication history

```toml
[release.artifacts]
bundles = true
runtime_image = true
testing_torrents_image = false
```

These independent booleans live in `konobangu-metadata.toml`; missing, mistyped or unknown fields are rejected. They select publication, not test coverage. All false means verification only, without dispatch, approval, a tag or a GitHub Release. Image-only releases still produce the version tag and a GitHub Release with the manifest/digests; bundle archives are not required. Do not duplicate these flags in IaC, repository variables or dispatch inputs, or expose flags that override committed intent.

`just release plan` reports the selection, source SHA, metadata SHA-256 (with checkout newlines normalized) and candidate image tags. `--format github-output` exports the same plan for job conditions. Local previews report a dirty checkout; automatic publication must resolve the plan from the exact clean, tested source. Read metadata from that SHA on retry, rather than from the latest branch tip. Default bundles and runtime images are selected; testing-torrents is opt-in. Runtime image publication and the complete release orchestrator still require the implementation below.

Git history proves the intended selection, not successful publication. Persist the metadata snapshot/hash and source SHA in the release manifest, together with actual successful archive hashes, image digests and workflow run IDs. Bundle receipts already carry the selection/hash, and local publication rejects changed metadata. The fixture publisher exports its digest and source/config hashes; its manual entry also requires the release ref and the committed opt-in flag. Candidate image tags exclude `latest`: promote that alias only after every selected output succeeds and the stable release is finalized. Reject retries that change the selection or source under an already published version; publish a new version instead of rewriting its history.

### Release contents

The candidate version is `0.1.0`, matching `konobangu-metadata.toml` and both changelogs; its tag will be `v0.1.0`. A versioned changelog does not establish that publication has happened. Maintain one application release, not a registry release for every workspace member.

| Deliverable | Release decision |
| --- | --- |
| Application bundles | Four target-named archives: `x86_64-unknown-linux-gnu`, `x86_64-unknown-linux-musl`, `x86_64-pc-windows-msvc`, `aarch64-apple-darwin`. Each includes `recorder-cli`/`.exe`, `webui/`, LICENSE, CHANGELOG, notices and a source/hash receipt. Add a secret-free configuration example in automated bundling; never package user data or credentials. |
| Build profile | Ship the default JXL + WebP, Rayon profile. Chili, serial and WebP-only variants are verification/build options, not separate release products. |
| Main runtime image | Publish public `ghcr.io/apeiraco/konobangu` for `linux/amd64`, using the already-verified musl executable and the same WebUI. Package it once in a minimal, non-root runtime with CA certificates; no Cargo, Node or codec build tools. Mount configuration, secrets and writable application data separately. PostgreSQL, the IdP and torrent clients remain external services. |
| Image tags and metadata | Publish the full version and `sha-<source SHA>`; move `latest` only after a stable release completes. Record the image digest in the release manifest. Include OCI source/revision/version/license labels, associate the package with the repository and explicitly enable public visibility on initial publication. Add Linux arm64 only after native artifact validation is available. |
| Workspace libraries | Keep Node packages private and Rust crates unpublished. `email`, `testing`, recorder bindings, `util`, `util-derive`, `fetch`, `downloader` and Rust `animeta` are internal build dependencies; distribute the application rather than npm/crates.io packages. Python `konobangu-animeta` and model demos are development/research tooling, with no PyPI/model release in this delivery. |
| Development binaries | Do not ship `mikan-doppel` (mock source), `auth-test-server`, `testcontainers-prune`, examples or `animeta-agent-cli` (currently empty). Email playground previews/exports are development outputs. |
| Test image | Select `ghcr.io/apeiraco/konobangu-testing-torrents` with `release.artifacts.testing_torrents_image`. The release orchestrator calls its reusable publisher only when selected; it is false by default and remains distinct from the runtime image. |
| Build images and docs | Keep the musl builder local to CI, without a user-facing image release. Publish docs through Pages from `master`; do not put the documentation site or compiler caches/declarations in application bundles. |

The main runtime image joins the portable bundles in this delivery because Docker is a primary self-hosting path. Its Dockerfile and automated publication are still to be implemented; the existing musl Dockerfile builds the executable and is not a production runtime image.

1. After `Verification report` succeeds for a push to `release`, resolve the committed metadata selection and dispatch `release.yaml` with the exact `source_sha`, `source_ref` and `tests_run_id` only when at least one artifact is selected. Add `actions: write` only to the dispatch job. Support manual retry of those same inputs; never auto-publish from PRs, `dev`, the default branch or an unrelated tag push.
2. Validate the source repository, verification workflow identity, completed successful run, push event, release branch and matching commit through the GitHub API. Require synchronized metadata and an exact versioned changelog. Checkout the tested SHA, not the mutable branch tip. Deploy the workflow on the default branch before depending on dispatch.
3. Upload the checked recorder executable, notices and target receipt from each existing platform job, and the production WebUI from the existing TypeScript job. Current CI retains only smoke receipts; these are insufficient for publication. Store the source SHA, version, target and hashes with each artifact. Consume artifacts from that validated run, without repeating builds or verification. Refactor local and CI bundling to share source/artifact validation; do not expose a manual `--verified` switch.
4. When `bundles` is selected, assemble target-named archives for Linux GNU/musl x64, Windows MSVC x64 and macOS arm64, including WebUI, LICENSE, CHANGELOG and third-party notices. Check candidate provenance and file hashes before generating a release manifest and SHA256SUMS.
5. Gate stable publication through `stable-release` on the `release` ref; infrastructure deployment policies allow that branch only, with no tag sources. Determine prerelease status from SemVer metadata. Give `contents: write` only to tagging/publication jobs and `packages: write` only to selected image publication jobs. Use `GITHUB_TOKEN`, with no npm/crates/PyPI publishing or signing secrets. Retain read-only repository defaults. Package and smoke-test the verified runtime image, publish its version/source tags without replacing a different existing candidate, and include its digest in the release manifest. Move `latest` after stable publication succeeds. [GitHub registry authentication](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry#authenticating-to-the-container-registry)
6. Create a missing version tag on the tested SHA, and reject a tag pointing elsewhere. Publish a draft, verify its downloaded artifacts, then expose the release. A retry must accept an identical published release and reject different source/artifacts; it must not overwrite a public release. Record the tested run ID and source SHA in the release manifest.
7. Accept a real release-branch push through verification, approval, downloadable bundles and a public pullable runtime image, plus failure/retry and tag-collision cases. Verify image startup, WebUI and API routing, secret-file configuration and persistent data mounts; codec smoke alone is insufficient. Apply the infrastructure manifest through its documented import/plan/apply process; configuring protection is not evidence of a successful publication. Exercise bundle-only, runtime-image-only, fixture-image-only, combined and empty selections; do not require unselected outputs. Keep the fixture publisher reusable under the same release orchestrator.

## R7 decisions

The Awa experiment preserves the accepted Apalis contracts. Verify published compatibility and licensing before starting; do not integrate speculative future versions.

1. Use public transaction APIs to prove enqueue/rollback, Cron occurrences and business publication in the same real SeaORM/SQLx transaction. Another connection or private storage table is not an acceptable substitute.
2. Preserve app-scoped, auth and task-control capabilities and the owner-only migration phase. Cover rollback after later mutation roots, cross-owner denial, and retry/cancel/delete transactions.
3. Exercise real workers with duplicate delivery, renewal, executor loss, cancellation recovery, attempt budgets, late results, resource deletion, restart and multi-instance Cron. Preserve timezone/DST, downtime coalescing and no-overlap behavior.
4. Identify lease/retry/recovery code that can be deleted and business fences, history and recovery that must remain. Reject the experiment if it needs a duplicate full executor or weakens correctness.
5. After success, complete runtime/lifecycle/roles, APIs/pages, codegen, CI, documentation and old-code removal in the same delivery. Migrate nonempty databases through stopped-write/drain/import and rehearsed backup restoration, retaining IDs, owners, history and terminal states. Never reset data or consume both queues concurrently.

## R8 decisions

Honor client Accept/q/exclusions and prefer eligible progressive JXL → ordinary JXL → WebP → original. Features take priority over small size differences; retain one fixed encode per profile without size-driven retries or downgrade. Do not enable automatic AVIF. Preserve originals, authorization and publication fences. Static defaults do not depend on the progressive gate, and static acceptance does not close R8.

Controlled native Linux or SSH verification is valid evidence. CI configuration and actual CI execution are recorded separately. Organize future work as complete delivery themes rather than numerous small experiment/implementation/cleanup rounds.

[English](001-SHORT-TERM-ROADMAP.md) | [中文](../../zh/roadmap/001-SHORT-TERM-ROADMAP.md)

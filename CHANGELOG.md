# Changelog

## Unreleased

### Added

- Persistent OIDC/PKCE sessions, isolated database roles and owner-scoped GraphQL transactions.
- Durable task runs/outbox, fenced execution, cancellation recovery and timezone-aware Cron scheduling.
- Native configuration interpolation and file secrets with redacted diagnostics.
- Versioned image derivatives, bounded media execution and provenance-aware backfill.
- Cross-platform development/release tooling and application, migration and platform verification.
- README-backed bilingual VitePress documentation with local search and GitHub Pages deployment at konobangu.apeiraco.com.

### Changed

- Upgrade dependencies; adopt injection-js, SecurityDept RxSignal, Apollo and TanStack Table 9.
- Name database privileges by application scope, authentication and task control; bootstrap NOLOGIN capabilities from one database URL, derive grants from Rust migration identifiers and consolidate unreleased migrations.
- Use RFC3339 timestamps and Temporal, loading missing browser capabilities before app startup.
- Encode covers with pinned JPXL Balanced 77 and static WebP80; share a par-core executor with Rayon, Chili or serial builds. Preserve originals and composite cover transparency on white.
- Preserve generated binding formatting and use native Node ESM path metadata in tooling.
- Use Vite email previews/exports, Playwright-owned fixtures and unit/integration/e2e test directories; share task migration identifiers in Rust.
- Share resource-based verification stages between Just and CI; isolate browser fixture builds and use one TypeScript reference graph for output management.
- Consolidate CLI/Just tooling and generated output; restore Zellij sessions, use kebab-case executables, and group configuration by service with explicit units.

### Fixed

- Preserve authorization, cache validators and Range behavior when negotiating image formats.
- Keep task publication, cancellation, retries and deletion behind transaction and lease fences.
- Install locked CLI dependencies before invoking tools in clean CI checkouts; apply file logging thresholds.

### Removed

- Browser token storage and Bearer/token endpoints, Tera, Jotai/DI forks and unused date adapters.
- Codec subprocesses, libjxl build machinery and tracked compiler declaration artifacts.

### Upgrade notes

Back up the database and storage, stop writers/workers, and apply owner-only migrations. Database rollback requires restoring the backup. Default startup initializes capability-restricted pools from one database URL; optional separate runtime/owner credentials remain available; replace configuration templates, flat auth/task/media fields and legacy JXL options. Automatic covers remain JXL + WebP; JPXL currently produces ordinary single-pass images.

See [authentication](docs/en/002-AUTHENTICATION-DECISION.md), [upgrades](docs/en/003-TASK-DELIVERY-AND-MIGRATION.md), and [media/configuration](docs/en/004-MEDIA-AND-CONFIGURATION.md).

[English](CHANGELOG.md) | [中文](docs/zh/CHANGELOG.md)

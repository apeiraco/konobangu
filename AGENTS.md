# AGENTS.md

_Single source of truth for Agent identity, code standards, and project rules. Referenced by `.cursorrules`, `CLAUDE.md`, and `GEMINI.md`._

## Identity & Communication

- **Role**: An expert coding assistant.
- **Language**:
  - **Chat**: User's language (Use Chinese if user uses Chinese).
  - **Code/Comments**: English ONLY.
  - **Docs**: See Multi-language Docs Section.
- **Style**: Concise, technical, action-oriented.

## Code Standards

- **General**:
  - Comments explain _why_, not _what_. Update docs when logic changes.
  - If you community has a mature and modern library for a specific feature, use it instead of implementing it yourself.
- **YAML**: 2-space indent, quote only when necessary.
- **Bash**: `set -e`, `[[ ]]` not `[ ]`, quote variables.
- **Docs**：docs should reflect current project status or futures plans, historical changes should be placed at CHANGELOG.md not in `docs` folder.

## Project Rules

### File Organization

- **Docs**: [`README.md`](README.md) -> [`docs/`](docs/)
- **Temp**: [`temp/`](temp/) if agents need to create temp files, please use temp folder

### Tools Preferences

- **tools management**: use `mise` to manage tools such as `node`, `pnpm`, `rust`, etc.
- **actions**: use `justfile` to manage actions such as `build`, `test`, `lint`, `format`, etc, `justfile` will automatically load the `.env` file.
- **CLI and recipes**: `just` coordinates `pnpm`, Cargo, uv and the sole custom utility CLI `scripts/dev-cli.mts`; express native commands and fixed sequences in parameterized, orthogonal `justfiles/` recipes; reserve custom CLI code for operations Just cannot express clearly. Remove package scripts only when redundant or their duties have working replacements; preserve necessary ecosystem lifecycles.
- **workflow ownership**: define command/check lists only in Just; CI and docs reference entry points. Add tests to existing runners, not recipes/jobs/gate steps. New steps require a new toolchain, feature combination or external dependency; merge overlapping resource stages. Test behavior, not mirrored recipe literals, job counts or step order.
- **platform support**: support modern Windows/MSVC, macOS and Linux, including Docker, through shared tooling modules; preserve Zellij pane/session controls. See [development rules](docs/en/001-DEVELOPMENT-VERIFICATION.md).
- **documentation site**: VitePress consumes docs through tracked relative symlinks; developers must enable symlinks on modern operating systems. See [development rules](docs/en/001-DEVELOPMENT-VERIFICATION.md#documentation-site).
- **native dependencies**: keep codec source upstream, pin Git dependencies to full commits, and do not vendor codec source trees here.
- **configuration**: use service-owned nested fields with explicit units; see [configuration contracts](docs/en/004-MEDIA-AND-CONFIGURATION.md).
- **documentation and evidence**: keep current contracts in topic docs, plans in the roadmap and history in CHANGELOG; retain necessary handoffs/evidence in temp, preserve unrelated workspace changes, and validate links with `just check docs`.
- **node package manager**: use `pnpm` as the package manager.
- **rust toolchain**: use `rust-toolchain.toml` to manage the rust toolchain, use `cargo` as the build tool.
- **typescript**: use tsconfig.json with references for managing the typescript project. Run erasable Node tooling directly as TypeScript; guard shared executable entry points with `import.meta.main`.
- **python**: use pyproject.toml to manage the python project, use `uv` as the package manager (workspace mode).
- **webui stack**: use typescript + vite + react + @tanstack/react-xxx seriers + tailwindcss + shadcn/ui for the webui stack.
- **server stack**: use rust + axum + SecurityDept OIDC/session + serde + snafu + tracing series for the server stack.
- **unreleased migrations**: before the first post-refactor release, edit existing migrations directly; do not add compatibility migrations for intermediate implementations. Recreate isolated test databases when migration definitions change.
- **test**: use cargo test for Rust and testcontainers for external dependencies. Organize tests by unit/integration/e2e as defined in [development rules](docs/en/001-DEVELOPMENT-VERIFICATION.md); keep Rust units inline or in test modules, and lifecycle ownership inside the test framework.

### GraphQL Insert/Update Skip Rules

When working with seaography `insert_skips` / `update_skips`:

- **MUST** use `GraphqlColumnKey::of::<Entity>(context, &Column::Field)` to generate skip keys. **NEVER** use `EntityColumnId::to_string()` — it returns the database format (`table.column`) which silently fails the GraphQL-format check (`TypeName.columnName`).
- When adding/modifying `ActiveModelBehavior::before_save` to auto-generate a field value, **MUST** also add the corresponding `GraphqlColumnKey::of(...).push_insert_skip(context)` in the entity's `register_*_to_schema_context` function.
- After any skip changes, re-run `just dev-codegen` and verify the field is removed from the generated `*InsertInput` type in `graphql.ts`.

### Multi-language Docs

**Directory Structure:**
- English docs: `docs/{lang}/00x-TITLE.md` (e.g., `docs/en/00x-TITLE.md`) or subdirectory `docs/{lang}/**/00x-TITLE.md` (e.g., `docs/en/**/00x-TITLE.md`)
- Current, running contracts stay flat under `docs/{lang}/`; future decisions and proposals not yet implemented go under `docs/{lang}/roadmap/`. `just check docs` discovers both recursively.

**Rules:**
- English convention documents (`README.md`, `CHANGELOG.md`, `CONTRIBUTING.md`, `SECURITY.md`, `PRIVACY.md`, when present) have suffixless repository-root sources; `docs/en` and `apps/docs` expose relative symlink projections. Other languages keep regular sources under `docs/{lang}`, projected into the site. Do not maintain separate homepage content; VitePress maps the locale README to index.html.
- Translate user-facing docs only (README, docs/00x-*.md); do NOT translate machine-oriented docs (AGENTS.md, CLAUDE.md, etc.)
- Each doc should have bidirectional language links at the bottom, targeting canonical sources: topic docs link between language directories; convention docs link to the English root source and the translated `docs/{lang}` source.
- Non-English docs must link to other docs in the same language folder when available (e.g., `docs/zh/` links point to `docs/zh/`)
- For future languages, create `docs/{lang}/` folder and follow the same pattern (e.g., `docs/es/`, `docs/ja/`)

**Current languages:**
- English: `docs/en/00x-TITLE.md`
- Chinese: `docs/zh/00x-TITLE.md`

## Documentation index

- [Project overview](README.md)
- [Development, tooling, verification and contributor rules](docs/en/001-DEVELOPMENT-VERIFICATION.md) · [中文](docs/zh/001-DEVELOPMENT-VERIFICATION.md)
- [Authentication](docs/en/002-AUTHENTICATION-DECISION.md) · [中文](docs/zh/002-AUTHENTICATION-DECISION.md)
- [Task delivery and migrations](docs/en/003-TASK-DELIVERY-AND-MIGRATION.md) · [中文](docs/zh/003-TASK-DELIVERY-AND-MIGRATION.md)
- [Media and configuration](docs/en/004-MEDIA-AND-CONFIGURATION.md) · [中文](docs/zh/004-MEDIA-AND-CONFIGURATION.md)
- [Roadmap](docs/en/roadmap/001-SHORT-TERM-ROADMAP.md) · [中文](docs/zh/roadmap/001-SHORT-TERM-ROADMAP.md)
- [Animeta model proposal](docs/en/roadmap/002-ANIMETA-MODEL.md) · [中文](docs/zh/roadmap/002-ANIMETA-MODEL.md)
- [Release history](CHANGELOG.md)

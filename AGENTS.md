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
- **node package manager**: use `pnpm` as the package manager.
- **rust toolchain**: use `rust-toolchain.toml` to manage the rust toolchain, use `cargo` as the build tool.
- **typescript**: use tsconfig.json with references for managing the typescript project.
- **python**: use pyproject.toml to manage the python project, use `uv` as the package manager (workspace mode).
- **webui stack**: use typescript + vite + react + @tanstack/react-xxx seriers + tailwindcss + shadcn/ui for the webui stack.
- **server stack**: use rust + axum + openconnectid + serde + snafu + tracing series for the server stack.
- **test**: use cargo test for unit test, use testcontainers for tests with external dependencies.

### GraphQL Insert/Update Skip Rules

When working with seaography `insert_skips` / `update_skips`:

- **MUST** use `GraphqlColumnKey::of::<Entity>(context, &Column::Field)` to generate skip keys. **NEVER** use `EntityColumnId::to_string()` — it returns the database format (`table.column`) which silently fails the GraphQL-format check (`TypeName.columnName`).
- When adding/modifying `ActiveModelBehavior::before_save` to auto-generate a field value, **MUST** also add the corresponding `GraphqlColumnKey::of(...).push_insert_skip(context)` in the entity's `register_*_to_schema_context` function.
- After any skip changes, re-run `just dev-codegen` and verify the field is removed from the generated `*InsertInput` type in `graphql.ts`.

### Multi-language Docs

**Directory Structure:**
- English docs: `docs/{lang}/00x-TITLE.md` (e.g., `docs/en/00x-TITLE.md`) or subdirectory `docs/{lang}/**/00x-TITLE.md` (e.g., `docs/en/**/00x-TITLE.md`)

**Rules:**
- Translate user-facing docs only (README, docs/00x-*.md); do NOT translate machine-oriented docs (AGENTS.md, CLAUDE.md, etc.)
- Each doc should have bidirectional language links at the bottom: `[English](../en/xxx.md) | [中文](xxx.md)` (in Chinese docs) or `[English](xxx.md) | [中文](../zh/xxx.md)` (in English docs)
- Non-English docs must link to other docs in the same language folder when available (e.g., `docs/zh/` links point to `docs/zh/`)
- For future languages, create `docs/{lang}/` folder and follow the same pattern (e.g., `docs/es/`, `docs/ja/`)

**Current languages:**
- English: `docs/en/00x-TITLE.md`
- Chinese: `docs/zh/00x-TITLE.md`

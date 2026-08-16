# Lint all (default), rust or ts. Select only known private recipes.
lint scope="all":
    @just {{if scope == "all" { "_lint-all" } else if scope == "rust" { "_lint-rust" } else if scope == "ts" { "_lint-ts" } else { error("Unknown lint scope: " + scope) }}}

# Fix all (default), rust or ts.
fix scope="all":
    @just {{if scope == "all" { "_fix-all" } else if scope == "rust" { "_fix-rust" } else if scope == "ts" { "_fix-ts" } else { error("Unknown fix scope: " + scope) }}}

# Check all (default), rust, ts, fmt, docs or types.
check scope="all":
    @just {{if scope == "all" { "_check-all" } else if scope == "rust" { "_check-rust" } else if scope == "ts" { "_check-ts" } else if scope == "fmt" { "_check-fmt" } else if scope == "docs" { "_check-docs" } else if scope == "types" { "_check-types" } else { error("Unknown check scope: " + scope) }}}

[private]
_lint-all: _lint-rust _lint-ts

# Explicitly enumerate valid feature combinations; --all-features is invalid here.
[private]
_lint-rust: _lint-workspace (_lint-recorder "media-par-rayon") (_lint-recorder "jxl,media-par-chili") (_lint-recorder "media-par-chili") (_lint-recorder "jxl") (_lint-recorder "")

[private]
_lint-workspace:
    cargo clippy --workspace --locked --features testcontainers,test-utils,playground --all-targets -- -D warnings

[private]
_lint-recorder features:
    cargo clippy -p recorder --locked --no-default-features --features {{if features == "" { "testcontainers,test-utils,playground" } else { features + ",testcontainers,test-utils,playground" }}} --all-targets -- -D warnings

[private]
_lint-ts:
    pnpm exec biome check

[private]
_fix-all: _fix-rust _fix-ts

[private]
_fix-rust:
    cargo clippy --workspace --fix --allow-dirty

[private]
_fix-ts:
    pnpm exec biome check --write

[private]
_check-all: _check-rust _check-ts _check-docs _check-types

[private]
_check-rust:
    cargo check --workspace --locked

[private]
_check-ts:
    pnpm exec tsc -b

[private]
_check-docs:
    node scripts/dev-cli.mts check-docs

[private]
_check-types:
    node scripts/dev-cli.mts types check

[private]
_check-fmt:
    cargo fmt -p recorder -p downloader -p testing-torrents -p util-derive --check

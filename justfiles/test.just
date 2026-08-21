# Optional integration and example coverage for the default workspace runner.
rust_test_features := "testcontainers,test-utils,playground"

# Run a native suite with runner filters: all, rust, webui, tooling, browser, media.
[positional-arguments]
test *args:
    node scripts/dev-cli.mts test {{recipe_args}}

[private]
_test-all:
    just test rust
    just test webui

# Cargo's all-targets mode excludes doctests, so run them separately.
[private]
_test-rust:
    cargo test --locked --workspace --all-targets --features {{rust_test_features}}
    cargo test --locked --workspace --features {{rust_test_features}} --doc

# Shared resource stages: all, static, rust, ts. Unknown stages fail before running.
verify stage="all":
    @just {{if stage == "all" { "_verify-all" } else if stage == "static" { "_verify-static" } else if stage == "rust" { "_verify-rust" } else if stage == "ts" { "_verify-ts" } else { error("Unknown verification stage: " + stage) }}}

# Generators complete before checking/building their TypeScript consumers.
[private]
_verify-all:
    just verify static
    just verify rust
    just verify ts

[private]
_verify-static:
    just release version check
    just test tooling
    just check docs
    just check fmt
    git diff --check HEAD

[private]
_verify-rust:
    docker info
    just lint rust
    node scripts/dev-cli.mts bindings snapshot temp/verification/bindings-before.json
    just test rust
    just test media
    just test browser
    node scripts/dev-cli.mts bindings verify temp/verification/bindings-before.json

[private]
_verify-ts:
    just types rebuild
    just build-packages
    just build-webui
    just build-docs
    just test webui
    just lint ts
    just check types

# The codec is a compilation choice, not host behavior: no media source is
# OS-conditional, so one runner owns this. `just test rust` covers the shipped
# JXL + WebP build, leaving the trimmed WebP-only build to cover every
# not(feature = "jxl") branch in the library, its test binaries and real codecs.
[private]
_test-media-trimmed:
    cargo test -p recorder --locked --no-default-features --features test-utils --lib media::
    cargo test -p recorder --locked --no-default-features --features test-utils --test media_executor --test config_media --test media_delivery --test media_http
    cargo build -p recorder --locked --no-default-features --bin recorder-cli
    node scripts/dev-cli.mts media smoke webp

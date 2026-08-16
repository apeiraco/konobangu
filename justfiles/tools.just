export-recorder-ts-bindings:
    cargo test -p recorder --locked --lib export_bindings

testcontainers-prune:
    cargo run -p recorder --bin testcontainers-prune --features testcontainers

# Inspect and invoke the project utility CLI.
[positional-arguments]
dev-cli *args:
    node scripts/dev-cli.mts {{recipe_args}}

# Manage compiler outputs: check, rebuild, clean [--legacy] [--project PATH].
[positional-arguments]
types *args:
    node scripts/dev-cli.mts types {{recipe_args}}

# Run recorder media backfill with native arguments.
[positional-arguments]
media-backfill *args:
    cargo run -p recorder --locked --bin media-backfill -- {{recipe_args}}

bench-media:
    cargo build -p recorder --locked --release --features jxl --bin recorder-cli
    cargo run -p recorder --locked --release --features jxl --example media_bench -- temp/bench/media

# Dependency maintenance remains a native Node ecosystem operation.
update-node:
    pnpm dlx npm-check-updates --deep -u
    pnpm install

# Run the torrent test service without an unnecessary package-script alias.
dev-torrents:
    pnpm --dir packages/testing-torrents exec node main.ts

# Only declared artifacts or dependency directories; never broad git clean -xdf.
[positional-arguments]
clean *args:
    node scripts/dev-cli.mts clean {{recipe_args}}

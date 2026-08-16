setup:
    mise install
    pnpm install --frozen-lockfile
    pnpm --dir packages/testing-torrents install --frozen-lockfile
    cargo check --workspace --locked
    uv sync
    pnpm exec prek install

prepare-dev:
    cargo install cargo-binstall
    cargo binstall sea-orm-cli cargo-llvm-cov cargo-nextest

prepare-dev-testcontainers:
    docker pull linuxserver/qbittorrent:latest
    docker pull ghcr.io/apeiraco/konobangu-testing-torrents:latest
    docker pull postgres:18-alpine

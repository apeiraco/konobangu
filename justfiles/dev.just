# Simple ecosystem commands stay with their native tools.
dev-webui:
    pnpm --filter webui exec vite

prod-webui: build-webui
    node scripts/dev-cli.mts copy-webui

dev-proxy:
    zellij --layout proxy.kdl

dev-recorder:
    watchexec -r -e rs,toml,yaml,json -- cargo run -p recorder --bin recorder-cli -- --environment=development --graceful-shutdown=false

prod-recorder: prod-webui
    cargo run --release -p recorder --bin recorder-cli -- --environment=production --working-dir=apps/recorder --graceful-shutdown=false

dev-recorder-migrate-up:
    cargo run -p recorder --locked --bin migrate-up -- --environment development --config-file apps/recorder/recorder.config.toml --dotenv-file apps/recorder/.env.development

dev-recorder-migrate-down:
    cargo run -p recorder --locked --bin migrate-down -- --environment development

dev-deps:
    docker compose -f devdeps.compose.yaml up

dev-deps-clean:
    docker compose -f devdeps.compose.yaml down -v

# Generate GraphQL once or continuously; watch does not wait for the backend.
dev-codegen mode="once":
    pnpm --filter webui exec graphql-codegen --config graphql-codegen.ts {{if mode == "once" { "" } else if mode == "watch" { "--watch" } else { error("Use once or watch") }}}

# The Zellij pane needs readiness waiting in addition to ordinary watch mode.
dev-codegen-wait:
    node scripts/dev-cli.mts wait-recorder
    just dev-codegen watch

dev-coverage:
    cargo llvm-cov test --html

dev-all:
    zellij --layout dev.kdl

dev-proxy-mikan:
    cargo run -p proxy --bin mikan-doppel

dev-proxy-whistle:
    pnpm exec cross-env "WHISTLE_MODE=prod|capture|keepXFF|x-forwarded-host|x-forwarded-proto" pnpm --filter proxy exec whistle run -p 8899 -t 30000 -D .

webui-preview:
    pnpm --filter webui exec vite preview

# Serve the bilingual VitePress site on loopback:5004.
dev-docs:
    pnpm --filter docs exec vitepress dev --host 127.0.0.1 --port 5004

# Preview the built documentation site on loopback:5004.
docs-preview:
    pnpm --filter docs exec vitepress preview --host 127.0.0.1 --port 5004

dev-email:
    pnpm --filter email-playground exec vite

recorder-playground:
    cargo run -p recorder --locked --features playground --example playground -- --environment development

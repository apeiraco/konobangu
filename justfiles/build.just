# Package artifacts (including bundled declarations) are separate from tsc -b outputs.
build-packages:
    pnpm exec tsdown --config scripts/tsdown.config.mts

# Build documentation from docs/en and docs/zh, checking internal links.
build-docs:
    pnpm --filter docs exec vitepress build

# Build the email preview application or export HTML templates.
build-email mode="build":
    pnpm --filter email-playground exec vite build {{if mode == "build" { "" } else if mode == "export" { "--outDir out" } else { error("Use build or export") }}}

build-webui:
    pnpm --filter webui exec vite build
    pnpm --filter webui exec biome check --write src/presentation/routeTree.gen.ts

# Build and collect a release artifact through the platform-aware utility.
[positional-arguments]
build-release *args:
    node scripts/dev-cli.mts build-release {{recipe_args}}

# Verify an existing artifact or build/test a fresh candidate.
[positional-arguments]
platform-check *args:
    node scripts/dev-cli.mts platform-check {{recipe_args}}

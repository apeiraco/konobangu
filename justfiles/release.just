# Metadata-aware release operations; tag/publish default to dry runs.
[positional-arguments]
release *args:
    node scripts/dev-cli.mts release {{recipe_args}}

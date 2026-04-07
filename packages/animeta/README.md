# konobangu-animeta (Python)

NLP datasets and tooling for anime metadata. Rust library code lives in the same crate (`src/`).

## Quick start

```bash
cd packages/animeta
uv sync
# Optional: export MIKAN_PARQUET=/path/to/tiny.parquet
uv run animeta-mikan-demo --limit 2000
# Without local parquet:
uv run animeta-mikan-demo --synthetic
```

Parquet columns: `fansub_name`, `original_name` (see `apps/recorder` merge pipeline).

# konobangu-animeta (Python)

NLP datasets and tooling for anime metadata. The Rust library lives in [`packages/animeta`](../animeta).

## Quick start

```bash
cd packages/animeta-py
uv sync
# Optional: export MIKAN_PARQUET=/path/to/tiny.parquet
uv run animeta-mikan-demo --limit 2000
# Without local parquet:
uv run animeta-mikan-demo --synthetic
```

Parquet columns: `fansub_name`, `original_name` (see `apps/recorder` merge pipeline).

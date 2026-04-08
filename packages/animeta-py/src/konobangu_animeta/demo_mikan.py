#!/usr/bin/env python3
"""Mikan classic episode titles: small end-to-end NLP demo (2025-era stack).

Shows patterns common in modern NLP pipelines:
  - Columnar IO with Polars (fast local parquet).
  - Hugging Face ``datasets`` for batched map / memory-friendly iteration.
  - Two ``transformers`` tokenizers (multilingual subword; no extra MeCab install).
  - Sentence embeddings with ``sentence-transformers`` (multilingual MiniLM).

Default parquet path matches the recorder test fixture layout. Override with env
``MIKAN_PARQUET`` or ``--parquet PATH``. Use ``--synthetic`` if you have no file yet.

Run from repo root::

    cd packages/animeta-py && uv sync && uv run animeta-mikan-demo

Or::

    uv run --directory packages/animeta-py animeta-mikan-demo --limit 500
"""

from __future__ import annotations

import argparse
import os
import random
import sys
from pathlib import Path

import polars as pl
import torch
from datasets import Dataset
from rich.console import Console
from rich.table import Table
from sentence_transformers import SentenceTransformer
from tqdm.auto import tqdm
from transformers import AutoTokenizer


def _repo_root() -> Path:
    # src/konobangu_animeta/demo_mikan.py -> parents[4] = workspace root
    return Path(__file__).resolve().parents[4]


def default_parquet_path() -> Path:
    env = os.environ.get("MIKAN_PARQUET")
    if env:
        return Path(env).expanduser().resolve()
    return _repo_root() / "resources/mikan/classic_episodes/parquet/tiny.parquet"


def load_titles(
    path: Path | None, *, synthetic: bool, limit: int | None
) -> pl.DataFrame:
    if synthetic:
        # original_name is site-style title; fansub is merged as [fansub] prefix (see merge pipeline).
        rows = [
            {
                "fansub_name": "Kamigami",
                "original_name": "進撃の巨人 The Final Season Part 2 - 16 (BD 1920x1080 AVC FLAC)",
            },
            {
                "fansub_name": "LoliHouse",
                "original_name": "孤独摇滚！ / Bocchi the Rock! - 12 [WebRip 1080p HEVC-10bit AAC][简繁内封字幕]",
            },
            {
                "fansub_name": "喵萌奶茶屋",
                "original_name": "[喵萌奶茶屋&LoliHouse] 间谍过家家 / Spy x Family - 25 [WebRip 1080p HEVC-10bit AAC][无字幕]",
            },
        ]
        df = pl.DataFrame(rows)
    else:
        p = path or default_parquet_path()
        if not p.is_file():
            raise FileNotFoundError(
                f"Parquet not found: {p}\n"
                "Set MIKAN_PARQUET or pass --parquet, or use --synthetic for a tiny in-memory sample."
            )
        df = pl.read_parquet(p)

    if limit is not None and limit > 0:
        df = df.head(limit)

    required = {"fansub_name", "original_name"}
    missing = required - set(df.columns)
    if missing:
        raise ValueError(f"Parquet missing columns {missing}; have {df.columns}")

    # Single field for NLP (matches how release names appear in the wild).
    text = pl.concat_str(
        [
            pl.lit("["),
            pl.col("fansub_name").fill_null(""),
            pl.lit("] "),
            pl.col("original_name").fill_null(""),
        ]
    ).alias("text")
    return df.with_columns(text)


def token_length_stats(encoded_lengths: list[int]) -> dict[str, float]:
    if not encoded_lengths:
        return {"count": 0, "mean": 0.0, "p95": 0.0, "max": 0.0}
    s = sorted(encoded_lengths)
    n = len(s)
    mean = sum(s) / n
    p95 = s[int(0.95 * (n - 1))]
    return {"count": float(n), "mean": mean, "p95": float(p95), "max": float(s[-1])}


def run_demo(
    df: pl.DataFrame,
    *,
    encode_batch: int,
    embed_pairs: int,
    device: str | None,
) -> None:
    console = Console()
    texts = df["text"].to_list()
    console.print(f"[bold]Rows[/bold]: {len(texts):,}")

    # --- Tokenizers: two multilingual subword models (no MeCab / fugashi) ---
    # For Japanese-heavy finetuning people often add tohoku + fugashi; keep the demo portable.
    tok_xlm = AutoTokenizer.from_pretrained("xlm-roberta-base")
    tok_mlm = AutoTokenizer.from_pretrained("bert-base-multilingual-cased")

    len_xlm: list[int] = []
    len_mlm: list[int] = []
    for t in tqdm(texts, desc="tokenize (local)", leave=False):
        len_xlm.append(len(tok_xlm.encode(t, add_special_tokens=True)))
        len_mlm.append(len(tok_mlm.encode(t, add_special_tokens=True)))

    st_xlm = token_length_stats(len_xlm)
    st_mlm = token_length_stats(len_mlm)

    table = Table(title="Token length (with special tokens)")
    table.add_column("Tokenizer", style="cyan")
    table.add_column("mean", justify="right")
    table.add_column("p95", justify="right")
    table.add_column("max", justify="right")
    table.add_row(
        "xlm-roberta-base",
        f"{st_xlm['mean']:.1f}",
        f"{st_xlm['p95']:.0f}",
        f"{st_xlm['max']:.0f}",
    )
    table.add_row(
        "bert-base-multilingual-cased",
        f"{st_mlm['mean']:.1f}",
        f"{st_mlm['p95']:.0f}",
        f"{st_mlm['max']:.0f}",
    )
    console.print(table)

    # --- Hugging Face Dataset + batched map (typical pretraining / finetune pattern) ---
    ds = Dataset.from_dict({"text": texts})

    def tokenize_batch(batch: dict[str, list]) -> dict[str, list]:
        out = tok_xlm(
            batch["text"],
            truncation=True,
            max_length=256,
            padding=False,
        )
        return {"input_ids": out["input_ids"], "attention_mask": out["attention_mask"]}

    ds_tok = ds.map(
        tokenize_batch,
        batched=True,
        batch_size=min(encode_batch, max(len(texts), 1)),
        remove_columns=["text"],
        desc="datasets.map (tokenize)",
    )
    console.print(
        f"[bold]HF Dataset[/bold]: columns={ds_tok.column_names}, num_rows={ds_tok.num_rows}"
    )

    # --- Sentence embeddings (retrieval / clustering / contrastive learning family) ---
    dev = device or ("cuda" if torch.cuda.is_available() else "cpu")
    model_id = "sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2"
    console.print(f"[bold]SentenceTransformer[/bold]: {model_id} on {dev}")

    st_model = SentenceTransformer(model_id, device=dev)
    emb = st_model.encode(
        texts,
        batch_size=min(encode_batch, max(len(texts), 1)),
        show_progress_bar=True,
        convert_to_numpy=True,
        normalize_embeddings=True,
    )
    console.print(f"[bold]Embedding shape[/bold]: {emb.shape} (L2-normalized)")

    if embed_pairs > 0 and len(texts) >= 2:
        rng = random.Random(42)
        pairs = min(embed_pairs, len(texts) * (len(texts) - 1) // 2)
        console.print(
            f"[bold]Random cosine similarities[/bold] (sample {pairs} pairs):"
        )
        sim_table = Table()
        sim_table.add_column("i", justify="right")
        sim_table.add_column("j", justify="right")
        sim_table.add_column("cosine", justify="right")
        sim_table.add_column("text_i (trunc)", overflow="ellipsis", max_width=48)
        sim_table.add_column("text_j (trunc)", overflow="ellipsis", max_width=48)
        seen: set[tuple[int, int]] = set()
        while len(seen) < pairs:
            i, j = rng.randrange(len(texts)), rng.randrange(len(texts))
            if i == j:
                continue
            a, b = (i, j) if i < j else (j, i)
            if (a, b) in seen:
                continue
            seen.add((a, b))
            cos = float((emb[a] * emb[b]).sum())
            ti = texts[i][:80] + ("…" if len(texts[i]) > 80 else "")
            tj = texts[j][:80] + ("…" if len(texts[j]) > 80 else "")
            sim_table.add_row(str(i), str(j), f"{cos:.4f}", ti, tj)
        console.print(sim_table)


def build_arg_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument(
        "--parquet",
        type=Path,
        default=None,
        help="Parquet path (default: MIKAN_PARQUET or recorder test tiny.parquet)",
    )
    p.add_argument(
        "--synthetic",
        action="store_true",
        help="Ignore parquet; use a few hard-coded rows (offline / CI friendly).",
    )
    p.add_argument("--limit", type=int, default=None, help="Max rows after load")
    p.add_argument(
        "--encode-batch",
        type=int,
        default=64,
        help="Batch size for HF map and sentence-transformers encode",
    )
    p.add_argument(
        "--embed-pairs",
        type=int,
        default=5,
        help="How many random pairs to print for cosine similarity",
    )
    p.add_argument(
        "--device",
        default=None,
        help="torch device for embeddings (default: cuda if available else cpu)",
    )
    return p


def main(argv: list[str] | None = None) -> int:
    args = build_arg_parser().parse_args(argv)
    try:
        df = load_titles(args.parquet, synthetic=args.synthetic, limit=args.limit)
    except (FileNotFoundError, ValueError) as e:
        print(e, file=sys.stderr)
        return 1
    run_demo(
        df,
        encode_batch=args.encode_batch,
        embed_pairs=args.embed_pairs,
        device=args.device,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

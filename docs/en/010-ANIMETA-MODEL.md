# 010 — Animeta Learned Parsing Plan (Python Train + Rust Infer)

## Status

Proposal (for future implementation; can evolve alongside current `legacy` / `probability` code)

## Audience

- Maintainers of `packages/animeta` (Rust) and `packages/animeta-py` (Python)
- Anyone planning to extend anime title / release-name parsing

## Purpose

Keep the **`OriginNameMeta` public contract** while moving from pure rules + hand-tuned scores toward a **small, data-driven model**, and align decomposition with **legacy (nom)** and **Aniparse-style** dimensions so labeling and evaluation stay coherent.

---

## 1. Why leave “compiler-style” rules

The legacy path is a **combinator parser**: `FansubComp` → scan until `EpisodeComp` / `MoiveComp` → `BangumiComps` (title + season) → `ExtraComps` (split remainder and try `SubtitleComp`, `ResolutionComp`, `SourceL1/L2`, etc.). See `OriginNameEpisode`, `OriginNameMovie`, and `OriginNameMeta` assembly.

Pros: **explainable, unit-testable**. Cons:

- **Span boundaries** depend on order and delimiter assumptions; new site patterns require **grammar edits**.
- Output is flattened into **`OriginNameMeta` fields**; intermediate structure (multi-segment title, stacked sources, ranges) is **lost**, which misaligns with multi-task / multi-span supervision.
- “Probability” tuned by hand does not scale.

Recommendation: **train in Python** (mature tooling), **infer in Rust** via **ONNX** (or similar), matching the rest of the binary stack.

---

## 2. Legacy decomposition (what the code actually parses)

Maps to `packages/animeta/src/extract/legacy.rs`; use as the **semantic source** for a labeling schema.

| Dimension | Legacy type / behavior | Maps to `OriginNameMeta` |
|-----------|--------------------------|---------------------------|
| Fansub / release | `FansubComp` | `fansub` |
| Title + season | `BangumiComps` | `name`, `season`, `season_raw` |
| Episode / collection | `EpisodeComp` | `episode_index` |
| Movie marker | `MoiveComp` | Movie vs episode branch; episode often `1` |
| Extras | `ExtraComp` heuristics on split tokens | `subtitle`, `resolution`, `source` |

**Note:** `ExtraComps` is **bucketed guessing**, not a global optimal segmentation—same pain point a learned **span + type** model addresses.

---

## 3. Aniparse-style dimensions (aligned with `Label`)

Typical block/element splits (Aniparse-like), aligned with `extract::probability::types::Label` for a **unified tag set** (trim as needed):

| Role | Notes | Example `Label` |
|------|--------|-----------------|
| Release group | Leading bracket block | `ReleaseGroup` |
| Title | Multi-language, slash-separated | `Title` |
| Season | `S2`, `第二季`, `2nd STAGE` | `SeasonPrefix`, `Season`, `SeriesType`, … |
| Episode / range | Single, range, batch | `EpisodePrefix`, `SequenceNumber`, `SequenceRange` |
| Video / audio | Codecs, container | `VideoTerm`, `AudioTerm` |
| Resolution | `1080p`, `1920x1080` | `VideoResolution` |
| Source | WebRip, BDRip, platform | `Source` |
| Subs / language | 简繁, 中日 | `Language`, `SubsTerm` |
| Promo / misc | 招募, batch tags | `ReleaseInformation`, `Other` |
| Structure | Brackets, delimiters | `Bracket`, `Delimiter`, `ContextDelimiter` |

The trained model need not replicate every enum value; NER tags must **project** to these semantics before **composing** `OriginNameMeta`.

---

## 4. Recommended architecture (Python + Rust)

### 4.1 Roles

| Phase | Stack | Notes |
|-------|--------|------|
| Data + silver labels | Python + JSON Schema | LLM batch labeling, QA, diff vs legacy |
| Training | PyTorch + small encoder | Sequence labeling first; optional sentence heads |
| Export | ONNX | Fix ops/shapes at export time |
| Inference | Rust + `ort` | Same process as `recorder` or sidecar |
| Fallback | `legacy::OriginNameRoot::parse_comp` | Low confidence or validation failure |

### 4.2 Model shape (suggested)

1. **Shared encoder** (char-level or small subword vocab; **char / UTF-8** often robust for filenames).
2. **Sequence head**: per character (or subword) **BIO / multi-type** tags for roles above.
3. **Optional sentence head**: coarse attributes (e.g. movie vs ep) to break ties.
4. **Optional open-set**: cluster unknown sources offline; online still store raw `source` string.

### 4.3 Mapping to `OriginNameMeta`

Keep the **JSON contract** unchanged:

- **Deterministic compose** from spans: `name` (merge policy for slashes / theater), `season` / `season_raw`, `episode_index`, `subtitle`, `source`, `fansub`, `resolution`.
- **Versioning**: bind `model_version` + `tokenizer_version` to preprocessing.

---

## 5. Data and supervision

| Source | Use |
|--------|-----|
| Mikan-style historical titles | In-domain wording |
| Real release filenames (if legally obtainable) | Close **domain gap** vs site-polished titles |
| LLM silver labels | Scale; fix schema and use consistency checks |
| Legacy output | Weak labels or filters; disputed rows for human gold |

---

## 6. Evaluation and rollout

- **Regression suite**: same `OriginNameMeta` equality idea; add **model vs legacy** diff reports.
- **Confidence gating**: fall back to legacy below threshold.
- **Gradual rollout**: slice by user/task; monitor per-field accuracy and fallback rate.

---

## 7. Suggested milestones

1. Freeze **label schema** + **compose rules** into `OriginNameMeta`.
2. Build **10k–100k** silver + small gold set.
3. Train baseline → ONNX → Rust `ort` E2E latency check.
4. Compare with `probability`; decide default path vs ensemble.

---

## 8. Relation to current code

- **`legacy`**: long-term **reference and fallback**; not removed by adding a model.
- **`probability`**: may shrink to **features**, **data prep**, or **ensemble**; driven by metrics.

---

[English](010-ANIMETA-MODEL.md) | [中文](../zh/010-ANIMETA-MODEL.md)

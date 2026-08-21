# 004 — Media and deployment configuration

JXL/WebP share one bounded queue, one owned Rayon pool and one admission budget. An admitted job starts as soon as a worker is free and delivers its own result immediately; the queue bounds waiting work and the budget bounds concurrent working sets. Normal builds generate JXL + WebP; automatic AVIF is rejected. Static display and [R8 progressive rendering](roadmap/001-SHORT-TERM-ROADMAP.md) have separate acceptance criteria.

The JPXL facade pins Git rev `1e2004aa0672259279cc24a52d4b8e264f675b77`, Balanced, quality 77 and threads 1, with the fixed bounded policy features. Each image receives one facade call. Library policy may use up to five score probes and three exact prices; the application adds no search, size-based format switching or lossless fallback. Output is ordinary single pass. WebP retains webp 0.3.1/libwebp-sys 0.9.6, quality 80, without per-item internal threading.

Covers apply orientation, composite accepted 8-bit sRGB samples over white in encoded sRGB space, then use Lanczos3 to limit the long edge to 1600 without upscaling. Both codecs consume the same normalized pixels; invisible RGB cannot contaminate transparent edges. Originals remain available. General WebP preserves alpha; general JXL RGBA and legacy lossless requests fail explicitly rather than silently compositing and claiming losslessness. The JPXL preset API is `{"mime_type":"image/jxl","preset_version":1}`, with `mimeType`/`presetVersion` in GraphQL input.

## Configuration sources

Figment parses one selected TOML, JSON or YAML/YML file using the existing filename priority. YAML uses `YamlExtended` for anchor/alias and merge keys; merge keys are an explicitly supported extension, not a promise about all YAML 1.2 processors. Defaults < file < `HOST`/`DATABASE_URL` aliases < canonical double-underscore environment variables. A selected dotenv file and process variables form a startup snapshot, with process variables taking precedence. Loading does not mutate the process environment.

Only effective file-origin string leaves, including strings in arrays, expand. Overridden placeholders do not fail startup. Keys, defaults, canonical values, secret-file contents and expansion results are final and never scanned again. Strings remain strings; native numeric/boolean/list structure and canonical Figment environment parsing retain their types.

| Syntax | Behavior |
| --- | --- |
| `$NAME`, `${NAME}` | ASCII identifier; missing or empty fails |
| `${NAME:-literal}` | Literal fallback for missing/empty; an empty fallback is allowed before field validation |
| `$$`, `$${NAME}` | Literal `$`, `${NAME}` |
| Nested expansions, `:?`, `:+`, `${NAME-default}`, command substitution, unclosed braces | Rejected with field path and redacted category |

Fallback literals cannot contain `$` or braces. A standalone dollar followed by an ordinary nonidentifier character stays literal. No tilde, shell, template evaluation or configuration inheritance is added. Quoted `{{ ... }}`, `{% ... %}` and `{# ... #}` remain ordinary strings; old unquoted templates fail native parsing with a redacted migration diagnostic.

```toml
[storage]
data_dir = "${DATA_ROOT:-/var/lib/konobangu}"
[media]
# Omit auto_optimize_formats to inherit compiled defaults.
# Optional WebP-only override: auto_optimize_formats = ["image/webp"]
[media.execution]
deadline_seconds = 30
queue_capacity = 8
concurrency = 1
```

```yaml
server:
  <<: &server_defaults
    binding: 0.0.0.0
    port: 5001
storage:
  data_dir: '${DATA_ROOT:-/var/lib/konobangu}'
```

## File secrets and logging

These six canonical variables accept an absolute path to a regular UTF-8 file (including symlinks to a regular file):

| Variable | Field |
| --- | --- |
| `AUTH__PROVIDER__PASSWORD_FILE` | `auth.provider.password` |
| `AUTH__PROVIDER__CLIENT_SECRET_FILE` | `auth.provider.client_secret` |
| `AUTH__SESSION__DATABASE_URL_FILE` | `auth.session.database_url` |
| `DATABASE__URL_FILE` | `database.url` |
| `DATABASE__MIGRATION__URL_FILE` | `database.migration.url` |
| `SCHEDULER__DATABASE__URL_FILE` | `scheduler.database.url` |

At most 64 KiB is read once. Empty, unreadable, oversized, non-UTF-8, directory, device or FIFO inputs fail startup. Ordinary canonical values and `_FILE` for the same field conflict, even if empty. Case-normalized duplicate environment targets also conflict. Unknown `_FILE` in application namespaces is rejected; unrelated tool variables are ignored. Alias `_FILE` and configuration-file `_file` syntax are unsupported.

File contents are literal, including trailing newline, quotes and `${...}`. Passwords are never trimmed. URI whitespace/newlines fail validation. Sensitive auth/database/task Debug fields and configuration/connection errors redact values; no configuration export endpoint is provided. Create files without a trailing newline and keep them outside committed configuration:

```text
node -e "require('node:fs').mkdirSync('secrets',{recursive:true}); require('node:fs').writeFileSync('secrets/basic-password','replace-with-deployment-password',{mode:0o600})"
```

Compose secrets are mounted as files, not interpolated into TOML:

```yaml
services:
  recorder:
    image: your-recorder-image
    environment:
      AUTH__PROVIDER__PASSWORD_FILE: /run/secrets/basic_password
      DATABASE__URL_FILE: /run/secrets/database_uri
    secrets:
      - basic_password
      - database_uri
secrets:
  basic_password:
    file: ./secrets/basic-password
  database_uri:
    file: ./secrets/database-uri
```

Provide the remaining native auth/server configuration separately. For OIDC use `AUTH__PROVIDER__CLIENT_SECRET_FILE`; the database URL initializes capabilities automatically; an optional migration URL can select a separate owner credential. No hot reload or secret rotation is implied.

## Logging configuration

`[logger]` in `recorder.config.toml` remains the logging entry, with an optional rolling-file example. `logger.enable` controls console output only; `logger.file_appender.enable` independently controls files. Formats are `compact`, `pretty` and `json`; levels are `off/error/warn/info/debug/trace`.

Global event filtering uses valid `RUST_LOG` → `logger.override_filter` → `logger.level` applied to recorder, sea_orm_migration, tower_http, sea_orm and sea_query. The file `level` further restricts globally allowed events; it cannot restore globally filtered events. There is currently no separate console filter for keeping debug in files while reducing console events; the file threshold does not override the global threshold. Invalid explicit `override_filter` fails initialization.

```toml
[logger]
enable = true
level = "info"
format = "compact"
pretty_backtrace = false
override_filter = "recorder=debug,tower_http=info,sea_orm=warn"

[logger.file_appender]
enable = true
non_blocking = true
level = "info"
format = "json"
rotation = "daily"
dir = "./logs"
filename_prefix = "recorder"
filename_suffix = "log"
max_log_files = 7
```

Relative `dir` resolves against the process working directory. Rotation supports `minutely/hourly/daily/never`; files remain usable with console output disabled. Canonical environment variables include `LOGGER__LEVEL=debug`, `LOGGER__FORMAT=json`, `LOGGER__OVERRIDE_FILTER=recorder=debug`, or the complete required `LOGGER__FILE_APPENDER__...` fields. `database.pool.log_queries` independently controls SQL log generation; logging filters still apply. The unimplemented old `logger.filter` key has been removed; unknown logger/file_appender keys fail. Use `override_filter` instead.

## Execution budgets and compatibility

| Configuration | Default and meaning |
| --- | --- |
| `limits.input_bytes` | 32 MiB source bytes |
| `limits.dimension` / `limits.pixels` | 8192 per edge / 16,777,216 pixels |
| `limits.decode_bytes` | 128 MiB decode allocation |
| `execution.working_set_bytes` | Shared 512 MiB admission estimate covering queued sources, decode/resize, codec workspace and output; not a hard RSS ceiling |
| `limits.output_bytes` | 64 MiB per output |
| `execution.concurrency` / `execution.queue_capacity` | One running / eight queued; concurrency sizes the encoder pool |
| `execution.deadline_seconds` | 30-second wait/result validity deadline, at most 300 seconds |

Timeout/cancellation rejects late results. A synchronous codec cannot be killed; the real task retains capacity and its budget until completion. Completed or skipped jobs release their working-set reservations before notifying callers; an observed completion means that job no longer consumes admission budget. Cancellation is checked between decode, normalization and encode; cancelled queued items are skipped. Shutdown stops admission, cancels queued work and waits for actual workers outside Tokio workers; the service exit path calls shutdown. Panics are caught inside each join branch so subsequent jobs can run. OOM, abort and native crashes still affect the service process; admission estimates do not provide per-job OS isolation.

Only static 8-bit SDR RGB/RGBA is optimized, after EXIF orientation. Embedded ICC, PNG cICP/HDR, gamma/chromaticities without an sRGB declaration, animation and other depths fail under the controlled fallback contract. Implicit and explicit sRGB are accepted. Failures preserve originals.

Legacy `max_encoder_bytes` (an RSS supervision threshold) is rejected; migrate explicitly to the `execution.working_set_bytes` admission estimate. Legacy `jxl_timeout_seconds` is rejected; use `media.execution.deadline_seconds`. `jxl_distance`, `jxl_quality`, `jxl_effort`, `jxl_speed`, `jxl_encoder_path` and `jxl_max_address_space_bytes` are explicitly rejected: neither JPXL quality nor in-process hard isolation can inherit their meaning. See [task upgrades](003-TASK-DELIVERY-AND-MIGRATION.md) for persisted-task migration and recovery.

Collection, workers and HTTP use one derivative plan. A small bounded manifest records source fingerprint/SHA256 and immutable derivative profile/path/length/SHA256. Full original extensions remain in derivative identities, so `a.jpg` and `a.png` are separate. The manifest is never an authorization source; every entry must match a path computed inside its source namespace. Manifest version 2 retains path compatibility; new profiles include the full JPXL revision, preset, white compositing and dimensions. Missing, corrupt, obsolete or source-invalid manifests fall back to the current original or 406 under Accept; sibling basename/mtime never supplies provenance. Legacy direct WebP/AVIF/JXL URLs remain readable. Backfill publishes current manifests without rewriting originals or stored URLs.

Source writes invalidate the manifest. New task publication checks its source version and execution fence, then takes the fixed full-path advisory resource lock. Encoding/source reads/large temporary writes occur before this short transaction. Derivatives publish atomically before the manifest. Owned temporary files are cleaned; historical immutable versions and originals are not garbage-collected automatically. Direct external filesystem edits require explicit rebuild and are detected through available fingerprints.

Public backfill is explicit and defaults to dry-run, 100 items. It never scans subscriber directories or runs from GET:

```bash
mise exec -- just media-backfill --config-file /etc/konobangu/recorder.config.toml
mise exec -- just media-backfill --config-file /etc/konobangu/recorder.config.toml --write --limit 100 --offset 0
```

Write mode requires the normal application/identity/scheduler configuration. Repeated enqueue skips equivalent active tasks and completed current profiles. Private images use the original owner's authorized task path.

## HTTP and pages

Recorder same-origin `/api/static/` image URLs use `optimize=accept`; external/signed/avatar/icon/data/blob URLs are preserved. Selection happens before conditions and Range. JXL requires explicit positive `image/jxl`; wildcards and `application/jxl` do not enable it. The most specific range determines q, zero excludes, and equal quality uses eligible progressive JXL → eligible existing ordinary JXL → WebP → original, independent of request order. Missing Accept selects original; malformed Accept returns a redacted 400, and exclusion of all available variants returns 406.

Negotiated 200/206/304/406 merge `Vary: Accept` with other Vary values. Public responses use `public, no-cache`, private use `private, no-cache`, and every private revalidation authorizes first. Derivatives have strong content-hash ETags. Originals without trusted content hashes use precise-path/metadata weak validators. Successful GET/HEAD conditions preserve cache headers and remove body/partial length; denials/errors never become 304. Only GET processes Range after conditions. The locked http-range 0.1.5 HttpRange::parse handles parsing and normalization, including leading zeros, OWS and mixed zero suffixes. A lone zero suffix or all-unsatisfiable NoOverlap maps to 416 with bytes */length; InvalidRange maps to a controlled 400. The application counts list members before parsing and returns full 200 above 16 to bound parser allocation. Returned start/length values are checked for zero length, overflow and representation bounds; overlaps also return full 200. Unknown units and zero-length files ignore Range, while duplicate Range headers remain rejected. The application accepts the library’s tolerance of skipping out-of-bounds starts first (for example bytes=99-2 on 10 bytes returns 416), without claiming stricter integer syntax. Multipart lengths use checked arithmetic and match streamed CRLF/body bytes. HEAD ignores Range/If-Range, returns complete metadata without opening a body stream, and still handles authorized 304. If-Range allows partial transfer only for a valid matching strong ETag of the selected representation; all date forms, weak/mismatched tags and malformed conditions return full 200 because current dates lack strong-validator evidence. Compressed image formats are excluded by the existing Tower compression predicate; bodies are not collected to compute validators.

Disable proxy buffering for progressive media delivery without changing authentication routing:

```nginx
location /api/static/ {
  proxy_pass http://recorder:5001;
  proxy_buffering off;
  gzip off;
}
```

Real-browser complete display and progressive screenshots are separate checks; complete decoding does not prove progressive rendering. Fixed-corpus checks use independent djxl/SSIMULACRA2 tools only for offline verification.

## Build and deployment

[Development verification](001-DEVELOPMENT-VERIFICATION.md) owns compilation configurations, native platform prerequisites and release checks. The single recorder executable embeds Rust JPXL and static libwebp/sharpyuv, aws-lc and other actual native dependencies. The project and JPXL are MIT; releases include dependency notices. Codec source stays in pinned Cargo Git/registry caches rather than repository copies.

Deployment needs configuration/data and HTTPS CA trust, without external codecs. Offline `recorder-cli media-smoke --output OUTPUT`, optionally `--input IMAGE`, exercises both production adapters without database, secrets or network initialization. It uses no worker protocol. Normal builds default to JXL + WebP; `--no-default-features` defaults to WebP. Explicit format lists, including `[]`, remain effective. Static defaults do not depend on R8.

## Configuration schema

Files use snake_case; environment variables are the exact uppercase path with `__` between segments. Use `_url` for connections, `_ms` or `_seconds` for durations, `_bytes` for budgets and `_concurrency`/`_capacity` for distinct execution limits. Do not add backend names to service contracts. Unknown fields in the root, database, auth, scheduler or media groups fail with redacted diagnostics. Defaults are typed; missing required credentials remain errors.

| Group | Ownership |
| --- | --- |
| `server`, `logger`, `storage`, `graphql`, `mikan` | HTTP, logging, files, GraphQL limits, source integration |
| `auth.provider` | `type`, Basic `username/password`, or OIDC `issuer/audience/client_id/client_secret/extra_scopes/extra_claims` |
| `auth.session` | optional `database_url` (inherits database.url), fixed `public_url`, `cookie_secure`, idle/absolute/login timeouts and cleanup interval in seconds |
| `database` | Default `url`; `pool.log_queries/min_connections/max_connections/connect_timeout_ms/idle_timeout_ms/acquire_timeout_ms`; optional owner `migration.url`, default-enabled `auto_run`, `legacy_oidc_issuer` |
| `scheduler.database/workers/cron` | Optional `url` (inherits database.url); `subscriber_concurrency/system_concurrency`; `poll_interval_seconds` |
| `media` | `auto_optimize_formats`; `encoding.webp.quality`, explicit `encoding.avif.quality/speed/threads`; `execution.deadline_seconds/queue_capacity/concurrency/working_set_bytes`; `limits.input_bytes/output_bytes/dimension/pixels/decode_bytes` |

The repository `recorder.config.toml` retains its existing pool policy: SQL logging enabled, both connection/idle timeouts at 500 ms and 1–10 connections. These explicit file overrides are distinct from typed defaults. Schema regrouping does not tune parameters; deployments can override their canonical paths. Startup runs migrations and capability grants before opening restricted runtime pools; `database.migration.auto_run` defaults to true. A separate owner URL and external migration control remain optional; see [013](002-AUTHENTICATION-DECISION.md) and [014](003-TASK-DELIVERY-AND-MIGRATION.md).

This is a breaking configuration change. Reorganize existing files and dotenv using this schema; remove HOST/DATABASE_URL aliases, flat auth fields, task.queue_database_uri and flat media knobs. A scheduler owns dispatch, execution, retries and cron; its public name is not a backend queue. Rust wire adapters preserve internal service/domain types without exposing their historical member names as configuration keys. The JPXL business preset remains compiled, with no invented quality mapping or backend knob.

Optional separate scheduler credentials can override the inherited database URL:

```toml
[scheduler.database]
url = "${SCHEDULER_URL}"
[scheduler.workers]
subscriber_concurrency = 2
system_concurrency = 2
[scheduler.cron]
poll_interval_seconds = 30
```

[English](004-MEDIA-AND-CONFIGURATION.md) | [中文](../zh/004-MEDIA-AND-CONFIGURATION.md)

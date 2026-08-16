# Konobangu

<img src="./assets/icon.png" alt="Konobangu" width="160" />

Self-hosted bangumi (anime) subscription and recording service: it watches configured sources, downloads matching episodes through a torrent client, and serves the collection through a web UI. Early-stage software with a Rust recorder, PostgreSQL, and a React WebUI.

## Install and run

Konobangu ships as a single `recorder-cli` executable that embeds the WebUI and both image codecs; it needs PostgreSQL 16+ and an OIDC provider (or a single-administrator Basic mode) for authentication. No public release has been tagged yet — see the [roadmap](docs/en/roadmap/001-SHORT-TERM-ROADMAP.md) for release status. Until then, build from source:

```sh
mise install                                   # Node, pnpm, Rust, just and other pinned tools
just setup                                      # frozen installs, workspace checks and git hooks
just build-release --target native              # or a cross target; see the development guide
```

Provide a `recorder.config.toml` next to the executable (or pass `--config-file`) with at least a database owner URL and one authentication provider; see [configuration](docs/en/004-MEDIA-AND-CONFIGURATION.md#configuration-sources) and [authentication](docs/en/002-AUTHENTICATION-DECISION.md#configuration-and-roles) for every field, including file-backed secrets for passwords and connection strings:

```toml
[database]
url = "postgres://konobangu:PASSWORD@localhost/konobangu"

[auth.provider]
type = "oidc" # or "basic" for a single administrator
issuer = "https://idp.example/"
audience = "konobangu"
client_id = "konobangu"
client_secret = "SERVER_ONLY_SECRET"

[auth.session]
public_url = "https://konobangu.example/"
```

Then run the executable; startup creates the restricted capability roles, applies migrations and opens its connection pools automatically:

```sh
./recorder-cli --environment production --config-file recorder.config.toml
```

## Documentation

[Documentation site](https://konobangu.apeiraco.com/)

| Topic | Document |
| --- | --- |
| Development, platform builds and verification | [Guide](docs/en/001-DEVELOPMENT-VERIFICATION.md) |
| Authentication and data access | [Contract](docs/en/002-AUTHENTICATION-DECISION.md) |
| Task delivery and database upgrades | [Operations](docs/en/003-TASK-DELIVERY-AND-MIGRATION.md) |
| Media and configuration | [Reference](docs/en/004-MEDIA-AND-CONFIGURATION.md) |
| Roadmap and remaining work | [Roadmap](docs/en/roadmap/001-SHORT-TERM-ROADMAP.md) |
| Animeta model proposal | [Proposal](docs/en/roadmap/002-ANIMETA-MODEL.md) |

The four guides above describe the current, running contract; `docs/{en,zh}/roadmap/` holds future decisions and proposals, not yet implemented. Release history: [CHANGELOG](CHANGELOG.md). Agent and contributor rules: [AGENTS.md](AGENTS.md). Historical iteration material under `temp/` is working residue, not a deployment manual.

### Contributing and sharing

Issues and pull requests are welcome on [GitHub](https://github.com/apeiraco/konobangu); start from the [development guide](docs/en/001-DEVELOPMENT-VERIFICATION.md) to prepare native Windows/macOS/Linux tools, install locked dependencies and run the verification gates (`just verify`) before sending a change. Keep current contracts in `docs/`, future proposals in `docs/*/roadmap/`, and delivered history in [CHANGELOG.md](CHANGELOG.md) — see [AGENTS.md](AGENTS.md) for the full project and documentation rules. Sharing this project (a link, a fork, a write-up) needs no separate permission beyond the license below.

## Repository layout

- `apps/recorder`: Rust HTTP/GraphQL service, migrations, task delivery and bounded in-process media encoding.
- `apps/webui`: React UI, SecurityDept authentication/RxSignal, injection-js DI and Apollo GraphQL data.
- `apps/docs`: VitePress documentation, projected from canonical repository documents through relative symlinks.
- `apps/proxy`, `apps/email-playground`: development tools.
- `packages/`: downloader, metadata, email and test helpers.
- `scripts/dev-cli.mts`: typed cross-platform development CLI, exposed through `just`.
- `justfiles/`: setup, development, build, quality, test, release and tool recipes; root justfile only configures/imports them.
- `build/`, `deploy/`: Docker build inputs and license notices.

## License and credits

MIT licensed, see [LICENSE](LICENSE); release artifacts bundle third-party notices generated from the locked dependency graph (see [deploy/licenses](deploy/licenses)). Konobangu embeds [JPXL](https://github.com/liminalism/JPXL) (also MIT) for JPEG XL encoding and static `libwebp`/`sharpyuv` for WebP. Contributions are credited through the Git history; see [CHANGELOG.md](CHANGELOG.md) for release notes.

[English](README.md) | [中文](docs/zh/README.md)

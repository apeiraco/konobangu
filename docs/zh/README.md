# Konobangu

<img src="../../assets/icon.png" alt="Konobangu" width="160" />

自托管番剧订阅与录制服务：监控配置的番剧源，通过 BT 客户端下载匹配分集，并用 Web UI 浏览收藏。基于 Rust recorder、PostgreSQL 和 React WebUI，仍处于早期阶段。

## 安装与运行

Konobangu 以单个 `recorder-cli` 可执行文件交付，内嵌 WebUI 和两种图片编码器；运行需要 PostgreSQL 16+，以及用于认证的 OIDC provider，或面向单管理员的 Basic 模式。尚未发布带 tag 的公开版本，发布状态见[路线图](roadmap/001-SHORT-TERM-ROADMAP.md)。目前从源码构建：

```sh
mise install                       # Node, pnpm, Rust, just and other pinned tools
just setup                         # frozen installs, workspace checks and git hooks
just build-release --target native # or a cross target; see the development guide
```

将 `recorder.config.toml` 放在可执行文件旁，或通过 `--config-file` 指定，至少提供数据库 owner URL 和一个认证 provider。全部字段见[配置](004-MEDIA-AND-CONFIGURATION.md#配置来源)与[认证](002-AUTHENTICATION-DECISION.md#配置与数据库角色)，包括通过文件提供密码和连接串等秘密配置的方式：

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

运行可执行文件；启动时自动建立受限能力角色、执行迁移并打开连接池：

```sh
./recorder-cli --environment production --config-file recorder.config.toml
```

## 文档

[文档站点](https://konobangu.apeiraco.com/zh/)

| 主题 | 文档 |
| --- | --- |
| 开发、平台构建与验证 | [指南](001-DEVELOPMENT-VERIFICATION.md) |
| 认证与数据访问 | [契约](002-AUTHENTICATION-DECISION.md) |
| 任务投递与数据库升级 | [运维](003-TASK-DELIVERY-AND-MIGRATION.md) |
| 媒体与配置 | [参考](004-MEDIA-AND-CONFIGURATION.md) |
| 路线图与后续工作 | [路线图](roadmap/001-SHORT-TERM-ROADMAP.md) |
| Animeta 模型提案 | [提案](roadmap/002-ANIMETA-MODEL.md) |

四份主题指南描述当前运行契约；`docs/{en,zh}/roadmap/` 维护未来决策与提案。发布历史见 [CHANGELOG](CHANGELOG.md)，Agent 与贡献规则见 [AGENTS.md](../../AGENTS.md)。`temp/` 中的轮次材料是工作证据，不是部署手册。

### 贡献与分享

欢迎在 [GitHub](https://github.com/apeiraco/konobangu) 提交 Issue 与 PR。提交前请按[开发指南](001-DEVELOPMENT-VERIFICATION.md)准备 Windows/macOS/Linux 工具、安装锁定依赖并运行 `just verify`。当前契约放在 `docs/`，未来提案放在 `docs/*/roadmap/`，交付历史放在 [CHANGELOG.md](CHANGELOG.md)；完整项目与文档规则见 [AGENTS.md](../../AGENTS.md)。分享本项目遵守许可证即可，无需额外许可。

## 仓库结构

- `apps/recorder`：Rust HTTP/GraphQL 服务、迁移、任务投递和有界进程内图片编码。
- `apps/webui`：React UI、SecurityDept 认证与 RxSignal、injection-js DI 和 Apollo GraphQL 数据。
- `apps/docs`：VitePress 文档站点，通过相对符号链接投影仓库文档真源。
- `apps/proxy`、`apps/email-playground`：开发工具。
- `packages/`：下载器、元数据、邮件和测试辅助库。
- `scripts/dev-cli.mts`：跨平台 TypeScript 开发工具，由 `just` 调度。
- `justfiles/`：安装、开发、构建、质量、测试、发布和工具 recipes。
- `build/`、`deploy/`：Docker 构建输入和许可证通知。

## 许可证与致谢

采用 MIT 许可证，见 [LICENSE](../../LICENSE)。发行制品包含基于锁定依赖图生成的第三方许可证通知，见 [deploy/licenses](../../deploy/licenses)。Konobangu 内嵌同样采用 MIT 协议的 [JPXL](https://github.com/liminalism/JPXL) 编码 JPEG XL，并静态链接 `libwebp`/`sharpyuv` 编码 WebP。贡献通过 Git 历史记录，发布说明见 [CHANGELOG.md](CHANGELOG.md)。

[English](../../README.md) | [中文](README.md)

# 001 — 短期路线图

本页只维护后续决策与验收条件。当前运行契约见 [认证](../002-AUTHENTICATION-DECISION.md)、[任务与升级](../003-TASK-DELIVERY-AND-MIGRATION.md)、[媒体与配置](../004-MEDIA-AND-CONFIGURATION.md)；已交付变更集中于 [CHANGELOG](../../../CHANGELOG.md)。轮次指导和证据属于 `temp/` 工作材料，不是部署操作手册。

## 发布门禁

本地 minor 发布候选已完成独立 review，CLI 干净检出的依赖安装顺序问题已关闭。自动发布仍有实施缺口，完成下述交付后才能视为自动发布流水线闭环；维护者目前仍可使用既有本地版本、tag 和发布工具。发布操作继续遵守 [开发验证契约](../001-DEVELOPMENT-VERIFICATION.md)；若继续修改生产源码或构建依赖，应按影响范围追加验证。架构与操作说明以对应的当前契约页为准。

后续 HMR 复审新发现开发门禁缺口：vite-plugin-monaco-editor 1.1.0 使用 Node 26 已移除的递归 rmdirSync，完整开发服务不能启动。发布前需修复 Monaco worker 集成，并在 mise 固定的 Node 上验证完整 Vite 开发配置、组件 Fast Refresh 和应用根更新；隔离 HMR fixture 通过不能关闭此项。本地复审记录位于 `temp/CONFIGURATION_BINARIES_RESOURCES_zh.md`。

## 保留的后续工作

| 工作项 | 方向 | 关闭条件 |
| --- | --- | --- |
| R6 模型工具链 | 以真实使用场景验证 Python/Animeta 模型加载、推理、数据集和资源成本；架构提案见 [Animeta](002-ANIMETA-MODEL.md) | 锁文件更新不能替代模型执行，需明确可复现样本、效果和资源预算 |
| R7 Awa 替换 | 默认候选为 Awa；等待运行时及官方 SeaORM 适配共同支持 SQLx 0.9 和项目 ORM 版本后启动实验 | 实验成功后完成正式迁移并独立验收，不能只提交探针 |
| R8 渐进 JXL | 保持静态 JXL 默认，真实浏览器验证响应尚未完成时出现可辨识预览和随后细化 | 从生产 HTTP 到实际页面完成录制；完整文件解码不等于渐进体验 |
| 会话存储适配 | 等待成熟 session store 支持 SQLx 0.9，收敛兼容连接池 | 保持持久撤销、重启、多实例、清理及总连接预算 |
| Rust stable | 上游 quirks_path 不再需要 nightly 后替换固定 nightly | Windows/macOS/Linux 默认、所有有效 feature 组合和裁剪分支全部通过 |

## 发布自动化

采用 Android-Credential-Provider-Fixer 的已测源码 dispatch 模型，与 SecurityDept 的方向一致；Outposts 的镜像构建/CD 模型无法覆盖本项目的跨平台发行 bundle。以下内容作为一次完整交付，不增加按功能域拆分的验证组。

### Metadata 选择与发布历史

```toml
[release.artifacts]
bundles = true
runtime_image = true
testing_torrents_image = false
```

三个独立布尔值位于 `konobangu-metadata.toml`，字段缺失、类型错误或未知字段均拒绝。它们选择发布内容，不裁剪测试覆盖。全 false 表示仅验证，不 dispatch、不审批、不创建 tag 或 GitHub Release。仅镜像发布仍创建版本 tag 和附带 manifest/digest 的 GitHub Release，不要求 bundle 压缩包。不在 IaC、仓库 variables 或 dispatch inputs 复制这些开关，也不提供覆盖已提交选择的命令参数。

`just release plan` 输出选择、source SHA、metadata SHA-256（统一检出换行后计算）与候选镜像标签，`--format github-output` 将同一计划提供给 job 条件。本地预览标记 dirty checkout；自动发布须从精确、干净、已测源码解析计划，重试也读取该 SHA 的 metadata，不读取分支新尖端。默认选中 bundle 与运行镜像，testing-torrents 显式 opt-in。主镜像发布与完整 release orchestrator 仍按下述步骤实施。

Git 历史证明的是发布意图，不能证明上传成功。release manifest 保存 metadata 快照/哈希、source SHA，以及实际成功的压缩包哈希、镜像 digest 和 workflow run ID。Bundle 回执已包含选择/哈希，本地发布拒绝 metadata 变化。Fixture publisher 导出 digest 和源码/配置哈希；其手动入口同样要求 release ref 与已提交的 opt-in 开关。候选镜像标签不含 `latest`，全部选中产物成功且稳定版完成后才统一提升该别名。同一版本公开后，重试不能改变选择或源码；调整发布范围应发布新版本，不重写历史。

### 发布内容

候选版本为 `0.1.0`，与 `konobangu-metadata.toml` 及双语 changelog 一致，对应 tag 为 `v0.1.0`。带版本的 changelog 不表示已完成发布。维护一个应用发行版本，不为每个 workspace 成员单独做 registry 发布。

| 交付项 | 发布决策 |
| --- | --- |
| 应用 bundle | 四个带 target 名称的压缩包：`x86_64-unknown-linux-gnu`、`x86_64-unknown-linux-musl`、`x86_64-pc-windows-msvc`、`aarch64-apple-darwin`。各包含 `recorder-cli`/`.exe`、`webui/`、LICENSE、CHANGELOG、notices 与 source/hash 回执。自动打包补充无 secrets 配置示例，不打包用户数据或凭据。 |
| 构建 profile | 发布默认 JXL + WebP、Rayon 配置；Chili、串行和 WebP-only 是验证/构建选项，不分别形成发行产品。 |
| 主运行镜像 | 发布 public `ghcr.io/apeiraco/konobangu`，首批为 `linux/amd64`，复用已验收 musl executable 与同版 WebUI。仅打包一次，使用包含 CA 证书的最小非 root runtime，不含 Cargo、Node 或 codec 构建工具。配置、secrets 和可写应用数据分别挂载；PostgreSQL、IdP 与 BT 客户端仍为外部服务。 |
| 镜像标签与元数据 | 使用完整版本和 `sha-<source SHA>` 标签，稳定版完成后才移动 `latest`。release manifest 记录镜像 digest。附带 OCI source/revision/version/license labels，关联源仓库，首次发布显式启用 public 可见性。Linux arm64 在具备原生制品验证后再增加。 |
| 工作区库 | Node 包保持 private，Rust crates 不发布。`email`、`testing`、recorder bindings、`util`、`util-derive`、`fetch`、`downloader` 和 Rust `animeta` 是内部构建依赖，交付应用，不拆为 npm/crates.io 发布。Python `konobangu-animeta` 与模型 demo 属于开发/研究工具，本期不发布 PyPI 包或模型。 |
| 开发二进制 | 不交付 `mikan-doppel`（mock 源）、`auth-test-server`、`testcontainers-prune`、examples 或 `animeta-agent-cli`（当前为空实现）。Email playground 预览/导出也是开发产物。 |
| 测试镜像 | `ghcr.io/apeiraco/konobangu-testing-torrents` 由 `release.artifacts.testing_torrents_image` 选择，release orchestrator 仅在选中时调用其 reusable publisher。默认 false，仍与主运行镜像职责不同。 |
| 构建镜像与文档 | musl builder 仅供 CI 本地使用，不作为用户镜像发布。文档由 `master` 的 Pages 流程发布，不将文档站或编译缓存/声明产物放进应用 bundle。 |

Docker 是主要自托管路径，因此主运行镜像与跨平台 bundle 放入本次完整发布交付。其 Dockerfile 与自动发布仍待实施；现有 musl Dockerfile 用于构建 executable，不能直接作为生产运行镜像。

1. `release` 推送的 `Verification report` 成功后，从已提交 metadata 解析选择，仅在至少一个产物被选中时携带精确 `source_sha`、`source_ref` 和 `tests_run_id` dispatch `release.yaml`，仅 dispatch job 获得 `actions: write`。允许按相同输入手动重试；PR、`dev`、默认分支和无关联 tag 推送不自动发布。
2. 通过 GitHub API 验证源仓库、verification workflow 身份、已完成且成功的运行、push 事件、release 分支和一致 commit。要求元数据同步及精确版本 CHANGELOG。检出已测试 SHA，不读取可变分支尖端；依赖 dispatch 前，workflow 文件必须先进入默认分支。
3. 在现有平台 job 上传已验证 recorder executable、notices 和目标回执，在现有 TypeScript job 上传生产 WebUI。当前 CI 只保留 smoke 回执，不足以发布。各制品记录 source SHA、版本、target 与哈希。发布消费该已验证运行的制品，不重复构建或门禁；重构本地与 CI 打包以共享源码/制品校验，不提供手动 `--verified` 开关。
4. 仅在 `bundles` 选中时，为 Linux GNU/musl x64、Windows MSVC x64、macOS arm64 生成带 target 名称的压缩包，包含 WebUI、LICENSE、CHANGELOG 和第三方 notices。验证候选来源及文件哈希，再生成 release manifest 和 SHA256SUMS。
5. 稳定版通过 `release` ref 的 `stable-release` 审批后发布，IaC 部署策略仅允许该分支，不允许 tag 来源。预发布状态按 SemVer 元数据判断。仅 tag/发布 job 获得 `contents: write`，仅选中的镜像发布 job 获得 `packages: write`，使用 `GITHUB_TOKEN`，不需要 npm/crates/PyPI 发布或签名 secrets，仓库默认权限仍为只读。对已验收产物打包运行镜像并 smoke，发布版本/source 标签时拒绝覆盖不同候选，manifest 记录 digest，稳定发布成功后再移动 `latest`。[GitHub registry 鉴权](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry#authenticating-to-the-container-registry)
6. 缺失版本 tag 时在已测 SHA 创建，已存在且指向其它 commit 时拒绝。先创建 draft、下载校验上传结果，再公开 release。重试接受完全相同的已发布版本，拒绝不同源码或制品，不能覆盖公开 release。manifest 记录测试 run ID 与 source SHA。
7. 验收一次真实 release 分支推送到验证、审批、可下载 bundle 和 public 可拉取运行镜像的完整路径，以及失败/重试、tag 冲突。镜像验证启动、WebUI/API 路由、secret 文件配置与持久数据挂载，仅 codec smoke 不足。按基础设施仓库的 import/plan/apply 流程应用清单；分支保护配置不等于发布成功。覆盖仅 bundle、仅运行镜像、仅 fixture 镜像、组合及全关闭的选择，不要求未选中的产物。fixture publisher 作为 reusable workflow 接入同一个 release orchestrator。

## R7 固定决策

Awa 实验不改变当前已验收的 Apalis 运行合约。开始前核对官方发布的兼容版本及许可证，不根据未来版本猜测提前接入。

1. 使用公开事务 API，在同一真实 SeaORM/SQLx 事务内验证 enqueue/rollback、Cron occurrence、业务结果发布；不得另开连接假装同一事务，或依赖内部表。
2. 保持 app_scoped、auth、task_control 能力边界及仅 owner 执行的迁移阶段。覆盖多根 mutation 后续失败、跨 owner 拒绝，以及 retry/cancel/delete 的事务边界。
3. 实际 worker 覆盖重复投递、跨续期长任务、失联、取消恢复、尝试预算、迟到结果、资源删除、重启和双实例 Cron。保留时区/DST、停机合并和不重叠语义。
4. 列出能够删除的 lease/retry/recovery 代码，以及仍必要的业务 fence、历史投影和恢复逻辑。若仍需完整重复执行器，或弱化正确性，判定实验失败并保留当前方案。
5. 通过后在同一交付阶段完成运行时、生命周期、角色、API/页面、codegen、CI、文档和旧机制删除。非空数据库采用停写/排空/导入及备份恢复，保留业务 ID、owner、历史和终态，不清库、不并行消费新旧队列。

## R8 固定决策

遵守客户端 Accept/q/exclusion，优先可用渐进 JXL → 普通 JXL → WebP → 原图。特性优先，不因体积微小差异重复编码或降级；不自动开启 AVIF。保留原图、固定单次编码配置、权限和发布 fence。静态默认不依赖渐进门禁，也不能把静态验收写成 R8 完成。

受控 Linux 主机或 SSH 原生验证可作为证据；配置 CI 与实际执行 CI 分别记录。后续轮次按完整交付主题组织，避免把相互依赖的实验、实现和清理拆成大量小轮次。

[English](../../en/roadmap/001-SHORT-TERM-ROADMAP.md) | [中文](001-SHORT-TERM-ROADMAP.md)

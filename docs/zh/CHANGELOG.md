# 更新记录

## 0.1.0

### 新增

- 持久化 OIDC/PKCE 会话、隔离数据库角色与 owner 作用域 GraphQL 事务。
- 持久化任务 runs/outbox、fence 执行、取消恢复与时区感知 Cron 调度。
- 原生配置插值、文件 secrets 和脱敏诊断。
- 带版本的图片衍生、有界媒体执行与来源可追溯的 backfill。
- 跨平台开发与发布工具，以及应用、迁移和平台验证。
- 基于 README 的双语 VitePress 文档站、本地搜索与 konobangu.apeiraco.com 的 GitHub Pages 发布。

### 调整

- 升级依赖，采用 injection-js、SecurityDept RxSignal、Apollo 和 TanStack Table 9。
- 按应用作用域、认证和任务控制命名数据库权限；从单个数据库 URL 初始化 NOLOGIN 能力角色，由 Rust 迁移标识符生成授权，合并未发布迁移。
- 使用 RFC3339 时间戳与 Temporal，在应用启动前加载缺失的浏览器能力。
- 封面使用锁定的 JPXL Balanced 77 与静态 WebP80，通过自有 Rayon pool 上的单一有界执行器完成；保留原图，透明封面合成白底。
- 保留生成 bindings 的格式，Node 工具使用原生 ESM 路径元数据。
- 使用 Vite 邮件预览与导出、Playwright 管理 fixtures、unit/integration/e2e 测试目录，并共享 Rust 任务迁移标识符。
- Just 与 CI 共享按资源划分的验证阶段，隔离浏览器 fixture 构建，并以同一 TypeScript 引用图管理输出。
- 验证仅由默认/release 分支推送及目标为这两个分支的 PR 触发，提供统一分支保护汇总检查。
- 通过已提交 metadata 选择 bundle/运行镜像/fixture 发布，发布计划与 bundle 回执记录源码/配置哈希。
- 整理 CLI/Just 工具和生成输出，恢复 Zellij sessions，使用 kebab-case 可执行文件名，配置按服务分组并标明单位。

### 修复

- 图片格式协商时保留授权、缓存 validator 与 Range 行为。
- 任务发布、取消、重试和删除受事务与租约 fence 保护。
- 干净 CI 检出先安装锁定 CLI 依赖再运行工具，并应用文件日志阈值。
- 发布元数据更新保留清单换行格式，发布说明支持 CRLF。
- 通过服务器 hash 核验处理 qBittorrent 重复添加冲突，并按 digest 固定集成测试镜像。
- 媒体任务先释放预算再通知完成，避免后续任务因资源释放竞态被错误限流。

### 移除

- 浏览器 token 存储和 Bearer/token 端点、Tera、Jotai/DI forks 与未使用的日期适配器。
- Codec 子进程、libjxl 构建流程和受版本控制的编译器声明产物。

### 升级说明

备份数据库与存储，停止写入者和 workers，并执行 owner-only 迁移；数据库回滚需要恢复备份。默认启动从单个数据库 URL 初始化受限连接池，也可使用独立运行或 owner 凭据。替换配置模板、平铺 auth/task/media 字段和旧 JXL 选项。自动封面保持 JXL + WebP；JPXL 当前输出普通单 pass 图片。

参见[认证](002-AUTHENTICATION-DECISION.md)、[升级](003-TASK-DELIVERY-AND-MIGRATION.md)和[媒体与配置](004-MEDIA-AND-CONFIGURATION.md)。

[English](../../CHANGELOG.md) | [中文](CHANGELOG.md)

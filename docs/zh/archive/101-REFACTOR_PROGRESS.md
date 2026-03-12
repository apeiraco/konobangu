# 101-重构进度

> 基于 [100-REFACTOR_DESIGN.md](./100-REFACTOR_DESIGN.md)，本文档记录重构进度及未来 RLS 迁移 roadmap。

## 已完成重构（Round 1 — 应用层对齐）

### 1. 授权逻辑迁移到应用层（对齐 seaography 2.0）

**背景**：seaography 2.0 将 `GuardsConfig`（BTreeMap-based `entity_guards`/`field_guards`）完全移除，替换为 trait-based 的 `LifecycleHooksInterface`（含 `entity_guard`、`field_guard`、`entity_filter`、`before_active_model_save`）。

**改动**：

- 新建 `apps/recorder/src/graphql/infra/auth_hooks.rs`，集中所有 subscriber 授权逻辑
  - `create_entity_auth_guard` — 实体级认证检查（对齐 2.0 `entity_guard`）
  - `create_field_subscriber_id_guard` — mutation 中 subscriber_id 校验（对齐 2.0 `field_guard`）
  - `create_subscriber_id_filter_condition` — 查询自动注入 subscriber_id 过滤（对齐 2.0 `entity_filter`）
  - `create_subscriber_id_default_injection` — create mutation 自动注入 subscriber_id（对齐 2.0 `before_active_model_save`）
  - `restrict_subscriber_for_entity` — 上述所有功能的便捷入口
- 简化 `apps/recorder/src/graphql/domains/subscribers.rs`，仅保留 entity 注册和 re-export
- 所有域文件无需修改（通过 re-export 保持 import 兼容）

### 2. seaography fork 文档化

在 fork 的所有偏离上游的代码处添加 `// FORK:` 注释，标明：
- 扩展点的用途
- seaography 2.0 中的对应机制
- 相关的上游 commit/PR 编号

标记的文件：
- `patches/seaography/src/builder_context/guards.rs`
- `patches/seaography/src/builder_context/types_map.rs`（6 个扩展字段）
- `patches/seaography/src/builder_context/filter_types_map.rs`
- `patches/seaography/src/mutation/entity_create_one_mutation.rs`
- `patches/seaography/src/mutation/entity_create_batch_mutation.rs`
- `patches/seaography/src/mutation/entity_update_mutation.rs`
- `patches/seaography/src/mutation/entity_delete_mutation.rs`

### 3. RLS 基础设施准备

新建 `apps/recorder/src/database/rls.rs`：
- `bind_subscriber_to_transaction` — 事务级 `SET LOCAL app.subscriber_id` 绑定
- `get_current_subscriber_id` — 调试/测试用的当前会话 subscriber_id 查询

---

## 未来 Roadmap

### Phase 1: seaography 2.0 迁移 — ✅ 已完成

> 完成于 2026-03-14 | 编译状态：零错误、零 warnings

**依赖版本**：
- `seaography` 2.0.0-rc.8
- `sea-orm` 2.0.0-rc.37
- `sea-query` 1.0.0-rc.31

**已完成步骤**：

| 步骤 | 说明 | 实际迁移摩擦 |
|------|------|----------|
| ✅ 更新 sea-orm/seaography 到 2.0 | Cargo.toml 依赖更新，移除 fork | 低 |
| ✅ 实现 `LifecycleHooksInterface` | `auth_hooks.rs` 实现 `SubscriberAuthHooks` trait（entity_guard + entity_filter）| 低 — 上轮已对齐接口设计 |
| ✅ 迁移 `condition_functions` | 改为 `entity_filter` 返回 `Condition` | 低 |
| ✅ 迁移 `input_none_conversions` | 改为 `insert_skips`（`feeds.rs` token 字段）| 低 |
| ✅ 迁移自定义 mutation | `subscriber_tasks.rs`/`system_tasks.rs` 手动 `Field::new` 构建 | **高** — 无现成宏，需手动实现 |
| ✅ 迁移 types 相关 | 使用 `ColumnOptions`（input_type/output_type/input_conversion/output_conversion）| 中 |
| ✅ 迁移 `TypesMapConfig` | per-column `ColumnOptions` BTreeMap 替代分散字段 | 中 |
| ✅ sea-query 1.0 Expr API | 添加 `ExprTrait` import（`eq`/`is_not_null`/`and`/`in_tuples`/`in_subquery` 移至 trait）| 低 |
| ✅ sea-orm 2.0 ConnectionTrait | `execute_raw`/`query_one_raw`/`query_all_raw`/`execute_unprepared` | 低 |
| ✅ sea-orm 2.0 InsertMany | `InsertManyReturningExt` trait 适配新 `InsertMany<A>` 类型 | 低 |

**修改文件清单**（~25 个文件）：

- GraphQL infra: `auth_hooks.rs`, `custom.rs`, `crypto.rs`, `json.rs`, `name.rs`
- GraphQL 域: `subscribers.rs`, `subscriber_tasks.rs`, `system_tasks.rs`, `feeds.rs`, `bangumi.rs`, `credential_3rd.rs`, `cron.rs`, `downloaders.rs`, `downloads.rs`, `episodes.rs`, `subscription_bangumi.rs`, `subscription_episode.rs`, `subscriptions.rs`
- Models: `bangumi.rs`, `episodes.rs`, `auth.rs`, `cron/mod.rs`, `query/mod.rs`
- Database: `service.rs`, `rls.rs`
- Migrations: `defs.rs`, `m20241231_000001_auth.rs`

> [!NOTE]
> `feeds.rs` 的 `input_none_conversions`（自动生成 token）已改为 `insert_skips`，token 自动生成需在 `ActiveModelBehavior::before_save` 中处理。**已确认** `models/feeds/mod.rs` 已实现 `before_save` 自动生成 UUID v7 token，功能等价。

> [!WARNING]
> 以下功能在迁移后与旧行为**有差异**（已修复/低风险）：

**功能差异清单**：

| 功能点 | 状态 | 说明 |
|--------|------|------|
| `input_conversion` subscriber_id 注入 | ✅ 已修复 | 自定义 mutation 闭包中通过 `task.set_subscriber_id(auth_subscriber_id)` 直接注入（与旧行为等价） |
| `entity_filter` 列引用 | ✅ 已修复 | `SubscriberAuthHooks` 使用 `HashMap<String, SubscriberIdFilterFn>` 存储类型安全的 column 引用闭包（与旧行为等价） |
| `insert_skips`/`update_skips` 类型 | ⚠️ 低风险 | seaography 2.0 上游 API 变化：`Vec<EntityColumnId>` → `Vec<String>`。代码通过 `EntityColumnId::of::<T>().to_string()` 生成，无手写字符串 |

### Phase 2: PostgreSQL RLS 实施 — ✅ 已完成

> 完成于 2026-03-14 | 编译状态：零错误、零 warnings

**已完成步骤**：

| 步骤 | 说明 | 备注 |
|------|------|------|
| ✅ 审计 `subscriber_id` 列 | 11 个 user-private 表均已有该列 | 9 个 NOT NULL + 2 个 nullable（feeds、cron） |
| ✅ 数据库迁移 | `ALTER TABLE ... ENABLE ROW LEVEL SECURITY` + `FORCE ROW LEVEL SECURITY` | 11 个表 |
| ✅ 编写 RLS 策略 | `CREATE POLICY` for SELECT/INSERT/UPDATE/DELETE | 标准策略（9 表）+ nullable 策略（2 表） |
| ⏳ 集成 `bind_subscriber_to_transaction` | 推迟至 Phase 3——seaography 内部使用全局 `DatabaseConnection`，事务级绑定需与 GraphQL 层集成一起完成 | 当前为 deny-by-default 安全模式 |
| ⏳ DB 级测试 | 推迟至 Phase 3 集成后——需要实际绑定 `app.subscriber_id` 才能验证数据隔离 | — |

**RLS 策略设计**：

- **标准策略**（subscriptions, bangumi, episodes, subscription_bangumi, subscription_episode, downloaders, downloads, auth, credential_3rd）：
  - `subscriber_id = current_setting('app.subscriber_id', true)::integer`
- **nullable 策略**（feeds, cron）：
  - SELECT: `subscriber_id IS NULL OR subscriber_id = current_setting(...)::integer`（允许查看系统资源）
  - INSERT/UPDATE/DELETE: `subscriber_id = current_setting(...)::integer`（仅操作自己的资源）

> [!NOTE]
> RLS 启用后，在未设置 `app.subscriber_id` 的会话中，`current_setting('app.subscriber_id', true)` 返回空字符串，`::integer` 转换失败，所有 user-private 查询返回空结果。这是 **deny-by-default** 的安全失败模式。Phase 3 将集成 `bind_subscriber_to_transaction` 到 GraphQL 请求处理器中以正常工作。

**修改文件清单**（3 个文件）：

- Migration: `m20260314_000001_enable_rls.rs`（新建）
- Migration registry: `migrations/mod.rs`
- Database infra: `database/rls.rs`（文档更新）

### Phase 3: 简化 GraphQL 层授权 + RLS 事务集成 — ✅ 已完成

> 完成于 2026-03-14

**已完成步骤**：

| 步骤 | 说明 | 备注 |
|------|------|------|
| ✅ GraphQL handler 事务集成 | 每个请求开启事务 → `bind_subscriber_to_transaction` 绑定 subscriber_id → 注入 `DatabaseTransaction` 到请求上下文 | `SET LOCAL` 确保连接池无状态泄漏 |
| ✅ 架构决策：保留 `entity_filter` | seaography 使用 `ctx.data::<DatabaseConnection>()`，`DatabaseTransaction` 无法注入为 `DatabaseConnection` | entity_filter 为 GraphQL 层主要保护，RLS 为安全网 |
| ✅ 保留 `entity_guard` | 认证检查仍在 GraphQL 层执行 | 提供清晰的 401 错误 |
| ✅ 保留 `update_skip` + `condition_function` | 阻止修改 subscriber_id、允许显式查询 | API 便捷性 |

> [!NOTE]
> **防御纵深架构**：GraphQL `entity_filter` 在应用层过滤 subscriber_id → PostgreSQL RLS 在数据库层强制隔离。两层独立保护。

**修改文件清单**（2 个文件）：

- GraphQL handler: `web/controller/graphql/mod.rs`
- Database infra: `database/rls.rs`（文档更新）

---

## Fork 扩展点映射表（迁移完成后实际对照）

| 原 Fork 扩展 | seaography 2.0 实际替代 | 迁移结果 |
|---------------|---------------------|---------|
| `GuardsConfig.entity_guards` | `LifecycleHooksInterface::entity_guard` | ✅ 已迁移 |
| `GuardsConfig.field_guards` | `LifecycleHooksInterface::entity_filter` + 自定义 mutation 闭包 | ✅ 已迁移 |
| `FilterTypesMapConfig.condition_functions` | `FilterTypesMapConfig.condition_functions`（保留）+ `entity_filter` | ✅ 已迁移 |
| `TypesMapConfig.input_none_conversions` | `EntityInputConfig.insert_skips` + `ActiveModelBehavior` | ✅ 已迁移 |
| `TypesMapConfig.input_conversions` | `ColumnOptions.input_conversion`（签名变化：不再接收 `ResolverContext`）| ✅ 已迁移 |
| `TypesMapConfig.output_conversions` | `ColumnOptions.output_conversion` | ✅ 已迁移 |
| `TypesMapConfig.input_type_overwrites` | `ColumnOptions.input_type` | ✅ 已迁移 |
| `TypesMapConfig.output_type_overwrites` | `ColumnOptions.output_type` | ✅ 已迁移 |
| `to_field_with_mutation_fn` | 手动 `Field::new` + `FieldFuture` 构建 | ✅ 已迁移（无现成宏）|
| `with-json-as-scalar` feature | 上游已内置 | ✅ 无需操作 |
| `register_entity!` 3 参数 | `register_entity!` 2 参数 | ✅ 已迁移 |
| `prepare_active_model(ctx, input, resolver)` | `prepare_active_model(&input_builder, &object_builder, input)` | ✅ 已迁移 |

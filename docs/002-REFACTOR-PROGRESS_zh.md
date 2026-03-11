# 重构进度：降低 seaography fork 对上游的偏离

> 基于 [001-REFACTOR-DESIGN_zh.md](./001-REFACTOR-DESIGN_zh.md)，本文档记录重构进度及未来 RLS 迁移 roadmap。

## 本次重构 — 已完成

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

### Phase 1: seaography 2.0 迁移

> 优先级：**高** | 依赖：上游 2.0 稳定版发布

| 步骤 | 说明 | 迁移摩擦 |
|------|------|----------|
| 更新 sea-orm 到 2.0 | Cargo.toml 依赖更新 | 低 — 独立操作 |
| 更新 seaography 到 2.0 | 移除 fork，使用上游 | 中 — 本次的 FORK 注释提供清晰的迁移路线 |
| 实现 `LifecycleHooksInterface` | 将 `auth_hooks.rs` 中的函数改为实现 trait | **低** — 本次已对齐接口设计 |
| 迁移 `condition_functions` | 改为 `entity_filter` | **低** — 逻辑已集中在 `auth_hooks.rs` |
| 迁移 `input_none_conversions` | 改为 `before_active_model_save` | **低** — 逻辑已集中 |
| 迁移自定义 mutation | 使用 2.0 的 custom mutation 宏 | 中 — 涉及 `custom.rs` 和域文件 |
| 迁移 types 相关 | 使用 2.0 的 per-column type options (#224) | 中 — 涉及 `credential_3rd`、task 类型 |

### Phase 2: PostgreSQL RLS 实施

> 优先级：**中** | 依赖：Phase 1 完成后开始

| 步骤 | 说明 |
|------|------|
| 审计 `subscriber_id` 列 | 确认所有 user-private 表都有该列 |
| 数据库迁移 | `ALTER TABLE ... ENABLE ROW LEVEL SECURITY` + `FORCE ROW LEVEL SECURITY` |
| 编写 RLS 策略 | `CREATE POLICY` for SELECT/INSERT/UPDATE/DELETE |
| 集成 `bind_subscriber_to_transaction` | 在 GraphQL 请求处理器中绑定 subscriber_id 到事务 |
| DB 级测试 | 验证 user A 无法访问 user B 的数据 |

### Phase 3: 简化 GraphQL 层授权

> 优先级：**低** | 依赖：Phase 2 验证通过后

| 步骤 | 说明 |
|------|------|
| 移除冗余 guards | RLS 已保证隔离后，Guards 可简化为仅做早期拒绝 |
| 移除冗余 filter conditions | RLS 已自动过滤，不再需要应用层重复过滤 |
| 保留 input_none_conversions | subscriber_id 自动注入仍需在 GraphQL 层完成 |

---

## Fork 扩展点映射表

| 当前 Fork 扩展 | seaography 2.0 替代 | 迁移难度 |
|---------------|---------------------|---------|
| `GuardsConfig.entity_guards` | `LifecycleHooksInterface::entity_guard` | 低 |
| `GuardsConfig.field_guards` | `LifecycleHooksInterface::field_guard` | 低 |
| `FilterTypesMapConfig.condition_functions` | `LifecycleHooksInterface::entity_filter` | 低 |
| `TypesMapConfig.input_none_conversions` | `LifecycleHooksInterface::before_active_model_save` | 低 |
| `TypesMapConfig.input_conversions` | Custom input type derive macros (#203) | 中 |
| `TypesMapConfig.output_conversions` | Output conversion macros (#217, #220) | 中 |
| `TypesMapConfig.input_type_overwrites` | Per-column type options (#224) | 中 |
| `TypesMapConfig.output_type_overwrites` | Per-column type options (#224) | 中 |
| `to_field_with_mutation_fn` | Custom mutation macros (#193) + lifecycle hooks | 中 |
| `with-json-as-scalar` feature | Upstream #222 | 无 — 已合入上游 |

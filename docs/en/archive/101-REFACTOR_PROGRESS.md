# 101-Refactor Progress

> Based on [100-REFACTOR_DESIGN.md](./100-REFACTOR_DESIGN.md), this document records refactoring progress and the future RLS migration roadmap.

## Completed Refactoring (Round 1 — Application Layer Alignment)

### 1. Authorization Logic Migration to Application Layer (Aligned with seaography 2.0)

**Background**: seaography 2.0 completely removed `GuardsConfig` (BTreeMap-based `entity_guards`/`field_guards`), replacing it with a trait-based `LifecycleHooksInterface` (containing `entity_guard`, `field_guard`, `entity_filter`, `before_active_model_save`).

**Changes**:

- Created `apps/recorder/src/graphql/infra/auth_hooks.rs`, centralizing all subscriber authorization logic
  - `create_entity_auth_guard` — Entity-level authentication checks (aligned with 2.0 `entity_guard`)
  - `create_field_subscriber_id_guard` — subscriber_id validation in mutations (aligned with 2.0 `field_guard`)
  - `create_subscriber_id_filter_condition` — Automatic subscriber_id filtering injection in queries (aligned with 2.0 `entity_filter`)
  - `create_subscriber_id_default_injection` — Automatic subscriber_id injection in create mutations (aligned with 2.0 `before_active_model_save`)
  - `restrict_subscriber_for_entity` — Convenient entry point for all above features
- Simplified `apps/recorder/src/graphql/domains/subscribers.rs`, keeping only entity registration and re-export
- No modifications needed for domain files (import compatibility maintained through re-exports)

### 2. seaography Fork Documentation

Added `// FORK:` comments at all code locations where the fork diverges from upstream, specifying:

- Purpose of the extension point
- Corresponding mechanism in seaography 2.0
- Related upstream commit/PR numbers

Marked files:

- `patches/seaography/src/builder_context/guards.rs`
- `patches/seaography/src/builder_context/types_map.rs` (6 extension fields)
- `patches/seaography/src/builder_context/filter_types_map.rs`
- `patches/seaography/src/mutation/entity_create_one_mutation.rs`
- `patches/seaography/src/mutation/entity_create_batch_mutation.rs`
- `patches/seaography/src/mutation/entity_update_mutation.rs`
- `patches/seaography/src/mutation/entity_delete_mutation.rs`

### 3. RLS Infrastructure Preparation

Created `apps/recorder/src/database/rls.rs`:

- `bind_subscriber_to_transaction` — Transaction-level `SET LOCAL app.subscriber_id` binding
- `get_current_subscriber_id` — Current session subscriber_id query for debugging/testing

---

## Future Roadmap

### Phase 1: seaography 2.0 Migration — ✅ Completed

> Completed on 2026-03-14 | Build status: Zero errors, Zero warnings

**Dependency versions**:

- `seaography` 2.0.0-rc.8
- `sea-orm` 2.0.0-rc.37
- `sea-query` 1.0.0-rc.31

**Completed steps**:

| Step | Description | Actual Migration Friction |
|------|-------------|--------------------------|
| ✅ Update sea-orm/seaography to 2.0 | Cargo.toml dependency update, remove fork | Low |
| ✅ Implement `LifecycleHooksInterface` | `auth_hooks.rs` implements `SubscriberAuthHooks` trait (entity_guard + entity_filter) | Low — interfaces aligned in previous round |
| ✅ Migrate `condition_functions` | Changed to `entity_filter` returning `Condition` | Low |
| ✅ Migrate `input_none_conversions` | Changed to `insert_skips` (token field in `feeds.rs`) | Low (**Note**: name format bug discovered post-migration, see Bug Fixes section) |
| ✅ Migrate custom mutations | `subscriber_tasks.rs`/`system_tasks.rs` manual `Field::new` construction | **High** — No existing macro, manual implementation required |
| ✅ Migrate types-related | Using `ColumnOptions` (input_type/output_type/input_conversion/output_conversion) | Medium |
| ✅ Migrate `TypesMapConfig` | Per-column `ColumnOptions` BTreeMap instead of scattered fields | Medium |
| ✅ sea-query 1.0 Expr API | Added `ExprTrait` import (`eq`/`is_not_null`/`and`/`in_tuples`/`in_subquery` moved to trait) | Low |
| ✅ sea-orm 2.0 ConnectionTrait | `execute_raw`/`query_one_raw`/`query_all_raw`/`execute_unprepared` | Low |
| ✅ sea-orm 2.0 InsertMany | `InsertManyReturningExt` trait adapting new `InsertMany<A>` type | Low |

**Modified files list** (~25 files):

- GraphQL infra: `auth_hooks.rs`, `custom.rs`, `crypto.rs`, `json.rs`, `name.rs`
- GraphQL domains: `subscribers.rs`, `subscriber_tasks.rs`, `system_tasks.rs`, `feeds.rs`, `bangumi.rs`, `credential_3rd.rs`, `cron.rs`, `downloaders.rs`, `downloads.rs`, `episodes.rs`, `subscription_bangumi.rs`, `subscription_episode.rs`, `subscriptions.rs`
- Models: `bangumi.rs`, `episodes.rs`, `auth.rs`, `cron/mod.rs`, `query/mod.rs`
- Database: `service.rs`, `rls.rs`
- Migrations: `defs.rs`, `m20241231_000001_auth.rs`

> [!NOTE]
> `feeds.rs` `input_none_conversions` (auto-generate token) changed to `insert_skips`. Token auto-generation needs to be handled in `ActiveModelBehavior::before_save`. **Confirmed** `models/feeds/mod.rs` has implemented `before_save` for automatic UUID v7 token generation, functionally equivalent.

> [!WARNING]
> The following features have **differences** from old behavior after migration (already fixed/low risk):

**Feature difference list**:

| Feature | Status | Description |
|---------|--------|-------------|
| `input_conversion` subscriber_id injection | ✅ Fixed | Custom mutation closures directly inject via `task.set_subscriber_id(auth_subscriber_id)` (equivalent to old behavior) |
| `entity_filter` column reference | ✅ Fixed | `SubscriberAuthHooks` uses `HashMap<String, SubscriberIdFilterFn>` storing type-safe column reference closures (equivalent to old behavior) |
| `insert_skips`/`update_skips` name format | ✅ Fixed | **Root cause**: `EntityColumnId::to_string()` returns database-layer format (`"feeds.subscriber_id"`), but `insert_skips` checks use GraphQL-layer format (`"Feeds.subscriberId"`), causing all skips to silently fail. **Fix**: uniformly use `get_entity_and_column_name<T>(context, column)` to generate the correct format (see Bug Fixes section) |

### Phase 2: PostgreSQL RLS Implementation — ✅ Completed

> Completed on 2026-03-14 | Build status: Zero errors, Zero warnings

**Completed steps**:

| Step | Description | Notes |
|------|-------------|-------|
| ✅ Audit `subscriber_id` columns | All 11 user-private tables have this column | 9 NOT NULL + 2 nullable (feeds, cron) |
| ✅ Database migration | `ALTER TABLE ... ENABLE ROW LEVEL SECURITY` + `FORCE ROW LEVEL SECURITY` | 11 tables |
| ✅ Write RLS policies | `CREATE POLICY` for SELECT/INSERT/UPDATE/DELETE | Standard policies (9 tables) + nullable policies (2 tables) |
| ⏳ Integrate `bind_subscriber_to_transaction` | Deferred to Phase 3 — seaography uses global `DatabaseConnection`, transaction-level binding needs GraphQL layer integration | Currently in deny-by-default security mode |
| ⏳ DB-level tests | Deferred to Phase 3 after integration — requires actual `app.subscriber_id` binding to verify data isolation | — |

**RLS policy design**:

- **Standard policies** (subscriptions, bangumi, episodes, subscription_bangumi, subscription_episode, downloaders, downloads, auth, credential_3rd):
  - `subscriber_id = current_setting('app.subscriber_id', true)::integer`
- **Nullable policies** (feeds, cron):
  - SELECT: `subscriber_id IS NULL OR subscriber_id = current_setting(...)::integer` (allows viewing system resources)
  - INSERT/UPDATE/DELETE: `subscriber_id = current_setting(...)::integer` (only operate on own resources)

> [!NOTE]
> After RLS is enabled, in sessions where `app.subscriber_id` is not set, `current_setting('app.subscriber_id', true)` returns an empty string, `::integer` conversion fails, and all user-private queries return empty results. This is a **deny-by-default** security failure mode. Phase 3 will integrate `bind_subscriber_to_transaction` into the GraphQL request handler for proper operation.

**Modified files list** (3 files):

- Migration: `m20260314_000001_enable_rls.rs` (new)
- Migration registry: `migrations/mod.rs`
- Database infra: `database/rls.rs` (documentation update)

### Phase 3: Simplified GraphQL Layer Authorization + RLS Transaction Integration — ✅ Completed

> Completed on 2026-03-14

**Completed steps**:

| Step | Description | Notes |
|------|-------------|-------|
| ✅ GraphQL handler transaction integration | Each request opens transaction → `bind_subscriber_to_transaction` binds subscriber_id → injects `DatabaseTransaction` into request context | `SET LOCAL` ensures connection pool is stateless |
| ✅ Architecture decision: Keep `entity_filter` | seaography uses `ctx.data::<DatabaseConnection>()`, `DatabaseTransaction` cannot be injected as `DatabaseConnection` | entity_filter is the main GraphQL layer protection, RLS is the safety net |
| ✅ Keep `entity_guard` | Authentication checks still execute in GraphQL layer | Provides clear 401 errors |
| ✅ Keep `update_skip` + `condition_function` | Blocks subscriber_id modification, allows explicit queries | API convenience |

> [!NOTE]
> **Defense in depth architecture**: GraphQL `entity_filter` filters subscriber_id at application layer → PostgreSQL RLS enforces isolation at database layer. Two independent protection layers.

**Modified files list** (2 files):

- GraphQL handler: `web/controller/graphql/mod.rs`
- Database infra: `database/rls.rs` (documentation update)

---

## Bug Fixes (Discovered Post-Migration)

### BF-1: `insert_skips` / `update_skips` Name Format Mismatch Causes Skips to Silently Fail

**Discovered**: 2026-03-17

**Symptom**: GraphQL frontend `FeedsInsertInput` had `token` field still as `NON_NULL` (required), and `SubscriptionsInsertInput`, `Credential3rdInsertInput`, etc. had `subscriberId` still as `NON_NULL`, causing TypeScript type errors on the frontend.

**Root Cause**:

`insert_skips` / `update_skips` are `Vec<String>`, and the check logic inside seaography (`seaography/src/inputs/entity_input.rs`) uses the **GraphQL-layer format**:

```rust
// seaography internal check
let full_name = format!("{}.{}", entity_object_builder.type_name::<T>(), column_name);
// produces "Feeds.subscriberId" (PascalCase entity name + camelCase column name)
self.context.entity_input.insert_skips.contains(&full_name)
```

But the migrated code incorrectly used `EntityColumnId::to_string()`:

```rust
// ❌ Wrong: returns database format "feeds.subscriber_id" (snake_case)
context.entity_input.insert_skips.push(entity_column_id.to_string());
```

The two formats never match, so every value pushed into the skip list can never be found by `contains` — all skips silently fail.

**Affected Scope**:
- `feeds.rs`: `token` field `insert_skips` fails → `FeedsInsertInput.token` remains `NON_NULL`
- `auth_hooks.rs`: `subscriberId` `insert_skips` and `update_skips` in `restrict_subscriber_for_entity` both fail → all entities registered through this function (Subscriptions, Credential3rd, Bangumi, Episodes, Downloaders, Downloads, Feeds, etc.) have `NON_NULL subscriberId` in their InsertInput
- `subscriber_tasks.rs` / `system_tasks.rs`: skips in `skip_columns_for_entity_input` and `restrict_*_for_entity` all fail

**Fix**:

Uniformly replaced with `get_entity_and_column_name<T>(context, column)`, which generates GraphQL-format names consistent with seaography's check logic via `context.entity_object.type_name` and `context.entity_object.column_name`:

```rust
// ✅ Correct: returns GraphQL format "Feeds.subscriberId"
let entity_column_key = get_entity_and_column_name::<T>(context, column);
context.entity_input.insert_skips.push(entity_column_key.clone());
context.entity_input.update_skips.push(entity_column_key);
```

**Modified Files**:
- `apps/recorder/src/graphql/infra/auth_hooks.rs`: `restrict_subscriber_for_entity`
- `apps/recorder/src/graphql/domains/feeds.rs`: `register_feeds_to_schema_context`
- `apps/recorder/src/graphql/domains/subscriber_tasks.rs`: `skip_columns_for_entity_input` + `restrict_subscriber_tasks_for_entity`
- `apps/recorder/src/graphql/domains/system_tasks.rs`: `skip_columns_for_entity_input` + `restrict_system_tasks_for_entity`

**Verification**: Post-fix, GraphQL introspection confirms `subscriberId` and `token` are correctly removed from all `InsertInput` types; TypeScript types regenerated by `graphql-codegen` pass `tsc --noEmit` with zero errors.

---

## Fork Extension Point Mapping Table (Actual Comparison After Migration Complete)

| Original Fork Extension | seaography 2.0 Actual Replacement | Migration Result |
|------------------------|-----------------------------------|------------------|
| `GuardsConfig.entity_guards` | `LifecycleHooksInterface::entity_guard` | ✅ Migrated |
| `GuardsConfig.field_guards` | `LifecycleHooksInterface::entity_filter` + custom mutation closures | ✅ Migrated |
| `FilterTypesMapConfig.condition_functions` | `FilterTypesMapConfig.condition_functions` (kept) + `entity_filter` | ✅ Migrated |
| `TypesMapConfig.input_none_conversions` | `EntityInputConfig.insert_skips` + `ActiveModelBehavior` | ✅ Migrated |
| `TypesMapConfig.input_conversions` | `ColumnOptions.input_conversion` (signature changed: no longer receives `ResolverContext`) | ✅ Migrated |
| `TypesMapConfig.output_conversions` | `ColumnOptions.output_conversion` | ✅ Migrated |
| `TypesMapConfig.input_type_overwrites` | `ColumnOptions.input_type` | ✅ Migrated |
| `TypesMapConfig.output_type_overwrites` | `ColumnOptions.output_type` | ✅ Migrated |
| `to_field_with_mutation_fn` | Manual `Field::new` + `FieldFuture` construction | ✅ Migrated (no existing macro) |
| `with-json-as-scalar` feature | Built-in upstream | ✅ No action needed |
| `register_entity!` 3 parameters | `register_entity!` 2 parameters | ✅ Migrated |
| `prepare_active_model(ctx, input, resolver)` | `prepare_active_model(&input_builder, &object_builder, input)` | ✅ Migrated |

---

English | [中文](../zh/archive/101-REFACTOR_PROGRESS.md)

# 100-Refactor Design

## Status

Draft

## Target Audience

- Developers working on backend, database, and GraphQL layer development
- AI agents performing code migration, refactoring, and review tasks

## Purpose

This document defines the plan for migrating from the current authorization model based on a forked `seaography-1.x.x` to a design centered on PostgreSQL Row-Level Security (RLS), where the GraphQL/application layer retains only limited and replaceable authorization logic.

This refactoring aims to achieve the following goals:

- Eliminate long-term dependency on a private Seaography fork for user-level authorization
- Make data isolation enforceable at the PostgreSQL layer
- Keep GraphQL authorization logic simple, localized, and framework-replaceable
- Preserve the productivity benefits of schema/entity reflection where practically feasible
- Reduce hidden authorization behavior embedded in framework internal mechanisms
- Produce migration paths that can withstand framework or language replacement

---

## Background

The existing system implements user permission control by forking and patching `seaography-1.x.x`.

This approach solved short-term needs but introduced several structural problems:

- Authorization behavior is partially hidden in the framework fork
- Upgrading is costly because security logic is entangled with upstream framework internals
- Authorization semantics are not portable to other GraphQL frameworks or other languages
- Correctness depends on application-layer behavior rather than database-enforced rules
- Future non-GraphQL entry points may accidentally bypass the current protection model

For this project, user-owned resources such as subscriptions, download tasks, and watch progress must never leak to other users. This requirement should not depend on the continued existence of Seaography-specific internal mechanisms.

Therefore, the target design is as follows:

- **PostgreSQL RLS is the source of truth for row-level data isolation**
- **GraphQL layer authorization is a secondary optimization and user experience layer**
- **Seaography-specific RBAC is not used as a core architectural dependency**
- **Seaography guards and filters are only used in a limited and replaceable manner**

---

## Non-Goals

This refactoring does **not** attempt to:

- Redesign the entire domain model
- Replace GraphQL with REST or other protocols
- Eliminate Seaography
- Create a generic enterprise RBAC system
- Pre-solve all future multi-tenant or sharing scenarios
- Optimize every query before security correctness is established

---

## Design Principles

### 1. Database-Enforced Ownership Over Framework-Enforced Ownership

If a row is private to a user, the database must enforce visibility and mutability constraints.

### 2. Authorization Logic Should Be Clear and Readable

Developers and AI agents should be able to identify where access control resides:

- Row ownership and row visibility: PostgreSQL policies
- Endpoint/mutation admission: Application or GraphQL guards
- Query convenience filtering: GraphQL filters or resolver composition

### 3. Prefer Portable Authorization Semantics

We should support designs that can be re-expressed as:

- Another GraphQL framework
- Another Rust tech stack
- Another language
- Future REST or task-processing entry points

### 4. Avoid Hidden Security Magic in Generated Code

Exposed generated schema is useful. But security assumptions hidden in code generation or fork internals are not.

### 5. Prefer Explicit Ownership Columns

Private tables should generally have direct `subscriber_id` columns rather than relying on indirect ownership inference through complex joins.

---

## Target Architecture

### Authorization Layers

The refactored system uses three layers with distinct responsibilities.

#### Layer A: PostgreSQL RLS (Enforced, Authoritative)

Responsible for:

- Which rows a user can `SELECT`
- Which rows a user can `UPDATE`
- Which rows a user can `DELETE`
- Which rows a user can `INSERT`
- Preventing accidental cross-user data leakage in any backend path

This is the hard security boundary.

#### Layer B: Application Request Context (Enforced)

Responsible for:

- Authenticating the caller
- Deriving `subscriber_id`, roles, and authentication state
- Binding the current user identity to the database session/transaction context
- Passing the authentication context to GraphQL resolvers

This layer connects identity to the database and API surface.

#### Layer C: GraphQL Guard/Filter Hooks (Optional but Recommended)

Responsible for:

- Early rejection of unauthenticated access
- Simplifying common query constraints
- Providing clearer API behavior and error responses
- Preserving ergonomics in the generated GraphQL API

This is a convenience and API-shaping layer, not a fundamental security layer.

---

## Why Not Use Seaography RBAC as a Foundation

Seaography RBAC may be useful in some projects but should not be viewed as core to this migration.

Reasons:

- It increases coupling with Seaography and SeaORM internals
- Makes future framework replacement more costly
- May recreate the current problems in a different form
- Row-level ownership is better expressed directly in PostgreSQL than in framework-specific authorization abstractions

Decision:

- **Do not** build a new permission system around Seaography RBAC
- **Do not** require Seaography RBAC to guarantee correctness
- **Do not** encode core row ownership guarantees solely in framework-level RBAC

Allowed usage:

- Future teams may evaluate Seaography RBAC for non-critical convenience scenarios
- Any such adoption must be additive and must not replace RLS-backed guarantees

---

## Resource Classification

Migration should first classify tables into access categories.

### Category 1: User-Private Tables

Examples:

- `subscription`
- `download_task`
- `watch_progress`
- `notification_rule`
- `user_feed_preference`
- `download_profile` (if user-scoped)

Rules:

- Must contain direct ownership information, typically `subscriber_id`
- Must be protected by RLS
- GraphQL access should default to current user scope

### Category 2: Shared or Global Reference Tables

Examples:

- `anime`
- `episode`
- `subtitle_group`
- Metadata cache or normalized catalog tables

Rules:

- Typically do not require per-user RLS
- Changes (if any) should be restricted to admin/operator rules
- Read access may be public or require authentication, depending on product design

### Category 3: Mixed-Scope Tables

Examples:

- Shared collections
- Collaborative watchlists
- System-generated recommendations with per-user visibility metadata

Rules:

- Defer advanced sharing designs unless currently needed
- Do not over-generalize prematurely
- Explicitly encode ownership and access semantics before exposing through generated APIs

---

## Canonical Security Model

### Ownership Model

For user-private rows:

- Each row explicitly belongs to a user unless otherwise explicitly designed
- Ownership is represented by a direct column, typically `subscriber_id`
- Authenticated users can only access rows where `row.subscriber_id == current_subscriber_id`

### Mutation Model

For user-private rows:

- Create: Rows must be created under the current authenticated user
- Update: Only rows owned by the current authenticated user can be updated
- Delete: Only rows owned by the current authenticated user can be deleted

### Field Source of Truth

For ownership fields (such as `subscriber_id`):

- Server should derive them from the authentication context
- Client-provided data should be ignored or rejected in normal user flows

---

## Database Design Requirements

### 1. Add Explicit Ownership Columns

Every user-private table must have direct ownership columns unless there is a very strong reason not to do so.

Preferred form:

```sql
subscriber_id uuid not null references app_user(id)
```

Avoid patterns that can only infer ownership through deep joins, such as:

- `download_task -> subscription -> profile -> user`
- `watch_progress -> episode_state -> library -> user`

These patterns make RLS harder to reason about and audit.

### 2. Enable RLS on Private Tables

For each user-private table:

```sql
ALTER TABLE subscription ENABLE ROW LEVEL SECURITY;
ALTER TABLE subscription FORCE ROW LEVEL SECURITY;
```

Using `FORCE ROW LEVEL SECURITY` is recommended so table ownership cannot silently bypass policies during usage.

### 3. Define Explicit Policies

Typical policy structure:

- `SELECT` policy using the current session user
- `INSERT` policy checking inserted row ownership
- `UPDATE` policy limiting old row eligibility and new row validity
- `DELETE` policy limiting deletable rows

Conceptual pattern:

```sql
CREATE POLICY subscription_select ON subscription
FOR SELECT
USING (subscriber_id = current_setting('app.subscriber_id', true)::uuid);

CREATE POLICY subscription_insert ON subscription
FOR INSERT
WITH CHECK (subscriber_id = current_setting('app.subscriber_id', true)::uuid);

CREATE POLICY subscription_update ON subscription
FOR UPDATE
USING (subscriber_id = current_setting('app.subscriber_id', true)::uuid)
WITH CHECK (subscriber_id = current_setting('app.subscriber_id', true)::uuid);

CREATE POLICY subscription_delete ON subscription
FOR DELETE
USING (subscriber_id = current_setting('app.subscriber_id', true)::uuid);
```

### 4. Runtime DB Roles Must Not Bypass RLS

Application runtime connections must not use roles that bypass RLS semantics.

Separation of concerns:

- Migration roles: Allow privilege escalation
- Runtime roles: Restricted, constrained by RLS

### 5. Current User Must Be Bound to DB Session or Transaction Scope

Each authenticated request must establish the current application user identity in a DB-visible way.

Recommended pattern:

```sql
SET LOCAL app.subscriber_id = '<authenticated-user-uuid>';
```

This should happen within the request transaction or request-bound connection context before executing user-scoped queries.

---

## GraphQL Layer Design

### GraphQL's Role After Refactoring

GraphQL is no longer responsible for the ultimate correctness of row-level data isolation.

Instead, GraphQL should:

- Provide ergonomic access to generated schemas
- Early-reject obviously invalid access
- Reduce unexpected behavior for clients
- Express application-level semantics not worth pushing into SQL policies

### Allowed Seaography Usage

Allowed and recommended:

- Shallow guards for authentication and coarse-grained routing admission
- Filter or lifecycle hooks that add user-scoped conditions for clarity and API ergonomics
- Limited custom resolvers or mutations where ownership must be derived from the server

Not recommended as a foundation:

- Deep framework-specific RBAC entanglement
- Re-forking Seaography to restore security-critical behavior
- Relying on generated CRUD mutations that allow clients to directly control ownership columns

---

## Guard and Filter Guidance

### Guards

Use guards for coarse-grained access checks, such as:

- User must be authenticated
- Admin-only mutations
- Anonymous callers cannot use a feature

Guards should answer questions like:

- Can this request enter this operation?

Guards should **not** be the sole mechanism preventing cross-user data access.

### Filters

Use filters for API convenience, such as automatically limiting user-private entities to the current user scope.

Examples:

- When querying `Subscription`, automatically scope to the current user in GraphQL
- When listing `DownloadTask`, pre-apply current user conditions

Filters are useful because they:

- Reduce accidental large-scope queries
- Improve API ergonomics
- Make generated GraphQL endpoint behavior more intuitive

But filters are still adjuncts to RLS.

---

## Custom Mutation Guidance

Generated CRUD mutations are usually too permissive for user-owned entities.

For user-private tables, prefer custom mutations when:

- Ownership must be derived from the authentication context
- Certain fields must be immutable to users
- Create/update flows require domain validation
- Batch operations require carefully controlled semantics

Examples:

- `createSubscription(animeId, ruleInput, profileId)`
- `pauseDownloadTask(taskId)`
- `removeSubscription(subscriptionId)`

Avoid generic patterns where users can freely submit or modify:

- `subscriber_id`
- Owner references
- Internal state fields
- Admin-only metadata

---

## Migration Strategy

Migration should be incremental, not a one-time rewrite.

### Phase 0: Discovery and Inventory

Create a detailed inventory of current authorization behavior.

Tasks:

- Identify all authorization logic implemented in the Seaography fork
- Map each behavior to one of the following targets:
  - RLS policies
  - GraphQL guards
  - GraphQL filters
  - Custom resolvers or mutations
  - Obsolete behaviors to remove
- Classify all tables by access category
- Identify tables missing explicit `subscriber_id`
- Identify all generated mutations currently exposing ownership-sensitive fields

Deliverables:

- Authorization behavior inventory
- Table classification matrix
- Migration risk list

### Phase 1: Schema Preparation

Tasks:

- Add `subscriber_id` columns where missing
- Backfill ownership data
- Add indexes required for user-scoped queries, typically including `(subscriber_id, id)` or `(subscriber_id, foreign_key)`
- Clean up rows with ambiguous ownership
- Define invariants for private tables

Exit criteria:

- Each user-private table has explicit ownership semantics
- Legacy ownership ambiguities eliminated or explicitly deferred

### Phase 2: Introduce RLS Shadow Mode

Tasks:

- Write RLS policies for user-private tables
- Add request logic for setting `app.subscriber_id`
- Add integration tests verifying DB-level isolation
- Temporarily keep existing application-layer authorization active

Recommended rollout:

- Enable RLS first in development and testing
- Use integration tests and manual cross-user scenarios for verification
- Compare with legacy fork authorization behavior

Exit criteria:

- DB-level tests prove cross-user isolation
- Application still has existing API behavior

### Phase 3: Refactor GraphQL Layer

Tasks:

- Remove authorization logic that only exists because the database does not enforce ownership
- Replace fork-specific behavior with guards, filters, and custom mutations where necessary
- Reduce generic generated mutations for sensitive entities
- Ensure ownership fields are server-derived

Exit criteria:

- GraphQL no longer depends on fork internals for correctness guarantees
- Generated schemas remain acceptable for developer productivity

### Phase 4: Remove Fork Dependency

Tasks:

- Switch from forked Seaography to upstream-compatible implementations where feasible
- Isolate any remaining framework glue code behind local application abstractions
- Remove dead compatibility code

Exit criteria:

- Security-critical behavior no longer depends on Seaography fork
- Fork can be archived or frozen

### Phase 5: Hardening and Simplification

Tasks:

- Audit private table exposure paths outside GraphQL
- Add regression tests for each user-private entity
- Document the new authorization model in developer documentation
- Simplify resolver code that became redundant after RLS adoption

Exit criteria:

- Authorization model can be understood without reading framework internals
- Future contributors can safely extend the system

---

## Required Tests

### Database-Level Tests

These are mandatory because correctness now primarily depends on RLS.

For each user-private table, test at least:

- User A can select their own rows
- User A cannot select User B's rows
- User A can insert their own rows
- User A cannot insert rows with User B's ownership
- User A can update their own rows
- User A cannot update User B's rows
- User A cannot change owned rows to another user
- User A can delete their own rows
- User A cannot delete User B's rows
- Unauthenticated or missing session behavior is rejected or empty by design

### API-Level Tests

Test that GraphQL behavior remains correct and ergonomic:

- Unauthenticated requests fail early at expected places
- User-scoped list queries only return their own rows
- Custom mutations do not accept client-controlled ownership fields
- Admin-only operations remain restricted

### Regression Tests Against Legacy Behavior

Where practically feasible, preserve expected product behavior while changing execution location.

Example tests:

- Users can still only see their own subscriptions in GraphQL queries
- Download task operations remain scoped to current user
- Existing client flows do not need to provide ownership fields in payload

---

## Implementation Rules for Developers and AI Agents

### Rule 1: Do Not Reintroduce Security-Critical Fork Patching

If a behavior is security-critical and currently implemented in the old fork, move it to:

- PostgreSQL RLS, or
- Explicit application-layer code

Do not hide it in patched framework internals again.

### Rule 2: Prefer Clarity Over Cleverness

When in doubt:

- Add explicit `subscriber_id`
- Write explicit policies
- Write explicit custom mutations
- Reject ambiguous ownership flows

### Rule 3: Do Not Trust Client-Provided Ownership Fields

For normal user flows, ownership must come from the authentication context, not the request payload.

### Rule 4: Generated CRUD Must Be Reviewed Per-Entity

Do not assume generated GraphQL CRUD is safe just because RLS exists.

Review whether:

- Mutation surface is semantically appropriate
- Hidden internal fields are exposed
- Ownership-related fields could be abused or lead to poor UX

### Rule 5: Keep GraphQL-Specific Authorization Shallow

GraphQL logic should be easy to rewrite in another framework.

Allowed forms:

- `require_auth(ctx)`
- `is_admin(ctx)`
- `entity_filter_for_current_user(ctx)`
- Custom mutation wrappers

Avoid building deeply framework-coupled policy DSLs unless there are compelling reasons.

### Rule 6: When Ownership Is Indirect, Stop and Redesign

If an AI agent encounters a table that can only derive ownership indirectly through multiple joins, it must not silently improvise fragile authorization shortcuts.

Instead, it should:

- Document the ambiguity
- Suggest adding explicit ownership columns
- Add migration/backfill plans
- Flag the area for human review if needed

### Rule 7: Security Correctness Over Code Generation Purity

It is acceptable to reduce auto-generated surface or replace generic mutations with custom mutations when needed.

### Rule 8: Maintain Auditability

New authorization behavior must be discoverable from:

- SQL migrations and policies
- Documented request context setup
- Resolver/guard/filter code

Reviewers should not need to check framework forks to understand access control.

---

## Suggested Project Structure Changes

Specific layouts may vary, but the system should strive for a structure like:

```text
docs/
  001-REFACTOR-DESIGN.md
  002-AUTHORIZATION-MODEL.md
  003-RLS-POLICIES.md
  004-GRAPHQL-AUTH-GUIDE.md

migrations/
  <timestamp>_add_subscriber_id_to_private_tables.sql
  <timestamp>_enable_rls_for_private_tables.sql
  <timestamp>_add_rls_policies.sql

src/
  auth/
    context.rs
    guards.rs
    session.rs
  db/
    rls.rs
    transaction.rs
  graphql/
    filters.rs
    hooks.rs
    mutations/
      create_subscription.rs
      remove_subscription.rs
      pause_download_task.rs
```

Focus is on separation of concerns, not exact filenames.

---

## Request Context Pattern

Each authenticated request should produce a request-scoped authentication context containing at least:

- `subscriber_id`
- Authentication state
- Coarse-grained role information if needed

This context should be injected into:

- Database execution paths so `app.subscriber_id` can be set
- GraphQL context so guards and filters can read it

Important constraints:

- Set DB session variables on transaction/request scope, not global pooled connections
- Ensure settings are applied before any user-scoped queries
- Ensure automatic cleanup through transaction scope or equivalent mechanism

---

## Common Refactoring Patterns

### Pattern A: Fork Hooks That Auto-Add Ownership Constraints

Old approach:

- Custom Seaography fork inserts user filters into generated query paths

New approach:

- Use RLS to enforce row visibility
- Optionally mirror the same scope in GraphQL filters for ergonomics

### Pattern B: Generated Mutations Accepting `subscriber_id`

Old approach:

- Client payload may specify ownership-related fields

New approach:

- Remove or wrap generic mutations
- Derive `subscriber_id` from authentication context on the server side
- Let RLS as a second line of defense reject inconsistent writes

### Pattern C: Admin Bypass Logic Implemented in Framework Internals

Old approach:

- Fork-specific admin branches deeply exist in the generated stack

New approach:

- Express admin behavior explicitly in guards, filters, or separate resolvers
- Use carefully designed DB roles/session strategies if needed, not hidden bypasses

---

## Risks and Mitigations

### Risk 1: RLS Accidentally Breaks Existing Queries

Causes:

- Missing session variables
- Incorrect policies
- Runtime role mismatch

Mitigation:

- Introduce DB-level integration tests early
- Enable RLS in development environment first
- Log request authentication context and transaction settings during migration

### Risk 2: Pooled Connections Leak Session State

Causes:

- Misusing session-level variables across pooled connections

Mitigation:

- Use `SET LOCAL` in transaction scope
- Avoid global session modification patterns for per-request identity
- Carefully review connection/transaction lifecycle

### Risk 3: Missing or Inconsistent Ownership Columns

Causes:

- Legacy schema design
- Historical data ambiguity

Mitigation:

- Treat ownership normalization as an explicit migration phase
- Backfill and validate before relying on RLS

### Risk 4: Generated GraphQL Surface Is Still Semantically Too Broad

Causes:

- Code generation exposes mutations that are technically safe under RLS but undesirable product-wise

Mitigation:

- Review generated operations per entity
- Replace sensitive generic mutations with custom mutations

### Risk 5: Team Continues to Think GraphQL Filters Are Security Boundary

Causes:

- Old mental model from framework fork authorization

Mitigation:

- Document that RLS is the source of truth
- Add test coverage at SQL layer, not just API layer
- Have reviewers explicitly check policy changes

---

## Acceptance Criteria

Migration is considered successful when all of the following are true.

### Security

- Every user-private table is protected by RLS
- Cross-user row leakage is prevented at the database level
- Runtime DB roles do not silently bypass RLS

### Architecture

- Security-critical authorization no longer depends on Seaography fork
- GraphQL authorization logic is shallow and replaceable
- Seaography RBAC is not required for correctness

### Product Behavior

- Users can still perform normal subscription, task, and progress workflows
- Clients do not need to submit ownership fields
- Unauthorized access fails safely and predictably

### Maintainability

- Developers can understand access control from documentation, migrations, and plain source code
- AI agents can extend the system without modifying framework internals
- Future migration from Seaography remains feasible

---

## Immediate Next Steps

1. Inventory all current fork-based authorization customizations
2. Classify all domain tables as private/shared/mixed-scope
3. Identify tables missing direct `subscriber_id`
4. Design backfill and migration for ownership columns
5. Implement request-scoped `app.subscriber_id` binding in DB path
6. Write initial RLS policies for one high-value private table (e.g., `subscription`)
7. Add DB-level isolation tests
8. Migrate that table's GraphQL access out of fork-specific logic
9. Repeat for `download_task`, `watch_progress`, and remaining private tables

---

English | [中文](../zh/archive/100-REFACTOR_DESIGN.md)

# 002 — Authentication and data access

## Current behavior and deployment boundary

The same-origin WebUI uses backend Authorization Code with PKCE S256 and an HttpOnly session cookie. Rust uses published SecurityDept OIDC/session 0.3.0; the browser uses SessionContextClient 0.3.0. Local behavior checks cover authentication, persistence and GraphQL/task isolation. See [verification](001-DEVELOPMENT-VERIFICATION.md) and [roadmap](roadmap/001-SHORT-TERM-ROADMAP.md).

This breaks the previous OIDC Bearer API, browser token storage, callback token JSON and Vite client-secret contract. Callback redirects only to a validated local path. RSS and subscriber static files require application authentication; possession of an RSS token alone no longer grants access. External readers must support application authentication. Machine API credentials are a future capability.

## Credential checks and system schedules

Credential checks use an independent `POST /api/credential3rd/{id}/check-available` command with JSON body `{}` and success response `{ "available": true }`. It requires current application authentication, exact Origin, `X-Konobangu-CSRF: 1` and `application/json`; responses are not cached. The previous `credential3rdCheckAvailable` GraphQL field and document are removed. Callers must migrate to the HTTP command.

An owner-bound short transaction reads the credential and PostgreSQL row version, then closes. External checking/login holds no business connection. A new owner-bound short transaction atomically matches owner, ID and version before committing encrypted cookies. The version is `xmin` read with the row, independent of editable timestamps. An intervening edit, deletion or another check's write yields 409 without stale cookie updates. Cross-owner access returns 403; remote failure returns 502 without cookie writes. Retrying must read the current credential again. Database rollback cannot undo requests already received by the third party.

All non-Basic Cron root, relation, update, delete and batch filters add `system_task_cron IS NULL`, ANDed with existing subscriber scope. Historical Basic/OIDC sharing does not confer system-schedule privileges; OIDC can still manage its own user schedules. Basic retains existing owner scope and the system-task input guard remains. Independent behavior checks confirm these production boundaries. Business records/outbox and queue recovery use the current [task contract](003-TASK-DELIVERY-AND-MIGRATION.md).

## Configuration and roles

Figment parses native TOML/JSON/YAML with defaults < selected file < canonical double-underscore overrides, such as AUTH__SESSION__PUBLIC_URL. Process variables precede dotenv; unquoted legacy templates fail with redacted migration errors, while quoted template markers remain literal. Merge the following into the existing server/logger/storage configuration. All passwords and hosts below are placeholders.

```toml
[database]
url = "postgres://konobangu:PASSWORD@localhost/konobangu"

[database.migration]
auto_run = true
# url = "postgres://migration_owner:PASSWORD@localhost/konobangu"
# legacy_oidc_issuer = "https://idp.example/"

[auth.provider]
type = "oidc"
issuer = "https://idp.example/"
audience = "konobangu"
client_id = "konobangu"
client_secret = "SERVER_ONLY_SECRET"
# extra_scopes = ["required-scope"]

[auth.session]
public_url = "https://konobangu.example/"
cookie_secure = true
idle_timeout_seconds = 1800
absolute_timeout_seconds = 43200
login_timeout_seconds = 300
cleanup_interval_seconds = 300
```

Register a confidential authorization_code client with PKCE S256 and the exact redirect `https://konobangu.example/api/auth/session/callback`. `auth.session.public_url` must be a fixed HTTP(S) origin without path/query/fragment/user information; Host/Forwarded never select callback origin. Secure defaults to true; disabling it is allowed only for HTTP loopback testing. All lifetimes must be positive.

`auth.provider.audience` must equal the client ID. A different audience or nonempty `auth.provider.extra_claims` fails startup: access-token claim restrictions cannot silently become ID-token principal restrictions. Move equivalent restrictions to the IdP before migration. Extra scopes are both requested and checked against granted scopes. Discovery/configuration failure never falls back to Basic or a development identity.

Runtime privileges belong to three fixed capability roles, all NOLOGIN, NOINHERIT, NOSUPERUSER and NOBYPASSRLS:

| Capability role | Responsibility |
| --- | --- |
| `konobangu_app_scoped_access` | Application data under subscriber-bound RLS; HTTP requests and worker business stages |
| `konobangu_auth_access` | External identity mapping, subscriber initialization and session/pending/grant persistence |
| `konobangu_task_control_access` | Cross-subscriber delivery, runs, outbox and Cron, including queue storage |

Default deployment needs one existing PostgreSQL login in database.url. auth.session.database_url and scheduler.database.url inherit it when omitted. database.migration.auto_run defaults to true: startup completes owner migrations and library initialization, creates and validates capability roles, then applies grants and memberships before opening runtime pools. Membership uses INHERIT FALSE, SET TRUE; the login does not automatically combine capability permissions. PostgreSQL 16+ is required; see [membership options](https://www.postgresql.org/docs/current/sql-grant.html) and [transaction-local role switching](https://www.postgresql.org/docs/current/sql-set-role.html). The bootstrap credential needs CREATEROLE, permission to manage existing capability memberships and ownership/grant rights for application objects. Managed-provider credentials must actually supply those privileges; database ownership alone is insufficient. Startup never silently disables RLS or falls back to unrestricted runtime queries on failure.

Each runtime pool activates its fixed capability on physical connection creation and restores it before checkout. Startup guards inspect current_user, requiring a non-owner, non-superuser role without privileges from other responsibilities. Business operations also execute SET LOCAL ROLE for app_scoped_access and bind subscriber identity inside their transaction. Commit, rollback and cancellation restore the restricted baseline and clear the local subscriber scope. Apalis and the mature session store issue queries through their own pools, so they need dedicated task-control/auth pool baselines rather than an unrelated outer transaction's SET LOCAL ROLE. Use direct PostgreSQL connections or session pooling; [transaction pooling](https://www.pgbouncer.org/features.html) cannot preserve this per-connection baseline and must not be used with the shared-login mode.

This is operation-level privilege reduction, not credential isolation against arbitrary SQL: session_user retains the bootstrap login's ability to RESET ROLE or switch to other granted capabilities. All URLs target the same database. Deployments requiring credential isolation can provide existing separate runtime URLs and an optional database.migration.url; startup grants each configured login only its required capability. To manage migrations externally, set auto_run=false after initializing the same role/grant contract with the existing migrate-up entry. These are optional deployment policies, not steps required for the default deployment. No runtime login or password is created or replaced by the application. --migration-preflight remains read-only.

Grants live in migrations/access.rs and reference migrations/defs.rs schema/table/column identifiers. The subscriber sequence comes from PostgreSQL pg_get_serial_sequence. Library objects are granted after their dedicated schemas initialize; application objects retain explicit grants. New application tables need explicit grant definitions. Repeated startup reapplies the same grants idempotently.

Identity uses two bounded pools: SeaORM/SQLx 0.9.0 max 3 and the mature store SQLx 0.8.6 max 2, both under auth_access. Both close on initialization failure and shutdown. The published store 0.15/core 0.14 adapter remains until a verified store supports SQLx 0.9.

Basic is an explicit single-administrator mode: replace OIDC fields with `type = "basic"`, `username` and `password` in `auth.provider`, retaining fixed origin/session configuration; the auth database URL inherits database.url by default, and build with `AUTH__PROVIDER__TYPE=basic`. Server verification owns authentication and Basic subscriber mapping. Logout returns 409 and the UI explains that clearing cookies cannot revoke browser-cached Basic credentials; close the browser or clear its credential cache. OIDC builds use only `AUTH__PROVIDER__TYPE=oidc`, without passing server auth configuration to Vite. Development Vite proxies `/api` to loopback recorder:5001; configure the fixed login origin as the browser-facing Vite:5000 origin. Basic verification also stays on the backend.

## Existing database migration and recovery

1. Stop recorder, workers and all writes. Back up the database with `pg_dump --format=custom` and preserve storage files; rehearse restoration in an isolated database. Protect sensitive backup data through the existing operational policy.
2. Inspect `public.auth` OIDC attribution, duplicate `(auth_type,pid)` and shared subscriber mappings. Set `database.migration.legacy_oidc_issuer` only when the previous issuer is known exactly; do not infer ownership from email/display name.
3. Run explicit owner migration:

```sh
mise exec -- cargo run -p recorder --locked --bin migrate-up -- --environment production --config-file /path/to/recorder.config.toml
```

Development can use `mise exec -- just dev-recorder-migrate-up`. The command prefers `database.migration.url`, otherwise uses the supplied owner `database.url`; it creates no business AppContext.

4. Apply role grants, configure the ordinary runtime roles and split identity pools, build WebUI, check startup/user-info/real login/isolation, then resume writes. Never configure an owner URI for runtime pools.

Unknown issuer attribution and conflicting legacy keys fail before identity DDL. Identity becomes the exact unique `(auth_type,issuer,pid)` key: Basic has NULL issuer, OIDC has a nonempty issuer. Auth moves into `auth_identity`; session/pending/grant use `auth_session`. Existing subscriber mappings and shared relationships are retained. New OIDC identities receive independent subscribers; transactional locking and uniqueness handle concurrent first login.

Historical owner-only 0.7 SQL bootstrap precedes the application transaction. The transaction imports business tasks and archives the old queue; public new-queue initialization and mature session store initialization follow. See [upgrade and recovery](003-TASK-DELIVERY-AND-MIGRATION.md). These component DDL steps are not one atomic commit. If store initialization fails, keep writes stopped, repair privileges, rerun idempotent up and apply grants.

Down fails explicitly because identities from different issuers cannot safely collapse. Recovery means stopping the new application, restoring the pre-migration backup with the matching old release/configuration, validating it, then reopening writes. New post-upgrade writes cannot be losslessly moved into the old identity model. Do not clear the database as a recovery mechanism.

## Session, CSRF and revocation

| Endpoint | Behavior |
| --- | --- |
| GET `/api/auth/session/login` | Creates a browser-bound one-time state and redirects to IdP |
| GET `/api/auth/session/callback` | Verifies browser/state/code/PKCE/nonce/token/scopes, rotates session ID, redirects locally |
| GET `/api/auth/session/user-info` | Minimal principal; unauthenticated/expired 401, no-store |
| POST `/api/auth/session/logout` | Revokes durable application grant before clearing session; idempotent OIDC logout, Basic 409 |

Cookie attributes are HttpOnly, Secure, SameSite=Lax, Path=/ and no Domain. Idle expiry defaults to 30 minutes and renews on access; grants expire absolutely after 12 hours without extension. Pending expires after five minutes and is consumed atomically by DELETE RETURNING. At most eight browser-bound states allow multiple tabs; callback checks the initiating browser before SDK consumption. Return paths reject external origins, scheme-relative targets, controls, backslashes and layered encoding before creating authenticated state. Logs omit callback query/code/state/session identifiers.

PostgreSQL persistence reuses tower-sessions-sqlx-store 0.15.0 through a core 0.14→0.15 Id/Record/error adapter, writing back any regenerated create ID. Instances share PostgreSQL without sticky sessions. Authentication checks the grant on every request. Logout persists revoked_at first, preventing late saves from restoring identity even if they recreate a session row. Cleanup runs every five minutes, retains revoked grants until absolute expiry, and stops with App shutdown.

Cookie writes, including GraphQL mutation, logout and credential checks, require exact Origin, `X-Konobangu-CSRF: 1`, and application/json with optional charset. Missing/wrong origin/header yields 403; wrong content type yields 415. SDK/Apollo send credentials and matching headers. CORS/SameSite do not replace these checks.

User-info refresh reloads application identity; no refresh token is stored or rotated. Local logout does not guarantee IdP-wide SSO logout or immediate propagation of IdP revocation. There is no background revocation propagation: local revocation or absolute expiry ends current application authentication.

## GraphQL, tasks and frontend ownership

Online execution uses explicit async-graphql dynamic resolvers. Seaography provides only pure schema/naming/filter helpers, without automatic database query/mutation/loaders. The HTTP operation owns an `Arc<Mutex<Option<DatabaseTransaction>>>` with trusted SET LOCAL subscriber identity. Roots, relations, request loaders, CRUD, batches and custom task mutations borrow that transaction; resolvers cannot commit. Locks release before nested resolution, and loaders never cache across requests. Mutation responds successfully only after commit; execution errors roll back and return data=null, and commit failures also reject success. Queries retain valid partial data and close their read transaction. A public extension preserves data and errors at schema-declared nullable boundaries, including alias paths; non-null failures still propagate. Nullable mutation-output errors also roll back the operation. Cancellation drops uncommitted transactions into rollback. Missing/empty GUC denies private rows.

Owners come from server identity mapping, never frontend subscriber_id. Creation injects the owner and updates cannot change it. Since FK checks may bypass RLS, GraphQL and task writes verify referenced-row visibility within the same owner transaction. Framework only_introspection excludes business execution even in mixed documents. System tasks/schedules require the explicit Basic administrator.

User tasks read verified subscriptions in a short transaction, close it before HTTP/image work, then commit resource upserts and associations together in owner-bound database stages. Credential checks use the independent command described above; background cookie synchronization commits only explicit short business stages, never inside GraphQL operations; rollback cannot undo external requests. GraphQL enqueue/retry/delete write application runs/outbox in the original operation transaction. Cron occurrence, runs/outbox and next_run commit together. Public queue delivery and fenced business stages follow the [task contract](003-TASK-DELIVERY-AND-MIGRATION.md), without an exactly-once external-effects claim.

FoundationEnvironment creates one official injection-js root shared by environment/native injector/SDK facade. Public useFactory/useValue and inject keep application code decorator-free. SessionContextClient owns its session resource; the root DestroyRef disposes SDK resources and AppRuntime owns listeners/React/Apollo. HTTP 403, network/5xx and parse errors do not loop into login; 401 enters the unauthenticated flow.

Each authenticated identity owns an ApolloClient/InMemoryCache. Logout/identity changes cancel and stop the old client before switching provider; epoch/current-identity checks reject late results. Ordinary refresh preserves the client. Query currently owns no private requests, so no duplicate cache layer is added. Apollo DeclareDefaultOptions defines query/watchQuery errorPolicy=all and mutation=none. Client-preset TypedDocumentNodes preserve result types; pages display partial data with errors, and mutation failures never count as successful writes.

The credential button uses the shared SDK transport, then refreshes active Apollo credential list/detail queries after success without adding an entity cache. Identity changes, root destruction and component cancellation cancel requests and reject late replies.

[English](002-AUTHENTICATION-DECISION.md) | [中文](../zh/002-AUTHENTICATION-DECISION.md)

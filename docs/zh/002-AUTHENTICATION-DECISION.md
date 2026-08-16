# 002 — 认证与数据访问

## 当前实现与部署边界

同源 WebUI 使用后端 Authorization Code + PKCE S256 和 HttpOnly cookie。Rust 使用发布版 SecurityDept OIDC/session 0.3.0；浏览器使用 SessionContextClient 0.3.0。身份、会话与 GraphQL/用户任务隔离已有本地行为验证。验证入口见 [开发验证](001-DEVELOPMENT-VERIFICATION.md)，验收状态见 [路线图](roadmap/001-SHORT-TERM-ROADMAP.md)。

这是 breaking change：移除 OIDC Bearer API、浏览器 token 存储、旧 callback token JSON 和 Vite client secret。callback 只跳转到校验后的站内路径。RSS 与 subscriber 私有静态文件也需要当前用户认证；持有 RSS token 不再单独构成访问权限，外部 RSS 客户端需支持应用认证。机器 API 凭据属于后续产品能力。

## 凭据检查与系统调度

凭据检查使用独立 `POST /api/credential3rd/{id}/check-available` 命令，JSON 请求体为 `{}`，成功响应为 `{ "available": true }`。需当前用户认证、精确 Origin、`X-Konobangu-CSRF: 1` 和 `application/json`，响应不缓存。旧 `credential3rdCheckAvailable` GraphQL 字段与 document 已移除；调用方必须迁移到 HTTP 命令。

命令先在绑定当前 subscriber 的短事务中读取凭据与 PostgreSQL 行版本并关闭事务，再进行外部检查/登录，最后在新的身份短事务中以 owner、ID 和行版本做原子条件更新，将加密 cookie 写入后提交。网络阶段不占用业务连接。版本取同次读取的 `xmin`，不依赖可修改的时间戳；期间编辑、删除或另一检查写回使版本变化时返回 409，不写入陈旧 cookie。跨 owner 为 403，远端检查失败为 502 且不写 cookie。重试须重新读取当前凭据；数据库回滚无法撤销第三方已接收的请求。

非 Basic 身份的 Cron root、关系、更新、删除与批量筛选统一增加 `system_task_cron IS NULL`，并与原 subscriber 范围做 AND。旧 Basic/OIDC 共享 subscriber 不授予系统调度权限；OIDC 仍可管理自己的用户 Cron。Basic 保留原 owner 范围，创建系统任务的输入检查也保留。上述生产边界已有独立行为验证；业务记录/outbox 与队列恢复遵循现行[任务合约](003-TASK-DELIVERY-AND-MIGRATION.md)。

## 配置与数据库角色

Figment 原生解析 TOML/JSON/YAML，优先级为 defaults < 配置文件 < canonical 双下划线变量（如 `AUTH__SESSION__PUBLIC_URL`）；进程环境优先于 dotenv，未加引号的旧模板报脱敏迁移错误，带引号的模板标记保持普通字符串。以下内容合并到现有服务配置，URI/password 是占位符，不能直接用于生产。保留现有 logger/server/storage 等配置。

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

IdP 注册 confidential client、authorization_code、PKCE S256，并配置精确 redirect URI `https://konobangu.example/api/auth/session/callback`。`auth.session.public_url` 必须是固定 HTTP(S) origin，不接受路径、query、fragment 或用户信息；callback 不信任 Host/Forwarded。Secure 默认开启，只有 HTTP loopback 测试可关闭。所有 TTL 必须为正数。

`auth.provider.audience` 必须等于 client ID；非空 `auth.provider.extra_claims` 或不同 audience 显式拒绝启动。旧 access-token claims 检查无法自动等价转换为 ID-token principal 检查，维护者必须先在 IdP 配置等价访问策略。extra scopes 同时请求并检查实际授权，不能把请求本身当作已授予。OIDC discovery/configuration 失败不会回退 Basic 或开发身份。

运行权限属于三个固定能力角色，均为 NOLOGIN、NOINHERIT、NOSUPERUSER、NOBYPASSRLS：

| 能力角色 | 职责 |
| --- | --- |
| `konobangu_app_scoped_access` | 绑定 subscriber 后，在 RLS 下访问应用数据；HTTP 请求与 worker 业务阶段共用 |
| `konobangu_auth_access` | 外部身份映射、subscriber 初始化与 session/pending/grant 持久化 |
| `konobangu_task_control_access` | 跨 subscriber 的任务投递、运行记录、outbox、Cron 与队列存储 |

默认部署只需要 database.url 中一个已有 PostgreSQL 登录；未配置的 auth.session.database_url 和 scheduler.database.url 继承它。database.migration.auto_run 默认 true：启动先完成 owner 迁移与库初始化，创建并校验能力角色，再应用授权和成员关系，最后打开运行连接池。成员关系使用 INHERIT FALSE、SET TRUE，登录不会自动合并三种能力。要求 PostgreSQL 16+；见 PostgreSQL [成员选项](https://www.postgresql.org/docs/current/sql-grant.html)与[事务级角色切换](https://www.postgresql.org/docs/current/sql-set-role.html)。初始化凭据需要 CREATEROLE、管理已有能力角色成员关系的权限，以及应用对象的所有权/授权权限。托管服务凭据须实际具有这些权限，数据库所有权本身并不足够。初始化失败不会静默关闭 RLS 或改用无限制运行查询。

各运行连接池在物理连接建立时切换到固定能力角色，每次借出连接前恢复该角色。启动检查以 current_user 验证非 owner、非 superuser 与职责间权限边界。业务事务同时执行 app_scoped_access 的 SET LOCAL ROLE 并绑定 subscriber；提交、回滚、取消后恢复受限基线并清除事务作用域。Apalis 与成熟 session store 在自己的连接池中执行查询，不能继承无关外层事务的 SET LOCAL ROLE，因此使用独立的任务控制/认证连接池基线。使用 PostgreSQL 直连或会话池；[事务池代理](https://www.pgbouncer.org/features.html)无法保持该连接基线，不得用于共用登录模式。

这提供操作级降权，不提供抵御任意 SQL 的凭据隔离：session_user 仍保有初始化登录的 RESET ROLE 或切换其它已授权能力的权限。所有 URL 指向同一数据库。需要凭据隔离时，可提供已有的独立运行 URL 与可选 database.migration.url；启动只向各配置登录授予对应能力。需要外部管理迁移时，先由既有 migrate-up 入口建立同一角色/授权契约，再设置 auto_run=false。这些是可选部署策略，不是默认部署步骤。应用不创建/替换运行登录或密码；--migration-preflight 保持只读。

授权集中于 migrations/access.rs，schema/表/列引用 migrations/defs.rs；subscriber 序列通过 PostgreSQL pg_get_serial_sequence 查询。库拥有的对象在专用 schema 初始化后授权，应用对象保留显式权限清单；新增应用表须更新授权定义。重复启动幂等重用角色并应用授权。

认证使用两个有界连接池：SeaORM/SQLx 0.9.0 最多 3 个连接，成熟 store SQLx 0.8.6 最多 2 个，均以 auth_access 执行查询。初始化失败与退出时关闭连接。store 0.15/core 0.14 adapter 保留至有经过验证的 SQLx 0.9 store。

Basic 是显式单管理员模式：用 `auth.provider` 中的 `type = "basic"`、`username` 和 `password` 替换 OIDC 字段，仍需 fixed origin 与会话配置，认证数据库 URL 默认继承 database.url，前端 `AUTH__PROVIDER__TYPE=basic`。服务器每次校验浏览器凭据并映射到 Basic subscriber；错误/缺失凭据不能被“始终登录”状态掩盖。Basic logout 返回 409，界面说明浏览器缓存凭据不能通过清 cookie 撤销；需要关闭浏览器或清除凭据缓存。OIDC 构建仅设置 `AUTH__PROVIDER__TYPE=oidc`，不把后端 auth 配置传给 Vite。开发 Vite 将 `/api` 代理到 loopback recorder:5001，登录固定 origin 应配置为浏览器访问的 Vite:5000；Basic 校验也由后端执行。

## 存量迁移与失败恢复

1. 停止 recorder、worker 和全部写入，备份数据库与存储文件；用 `pg_dump --format=custom` 保存数据库，并在隔离数据库演练恢复。备份涉及敏感数据，按现有运维策略保管。
2. 检查 `public.auth` 的 OIDC 归属、重复 `(auth_type,pid)` 与共享 subscriber。只有明确的旧 issuer 才设置 `database.migration.legacy_oidc_issuer`，不能按 email 或显示名猜测。
3. 显式执行 owner 迁移：

```sh
mise exec -- cargo run -p recorder --locked --bin migrate-up -- --environment production --config-file /path/to/recorder.config.toml
```

开发环境也可用 `mise exec -- just dev-recorder-migrate-up`。显式命令优先使用 `database.migration.url`，否则使用指定配置的 owner `database.url`；不创建业务 AppContext。

4. 执行角色授权，配置普通运行角色及拆分后的身份池，构建 WebUI，检查服务器启动、user-info、真实登录和隔离后再恢复写入。不要将 owner URI 配给业务、身份或队列池。

迁移在 DDL 前拒绝未知 OIDC issuer 和冲突。身份改为精确 `(auth_type,issuer,pid)` 唯一键，Basic issuer 为 NULL，OIDC issuer 非空；auth 移到 `auth_identity`，session/pending/grant 位于 `auth_session`。保留所有旧 subscriber 映射及已有共享关系，新 OIDC 身份创建独立 subscriber。并发首次映射由事务、数据库锁和唯一索引收口。

owner-only 历史 0.7 SQL bootstrap 在应用事务之前；事务导入业务任务并归档旧队列，随后执行公开新 queue migrator 与成熟 store 初始化。阶段失败保持停写，修复后幂等重跑再授权，详见[升级与恢复](003-TASK-DELIVERY-AND-MIGRATION.md)。不能宣称整个跨组件 DDL 只有一个原子提交。

跨 issuer 身份不能安全合并，down 明确报错。恢复边界是停止新版本、恢复迁移前备份和匹配的旧版本/配置，再验证后开写；升级后新增写入不能无损回退到旧身份结构。不得用清库替代恢复。

## 会话、CSRF 与撤销

| 路由 | 行为 |
| --- | --- |
| GET `/api/auth/session/login` | 创建当前浏览器的一次性 state，跳转 IdP |
| GET `/api/auth/session/callback` | 验证浏览器绑定、state/code/PKCE/nonce/token 与 scope，轮换 session ID，跳转站内 |
| GET `/api/auth/session/user-info` | 最小 principal；未登录/过期 401，响应 no-store |
| POST `/api/auth/session/logout` | 持久撤销应用 grant 后清 session；重复 OIDC 退出幂等；Basic 409 |

cookie 为 HttpOnly、Secure、SameSite=Lax、Path=/、无 Domain。idle 默认 30 分钟，每次访问续期；grant 绝对期限默认 12 小时，不随访问延长。pending 默认 5 分钟，原子 DELETE RETURNING 消费；浏览器保存最多 8 个关联 state，callback 在 SDK 消费前比对，另一浏览器不能消耗原事务。return target 在认证建立之前拒绝外域、scheme-relative、控制字符、反斜线与多层编码绕过。日志不记录 query 中的 code/state 或 session ID。

session store 复用 tower-sessions-sqlx-store 0.15.0，仅做 core 0.14→0.15 Id/Record/error 转换，create 后写回新 ID。多个实例共用 PostgreSQL，无 sticky-session 要求。每次认证读取 grant，缺失/过期/撤销均拒绝；logout 先持久化 revoked_at，迟到 save 即使重建 session 行也无法恢复身份。每 5 分钟清 session/pending/绝对过期 grant；撤销记录保留到绝对期限，清理任务跟随 App shutdown。

所有 cookie 写入需要精确 Origin、`X-Konobangu-CSRF: 1` 和 `application/json`（可带 charset），包括 GraphQL mutation、logout 与凭据检查命令。缺失/错误 Origin/header 为 403，错误内容类型为 415。前端 SDK/Apollo 统一发送凭据与这些字段。CORS/SameSite 不能替代这些检查。

user-info refresh 仅刷新应用身份，不保存或轮换 refresh token；本地退出不保证 IdP 全局 SSO 退出，也不立即感知 IdP 侧撤销。没有后台 IdP 撤销传播，应用 grant 到期或本地退出才终止当前应用认证。

## GraphQL、任务与前端状态归属

线上使用显式 async-graphql dynamic resolver。Seaography 仅提供类型/命名/过滤 helper，不注册其自动数据库 query/mutation/loader。HTTP operation 拥有 `Arc<Mutex<Option<DatabaseTransaction>>>`，在同一事务 SET LOCAL subscriber；root、关系、请求级 loader、自定义 CRUD、批量和任务 mutation 都借用此事务，resolver 不能自行 commit。关系递归前释放锁；loader 不跨请求缓存。mutation 成功提交之后才响应，有 errors 则回滚并返回 data=null，commit 失败同样不返回成功；query 保留合法 partial data，关闭只读事务。公开 extension 接口在 schema 声明的可空字段边界保留数据与 errors（含别名路径），非空字段错误继续传播；mutation 的可空输出错误同样触发 operation 回滚。取消时未提交事务 drop 回滚。未绑定/清空 GUC 时 RLS 返回无私有行。

owner 来自服务器认证映射，不接受前端 subscriber_id。新增资源自动注入 owner；修改 owner 输入被跳过。PostgreSQL FK 检查可能绕过 RLS，所以 GraphQL 写入和用户任务写入还检查所引用资源在同一身份事务内可见。introspection 使用框架 only_introspection，混合业务字段也不执行。系统任务/系统 cron 仅允许显式 Basic 管理员。

用户任务先短事务读取已验证 subscription，再关闭事务进行 HTTP/图片处理；资源 upsert 与关联以绑定 owner 的短事务一起提交。第三方凭据检查使用上述独立命令；后台 cookie 同步仅在明确的短业务阶段提交，不在 GraphQL operation 内调用。数据库回滚无法撤销已发出的外部请求。GraphQL 任务 enqueue/retry/delete 在原 operation 事务写入应用 runs/outbox；Cron occurrence、runs/outbox 和 next_run 同一事务提交。公开队列投递与受 fence 保护的业务阶段遵循[任务合约](003-TASK-DELIVERY-AND-MIGRATION.md)，不宣称外部效果 exactly-once。

前端 FoundationEnvironment 创建唯一 injection-js 根，environment/native injector/SDK facade 共用实例，公开 useFactory/useValue 和 inject 完成无装饰器注入。SessionContextClient 拥有只读消费的 session resource；根 DestroyRef 负责 SDK dispose，AppRuntime 负责 listener/React/Apollo 资源。403、网络/5xx、解析错误不自动进入登录循环；401 才进入未认证流程。

每个认证身份拥有独立 ApolloClient/InMemoryCache，退出/换身份先取消和 stop 旧 client，再切换 provider；epoch 与当前身份检查阻止晚到响应写入新缓存，普通 refresh 不重建 client。Query 目前没有私有请求，不新增重复缓存层。Apollo DeclareDefaultOptions 约束 query/watchQuery errorPolicy=all 与 mutation=none；client preset 生成 TypedDocumentNode，页面展示 partial data 与错误，mutation 错误不能作为成功结果。

凭据检查按钮使用同一 SDK transport；成功后刷新 Apollo 已激活的凭据列表/详情查询，不维护第二份实体缓存。身份变化、根销毁或组件取消会取消请求并拒绝迟到结果。

[English](../en/002-AUTHENTICATION-DECISION.md) | [中文](002-AUTHENTICATION-DECISION.md)

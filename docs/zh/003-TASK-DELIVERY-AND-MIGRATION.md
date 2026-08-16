# 003 — 任务投递与数据库升级

## 业务记录与 API

应用拥有 `task_runs` 与 `task_outbox`。SeaORM/migration 2.0.4、SQLx 0.9.0 执行业务事务，Seaography 2.0.0-rc.9 提供 schema helper。Apalis 1.0.0-rc.10、apalis-postgres 1.0.0-rc.9 负责投递、领取、heartbeat 和 orphan recovery。预发布版本精确固定，使用公开 API，无 fork。

用户/系统任务列表和详情读取 `task_runs` 上的应用视图。创建和重试在原 GraphQL operation 事务内写 run/outbox；后续 root/batch 失败时两者共同回滚，resolver 不访问队列网络。创建返回已排队任务 ID，不代表采集完成。浏览器实体由 Apollo 管理，四张管理表采用原生 Table 9.2.4 服务端分页/排序/筛选；排序、筛选、页大小变化回到第零页，选择使用稳定业务 ID。

状态保留 Pending/Scheduled/Running/Done/Failed/Killed，由任务命令与执行器改变；不暴露任意 owner/generation/次数/状态更新。任务 API 移除队列 lock/priority 实现字段，新增 `generation`、`cancelRequestedAt`。用户任务 owner 来自当前认证身份；系统任务管理始终要求 Basic，历史共享 subscriber 不赋予 OIDC 系统权限。外键引用必须属于同一 owner。

删除是取消或归档：Pending/Scheduled 变 Killed，Running 写取消请求；终态归档并从普通查询隐藏。持久 tombstone 防止复活。取消的 Running 在 lease 过期或无 lease 时原子转 Killed，记录 done_at 并清空 token/lease，不增加执行次数；取消、重放和现有 dispatcher 复用同一规则，不依赖另一份 envelope。重试只接受没有有效执行者的 Failed/Killed，重新验证输入/引用，推进 generation、重置次数并写新 outbox；并发重试只允许一个成功，其余明确冲突。无权限和冲突不得显示操作成功。

订阅删除是原 IdentityOperation 内的显式业务命令：事务级订阅 intent 排除新增任务引用，先锁 task 行再删除父资源，取消活动任务并保留历史/outbox。已有 fenced 业务阶段先结束，删除后新的阶段拒绝已取消 token。可空 live subscription FK 使用 ON DELETE SET NULL，payload ID 仅作历史；retry/replay 重新校验资源，详情显示已删除引用。Cron 创建遇到删除 intent 时非阻塞回滚，等待下次轮询。

## GraphQL 时间格式升级

schema context 启用 Seaography 官方 `types.timestamp_rfc3339=true`。DateTimeUtc 输出由 Chrono 展示字符串规范化为带 offset 的 RFC3339，保留数据库微秒；现有 scalar、过滤/输入类型及其解析器保持不变，没有自建序列化器或迁移数据库时间。当前 Time 日期过滤输入继续使用既有 `YYYY-MM-DD HH:mm:ss.f` 格式，此输出开关不增加 RFC3339 filter 输入支持。使用输出字符串做缓存或比较的调用方应按时间语义处理。WebUI 共享 Temporal.Instant 边界兼容旧 Chrono UTC/offset 值；无 offset 不解释为浏览器本地时间。真实普通角色 GraphQL 回归验证持久时间的 RFC3339/微秒输出及既有日期过滤。

## 投递、重试与 fencing

task_control 角色通过 `FOR UPDATE SKIP LOCKED` 领取到期 outbox，提交短事务后经公开 TaskSink 投递，再短事务确认。投递 lease 为 60 秒；失败保留记录并按 1/2/4/8/16/32/60 秒退避，之后上限 60 秒。重启轮询恢复过期领取。幂等 key 为 `task_id:generation`，envelope 仅包含这两个字段；worker 从可信 run 读取 owner 与版本化 payload。

业务次数只计实际执行，不随重复投递消耗。新任务最多执行三次，包含首次，重试间隔 5/30 秒。网络、HTTP 5xx 和可恢复数据库错误可重试；4xx、非法输入和 owner 失败不可重试。队列次数使用独立较高上限。业务错误仍作为错误进入公开 worker middleware；关闭信号打断重试等待，保留持久状态并停止 dispatcher/scheduler。

领取通过 generation/token CAS 替换执行 UUID。业务 lease 为 120 秒，每 30 秒续期；过期执行可接管。每个私有数据库阶段绑定可信 subscriber，提交前检查当前 token、有效 lease 与未取消状态。旧 generation、终态重复投递和旧 token 不能提交新结果。文件先暂存，再在短 fence 锁内原子替换；关联/来源主键使用唯一约束与 upsert。数据库事务不跨外部 HTTP。续期 SQL 等锁时仍 poll 业务执行；取消、续期错误/lease 失效与退出先停止并释放另一分支，再写终态。PostgreSQL 的在途语句由同一普通业务/scheduler 角色的短生命周期控制连接按 token 取消，事务局部 application_name 标记避免取消无关查询。控制连接显式关闭，不增加永久 pool 或权限；身份 3+2 预算不变。服务退出发送停止信号并 join worker 与轮询循环。

外部读取允许重放；后来取消不能撤回已提交的有效阶段或已发出的请求。当前合约是持久投递与带 fencing、可重复的业务阶段，不承诺外部副作用 exactly-once。

## Cron

Cron 由应用拥有，沿用 croner 4 + chrono-tz。一个短事务锁 due plan，写 task/outbox 并推进 `next_run`；`(cron_id,scheduled_at_utc)` 唯一。轮询是正确性来源，不依赖 NOTIFY。task_control 无私有应用或认证/session 表权限。

时间持久为 UTC，按配置 IANA timezone 计算。DST 重复本地时刻产生两个 UTC occurrence，缺失时刻跳过。停机漏触发合并为一次，并推进到当前时间之后。同一 Cron 有活动任务时跳过该 occurrence，记录原因且不积累无限 backlog。禁用/修改影响未来调度；数据库失败时 occurrence、outbox 与计划推进共同回滚。

## Owner 升级与恢复

存量安装与新数据库均使用停写流程。运行池不得使用 owner URI；运维控制升级时保持 `database.auto_migrate=false`。参见[认证与角色](002-AUTHENTICATION-DECISION.md)。

1. 停 WebUI 写入、recorder 及所有旧 worker/Cron/dispatcher；保留旧版本、配置和存储。迁移前备份并在隔离库演练恢复：

```sh
pg_dump --format=custom --file=/protected/konobangu-before.dump "$MIGRATION_DATABASE_URL"
pg_restore --exit-on-error --dbname="$ISOLATED_RESTORE_DATABASE_URL" /protected/konobangu-before.dump
```

2. 必要时配置精确已知 `database.migration.legacy_oidc_issuer`，不按名字/email 猜 owner。执行只读 owner preflight：

```sh
mise exec -- cargo run -p recorder --locked --bin migrate-up -- --environment production --config-file /path/to/recorder.config.toml --migration-preflight
```

报告列出旧任务数量及逐项问题。未知 schema/history/业务版本、混合或被改动 SQLx history、未知 kind/status/payload、无法归属的 owner/引用、重复 ID/occurrence 和近期旧 worker heartbeat 均在任务 DDL 前拒绝。应修正归属或停 worker，不改 checksum 绕过。

3. 移除 `--migration-preflight` 执行 up。优先使用 `database.migration.url`，否则指定 `database.url` 必须是 owner。各阶段可重入，不是跨组件单一原子提交：

| 阶段 | 结果 |
| --- | --- |
| 必要的历史 bootstrap | 由项目 Rust migration 创建重放本项目迁移所需的空 jobs/workers 兼容表；已有旧 schema 只校验并保留 |
| 应用事务 | 身份迁移及显式 version-0 payload 转换；导入 ID/owner/引用/时间/次数/预算/错误；创建 run/outbox、替换视图并移除旧业务队列触发器 |
| 归档 | 保留 `apalis_legacy_v07` 和确认属于旧队列的 history，撤销普通角色访问 |
| 新队列初始化 | 公开 PostgresStorage migrator 显式创建 `apalis`，history 写入 `apalis._sqlx_migrations` |
| 成熟 session store | owner 通过发布 store 的 SQLx 0.8.6 临时池初始化既有 schema，随后关闭 |

Done/Failed/Killed 不自动投递；Pending/Scheduled 写 outbox；已停止的 Running 转 recovering Pending，保留旧执行信息与合法预算，恢复领取不重复增加已计次数。旧 payload 经注册的 typed task 定义转换。旧 schema 仅 owner 可读归档，不由新 worker 消费。

4. 任何阶段失败都保持停写，修复后重跑 up；不得恢复并发旧消费。owner 入口自动建立 NOLOGIN 能力角色、对象授权与配置登录的成员关系，无需额外授权命令；默认新部署在启动时执行这些阶段。app_scoped_access 有绑定 subscriber 的 task/outbox 权限；task_control_access 有跨 subscriber 的 queue、Cron、task/outbox 权限；auth_access 无私有应用、任务或队列权限。见[角色与连接池契约](002-AUTHENTICATION-DECISION.md)。

5. 按 status/owner/引用核对计数、计划时间、预算和抽样 payload；确认归档隔离、新 history、生产登录、任务创建/查询/执行/重试/取消及跨 owner 拒绝，再恢复写入。

回退为停写，恢复迁移前备份、存储及匹配旧版本/配置。down 明确拒绝不可逆身份/任务迁移；新写入不能无损转回旧模型。不得清库，不得双版本同时消费。归档清理需后续运维显式决定。

## 相关契约

配置插值、secret 与媒体能力见[媒体与配置](004-MEDIA-AND-CONFIGURATION.md)；身份/session 连接池与角色见[认证](002-AUTHENTICATION-DECISION.md)；工具链、平台构建与验证见[开发验证](001-DEVELOPMENT-VERIFICATION.md)。

任务投递迁移中的表和列名称由 migrations/defs.rs 的 Rust 标识符定义。SeaQuery 生成其可表达的结构与查询；PostgreSQL 专有策略、部分索引和切换 SQL 在 Rust 中引用这些标识符。Apalis 0.7 归档契约独立固定，不随当前应用实体调整，避免未来实体重命名改变历史源 schema。

[English](../en/003-TASK-DELIVERY-AND-MIGRATION.md) | [中文](003-TASK-DELIVERY-AND-MIGRATION.md)

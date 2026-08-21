# 004 — 图片与部署配置

JXL/WebP 共用一个有界队列、一个自有 Rayon pool 和一份 admission 预算。获得 admission 的任务在有空闲 worker 时立即开始，并各自立即交付结果；队列限制等待中的工作量，预算限制并发工作集。正常构建默认生成 JXL + WebP，自动 AVIF 拒绝。静态显示与 [R8 渐进体验](roadmap/001-SHORT-TERM-ROADMAP.md) 分开验收。

JPXL facade 固定 Git rev `1e2004aa0672259279cc24a52d4b8e264f675b77`，Balanced、quality 77、threads 1；开启固定的有界 policy features，单图只有一次 facade 调用。库内策略最多 5 次评分 probe、3 次精确计价，不添加应用层搜索、体积切换或无损兜底。当前输出普通单 pass。WebP 保留 webp 0.3.1/libwebp-sys 0.9.6、quality 80，单任务不启用内部线程。

封面先应用方向，将接受的 8-bit sRGB 样本在编码后的 sRGB 空间按 alpha 合成白底，再用 Lanczos3 将长边限制到 1600，不放大小图。JXL/WebP 使用同一规范化像素，隐藏 RGB 不污染透明边缘，原图保留。通用 WebP API 保留 alpha；通用 JXL 的 RGBA 或旧无损请求明确失败，不静默白底后宣称无损。JPXL preset API 为 `{"mime_type":"image/jxl","preset_version":1}`，GraphQL 输入为 `mimeType`/`presetVersion`。

## 配置来源

Figment 按原文件名优先级选择单个 TOML/JSON/YAML/YML。YAML 使用 `YamlExtended` 支持 anchor/alias/merge key；merge key 是显式扩展，不承诺所有 YAML 1.2 处理器支持。优先级为 defaults < file < canonical 双下划线变量。已选 dotenv 与进程环境形成启动快照，进程优先；加载不修改进程环境。

只展开最终生效的 file-origin 字符串叶子，包括数组内字符串；被覆盖的占位符不会报错。键、默认值、canonical 值、文件 secret 和展开结果都不再展开。字符串仍为字符串；原生数字/布尔/列表结构及 canonical Figment 环境类型解析保持类型。

| 语法 | 行为 |
| --- | --- |
| `$NAME`、`${NAME}` | ASCII 名称；缺失/空值失败 |
| `${NAME:-literal}` | 缺失/空值用字面默认；允许空默认，再做字段校验 |
| `$$`、`$${NAME}` | 字面 `$`、`${NAME}` |
| 嵌套、`:?`、`:+`、`${NAME-default}`、命令替换、未闭合花括号 | 返回字段路径及脱敏类别 |

默认 literal 不允许 `$` 或花括号。单独 `$` 后普通非名称字符保留字面值。没有 tilde/shell/模板执行或配置继承。已加引号的 `{{ ... }}`、`{% ... %}`、`{# ... #}` 保持普通字符串；旧未加引号模板由原生解析器拒绝并给脱敏迁移提示。

```toml
[storage]
data_dir = "${DATA_ROOT:-/var/lib/konobangu}"
[media]
# Omit auto_optimize_formats to inherit compiled defaults.
# Optional WebP-only override: auto_optimize_formats = ["image/webp"]
[media.execution]
deadline_seconds = 30
queue_capacity = 8
concurrency = 1
```

```yaml
server:
  <<: &server_defaults
    binding: 0.0.0.0
    port: 5001
storage:
  data_dir: '${DATA_ROOT:-/var/lib/konobangu}'
```

## 文件 secret 与日志

以下六个 canonical 变量接受 UTF-8 普通文件的绝对路径，也允许符号链接指向普通文件：

| 变量 | 字段 |
| --- | --- |
| `AUTH__PROVIDER__PASSWORD_FILE` | `auth.provider.password` |
| `AUTH__PROVIDER__CLIENT_SECRET_FILE` | `auth.provider.client_secret` |
| `AUTH__SESSION__DATABASE_URL_FILE` | `auth.session.database_url` |
| `DATABASE__URL_FILE` | `database.url` |
| `DATABASE__MIGRATION__URL_FILE` | `database.migration.url` |
| `SCHEDULER__DATABASE__URL_FILE` | `scheduler.database.url` |

启动只读一次，最多 64 KiB。空、不可读、超限、非 UTF-8、目录、设备、FIFO 拒绝。同目标普通 canonical 与 `_FILE` 冲突，空值也算；大小写归一重复目标也失败。应用命名空间内未知 `_FILE` 拒绝，无关工具变量忽略。不支持别名 `_FILE` 或配置文件 `_file`。

内容保持字面值，包括尾换行、引号和 `${...}`；密码不 trim，URI 中空白/换行由校验拒绝。auth/database/task 敏感 Debug 与配置/连接错误脱敏，没有配置导出端点。文件应无尾换行并放在已提交配置之外：

```text
node -e "require('node:fs').mkdirSync('secrets',{recursive:true}); require('node:fs').writeFileSync('secrets/basic-password','replace-with-deployment-password',{mode:0o600})"
```

Compose secrets 挂载为文件，不将秘密插入 TOML：

```yaml
services:
  recorder:
    image: your-recorder-image
    environment:
      AUTH__PROVIDER__PASSWORD_FILE: /run/secrets/basic_password
      DATABASE__URL_FILE: /run/secrets/database_uri
    secrets:
      - basic_password
      - database_uri
secrets:
  basic_password:
    file: ./secrets/basic-password
  database_uri:
    file: ./secrets/database-uri
```

其余原生 auth/server 配置单独提供。OIDC 使用 `AUTH__PROVIDER__CLIENT_SECRET_FILE`；数据库 URL 自动初始化能力角色，可选迁移 URL 用于选择独立 owner 凭据。不承诺热加载/secret 轮换。

## 日志配置

`recorder.config.toml` 的 `[logger]` 仍是日志入口，示例包含可选的滚动文件表。`logger.enable` 仅控制控制台输出；文件输出由 `logger.file_appender.enable` 独立控制。支持 `compact`、`pretty`、`json` 格式和 `off/error/warn/info/debug/trace` 级别。

全局事件过滤优先级为有效 `RUST_LOG` → `logger.override_filter` → `logger.level` 对 recorder、sea_orm_migration、tower_http、sea_orm、sea_query 的默认模块过滤。文件 `level` 进一步限制全局允许的事件，不能重新启用已被全局过滤的事件。要让文件保留 debug 而控制台保留更少事件，当前没有独立的控制台过滤字段；不要把文件阈值误认为覆盖全局阈值。无效显式 `override_filter` 会使初始化失败。

```toml
[logger]
enable = true
level = "info"
format = "compact"
pretty_backtrace = false
override_filter = "recorder=debug,tower_http=info,sea_orm=warn"

[logger.file_appender]
enable = true
non_blocking = true
level = "info"
format = "json"
rotation = "daily"
dir = "./logs"
filename_prefix = "recorder"
filename_suffix = "log"
max_log_files = 7
```

相对 `dir` 基于进程工作目录。轮转支持 `minutely/hourly/daily/never`；关闭控制台后文件仍可工作。也可使用 canonical 环境变量，例如 `LOGGER__LEVEL=debug`、`LOGGER__FORMAT=json`、`LOGGER__OVERRIDE_FILTER=recorder=debug`，或完整提供 `LOGGER__FILE_APPENDER__...` 表所需字段。`database.pool.log_queries` 独立决定是否产生 SQL 日志，最终仍受日志过滤器限制。未实现的旧 `logger.filter` 已移除；logger 与 file_appender 的未知键报错，使用 `override_filter`。

## 执行预算与兼容

| 配置 | 默认值与含义 |
| --- | --- |
| `limits.input_bytes` | 源字节 32 MiB |
| `limits.dimension` / `limits.pixels` | 每边 8192 / 16,777,216 像素 |
| `limits.decode_bytes` | 解码分配 128 MiB |
| `execution.working_set_bytes` | JXL/WebP 共用 admission 估算预算 512 MiB，含排队源字节、解码/缩放、codec 与输出；不等于 RSS 硬上限 |
| `limits.output_bytes` | 每项输出 64 MiB |
| `execution.concurrency` / `execution.queue_capacity` | 同时执行 1 / 排队 8；concurrency 决定编码 pool 的线程数 |
| `execution.deadline_seconds` | 等待与结果有效期 30 秒，最大 300 秒 |

超时/取消丢弃迟到结果；进入同步 codec 后不能强杀线程，实际任务结束前仍占用容量与预算。已完成或跳过的任务先释放工作集预算，再通知调用方；调用方收到完成结果时，该任务已不再占用预算。解码、规范化及编码之间检查取消；排队取消项跳过。shutdown 停止 admission、取消未执行项，在 Tokio worker 之外等待真实工作完成；服务退出路径调用该 shutdown。各 join 分支内部捕获 panic，之后可继续调度。OOM、abort、native 崩溃仍影响服务进程，估算预算不能提供逐任务 OS 隔离。

仅优化静态 8-bit SDR RGB/RGBA，应用 EXIF orientation。嵌入 ICC、PNG cICP/HDR 标记、无 sRGB 声明的 gamma/chromaticities、动画或其它位深受控失败；接受隐式与显式 sRGB。任何失败保留原图。

旧 `max_encoder_bytes`（RSS 监督阈值）被拒绝，需显式改为 `execution.working_set_bytes` admission 估算。旧 `jxl_timeout_seconds` 拒绝，改用 `media.execution.deadline_seconds`。旧 `jxl_distance`、`jxl_quality`、`jxl_effort`、`jxl_speed`、`jxl_encoder_path`、`jxl_max_address_space_bytes` 明确拒绝；这些字段不能对应 JPXL quality 或进程内硬隔离。任务迁移及恢复见 [任务与升级](003-TASK-DELIVERY-AND-MIGRATION.md)。

采集、worker 与 HTTP 共用衍生计划。有界 manifest 保存 source fingerprint/SHA256，以及不可变衍生的 profile/path/length/SHA256。路径保留完整原扩展名，`a.jpg` 与 `a.png` 不混同。manifest 不作为授权源，每条路径须符合该 source namespace 的正式构造。manifest version 2 保持路径兼容，新 profile 包含 JPXL 完整 rev、preset 与白底/尺寸策略。缺失、损坏、旧 profile 或源失效时按 Accept 回退当前原图或返回 406，不凭同 basename/mtime 的 sibling 猜测来源。旧 WebP/AVIF/JXL 直接 URL 仍可读取；backfill 发布当前 manifest，不改写原图或已有 URL。

正式源写入使 manifest 失效。发布先检查源版本和任务 fence，再获取固定完整路径 advisory resource lock；编码、读源和大临时文件写在短事务之外。衍生原子发布后才发布 manifest；清理本次 owned 临时文件，历史不可变版本和原图不自动 GC。外部直接改文件通过可用 fingerprint 检测，并需显式重建。

公共 backfill 显式执行，默认 dry-run、100 个，不扫描 subscriber 目录，GET 不触发：

```bash
mise exec -- just media-backfill --config-file /etc/konobangu/recorder.config.toml
mise exec -- just media-backfill --config-file /etc/konobangu/recorder.config.toml --write --limit 100 --offset 0
```

写模式需要正常 application/identity/scheduler 配置。重复入队跳过等价活跃任务与已完成当前 profile。私有图通过原 owner 授权任务路径生成。

## HTTP 与页面

Img 只给本 recorder 同源 `/api/static/` 加 `optimize=accept`，保留外部/签名/头像/icon/data/blob URL。格式选择先于条件和 Range。JXL 必须显式正 q 的 image/jxl，wildcard/application/jxl 不启用。最具体范围定 q，0 排除；平局 合格渐进 JXL → 已存在的合格普通 JXL → WebP → 原图，不依赖头顺序。无 Accept 发原图，畸形 400，全部排除 406。

协商 200/206/304/406 合并 Vary: Accept。public 使用 public,no-cache，private 使用 private,no-cache，每次私有重验先授权。衍生使用内容 hash 强 ETag；无可信原图内容 hash 时使用路径/精确 metadata 弱 validator。成功 GET/HEAD 条件保留缓存头、清空 body/部分长度；拒绝/错误不转304。只有 GET 在条件请求后处理 Range。锁定 http-range 0.1.5 的 HttpRange::parse 解析并归一范围，支持前导零、OWS 和混合零后缀；单独零后缀/全不可满足的 NoOverlap 返回 416 及 bytes */length，InvalidRange 返回受控 400。应用在解析前统计列表成员，超过 16 个则完整 200，避免 parser 分配任意长列表；对库返回的 start/length 检查零长度、加法溢出及表示边界，重叠仍回完整 200。未知单位、空文件忽略 Range，重复 Range header 仍拒绝。接受该库对起点超界成员先跳过的容错（例如 bytes=99-2 对 10 bytes 返回 416），不承诺比库更严格的整数语法。multipart 长度使用 checked arithmetic，与实际 CRLF/body 一致。HEAD 忽略 Range/If-Range，直接返回完整 metadata，不打开 body 流，并保留授权后的 304。If-Range 仅当前表示合法强 ETag 匹配才返回部分传输；日期形式、弱/不匹配 tag、非法条件均返回完整 200，当前秒级日期没有强 validator 证据。现有 Tower compression predicate 排除已压缩 image，不为 validator collect body。

部署关闭媒体代理缓冲，保持原授权路由：

```nginx
location /api/static/ {
  proxy_pass http://recorder:5001;
  proxy_buffering off;
  gzip off;
}
```

真实浏览器完整显示与渐进截图分开验收；完整解码不算渐进。固定语料通过独立 djxl/SSIMULACRA2 验证，工具只用于离线测试。

## 构建与部署

[开发验证](001-DEVELOPMENT-VERIFICATION.md) 是编译配置、平台构建和门禁的权威入口。recorder 单可执行文件包含 JPXL Rust 编码与 libwebp/sharpyuv、aws-lc 等静态原生依赖；项目及 JPXL 均为 MIT，发行附带实际依赖许可证。源码仅来自 Cargo 固定 Git/registry 缓存，仓库不复制 codec 源码。

生产需要配置/数据及 HTTPS CA 信任，不需要外部 codec。离线 `recorder-cli media-smoke --output OUTPUT` 可通过真实适配器生成两种封面；可加 `--input IMAGE`。它不初始化数据库、secret 或网络，也不使用旧 worker 协议。正常构建默认 JXL + WebP；`--no-default-features` 默认 WebP，包括 `[]` 的显式格式列表始终有效。静态默认不依赖 R8 门禁。

## 配置体系

文件统一 snake_case，环境变量使用完全相同的字段路径转大写、层级用 `__` 分隔。连接统一 `_url`，时长显式 `_ms`/`_seconds`，容量显式 `_bytes`，并发与排队分别 `_concurrency`/`_capacity`。服务契约不使用存储 backend 的名字。根、database、auth、scheduler、media、logger 中未知字段给脱敏错误；默认值由类型管理，缺少必要凭据仍报错。

| 分组 | 归属 |
| --- | --- |
| `server`、`logger`、`storage`、`graphql`、`mikan` | HTTP、日志、文件、GraphQL 限制、源站集成 |
| `auth.provider` | `type`；Basic 的 `username/password`；OIDC 的 `issuer/audience/client_id/client_secret/extra_scopes/extra_claims` |
| `auth.session` | 可选 `database_url`（继承 database.url）、固定 `public_url`、`cookie_secure`；秒数形式的 idle/absolute/login timeout 与 cleanup interval |
| `database` | 默认 `url`；`pool.log_queries/min_connections/max_connections/connect_timeout_ms/idle_timeout_ms/acquire_timeout_ms`；可选 owner `migration.url`、默认开启的 `auto_run`、`legacy_oidc_issuer` |
| `scheduler.database/workers/cron` | 可选 `url`（继承 database.url）；`subscriber_concurrency/system_concurrency`；`poll_interval_seconds` |
| `media` | `auto_optimize_formats`；`encoding.webp.quality` 和显式 API 的 `encoding.avif.quality/speed/threads`；`execution.deadline_seconds/queue_capacity/concurrency/working_set_bytes`；`limits.input_bytes/output_bytes/dimension/pixels/decode_bytes` |

仓库 `recorder.config.toml` 保留原有连接池行为：SQL 日志开启、连接/空闲超时均为 500 ms、连接数 1–10；这些是该文件的显式覆盖，不能与类型默认值混淆。字段重组不自动调优参数，部署可通过对应 canonical 路径覆盖。启动先迁移并配置能力授权，再打开受限运行连接池；`database.migration.auto_run` 默认 true。独立 owner URL 与外部迁移控制均为可选策略，见 [013](002-AUTHENTICATION-DECISION.md) 与 [014](003-TASK-DELIVERY-AND-MIGRATION.md)。

这是配置 breaking change：按此表重组已有文件与 dotenv，移除 HOST/DATABASE_URL 别名、平铺 auth、task.queue_database_uri 和平铺 media 参数。scheduler 负责投递、执行、重试与 Cron，公开名字不绑定队列 backend。Rust wire adapter 保留内部服务/领域类型，其历史成员名不成为公开配置字段。JPXL 业务 preset 仍固定在代码中，不虚构画质映射或新增 backend 参数。

可选的独立调度凭据可覆盖默认继承的数据库 URL：

```toml
[scheduler.database]
url = "${SCHEDULER_URL}"
[scheduler.workers]
subscriber_concurrency = 2
system_concurrency = 2
[scheduler.cron]
poll_interval_seconds = 30
```

[English](../en/004-MEDIA-AND-CONFIGURATION.md) | [中文](004-MEDIA-AND-CONFIGURATION.md)

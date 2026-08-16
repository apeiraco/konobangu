# 001 — 短期路线图

本页只维护后续决策与验收条件。当前运行契约见 [认证](../002-AUTHENTICATION-DECISION.md)、[任务与升级](../003-TASK-DELIVERY-AND-MIGRATION.md)、[媒体与配置](../004-MEDIA-AND-CONFIGURATION.md)；已交付变更集中于 [CHANGELOG](../../../CHANGELOG.md)。轮次指导和证据属于 `temp/` 工作材料，不是部署操作手册。

## 发布门禁

当前 minor 发布候选已完成独立 review，CLI 干净检出的依赖安装顺序问题已关闭。现等待维护者决定版本、tag 和发布，不追加下一轮实施计划。发布操作继续遵守 [开发验证契约](../001-DEVELOPMENT-VERIFICATION.md)；若继续修改生产源码或构建依赖，应按影响范围追加验证。架构与操作说明以对应的当前契约页为准。

后续 HMR 复审新发现开发门禁缺口：vite-plugin-monaco-editor 1.1.0 使用 Node 26 已移除的递归 rmdirSync，完整开发服务不能启动。发布前需修复 Monaco worker 集成，并在 mise 固定的 Node 上验证完整 Vite 开发配置、组件 Fast Refresh 和应用根更新；隔离 HMR fixture 通过不能关闭此项。本地复审记录位于 `temp/CONFIGURATION_BINARIES_RESOURCES_zh.md`。

## 保留的后续工作

| 工作项 | 方向 | 关闭条件 |
| --- | --- | --- |
| R6 模型工具链 | 以真实使用场景验证 Python/Animeta 模型加载、推理、数据集和资源成本；架构提案见 [Animeta](002-ANIMETA-MODEL.md) | 锁文件更新不能替代模型执行，需明确可复现样本、效果和资源预算 |
| R7 Awa 替换 | 默认候选为 Awa；等待运行时及官方 SeaORM 适配共同支持 SQLx 0.9 和项目 ORM 版本后启动实验 | 实验成功后完成正式迁移并独立验收，不能只提交探针 |
| R8 渐进 JXL | 保持静态 JXL 默认，真实浏览器验证响应尚未完成时出现可辨识预览和随后细化 | 从生产 HTTP 到实际页面完成录制；完整文件解码不等于渐进体验 |
| 会话存储适配 | 等待成熟 session store 支持 SQLx 0.9，收敛兼容连接池 | 保持持久撤销、重启、多实例、清理及总连接预算 |
| Rust stable | 上游 quirks_path 不再需要 nightly 后替换固定 nightly | Windows/macOS/Linux 默认、所有有效 feature 组合和裁剪分支全部通过 |

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

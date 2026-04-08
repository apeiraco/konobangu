# 010 — Animeta 学习型解析方案（Python 训练 + Rust 推理）

## 状态

提案（面向未来实现；与当前 `legacy` / `probability` 代码可并行演进）

## 目标受众

- 维护 `packages/animeta`（Rust）与 `packages/animeta-py`（Python）的开发者
- 需要扩展番剧名、资源文件名解析能力的架构讨论

## 目的

在保留 **`OriginNameMeta` 对外契约** 的前提下，用**数据驱动**的小模型替代（或增强）纯规则 + 手工打分的「概率引擎」，并明确与 **legacy（nom 规则）** 及 **Aniparse 式拆解维度** 的对应关系，便于语料标注与评测。

---

## 1. 为何要从「编译器式规则」走向学习型

当前 legacy 路径本质是 **组合子解析器**：`FansubComp` → 扫描至 `EpisodeComp` / `MoiveComp` → `BangumiComps`（标题 + 季）→ `ExtraComps`（按分隔拆桶再尝试匹配字幕 / 分辨率 / 来源等）。见 `OriginNameEpisode`、`OriginNameMovie` 与 `OriginNameMeta` 的装配逻辑。

这种方式的优点是**可解释、可单测**；缺点是：

- **实体边界**与**语素**（何为一集标题、何为宣发块）强依赖书写顺序与分隔符假设，新站点格式往往要**改文法**。
- 输出被压平成 **`OriginNameMeta` 七字段**，中间结构（多段标题、多级来源、合集范围等）**无法自然表达**，与「多任务、多 span」的标注需求不对齐。
- 「概率」分支若仍靠人工调分与 tie-break，规模一大就难以维护。

因此推荐：**训练侧在 Python 用成熟生态做序列建模与导出；推理侧在 Rust 用 ONNX 等固定图**，与现有二进制栈一致。

---

## 2. Legacy 拆解维度（实现层面的真实结构）

下列与 `packages/animeta/src/extract/legacy.rs` 中的类型对应，可作为**标注 schema 的语义来源**。

| 维度 | Legacy 类型 / 行为 | 聚合到 `OriginNameMeta` |
|------|---------------------|-------------------------|
| 字幕组 / 发布方 | `FansubComp`（首部括号块等） | `fansub` |
| 番剧标题 + 季 | `BangumiComps`（`name` + `SeasonComp`） | `name`, `season`, `season_raw` |
| 话数 / 合集 | `EpisodeComp`（括号、`EP`、`第N话`、`- N`、合集后缀等） | `episode_index` |
| 剧场版标识 | `MoiveComp`（与 Episode/Movie 根择一） | 影响是否走 Movie 分支；话数常置 1 |
| 附加元数据 | `ExtraComps`：对剩余串按分隔拆分后**依次尝试** `SubtitleComp`、`ResolutionComp`、`SourceL1/L2`、`RegionLimitComp` | `subtitle`, `resolution`, `source` |

**要点**：`ExtraComps` 是**启发式分桶**，不是全局最优切分；这与学习型方案中「显式 span + 类型」的标注目标一致，也是规则难以拆「语素」的根源。

---

## 3. Aniparse 式拆解维度（与 probability `Label` 对齐）

参考 Aniparse 类工具常见的**区块 / 元素**划分，并与当前 `extract::probability::types::Label` 对照，便于统一**训练标签集**（可略作增删）：

| 语义角色 | 说明 | 与 `Label` 的对应（示例） |
|----------|------|---------------------------|
| 发布组 / 字幕组 | 首部或固定括号内的组名 | `ReleaseGroup` |
| 标题主名 | 含多语言 slash 的展示名 | `Title` |
| 季 / 期 / ordinal | `S2`、`第二季`、`2nd STAGE` | `SeasonPrefix`, `Season`, `SeriesType` 等 |
| 话数 / 范围 | 单集、区间、合集 | `EpisodePrefix`, `SequenceNumber`, `SequenceRange` |
| 视频 / 音频 | 编码、容器、bit 深 | `VideoTerm`, `AudioTerm` |
| 分辨率 | `1080p`、`1920x1080` | `VideoResolution` |
| 来源 | WebRip、BDRip、平台名 | `Source` |
| 字幕 / 语言 | 简繁、中日 | `Language`, `SubsTerm` |
| 其它宣发 | 招募、合集标记等 | `ReleaseInformation`, `Other` |
| 结构符号 | 括号、分隔符 | `Bracket`, `Delimiter`, `ContextDelimiter` |

学习型方案不必逐 token 复刻上述枚举；但 **NER 标签集**应能**无损投影**回上述语义，再 **compose** 成 `OriginNameMeta`（见下文）。

---

## 4. 推荐总体架构（Python + Rust）

### 4.1 分工

| 阶段 | 语言 / 工具 | 说明 |
|------|-------------|------|
| 数据与银标 | Python + JSON Schema | LLM 批量标注、人工抽检、与 legacy 对比 |
| 训练 | PyTorch + 小 Encoder（或多任务头） | 序列标注为主，整句分类为辅 |
| 导出 | ONNX | 算子与动态 shape 在导出时固化 |
| 推理 | Rust + `ort`（ONNX Runtime） | 与 `recorder` / `animeta` 同进程或侧车 |
| 兜底 | 现有 `legacy::OriginNameRoot::parse_comp` | 低置信度或 schema 校验失败时回退 |

### 4.2 模型形态（建议）

1. **共享编码器**（字符级或小词表子词；文件名建议**字符级或字节级 UTF-8** 以降低与特殊符号的耦合）。
2. **序列头**：对每字符（或子词）预测 **BIO / 多类型** 标签，覆盖上表语义角色（可合并稀有类为 `OTHER`）。
3. **句子级头（可选）**：对整句预测「是否有剧场版」「主要来源 coarse 类」等，辅助 resolve 冲突。
4. **开放集（可选）**：未知来源 embedding 离线聚类或单独 metric 头；线上一律写入 `source` 字符串，不强行对齐封闭类别表。

### 4.3 与 `OriginNameMeta` 的映射

保持 **JSON 契约不变**（见 `OriginNameMeta` 定义）：

- 从 span 集合 **确定性规则** 合成：`name`（多段 `Title` 合并策略需固定，如 slash 连接与首段剧场版）、`season` / `season_raw`、`episode_index`、`subtitle`、`source`、`fansub`、`resolution`。
- **版本化**：`model_version` + `tokenizer_version` 与推理绑定，避免仅换模型不换预处理。

---

## 5. 语料与监督

| 来源 | 用途 |
|------|------|
| Mikan 等站点历史番剧名 / 展示名 | 域内语言分布、标题多样性 |
| 真实资源文件名（若可合法收集） | 缩小与 BT 命名的 **domain gap** |
| LLM 银标 | 快速扩量；需 schema 固定与一致性规则 |
| Legacy 输出 | 弱标签或一致性过滤；争议样本人工标注 |

银标必须经过 **schema 校验 + 与 legacy 差异统计**；高价值错误类型进入金标或小批量人工修正。

---

## 6. 评测与上线策略

- **单元对齐**：延续现有 `OriginNameMeta` 相等性测试思路，增加「模型 vs legacy」**双跑 diff** 报表。
- **置信度阈值**：低于阈值走 legacy，避免线上回归。
- **灰度**：按用户/任务类型放量，监控字段级准确率与回退率。

---

## 7. 里程碑（建议）

1. 冻结 **标注 schema**（span 类型 + 与 `OriginNameMeta` 的合成规则文档）。
2. 构建 **10k～100k** 级银标 + 小量金标。
3. PyTorch 训基线模型 → ONNX → Rust `ort` 打通端到端延迟测试。
4. 与 `probability` 并行对比，再决定是否默认走模型或仅增强部分字段。

---

## 8. 与当前代码的关系

- **`legacy`**：长期作为**金标准与兜底**，不因引入模型而删除。
- **`probability`**：可逐步**收缩**为特征提取或训练数据预处理，或与模型 **ensemble**；具体以评测为准。

参考论文和项目：

- [GoLLIE: Guideline following Large Language Model for Information Extraction](https://github.com/hitz-zentroa/GoLLIE)
- [GLiNER: Generalist and Lightweight Model for Named Entity Recognition](https://github.com/urchade/GLiNER)
- [GLiNER2: Unified Schema-Based Information Extraction and Text Classification](https://github.com/fastino-ai/GLiNER2)
- [ZERONER: ueling Zero-Shot Named Entity Recognition
via Entity Type Descriptions](https://aclanthology.org/2025.findings-acl.805.pdf)
- [OpenBioNER-v2](https://huggingface.co/blog/alecocc/openbioner-v2)
- []()

---

[English](../en/010-ANIMETA-MODEL.md) | [中文](010-ANIMETA-MODEL.md)

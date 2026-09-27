# 未被接线的契约词汇：`IoObservation`（2026-09-28）

本文件记录一次扫描发现的**文档/代码分歧**，供维护者裁决。**不含任何代码或测试改动。**

基线：`7b4d0f5`（本线第五十一个切片 `987b6e5` 的门禁记录之后）。

## 发现

`neko_carrier::IoObservation` —— **整个类型在全工作区只出现一次，即它自己的声明**
（`crates/neko-carrier/src/lib.rs:909`）。它：
- 从未被当作类型使用（没有任何签名、字段或 `let` 提到它）
- 从未被构造
- 从未被匹配
- 没有 `impl` 块
- 没有任何形式的再导出（`pub use`）或别名导入

换句话说，**这个类型是死的**。

但 `docs/decisions.md` 的 **D054**（2026-08-29，状态「Candidate architecture
hardening — no wire change」）用**现在时**断言它是契约的一部分：

> The generic `Carrier` contract now exposes only `CarrierLimits`, `CarrierError`,
> and opaque `IoObservation` vocabulary.

所以这是**一个被决策记录写成契约、却从未接线的词汇**。它的文档注释也表明了意图：
「Observation returned by an adapter when an operation has no payload.」

## 为什么之前的扫描没抓到

我此前做「枚举变体级零引用」扫描时，统计的是 `IoObservation::WouldBlock` /
`IoObservation::Closed`，得到 3 和 2 次——**但这些全部是同名变体污染**
（`UdpError::WouldBlock`、`CarrierError::Closed` 等等；在整个 neko-carrier 里
`WouldBlock` 出现 11 次、`Closed` 出现 36 次，绝大多数属于别的类型）。于是我漏掉了
「**这个类型本身**零使用」这一事实。

**方法论教训（值得复用）**：对枚举做零引用扫描时，**只统计 `Enum::Variant` 是不够的**
——同名变体会被其它类型污染。**必须同时统计「类型名本身」的出现次数**；若类型名只出现
一次（声明处），那么**它的全部变体无论变体名统计结果如何都是死的**。

## 为什么**不**删除

与 `PmtuSendOutcome::Sent` 同类：**这是被文档化的（契约级）未实现意图，而不是复制残留**。
删除它会**与一条已记录的决策（D054）相矛盾**；而修改 D054 或把它接线，都属于架构决策，
**超出本守卫的自主范围**。因此：

- **保留该类型**，不在本轮删除
- 记为**维护者裁决项**：要么把 `IoObservation` 接进 Carrier 契约（如 D054 所描述），
  要么修正 D054 并删除该类型

## 与先前记录的其它保留项的关系

这批「被文档化但未实现/未接线」的项现在共 2 个：

| 项 | 文档依据 | 状态 |
|---|---|---|
| `neko_reliable::PmtuSendOutcome::Sent` | 无明确文档，但「发送结果」枚举的成功臂语义自然 | 保留 |
| `neko_carrier::IoObservation` | **D054 明写它是 Carrier 契约词汇** | 保留 |

另有 3 个受限核心 crate（`neko-session`）里语义合理的未实现变体
（`StreamState::Reset`、`RuntimeError::Deadline`、`LedgerError::ContextMismatch`），
已在 `docs/notes/dead-enum-variants-20260927.md` 记录为保留。

## Evidence boundary

仓库级静态引用扫描，开发者本地执行；不是审阅者本地、不是托管 CI、不是网络或性能证据。
未改动任何生产源码或测试（本文件是唯一新增内容），因此不需要 exact-tree 门禁。
无 H-I4-119 / D019 / release flag / policy 值变更。`READY_LIVE: none`。

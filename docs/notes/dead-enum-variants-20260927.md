# 未被引用的公开枚举变体（删除候选）— 2026-09-27

本文件记录一次仓库级扫描的发现：**11 个公开枚举变体在工作区中从未被构造、也从未被匹配**。
它们属于删除候选，**不是**覆盖候选——本文件不附带任何测试改动，也不需要门禁。

基线：`e58bdae`（本线第十一个切片 `355c6c8` 的门禁记录之后）。
扫描范围：`crates/**`（所有 `src/lib.rs` 与 `tests/*.rs`）。

## 发现方法（可复现）

对每个 `pub enum` 提取变体，统计形如 `Enum::Variant` 的引用：

```sh
grep -ro -- "EnumName::VariantName" crates/ | wc -l
```

**只统计 `Enum::Variant` 形式是刻意的保守选择**：本仓库的代码风格一律写限定路径（实测 `StreamState::Open`
`PathError::InvalidTransition` 等都是限定形式）。为排除「有 `use Enum::*` 后写裸变体名」的可能，
另外确认了**这些枚举在工作区中都没有 glob 导入**，并且**这 6 个枚举都没有任何 `impl` 块**——
因此不存在「`Self::Variant` 其实属于本枚举」的漏计。那几处 `Self::MessageTooLarge` 经逐行确认属于
`impl Display for UdpError`、`impl From<UdpError> for CarrierError`、`impl From<TcpError> for CarrierError`，
指向的是**别的**错误类型。

## 发现（11 个变体，4 个枚举，3 个 crate）

| 枚举 | 从未被引用 | 已被生产/匹配的 |
|---|---|---|
| `neko_session::StreamState` | `Reset` | Open, HalfClosedLocal, HalfClosedRemote, Closed |
| `neko_session::RuntimeError` | `Deadline` | 其余 12 个 |
| `neko_session::LedgerError` | `ContextMismatch` | 其余 12 个 |
| `neko_reliable::PmtuSendOutcome` | `Sent` | RetryAt, BaseIncompatible |
| `neko_carrier::PathError` | `ValidationDomain`, `ActivePathRequired` | 其余 9 个 |
| `neko_carrier::MemoryPairError` | `MessageTooLarge`, `QueueFull`, `Closed`, `PeerClosed`, `ArithmeticOverflow` | InvalidLimits, StatePoisoned |

三处最值得注意的：

1. **`neko_reliable::PmtuSendOutcome::Sent`**——PLPMTUD 的发送结果类型。唯一的生产者是
   `Plpmtud::on_emsgsize`，它只返回 `BaseIncompatible` 或 `RetryAt`。也就是说这个三元结果类型里
   **有一个结果永远不会出现**，对其做穷尽匹配的调用方会得到一条不可达分支。该类型还没有 `impl`。

2. **`neko_carrier::MemoryPairError`**——7 个变体里只有 2 个会被 `MemoryEndpoint` 生产
   （`new` 的 `InvalidLimits`、`lock` 的 `StatePoisoned`）。其余 5 个的**变体名与 `UdpError` 完全同构**
   （`MessageTooLarge`/`QueueFull`/`Closed`/`PeerClosed`/`ArithmeticOverflow`），看起来是从 `UdpError`
   复制过来而未裁剪。内存对的 `send` 走的是 `CarrierError`，不是这个类型。

3. **`neko_session::StreamState::Reset`**——流状态枚举声明了 `Reset` 态，但整个 crate 里这个词只出现在
   定义行本身：`Open`/`Closed`/`HalfClosedLocal`/`HalfClosedRemote` 都在用，**reset 从未被建模**。
   一个消费方（或未来的实现者）读这个枚举会以为 reset 是被建模的状态之一。

## 为什么**不**写成测试

对没有消费方的接口断言其字面值，钉住的是死表面而不是行为。例如给
`PmtuSendOutcome::Sent` 写一个 `assert_eq!` 只能验证这个变体仍然能被命名，不能验证任何真实路径——
而这正是标准明确禁止的做法（与先前记录的三个零引用常量同类）。

## 为什么**不**在本轮删除

这 11 个变体全部是**公开 API**（`pub enum`，位于库 crate）。删除它们会改变
`neko-session` / `neko-carrier` / `neko-reliable` 的公开表面，属于架构/API 变更，超出本守卫的
自主范围（守卫禁区内含核心 Session/Carrier/wire 架构变更）。因此记录为**维护者决策项**而非自行执行。

一个必须如实说明的边界：本仓库外部的消费者可以匹配这些 `pub` 变体，所以「工作区内零引用」不等于
「世界上零引用」。本发现只主张**本工作区内无生产者、无匹配者**；这正是它们可能成为不可达分支的原因。

## 一个解释：为什么它们能积累下来

Rust 的 `dead_code` lint **不会**对库 crate 的 `pub` 项报警——`pub` 项被视为公开 API 的一部分。
所以一个从未被生产的公开变体不会引发任何编译期警告，只能靠这种仓库级引用扫描发现。

## Evidence boundary

仓库级静态引用扫描，开发者本地执行；不是审阅者本地、不是托管 CI、不是网络或性能证据。
未改动任何生产源码（本文件是唯一的新增内容），因此不需要 exact-tree 门禁。
无 H-I4-119 / D019 / release flag / policy 值变更。`READY_LIVE: none`。

# 猫娘 Design Decision Log

这个文件记录讨论中已经形成的设计方向，避免后续 Agent 把“曾经讨论过”误认为“已经定稿”，也避免真正做出的决定只存在于聊天记录里。

状态定义：

- **Accepted**：当前按此推进，除非新实验推翻。
- **Provisional**：暂时采用，仍需要实现/测试确认。
- **Research**：只进入研究列表，不承诺实现。
- **Superseded**：被后续决定替代，但保留历史。

---

## 2026-08-26 — D001：Rust 起手式

**Status: Accepted**

第一实现语言选择 Rust。

理由不是“Rust 一定最快”，而是本项目本身是传输栈研究：wire codec、状态机、buffer ownership、fuzz、边界安全、低层 socket 行为等都值得直接暴露出来研究。

当前没有要求本地 Windows 必须配置 Rust。主要开发/实验方向转为 Linux server；VPS 提供真实 WAN peer；QEMU 与 Linux 网络工具提供可控实验环境。

尚未决定：async runtime、TLS/Noise 库、buffer model、workspace crate 拆分。

---

## 2026-08-26 — D002：猫娘从 UDP protocol 改为 carrier-agnostic Session transport

**Status: Accepted**

早期基线把项目定义为“UDP + AEAD + 自定义可靠传输”。这个方向仍然是第一实现切片，但不再定义猫娘本身。

新的核心抽象：

```text
Application / Tunnel
        -> Nekomusume Session
        -> Carrier Manager
        -> UDP / TCP / experimental carriers
```

TCP、UDP 等是底层 Carrier。逻辑 Session 不绑定某个具体 transport connection。

`docs/design-handoff.md` 保留为历史交接基线；最新架构方向见 `docs/carrier-architecture.md`。

---

## 2026-08-26 — D003：核心问题选择“异构 Carrier 生存性”（B）

**Status: Accepted**

项目不是主要为了同时提供 reliable stream + unreliable datagram（问题 A）。

当前核心问题是：

> 当网络环境让 UDP、TCP 或其他通信原语中的一部分不可用/严重退化时，能否让同一个逻辑隧道在异构 Carrier 之间 failover / migrate（问题 B）？

这也是 TCP + UDP 共存的主要理由。

---

## 2026-08-26 — D004：第一阶段只做 failover，不做 TCP/UDP 同时条带化

**Status: Accepted**

早期目标：

```text
UDP = primary
TCP = fallback
```

先证明 Session 可以跨 Carrier 恢复，再研究 concurrent heterogeneous multipath。

禁止在早期简单按 packet 轮流分发到 TCP/UDP；不同排队、重传和有序语义可能制造跨协议 Head-of-Line blocking。

---

## 2026-08-26 — D005：Path feedback 与 Session delivery state 分层

**Status: Provisional**

UDP Carrier 可以拥有 packet-level ACK / RTT / loss / cwnd；TCP Carrier 使用 TCP 自身可靠性，不重复实现 TCP packet ACK。

但 Session 仍需维护跨 Carrier 的逻辑交付状态，例如 `stream_id + offset ranges`，用于 failover 后判断已确认/不确定数据并去重。

具体 ACK 编码尚未冻结，进入 M0 研究。

---

## 2026-08-26 — D006：实验性 Carrier 列表

**Status: Research**

当前候选：

- Tier 0：UDP、TCP（必须实现）
- Tier 1：ICMP Echo、Raw IP experimental protocol（必须做 reachability 实验，不承诺成为常用路径）
- Tier 2：SCTP、DCCP（重点研究/可能实验实现）
- Tier 3：UDP-Lite、GRE、IP-in-IP、ESP、HTTP/WebSocket/MASQUE 等（协议考古与对照）

Raw IP 实验号必须显式配置，默认不开启；不得把实验值当成互联网正式协议号。

---

## 2026-08-26 — D007：Reachability Matrix 作为正式实验产物

**Status: Accepted**

不凭经验猜测“公网一般会不会放行”。对 Linux server、VPS、QEMU/虚拟拓扑以及后续其他端点运行可重复 reachability probe，记录协议、IPv4/IPv6、端口、payload、RTT、loss、双向持续可用性、权限和网络环境元数据。

项目的人类可读探测结果固定为：

```text
如果通：喵~！
如果不通：喵呜呜呜呜…
```

同时必须存在机器可读状态与正确 exit code。

---

## 2026-08-26 — D008：真实 WAN 与可控网络实验并行

**Status: Accepted**

- Linux server + VPS：真实互联网验证。
- QEMU / netns / veth / tc netem：可重复故障注入与网络模型。

真实 WAN 用来判断“现实是否买账”；可控环境用来解释“为什么”。二者不能互相替代。

---

## 未决事项

- M0 crate/workspace 结构。
- v0 magic/version 与 canonical integer encoding。
- Session delivery ACK 具体格式。
- UDP Carrier 的第一拥塞控制基线。
- UDP/TCP migration handshake 与 anti-replay 细节。
- Carrier scoring 与 hysteresis。
- ICMP / Raw-IP probe 的 payload 与安全边界。
- 第一批真实测试节点和矩阵规模。

---

## 2026-08-26 — Research integration index

**Status: Accepted as documentation governance; implementation decisions remain gated**

The standards, security and transport research are indexed by [`docs/m0-spec-plan.md`](m0-spec-plan.md). The versioned document [`docs/specs/nekomusume-session-v0.md`](specs/nekomusume-session-v0.md) is the sole normative-source entry point; it remains provisional until its gates are reviewed. Research files are non-normative; this log records status and links rather than duplicating wire rules. `docs/design-handoff.md` is historical/non-normative.

## 2026-08-26 — D009：M0 先落地边界，不落地传输

**Status: Accepted**

M0 仅建立 virtual Cargo workspace 与 `neko-wire`、`neko-session`、`neko-carrier`、`neko-cli` 四个 crate 边界。当前不实现 UDP/TCP、密码学、wire codec、Session 状态机或 failover；这些能力必须等待对应规范与安全审查门禁。crate 代码只提供可编译的占位标记，避免 README、路线图或规范产生已实现的虚假声明。

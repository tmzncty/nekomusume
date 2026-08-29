# 猫娘 Roadmap

本路线图记录当前研究推进顺序；实现状态以 [`docs/status.md`](docs/status.md) 为唯一来源。它可以被实验结果修改，但修改必须同步到 `docs/decisions.md`，不能让关键设计只存在于聊天记录。

## Milestone 0 — Rust 骨架、抽象与测试向量

目标：先让“Session 是什么、Carrier 是什么、字节长什么样、状态怎么走”可验证。

- [x] 实现语言：Rust
- [x] 建立 Cargo workspace / crate 边界
- [x] 定义 `Session` / `Carrier` / `Path` 的最小抽象
- [x] 明确协议版本与 magic
- [x] 固定第一版 session record / UDP packet header
- [x] 定义 frame type 与字段编码
- [x] 定义 Session delivery state / acknowledgement 语义
- [x] 定义错误码与兼容策略
- [x] 写状态机文档
- [x] 至少 20 个 golden encode/decode tests
- [x] fuzz：畸形输入不得 panic、越界或无限分配
- [x] CLI skeleton：`neko client` / `neko server` / `neko probe`

M0 不要求 TCP/UDP 已经真正搬运应用数据，但抽象不能把 Session 写死成 UDP socket。

## Milestone 1 — UDP Carrier 单流加密回显

- [x] Linux UDP socket
- [x] 使用成熟握手与加密实现
- [x] 加密 session record
- [x] 单个双向 stream
- [x] CLOSE
- [x] 本机 loopback 稳定回显；QEMU / 局域网待独立授权实验
- [x] VPS 真实 WAN 基础连通（IPv4 TCP/UDP authenticated echo；IPv6 基础路由与端口已验证）

## Milestone 2 — UDP reliable packet engine

- [x] packet number
- [x] ACK ranges
- [x] RTT 估计
- [x] packet/time threshold loss detection
- [x] PTO
- [x] frame-level retransmission
- [x] Reno 风格第一拥塞控制基线
- [x] pacing
- [x] 1% / 5% / 10% 确定性首发丢包下数据完整性测试

这一层属于 UDP Carrier；不要把 UDP packet ACK 误当成跨 Carrier 的 Session delivery acknowledgement。

## Milestone 3 — TCP Carrier 与真正的异构 failover

这是猫娘区别于“又一个 UDP 协议”的第一个关键里程碑。

- [x] TCP Carrier framing
- [x] TCP/UDP capability model
- [x] Session delivery state 可跨 Carrier 表达
- [x] UDP primary + TCP fallback
- [x] UDP 失效时切换 TCP
- [x] 对 uncertain data 做安全重发与接收端去重
- [x] TCP 上不重复实现 packet-level TCP ACK
- [x] path validation / anti-replay 基础
- [x] failover latency / duplicate bytes / recovery success 指标

第一阶段禁止为了聚合带宽而把相邻数据简单轮流扔给 TCP/UDP。

## Milestone 4 — 多 stream、流控与 Carrier Manager

- [x] 多 stream
- [x] connection/session-level flow control
- [x] per-stream flow control
- [x] 公平调度
- [x] 大流不得显著阻塞交互小流
- [x] 每连接与全局资源上限
- [x] Carrier health probe
- [x] Carrier scoring
- [x] failover hysteresis，避免路径来回抖动
- [x] UDP 恢复后的迁回策略（validated generation + health margin + hold gate）

## Milestone 5 — Benchmark 与真实 WAN validation

> Release-readiness gate remains blocked; see [`docs/spec/m5-release-readiness-gate.md`](docs/spec/m5-release-readiness-gate.md).

### 可控环境

使用 QEMU / netns / veth / `tc netem`：

- [x] baseline（确定性 fixture + netns 实证）
- [x] variable RTT（建模 fixture + netns 20ms 实证）
- [x] random loss（1/5/10% 确定性 fixture + netns 实证）
- [x] burst loss（netns gemodel 实证）
- [x] reorder（确定性 reversal + netns 实证）
- [x] bandwidth changes（建模 fixture + netns 10mbit 实证）
- [x] carrier hard-failure / blackhole（隔离 failover + netns 100% loss 实证）

### 真实环境

Linux server <-> VPS：

- [x] UDP 正常路径（single authenticated echo）
- [ ] UDP 退化 / TCP fallback
- [ ] 长连接稳定性
- [ ] NAT / endpoint change（条件允许时）
- [ ] 与 HY2 在同服务器、同线路、同 MTU、同安全等级、同应用流量下比较
- [x] 报告 median / P95 / failures（bounded authenticated baseline；非性能结论）

## Experimental Track A — Reachability Matrix

建立独立 `neko probe` 工具，研究真实网络还能放行哪些底层通信原语。

第一批候选：

- [x] TCP IPv4 / IPv6（bounded probe）
- [x] UDP IPv4 / IPv6（bounded probe）
- [ ] ICMP Echo / ICMPv6 Echo
- [ ] SCTP
- [ ] DCCP
- [ ] GRE
- [ ] ESP
- [ ] Raw IP experimental protocol 253/254（显式启用）

记录：

- reachability；
- RTT / first response；
- loss；
- usable payload；
- sustained bidirectional viability；
- privileges；
- kernel / VPS / network metadata。

人类可读输出：

```text
pass: 喵~！
fail: 喵呜呜呜呜…
```

同时输出结构化结果与正确 exit code。

## Experimental Track B — 其他 Carrier / 协议参考

### 重点阅读/候选实验

- [ ] SCTP：multi-stream / multi-homing / per-path congestion state
- [ ] DCCP：congestion-controlled unreliable datagrams

### 协议考古 / 对照

- [ ] UDP-Lite
- [ ] GRE
- [ ] IP-in-IP
- [ ] ESP/IPsec
- [ ] HTTP CONNECT / WebSocket
- [ ] MASQUE / CONNECT-IP

进入列表不等于承诺实现。

## Experimental Track C — 后期增强

仅根据前面实验决定：

- [x] bounded authenticated PLPMTUD state
- [x] bounded XOR FEC candidate（未启用，证据不足以选择）
- [x] bounded authenticated unreliable datagram API
- [x] bounded synchronized key update
- [x] 0-RTT gate closed（明确禁用；未实现）
- [x] concurrent UDP + TCP gate closed（明确禁用；无受控收益证据）
- [x] heterogeneous multipath aggregation gate closed（明确禁用；DSN/重排/拥塞耦合未闭合）

## 当前第一轮研究问题

1. Cargo workspace 应如何拆：`wire` / `session` / `carrier-*` / `crypto` / `probe` 是否足够？
2. Session delivery acknowledgement 使用 offset range、message receipt，还是两者并存？
3. v0 packet/session record header 是否先固定长度以方便抓包和 fuzz？
4. 第一版握手直接复用 TLS 1.3 还是 Noise library？
5. UDP -> TCP failover 如何定义“确认收到 / uncertain / 可安全重发”？
6. Carrier score 如何避免因为瞬时 loss/RTT 变化不断切换？
7. ICMP 与 Raw-IP 在真实 VPS/运营商网络的持续双向可用程度到底如何？

## M0 research record

The executable pre-implementation plan is [`docs/m0-spec-plan.md`](docs/m0-spec-plan.md). The research files under [`docs/research/`](docs/research/) do not mark implementation work complete; all existing unchecked M0 items remain unchecked until code and tests exist.

# 猫娘 Roadmap

本路线图记录当前研究推进顺序；实现状态以 [`docs/status.md`](docs/status.md) 为唯一来源。它可以被实验结果修改，但修改必须同步到 `docs/decisions.md`，不能让关键设计只存在于聊天记录。

## Milestone 0 — Rust 骨架、抽象与测试向量

目标：先让“Session 是什么、Carrier 是什么、字节长什么样、状态怎么走”可验证。

- [x] 实现语言：Rust
- [ ] 建立 Cargo workspace / crate 边界
- [ ] 定义 `Session` / `Carrier` / `Path` 的最小抽象
- [ ] 明确协议版本与 magic
- [ ] 固定第一版 session record / UDP packet header
- [ ] 定义 frame type 与字段编码
- [ ] 定义 Session delivery state / acknowledgement 语义
- [ ] 定义错误码与兼容策略
- [ ] 写状态机文档
- [ ] 至少 20 个 golden encode/decode tests
- [ ] fuzz：畸形输入不得 panic、越界或无限分配
- [ ] CLI skeleton：`neko client` / `neko server` / `neko probe`

M0 不要求 TCP/UDP 已经真正搬运应用数据，但抽象不能把 Session 写死成 UDP socket。

## Milestone 1 — UDP Carrier 单流加密回显

- [ ] Linux UDP socket
- [ ] 使用成熟握手与加密实现
- [ ] 加密 session record
- [ ] 单个双向 stream
- [ ] CLOSE
- [ ] 本机 / QEMU / 局域网稳定回显
- [ ] VPS 真实 WAN 基础连通

## Milestone 2 — UDP reliable packet engine

- [ ] packet number
- [ ] ACK ranges
- [ ] RTT 估计
- [ ] packet/time threshold loss detection
- [ ] PTO
- [ ] frame-level retransmission
- [ ] Reno/CUBIC 风格第一拥塞控制基线
- [ ] pacing
- [ ] 1% / 5% / 10% 丢包下数据完整性测试

这一层属于 UDP Carrier；不要把 UDP packet ACK 误当成跨 Carrier 的 Session delivery acknowledgement。

## Milestone 3 — TCP Carrier 与真正的异构 failover

这是猫娘区别于“又一个 UDP 协议”的第一个关键里程碑。

- [ ] TCP Carrier framing
- [ ] TCP/UDP capability model
- [ ] Session delivery state 可跨 Carrier 表达
- [ ] UDP primary + TCP fallback
- [ ] UDP 失效时切换 TCP
- [ ] 对 uncertain data 做安全重发与接收端去重
- [ ] TCP 上不重复实现 packet-level TCP ACK
- [ ] path validation / anti-replay 基础
- [ ] failover latency / duplicate bytes / recovery success 指标

第一阶段禁止为了聚合带宽而把相邻数据简单轮流扔给 TCP/UDP。

## Milestone 4 — 多 stream、流控与 Carrier Manager

- [ ] 多 stream
- [ ] connection/session-level flow control
- [ ] per-stream flow control
- [ ] 公平调度
- [ ] 大流不得显著阻塞交互小流
- [ ] 每连接与全局资源上限
- [ ] Carrier health probe
- [ ] Carrier scoring
- [ ] failover hysteresis，避免路径来回抖动
- [ ] UDP 恢复后的迁回策略

## Milestone 5 — Benchmark 与真实 WAN validation

### 可控环境

使用 QEMU / netns / veth / `tc netem`：

- [ ] baseline
- [ ] variable RTT
- [ ] random loss
- [ ] burst loss
- [ ] reorder
- [ ] bandwidth changes
- [ ] carrier hard-failure / blackhole

### 真实环境

Linux server <-> VPS：

- [ ] UDP 正常路径
- [ ] UDP 退化 / TCP fallback
- [ ] 长连接稳定性
- [ ] NAT / endpoint change（条件允许时）
- [ ] 与 HY2 在同服务器、同线路、同 MTU、同安全等级、同应用流量下比较
- [ ] 报告 median / P95 / failures，而不是只贴最好成绩

## Experimental Track A — Reachability Matrix

建立独立 `neko probe` 工具，研究真实网络还能放行哪些底层通信原语。

第一批候选：

- [ ] TCP IPv4 / IPv6
- [ ] UDP IPv4 / IPv6
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

- [ ] PMTUD
- [ ] FEC
- [ ] unreliable datagram API
- [ ] key update
- [ ] 0-RTT
- [ ] concurrent UDP + TCP
- [ ] heterogeneous multipath aggregation

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

# 猫娘（nekomusume）

一个公开研究用的实验性、**carrier-agnostic** 加密 Session transport / tunnel 项目。

> 为什么叫猫娘？别问，问就是猫娘。

## 这是什么

猫娘不再被定义成“另一个 UDP 协议”。UDP 是第一个 Carrier，但不是协议本体。

核心目标是研究：**当真实网络只剩下部分通信原语可用时，一个逻辑加密 Session 能否在 UDP、TCP 以及实验性 Carrier 之间 failover / migrate，并尽量保持隧道存活。**

```text
Application / Tunnel
        |
        v
Nekomusume Session
        |
        v
Carrier Manager
   /       |       \
 UDP      TCP    experimental
```

Hysteria2 仍然是重要 benchmark competitor，但“超过 HY2”不是项目定义。先证明行为，再谈性能。

## 当前状态

**Research bootstrap / pre-Milestone 0**。权威状态表见 [`docs/status.md`](docs/status.md)；其中 `implemented` 仅表示仓库证据存在，不表示安全、协议冻结、互操作或生产就绪。当前发布门禁见 [`docs/spec/m5-release-readiness-gate.md`](docs/spec/m5-release-readiness-gate.md)，仍为 blocked。

已形成的主要方向：

- 实现语言：**Rust**；
- 逻辑 Session 与底层 Carrier 解耦；
- UDP：第一主力 Carrier；
- TCP：首要 fallback Carrier；
- 第一阶段只做 failover / migration，不做 UDP+TCP 同时条带化；
- ICMP Echo / Raw IP experimental protocol：仅为研究队列，当前被治理状态阻断，不进行公网暴露；
- SCTP / DCCP：重点参考与实验候选；
- Linux server + VPS：真实 WAN；
- QEMU / netns / veth / `tc netem`：可控网络实验。

最新架构方向：[`docs/carrier-architecture.md`](docs/carrier-architecture.md)

设计决定日志：[`docs/decisions.md`](docs/decisions.md)

早期交接基线：[`docs/design-handoff.md`](docs/design-handoff.md)

> `design-handoff.md` 中的“UDP + AEAD + 自定义可靠传输”仍是重要第一实现输入，但已经被更高层的 carrier-agnostic 架构重新定位为“第一个 Carrier 实现”，而不是猫娘的最终定义。

## 原则

1. **先测量，再宣称。** 没有 benchmark 结果，不写“比 HY2 更好”。
2. **不自研密码算法。** 使用成熟 TLS / Noise / AEAD 实现。
3. **Session 高于 Carrier。** TCP/UDP/ICMP 等负责搬运，逻辑会话状态属于猫娘。
4. **先 failover，再 aggregation。** 先证明 UDP <-> TCP 迁移，再研究异构多路径并发。
5. **协议核心与代理业务解耦。** 隧道是主要用途，但 transport/session 本身应可独立测试。
6. **所有关键行为必须可复现。** wire format、状态机、错误码、测试向量、netem 条件和 benchmark 都留下记录。
7. **公开研究不公开生产秘密。** 不提交生产密钥、真实生产拓扑或不必要的地址信息。

## 初步路线

- M0：Rust workspace、Session/Carrier 抽象、wire format、错误码、状态机、golden vectors
- M1：UDP Carrier + 成熟加密库 + 单流加密回显
- M2：UDP reliable packet engine：ACK、RTT、loss、retransmit、CC baseline
- M3：TCP Carrier + Session delivery state + UDP -> TCP failover / resume
- M4：多 stream、流控、调度、Carrier scoring、受控网络实验
- M5：与 HY2 公平 benchmark + 真实 WAN validation
- Experimental：ICMP / Raw-IP reachability、SCTP/DCCP、PMTUD/FEC/datagram、异构 multipath

详细定义见 [`ROADMAP.md`](ROADMAP.md)。

## Reachability

猫娘会单独测试“今天的真实网络到底允许哪些 communication primitives 活着”。

人类可读输出已经冻结：

```text
如果通：
喵~！

如果不通：
喵呜呜呜呜…
```

当然，机器可读结果仍然要有 JSON / exit code，不能让 CI 猜猫叫。

## 当前不做的事情

- 不把实验版本直接替换生产隧道；
- 不为了跑分关闭认证、完整性保护或资源限制；
- 不把理论优势写成已经验证的性能结论；
- 不在 M0/M1 就同时实现 FEC、多路径、复杂伪装和所有候选 Carrier；
- 不把实验 IP protocol number 当成正式公网协议号。

## 许可证

本项目采用双许可证，可任选 MIT 或 Apache-2.0。完整文本见 [LICENSE-MIT](LICENSE-MIT) 与 [LICENSE-APACHE](LICENSE-APACHE)。

## Research and M0 planning

The standards, threat model and recovery studies are indexed in [`docs/m0-spec-plan.md`](docs/m0-spec-plan.md), with the provisional normative-source entry point at [`docs/specs/nekomusume-session-v0.md`](docs/specs/nekomusume-session-v0.md), and [`docs/research/`](docs/research/). They are research/planning documents, not implemented features. [`docs/design-handoff.md`](docs/design-handoff.md) is historical and non-normative.

## Local verification

本地验证是当前主路径：`scripts/check.sh` 运行 Rust 格式、检查、测试和
Clippy；`scripts/fuzz-smoke.sh` 运行 nightly `cargo-fuzz` decode smoke，可用
`FUZZ_TIME` 与 `FUZZ_MAX_LEN` 覆盖默认的 30 秒与 8192 bytes。GitHub Actions
仅作为复核门禁，不替代本地验证。脚本只执行本地构建与测试，不修改网络服务。

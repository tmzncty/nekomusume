# IMPLEMENTATION_PLAN.md — Nekomusume

> Status is governed by [`docs/status.md`](docs/status.md). A checked planning item is not evidence of implementation or approval.

本文件把 `ROADMAP.md` 转成 agent 可连续领取的施工顺序。若代码已经完成某项，以代码和测试为准，不重复实现。

## Phase 0 — Baseline audit

- [x] 运行 `./scripts/check.sh`，记录当前基线；
- [x] 检查 workspace/crate 实际边界与 `docs/m0-spec-plan.md` 是否一致；
- [x] 列出当前已实现的 wire/session/CLI 能力；
- [x] 将“研究完成但代码未完成”的事项保持未完成状态。

验收：有一份简短、基于当前仓库事实的 baseline note；不得把旧文档计划当实现。

## Phase 1 — M0 normative wire/session core

### 1.1 crate boundary

目标：最小拆分支持 `wire`、`session`、carrier 抽象和 CLI，不为未来功能提前建大量空 crate。

- [x] 固定 workspace 成员；
- [x] 依赖方向无循环；
- [x] core 不依赖具体 UDP/TCP socket。

### 1.2 protocol identity

- [x] version/magic；
- [x] session record/header；
- [x] frame type 与字段编码；
- [x] error code；
- [x] unknown version/type 行为；
- [x] compatibility policy。

所有行为同步到 `docs/specs/nekomusume-session-v0.md`。

### 1.3 golden vectors

至少覆盖：

- 最小合法 record；
- 多 frame；
- 边界长度；
- unknown type；
- truncation；
- 超限长度；
- 非法 enum；
- deterministic encode；
- decode→encode round-trip；
- version rejection/negotiation baseline。

目标：≥20 个稳定向量。

### 1.4 parser hardening

- [x] 所有外部长度有上限；
- [x] 畸形输入不 panic；
- [x] 不无限分配；
- [x] fuzz target 覆盖主要 decode path；
- [x] corpus 放入最小 regression seeds。

验收：`check.sh` + `fuzz-smoke.sh`。

## Phase 2 — Session state semantics

- [x] 定义 Session lifecycle；
- [x] 定义 delivery acknowledgement；
- [x] 明确 confirmed / uncertain / delivered / closed 等状态；
- [x] 设计跨 Carrier 可序列化状态；
- [x] ACK 语义与 UDP packet ACK 分离；
- [x] 单元测试覆盖重复、乱序、重放、关闭边界。

验收：不创建 socket 也能用纯状态机测试一次 Session delivery 流程。

## Phase 3 — CLI skeleton

- [ ] `neko client`；
- [ ] `neko server`；
- [ ] `neko probe`；
- [x] `--help`/错误 exit code；
- [ ] 结构化日志基础；
- [ ] 不在 CLI 中堆协议状态逻辑。

验收：CLI 可运行、参数错误稳定、核心 crate 可独立测试。

## Phase 4 — M1 UDP Carrier + encrypted echo

- [x] Linux UDP Carrier（loopback-only）；
- [x] 成熟握手/AEAD 库接入；
- [x] 身份/密钥配置最小模型；
- [x] encrypted session record；
- [x] 单个双向 stream；
- [x] CLOSE；
- [x] loopback echo；netns 待后续实验；
- [x] corruption/authentication failure tests；
- [x] replay 基础边界。

真实 WAN 只有在本地测试稳定后再做。

验收：同一 payload 在 loopback/netns 下完整往返，抓包中应用明文不可见。

## Phase 5 — M2 reliable UDP engine

按依赖顺序：

1. packet number；
2. ACK range encoding/decoding；
3. RTT estimator；
4. packet/time threshold loss detection；
5. PTO；
6. frame-level retransmission；
7. congestion-control baseline；
8. pacing；
9. resource limits。

测试矩阵：0/1/5/10% random loss、burst loss、reorder、RTT change、blackhole。

验收：数据完整性稳定；输出 retransmit/loss/RTT 指标。

## Phase 6 — M3 TCP Carrier and heterogeneous failover

- [ ] TCP framing；
- [ ] capability model；
- [ ] UDP primary / TCP fallback；
- [ ] hard-failure detection；
- [ ] uncertain data safe resend；
- [ ] receiver deduplication；
- [ ] anti-replay/path validation baseline；
- [ ] recovery metrics；
- [ ] UDP recovery/migration-back policy先写设计再实现。

核心验收场景：传输中途 UDP blackhole，Session 通过 TCP 恢复，最终字节流无丢失且重复可测/受控。

## Phase 7 — M4 multi-stream and Carrier Manager

- [ ] stream ids/lifecycle；
- [ ] per-stream + session flow control；
- [ ] scheduler fairness；
- [ ] global/per-connection limits；
- [ ] health probes；
- [ ] scoring；
- [ ] hysteresis；
- [ ] migration-back。

验收：大 bulk stream 不显著饿死交互小 stream；路径抖动不导致频繁切换。

## Phase 8 — `neko probe` reachability track

先支持安全、无需特权或最常见路径：

- [ ] TCP IPv4/IPv6；
- [ ] UDP IPv4/IPv6；
- [ ] ICMP Echo（显式权限检查）；
- [ ] SCTP/DCCP/GRE/ESP/Raw-IP 仅在明确启用时测试。

机器输出：JSON + exit code；人类输出保留：

```text
pass: 喵~！
fail: 喵呜呜呜呜…
```

记录环境、权限、RTT、loss、usable payload、持续双向能力。

## Phase 9 — Benchmark harness

- [ ] netns/veth/netem 自动化；
- [ ] baseline/RTT/loss/reorder/bandwidth/blackhole 场景；
- [ ] 结果机器可读；
- [ ] median/P95/failure；
- [ ] HY2 对照脚本与条件说明；
- [ ] 真实 WAN 结果与可控实验分开保存。

不要为 benchmark 关闭安全机制。

## Phase 10 — Experimental enhancements

只有前面结果给出理由后领取：

- PMTUD；
- FEC；
- unreliable datagram API；
- key update；
- 0-RTT；
- concurrent carriers；
- heterogeneous aggregation。

每项先写 decision/research，回答“解决了哪个已观察到的问题”。

## 每个切片的完成门禁

- 代码/研究产物真实存在；
- 测试覆盖关键行为；
- `./scripts/check.sh` 通过；
- parser/wire 改动运行 fuzz smoke；
- 网络实验不污染生产环境；
- 规范同步；
- checkpoint commit；
- 下一任务依赖明确。

# AGENTS.md — Nekomusume execution contract

本文件定义自动化 coding/research agent 在本仓库中的默认施工规则。目标不是让 agent 重新讨论项目方向，而是让它在现有设计边界内持续、可验证地推进。

## 0. 开工前必须阅读

按顺序阅读：

1. `README.md`
2. `ROADMAP.md`
3. `SECURITY.md`
4. `docs/standing-vps-lab-authorization.md`
5. `IMPLEMENTATION_PLAN.md`
6. `docs/decisions.md`
7. `docs/carrier-architecture.md`
8. `docs/m0-spec-plan.md`
9. `docs/specs/nekomusume-session-v0.md`
10. 与当前任务直接相关的 `docs/research/`、crate 和测试

`docs/design-handoff.md` 是历史交接材料，不是当前最高优先级规范。

`docs/standing-vps-lab-authorization.md` 是维护者已经给出的持续有效 VPS/WAN 实验授权。只要实验处于该文件的范围和边界内，就不得再把“需要逐次 WAN 授权”“需要重新批准 40080/40081 listener”或“缺 count/bytes/duration/port”当作外部 blocker。

## 1. 不可破坏的架构边界

- **Session 高于 Carrier。** 不得把逻辑 Session 状态重新写死为 UDP socket/TCP stream 状态。
- UDP 是第一个主力 Carrier，不是协议本体。
- TCP 是第一 fallback；M3 之前不要提前做 UDP+TCP 条带化聚合。
- Session delivery acknowledgement 与 UDP packet ACK 必须分层，不得混为一个可靠性机制。
- 不自研密码算法。握手、AEAD、KDF 等必须使用成熟、维护中的密码库，并记录选择依据。
- 协议 wire format、状态机、错误码、版本兼容、资源上限必须有机器可验证表示。
- 理论优势不是 benchmark 结论；没有同条件数据，不得写“比 HY2 更快/更稳”。

若实现需要修改这些边界，先更新 `docs/decisions.md`，写清：问题、选项、证据、决定、回滚条件；不得让架构变化只存在于 commit message 或聊天记录。

## 2. 规范优先级

发生冲突时，优先级默认是：

1. 已提交的明确安全约束与 `SECURITY.md`
2. `docs/standing-vps-lab-authorization.md` 中的实验授权边界
3. `docs/decisions.md` 中仍有效的决定
4. `docs/specs/` 下当前规范
5. `docs/carrier-architecture.md`
6. `ROADMAP.md` / `IMPLEMENTATION_PLAN.md`
7. 研究笔记
8. 历史 handoff

Standing authorization 只解决“管理员已经允许哪些自有 VPS 实验”的问题，不覆盖 `SECURITY.md`，也不能授权第三方目标、生产网络修改或该文件明确要求重新批准的高权限/长时/高流量实验。

如果无法判断某条决定是否仍有效，先通过仓库历史和现有实现核对，不要自行拼接两个互相冲突的设计。

## 3. 默认工作方式

每轮工作：

1. 检查当前 git 状态、最近提交、现有测试和未完成任务；
2. 从 `IMPLEMENTATION_PLAN.md` 中选择**依赖已满足的最早未完成项**；
3. 先确认已有代码/研究是否已经部分完成，禁止重复造第二套；
4. 实现最小完整切片；
5. 补测试、golden vector、文档或 benchmark fixture；
6. 运行本地验证；
7. 修复本轮引入的问题；
8. 提交语义明确的 checkpoint；
9. 在没有真正 blocker 时继续下一切片。

不要把“写了计划”“加了 TODO”“建了空 crate”当完成。

当一个 WAN 节点失败或 blocked 时，必须继续检查所有不依赖该节点的 READY 工作。不得把单个 WAN failure、缺少 previous release、缺少 native ARM host 或外部 review 自动升级为整个项目停止条件。

## 4. 本地验证门禁

正常代码改动完成后至少运行：

```bash
./scripts/check.sh
```

涉及 wire decode/parser、外部输入或长度字段时，还应运行：

```bash
./scripts/fuzz-smoke.sh
```

若当前环境缺少 nightly/cargo-fuzz，可明确记录该项未运行以及原因，但不得假报通过。

涉及网络行为时，优先使用：

1. unit/integration tests；
2. loopback；
3. netns/veth；
4. QEMU/局域网；
5. 真实 WAN。

第 5 层是否可直接执行，以 `docs/standing-vps-lab-authorization.md` 为准。处于 standing authorization 范围内的自有客户端 ↔ 自有 VPS 普通 bounded TCP/UDP 实验已经获得明确授权，无需再次逐次询问。

## 5. 网络实验安全

- 默认不修改宿主机生产路由、防火墙、代理、隧道或系统服务。
- netns/veth 实验必须有清理路径；失败后也应尽量恢复。
- 不提交真实生产密钥、cookie、token、私网拓扑、无必要公网地址。
- Raw IP、ICMP carrier、SCTP、DCCP、GRE、ESP 等高权限/非常规 reachability 测试必须符合 `docs/standing-vps-lab-authorization.md`；其中要求单独授权的项目不得作为普通 test suite 的隐式副作用。
- 不扫描第三方地址段，不做规避访问控制的实验。
- 公网 benchmark 只能针对自己控制或明确授权的端点。
- 自有客户端 ↔ 自有 VPS 的普通 bounded TCP/UDP listener、Session、diagnostic、benchmark、HY2 comparison、bounded capture、package rehearsal 和 cleanup 已由 `docs/standing-vps-lab-authorization.md` 持续授权。
- 同一失败实验不得在 instrumentation/code/configuration/hypothesis/capture coverage 等均无变化时机械重跑；有新的诊断变量且仍在 standing authorization 边界内时，可以直接继续，不需要重新请求许可。
- 实验结束必须清理本轮产生的 listener/process/temp runtime，并尽量验证无残留。

## 6. Wire / parser 规则

任何来自网络的字段都视为不可信：

- 长度和计数必须有上限；
- 不得因畸形输入 panic；
- 不得无限分配；
- 不得未检查整数溢出/截断；
- unknown version/frame/type 的行为必须明确；
- encode/decode 必须有稳定 golden vectors；
- 兼容性变化必须修改规范与测试向量。

序列化结果应尽量 deterministic，便于抓包、回归和跨版本比对。

## 7. Reliable UDP 规则

进入 M2 后：

- packet number、ACK range、RTT、loss detection、PTO、retransmission、CC/pacing 分开建模；
- frame retransmission 不等于 packet retransmission；
- 不以简单 sleep 循环冒充 pacing；
- 拥塞控制至少提供可重复 netem 测试；
- 数据完整性优先于吞吐跑分。

## 8. Failover 规则

M3 的核心验收不是“TCP 也能传数据”，而是：

- UDP primary 正常工作；
- UDP hard-failure/blackhole 可检测；
- Session 状态不丢失地进入 TCP fallback；
- uncertain data 有清楚的安全重发/去重语义；
- 恢复过程有指标：恢复时间、重复字节、丢失字节、成功率；
- 不在 TCP 上重复实现一套无意义的 packet ACK。

## 9. Benchmark 规则

与 HY2 或其他实现比较时必须尽可能固定：

- server/VPS；
- 路线与时间窗口；
- MTU；
- 安全等级；
- 应用负载；
- netem 条件；
- 测试时长与样本数。

报告 median/P95/failure count，不只报告最好成绩。负结果也应保存。

## 10. Research 任务规则

研究任务必须产出可追溯文件，并区分：

- 标准事实；
- 既有实现行为；
- 本项目设计选择；
- 待验证假设。

优先 RFC、IETF draft、内核/库官方文档、论文和上游源码。博客可辅助，不可替代关键规范来源。

## 11. Commit 规则

建议 checkpoint 类型：

- `spec:` 协议/规范改变
- `feat:` 可运行能力
- `test:` 测试/fuzz/fixture
- `research:` 可复核调研
- `bench:` benchmark harness/result
- `docs:` 纯文档
- `fix:` correctness 修复

一个 checkpoint 应能回答“这一笔新增了什么可验证能力”。不要把多个独立里程碑揉成一个巨型提交。

## 12. Stop conditions

出现以下情况时停止当前方向并记录 blocker/decision：

- 必须自研密码原语才能继续；
- 设计迫使 Session 重新绑定单一 Carrier；
- 需要关闭认证/完整性保护才能获得性能；
- 未解决资源上限就准备暴露公网服务；
- benchmark 条件无法做到基本公平；
- 真实网络实验需要影响非授权第三方；
- fuzz/测试发现 parser correctness 问题但下一任务只是加功能；
- 当前动作超出 `docs/standing-vps-lab-authorization.md` 的持续授权边界，且确实需要新的维护者许可。

以下内容**不是** stop condition：

- standing authorization 已覆盖的普通自有 VPS TCP/UDP 实验缺少再次确认；
- standing authorization 已提供默认 profile 时缺少逐次 `count/bytes/duration/port`；
- 一个独立 WAN 节点失败但仍存在不依赖它的 READY 工作。

## 13. 完成定义

一个任务只有在以下条件满足时才算完成：

- 实现存在而不是只有文档；
- 对应测试/fixture 存在；
- 本地门禁通过或明确记录不可运行项；
- 规范与代码没有明显漂移；
- 没有把新的关键架构决定留在未记录状态；
- 对外宣称与实际验证程度一致。

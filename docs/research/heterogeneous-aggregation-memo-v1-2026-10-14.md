# 异构 Multipath 聚合研究备忘录 v1（Heterogeneous Aggregation Research Memo）

> Status: research memo v1 — **not a design, not a gate application, not an
> implementation approval**. Zero code / state-machine / wire change is
> proposed or implied by this document.
>
> Work item: `doctrine-kickoff-20261014 / w-l6-agg-memo`. Line: ROADMAP
> 「异构聚合研究」(L6). Baseline: main `c45308d`.
>
> 红线遵守（AGENTS.md §1.1 原文：「聚合实验不偷改 failover exactly-once
> 条件」）：本备忘录的所有提案均**不修改** M3 failover 的
> uncertain → confirm → dedup 状态转移语义、`FailoverController` 的单
> active 转移条件、或 `neko-session` DeliveryLedger 的 overlap/conflict
> 不变量。每个提案就地重申兼容性（§2.x「红线」段）。聚合启用仍被
> `docs/spec/m4-concurrent-multipath-gate.md` 关闭；本研究不申请开 gate，
> 只为重开条件积累语义地基。

## 0. TL;DR

1. **语义位置**：聚合 = Session 层之下的**发送端调度器扩展** + 接收端
   既有 DeliveryLedger 重组语义的复用。neko 的 DSN 地基原生存在——
   `DataId` / `LogicalRangeId(stream, offset)` 就是连接级数据身份，
   与 per-path packet number 已分离（carrier-architecture.md §5.2）。
   缺的不是身份，是**并发 assignment 语义**（`ConcurrentCarrierManager`
   现为 single-active 迁移模型，`assign` 只绑 sole active owner）。
2. **四大问题判读**：DSN——**已绕开**（身份已全局）；reordering——
   **必须解决但重组器已存在**，缺口在 gap-time 有界性；HoL——
   **绕不开**（stream 内有序是 Session 契约），只能调度层吸收；
   congestion coupling——**必须解决**，TCP carrier 的内核 CC 不可注入
   是硬约束，一期形态应把 TCP 路视为不受控容量源。
3. **W5 边界**：W5（D077）是 **app 层同构多流**（socket-per-stream +
   确定性分片不交错 + 合取 receipt），是异构聚合的退化特例。它的
   406×/98.8% 数字、零重组、零 HoL、无 CC 耦合**均不可外推**到单
   Session 交错聚合。可共享的只有「per-connection 钳制是运营商策略、
   与层级无关」这一现象判断和「确定性分片可行」的方法启示。
4. **实验阶梯**：S0 socket-free 状态模型（M4 gate 明文要求的第一步）→
   S1 netns 受控异构路径 → S2 WAN。公平性与 HoL 调度收益只在 S1 可证；
   WAN 只回答「真实路径买不买账」。

## 1. 语义位置：聚合发生在哪一层

### 1.1 分层坐标

```text
┌──────────────────────────────────────────────────────┐
│ Application / Tunnel                                  │
├──────────────────────────────────────────────────────┤
│ Nekomusume Session                                    │
│   DeliveryLedger：stream+offset 身份、overlap 幂等/   │
│   conflict、confirmed watermark —— 接收端重组点(已有) │
├──────────────────────── 唯一接缝 ────────────────────┤
│ Carrier Manager                                       │
│   FailoverController(单active) /                     │
│   ConcurrentCarrierManager(standby-warm迁移)          │
│   ▲ 聚合的发送端扩展点：assignment 从                 │
│     「sole active owner」并发化为「owner 集合调度」    │
├──────────┬──────────┬──────────┬─────────────────────┤
│ UDP      │ TCP      │ ICMP     │ ...                  │
│ carrier-local PN │ (无PN,可靠流) │                   │
└──────────┴──────────┴──────────┴─────────────────────┘
```

- **发送端**：聚合发生在 Carrier Manager 一层——决定哪个
  `LogicalRangeId` 由哪条 path 携带。现有代码里这一决策是平凡的
  （`assign` 绑唯一 active owner，`replay_uncertain`/`finish_drain`
  在切换/失败后重绑，lib.rs ConcurrentCarrierManager，单测覆盖）。
  聚合 = 把这个单点决策并发化。**这层扩展不触碰 Session 语义。**
- **接收端**：重组点就是 `neko-session` DeliveryLedger——insert 的
  offset-range 语义、exact-duplicate 幂等、novel-byte overlap 拒绝
  （session-v0 spec「Delivery segment overlap invariant」）与
  `FailoverController::receive` 的 first-delivery/duplicate 判定
  （lib.rs，M3 单测含「received exactly once」断言）已经是跨 carrier
  的逻辑重组器，因为身份（DataId）从不绑定 carrier。
- **身份层**：`DataId(u64)` 与 `LogicalRangeId { stream, offset }` 是
  连接级 DSN 的现成形式。per-path 序列（UDP Recovery 的 PN 空间、
  TCP 的字节流序）从未被提升为数据身份——这正是 RFC 8684 MPTCP 的
  DSN/subflow-seq 分离同构（protocol-standards.md 已收录该先例，
  并明文「Do not import MPTCP scheduler/token format」）。

### 1.2 与 failover exactly-once 为什么不冲突（总论证）

M3 机制（crates/neko-carrier/src/lib.rs FailoverController + m3
spec/单测）的 exactly-once 由三个不依赖「同一时刻只有一条活路」的
构件支撑：

| 构件 | 现状 | 聚合下的角色 |
|---|---|---|
| 全局 record 身份（DataId，跨 carrier 幸存） | 已有 | 不变——身份本就 carrier 无关 |
| 接收端 idempotent dedup（exact dup 幂等 / 冲突拒绝） | 已有（receive 单测：duplicate 吸收、conflict 报错） | 不变——多路到达只是 dup 来源变多 |
| confirm 收紧 uncertain（bounded 容量、fail-closed） | 已有 | 不变——per-range confirm 与路径数无关 |

exactly-once 的实现里**唯一**的「单路假设」是 `active: ActiveCarrier`
单值与 `tcp_resend()` 的 WrongCarrier guard——那是**调度状态**，不是
交付语义。failover 模式「先后」与聚合模式「同时」共享同一身份/确认/
去重机制；区别只在 range→path 的绑定是一时一个还是一时多个。
因此兼容性判据可以一句话写死：

> **兼容性判据（本备忘录所有提案共守）**：任何聚合形态只允许
> （a）在 Carrier Manager 层新增 owner 集合与调度；（b）复用
> DataId/LogicalRangeId 身份、confirm/receive 语义与 DeliveryLedger
> 不变量原样；（c）把「单 active」读取为「单 active 集合」的扩展。
> 不允许改 uncertain→confirm 转移条件、dedup 幂等/冲突语义、或
> ledger overlap 规则本身。

## 2. 四大经典问题在 neko 语义约束下的判读

经典问题谱系以 MPTCP 为主要参照（RFC 8684 路径/身份管理；RFC 6356
coupled congestion control；SCTP RFC 9260 CMT 对照），与 M4 gate 的
9 项缺失设计要件逐一对得上（该 gate 文档是本研究线的直接前身）。

### 2.1 DSN 序列空间（per-path 单调 vs 全局）

- **经典问题**：MPTCP 用连接级 64-bit DSN + per-subflow 序列双重编号，
  代价是映射表与窗口管理复杂度（RFC 8684 §3.3）。
- **neko 判读：已绕开**。`DataId`/`LogicalRangeId(stream, offset)` 已是
  全局身份，per-path PN 只属于 UDP Recovery 的 carrier-local 证据域
  （m3-runtime-contract：「packet-number allocation … remain
  carrier-local evidence」）。M4 gate 要件 1「connection-level data
  sequence independent of carrier packet numbers」在**身份层已满足**。
- **真正缺口**：assignment 的并发语义。`ConcurrentCarrierManager` 的
  `ranges: BTreeMap<LogicalRangeId, AssignedRange>` 数据结构已是
  「range → owner」映射，但 `assign` 只接受 sole active owner
  （`NoActive` 错误路径）。并发化是这个函数的语义扩展：owner 从
  `ConcurrentPathKey` 单值变为集合。这是**状态模型研究**，不触碰 wire。
- **红线声明**：DSN 复用既有身份，不改任何确认/去重语义。兼容。

### 2.2 Reordering（接收端重组窗口）

- **经典问题**：异构 RTT/带宽下到达序偏离发送序，接收端需要重组缓冲，
  且缓冲必须有界（M4 gate 要件 3「receiver memory and gap-time limits
  under asymmetric RTT and blackholes」）。
- **neko 判读：必须解决，但重组器已存在，缺口有界**。
  - 已有：DeliveryLedger 的 bounded insert（offset 语义、byte-limit、
    conflict 拒绝）+ `receive` 的 DataId dedup。跨 carrier dedup 在 M3
    单测已证（fresh-AEAD duplicate 同 DataId 被逻辑去重）。
  - 缺口 A（gap-time 语义）：ledger 的 overlap invariant 禁止
    「bridge an uncovered gap」的合并——failover 模式下这正确（gap =
    可能的 conflict）。聚合模式下慢路 range 在途时快路 range 连续
    到达，会产生**预期中的临时 gap**。两种候选处理：(i) 调度层保证
    「同一 stream 的 range 到达序无 gap」（大 chunk 交替分配，见
    §2.3），ledger 语义零改动；(ii) 引入「挂起 gap」状态——**这改
    ledger 语义，踩红线，一期排除**。
  - 缺口 B（乱序 insert 容忍度）：**已考证（OQ1 回填，2026-10-14，
    读 crates/neko-session/src/lib.rs insert/confirm_received 路径，
    零改动）**。三条结论：
    1. watermark 是 **confirmed 前沿**（仅 `confirm_received` 推进，
       insert 不推进；可跳 hole）。非重叠 insert 只受**前瞻** bound
       （`offset ≤ watermark + max_reorder`，默认 64B、inclusive；
       另有 `max_offset_jump` 1MiB），**后向（落后于 watermark）无限
       容忍**；带 hole 的非重叠 insert 被接受——spec「never
       synthesize bytes across a hole」只约束重叠 merge 路径（单测
       `insert(1,10)` 过 / `insert(1,11)` ReorderLimit 锚定语义）。
    2. **聚合含义**：快路 range 先到先 confirm 推高 watermark 后，
       慢路 range 后到时 offset 落于 watermark 之下，**天然被接受**
       （ReorderLimit 只拦超前方向）；前瞻 64B 默认值对聚合太紧
       （须按 RTT 失配 × 带宽配置），但 `max_reorder` 是 per-ledger
       可配参数（测试含 `u64::MAX` 案例），调参即可、零语义改动。
    3. **watermark 连续性归属**：confirm 无条件跳 hole 推进（`end >
       watermark` 即推）——它是「最高 confirmed end」而非「连续交付
       水位」；m3-runtime-contract 的「exactly once in offset order」
       连续性责任在 Session runtime 读侧。聚合下「快路确认越过未确认
       慢路区间」是 ledger 允许的形态（单 active 下不出现，聚合形态
       归 S0 研究域）。
- **红线声明**：候选处理 (i) 零 ledger 改动；(ii) 违反兼容性判据 (b)，
  明确排除。一期聚合必须以「到达序无 gap」为调度器约束。

### 2.3 HoL blocking（异构 RTT 下快流等慢流）

- **经典问题**：单一有序字节流上，慢路径的在途字节卡住快路径已到
  数据的交付（connection-level HoL）。MPTCP 用 DSN 映射把 subflow 内
  HoL 解掉，但 connection 级仍受 scheduler 决策影响。
- **neko 判读：绕不开，只能调度层吸收**。stream 内有序交付是 Session
  契约（m3-runtime-contract：「Application receives each byte range
  exactly once in offset order」），不能放弃。carrier-architecture.md
  §6 明文第一阶段禁 round-robin 条带化的理由正是「跨协议 Head-of-Line
  blocking」——该禁令与 M4 gate 一致，本研究不挑战它。
- **吸收手段**（均为 scheduler 研究域，零 Session 改动）：
  1. **大 chunk 交替**（保守）：慢路拿大粒度不相交区间，快路小粒度，
     使「等待慢路 chunk」的暴露时间 ≈ 慢路 RTT ×（快路 chunk 数/
     慢路 chunk 数）的调度问题。W5 的确定性分片（D077 条款 2）是
     其同构退化特例——区间不相交 + 每流独立有序 ⇒ 结构性零 HoL。
  2. **乱序交付窗口**（OQ1 考证后确认成立）：ledger 对「落后于
     watermark 的非重叠 insert」无限容忍（§2.2 缺口 B），快路 range
     可先落 ledger、先 confirm，交付顺序由 runtime 读侧保序——HoL
     从「阻塞接收」降级为「阻塞 watermark 推进」，代价转移为 bounded
     重组内存与 `max_reorder` 前瞻调参（默认 64B，per-ledger 可配）。
  3. unreliable datagram 流量（session-v0 已预留 datagram-shaped
     message 语义）天然无 HoL——若聚合一期只跑 datagram 流量，
     HoL 问题整体消失。这是**形态选择**，不是设计奇迹。
- **红线声明**：三种手段都不动 ledger/确认语义。兼容。

### 2.4 Congestion coupling（多路径共享瓶颈的耦合控制）

- **经典问题**：N 条路共享瓶颈各自独立 CC 会抢 N 倍公平份额
  （RFC 6356 的动机），聚合必须做耦合或至少记账。
- **neko 判读：必须解决，但一期有结构性简化路径**。
  - 硬约束：TCP carrier 的 CC 在内核，neko 无法注入 cwnd——对 TCP 路
    做「耦合控制」不可行（改内核/`TCP_CONGESTION` 定制是 out of
    scope）。M4 gate 要件 4「congestion coupling so TCP and UDP do
    not compete unfairly for one bottleneck」在此约束下只能部分满足。
  - 一期形态建议（研究假设，非设计）：TCP 路 = **不受控容量源**
    （类似固定速率腿），UDP 路自适应并对「观测到的 TCP 路吞吐」做
    记账式避让；等价于 RFC 6356 的耦合在「一条腿不可控」时的退化。
    DCCP carrier（Tier 2 候选，carrier-architecture.md §7）是「底层
    CC + Session 可靠」的对照实验位，留二期。
  - 本地证据现状：W5 实测 N=4 与 N=8 聚合持平（199/193 Mbps，
    D076 实验 A）——上界按流数而非拥塞信号饱和，与 per-connection
    钳制形态一致（机制未定，D076-C 因果降级在案）。要点是：
    **「无耦合多流未观察到公平性事故」不构成耦合问题不存在的
    证据**——该路径若瓶颈非拥塞队列，公平性问题根本不被激发。
    真拥塞共享瓶颈场景 W5 从未测过（bjlh↔us-209 出向是套餐顶），
    必须 netns 造（S1）。
- **红线声明**：CC 协调完全在 scheduler/carrier-local 域，无 Session
  语义触碰。兼容。

### 2.5 其余 M4 gate 要件的落点（简表）

| M4 gate 要件 | 本备忘录落点 |
|---|---|
| 5. ACK attribution 不提升 carrier 反馈为 Session 交付 | 两层确认语义（carrier-architecture §5）已隔离；聚合的 confirm 仍是 Session 层显式事件 |
| 6. 多 carrier 重发 ownership | OQ2（复写 vs 重指派） |
| 7. scheduler 行为（TCP HOL vs UDP loss/reorder） | §2.3 手段 1/2 的 A/B 实验域 |
| 8. path-generation / key-phase / migration 交互 | OQ6；migration-back 状态机（ADR）与聚合模式映射未研究 |
| 9. 与 failover-only 的受控对比 | S1 实验阶梯的对照组定义 |

## 3. 与 W5 实测证据的差异边界（不外推声明）

W5（D074 条款 4 → D076 门控 → D077 实现验收）实测了什么、没测什么，
逐项对照：

| 维度 | W5（D077 已实测） | 单 Session 异构聚合（本研究线） |
|---|---|---|
| 层级 | Session **之上**：N 个完整独立 Session（socket-per-stream，每流独立握手/SecureSession/SessionRuntime） | Session **之下**：1 个 Session、N 条 path 并发携带 range |
| carrier/路径 | 同构：同 carrier（TCP）同路径族 | 异构：TCP+UDP、跨路径（定义域） |
| 数据身份 | 无全局 DSN——确定性分片双侧独立推导（shard 基数平移），wire offset 连接本地 0 基 | 必须用全局 DSN（§2.1） |
| 交错 | **不交错**：流 i 拿不相交区间，各自连续 | 调度器可交错（否则退化为 W5 形态） |
| 重组 | 零重组（区间不相交 + 各流独立确认） | 交错则必有重组（§2.2） |
| HoL | 结构性零 HoL | 有暴露面，需调度吸收（§2.3） |
| CC 耦合 | 无（每连接独立 Reno；实测路径瓶颈是 policer 非拥塞点） | 必须设计（§2.4），W5 数据对此无证据力 |
| 失败语义 | 合取 receipt：任一流死 → 全体 fail-closed 退出 | 路径死 → range 重指派（failover 机制的并发化），Session 存活 |
| 实测数字 | 406×（D076 实验A，晨窗）、98.8% R2（D077，US→CN N=4） | 无——本研究线尚无任何实测 |

**可从 W5 继承的**（层级无关的现象/方法）：

- 「单连接无解，多流（≥4）有解」的 per-connection 钳制判读——运营商
  per-connection 策略对 Session 层聚合同样存在（一条 TCP carrier 仍
  是一条连接），这是聚合的**动机证据**而非收益证据。
- 确定性分片方法的可行性（双侧独立推导、SHA 交叉验证两日一致）。
- 时间窗纪律（§3.3：入向结论须带时间戳、晨开午关）——未来任何聚合
  WAN 实测直接继承。
- D076-C 教训：机制未分离时不写因果级表述。

**不可外推的**：

- W5 的 406×/98.8%/247-311 Mbps 全部是「同构 + 不交错 + app 层」形态
  的数字，**不预测任何交错/异构形态的吞吐**——交错引入重组与 HoL
  成本，异构引入 RTT 失配，两者都可能显著改变结论方向。
- W5 的零 HoL/零重组是形态给的，不是机制给的；Session 内聚合若交错，
  这两项从零成本变正成本。
- W5 的「无耦合无公平性事故」是路径特性（policer 瓶颈）给的，不可
  当作耦合问题不存在的证据。

**关系一句话**：W5 是异构聚合在「同构、不交错、上移到 app 层」极限
下的退化特例。它证明了这条路的最保守形态可用；它没有为任何更激进
形态提供证据。

## 4. Open questions 与实验设计阶梯

> OQ1 已完成（2026-10-14）：结论回填 §2.2 缺口 B——乱序 insert 后向
> 无限容忍、前瞻 `max_reorder` 可配；confirm 的 ledger 全局 context
> 单调性对并发路是硬约束（强化 OQ6）。考证方法：只读
> crates/neko-session/src/lib.rs insert/confirm_received 路径 + 单测
> 锚定，零改动。

### 4.1 Open questions

- **OQ1（考证，已完成 2026-10-14）**：结论回填 §2.2 缺口 B——
  非重叠乱序 insert 后向无限容忍、前瞻受 `max_reorder`（默认 64B，
  per-ledger 可配）bound；「无 gap 纯乱序 insert」在现有 ledger 语义
  下**免费**，§2.3 手段 2 成立。附带发现：`confirm_received` 的
  ledger 全局 context 单调性（component-wise monotonic）对并发路是
  硬约束——不同 path_generation 的成员不能并存于确认流（详见 §2.2
  缺口 B 第 1/3 条与 OQ6）。
- **OQ2**：同一 range 双路复写 vs 单路指派 + 失败重指派。复写提速
  时延但 duplicate bytes 上升（M3 metrics 有 duplicate_bytes 计数可
  观测）；dedup 已有，问题纯属 scheduler 收益权衡。
- **OQ3**：gap-time 上界与 `max_uncertain_{entries,bytes}` 的联合
  bound——RTT 失配最大多少倍时重组缓冲仍有界（M4 要件 3 的定量版）。
- **OQ4**：TCP 路不可控假设下，记账式速率协调的可行域与失效模式
  （TCP 吞吐波动大时记账滞后是否震荡）。
- **OQ5**：D076 遗留的 N=2→N=4 跳变机制未定（流数阈值 vs 拥塞协同），
  它同时是聚合路数自适应 scheduler 的输入——两线应共享复检协议
  （docs/experiments/d076-causal-recheck.md）产出。
- **OQ6**：聚合模式与 migration-back 状态机（m4 ADR：TCP_HOLD/
  UDP_ACTIVE + hold counter）的映射——「active 集合」变化如何投影到
  既有单 active 状态机，或需要并行的集合版状态机（S0 模型域）。
  **OQ1 考证强化了本题**：`confirm_received` 的 ledger 全局
  component-wise 单调（OldEpoch/InvalidMigration guard）要求聚合成员
  的确认 context 不回退——若并发成员携带不同 path_generation
  （如旧代 UDP 路 + 新代 TCP 路并存），后到的旧代确认会被拒绝。
  候选方向（S0 域）：聚合集合成员共享统一 generation 空间（成员
  变更原子 bump），或确认 context 与成员身份解耦。这两条与
  `ConcurrentCarrierManager` 现有 per-path generation 语义的张力
  是 S0 状态模型的核心设计输入。

### 4.2 实验阶梯（先模拟后实测）

M4 gate 重开条件原文即阶梯：「socket-free DSN/reordering/
congestion-coupling model first, deterministic asymmetric-path and
blackhole tests, then isolated netns A/B results」。

**S0 — socket-free 状态模型（可证：正确性类问题）**

- 域：`ConcurrentCarrierManager` 风格的纯状态机多 owner 扩展语义
  模拟（研究原型，不进主代码）+ 确定性时钟。零 socket 零 netns。
- 可证：多路 assign/confirm/dedup 的不变量（每 range 至少一个
  owner、confirm 后无重发、bounded uncertain 容量 fail-closed）、
  gap-time 上界的存在性（OQ3 的解析/枚举验证）、OQ6 状态机映射的
  穷举迁移正确性。
- 不可证：任何与时延/吞吐有关的行为。

**S1 — netns lab：netem 受控异构路径（可证：行为类问题）**

- 拓扑：双/三路 veth + netem，参数矩阵 {RTT 比 1:2–1:10、loss
  0–5%、带宽比、共享瓶颈单队列 vs 分离}。
- 可证：重组窗口实际行为（§2.2 缺口 A 的实证）、HoL 调度收益
  （A/B：大 chunk 交替 vs 逐 record 交错 vs 复写）、**无耦合双路
  共享瓶颈的公平性破坏**（§2.4——W5 未测、只有 S1 能造）、TCP 路
  记账式避让的失效模式（OQ4）。
- 对照组（M4 要件 9）：同参数 failover-only（单路+切换）vs 聚合，
  声明指标（吞吐/时延/duplicate bytes/watermark 停进时长）。
- 不可证：真实运营商钳制形态、真实 middlebox 行为。

**S2 — WAN（standing authorization 范围；可证：现实买账类问题）**

- 只回答：真实 RTT 失配（如 bjlh↔us-209 165ms vs 本地中转腿）与
  per-connection 钳制形态下，聚合是否有净收益；TCP+UDP 混合在真实
  双栈下的可用性。
- 不可证：受控公平性（WAN 无法固定瓶颈）——凡 S1 未过的公平性
  断言不得进 S2。
- 纪律继承：入向吞吐结论带时间戳（§3.3）；双锚回归
  （bjlh↔us-209 1170/256MiB ±10%）不受实验污染；D076-C 因果措辞
  纪律。

### 4.3 阶梯推进条件（研究线内部，非 gate 声明）

S0 未产出自洽模型前不开 S1；S1 未给出「聚合 ≥ failover-only 在
声明指标上」的受控 A/B 结论前不开 S2。每一阶产出落
`docs/research/` 或 `docs/experiments/`，走 PR（佩丽卡审）。

## 5. 引证清单

仓库内（normative/evidence）：

- docs/specs/nekomusume-session-v0.md — Delivery segment overlap
  invariant、Context commit invariant、unreliable datagram boundary
- docs/carrier-architecture.md — §5.2 path/session 序列空间分离、
  §6 一阶段禁条带化、§7 carrier 能力表（含 DCCP）、§12 研究问题 6
- docs/spec/m3-tcp-failover.md — FailoverController 语义、uncertain
  边界审计
- docs/m3-runtime-contract-2026-08-29.md — 两层确认、「Application
  receives each byte range exactly once in offset order」
- docs/spec/m4-concurrent-multipath-gate.md — 9 要件清单与重开条件
  （本研究线前身）
- docs/adr/m4-migration-back-state-machine.md — 单 active 状态机与
  guards
- docs/decisions.md — D074（条款 3/4 R1 与多流互补声明）、D076
  （per-connection 钳制 + D076-C 因果更正）、D077（W5 实现与验收、
  时间窗附录）
- docs/limits/2026-10-13-international-matrix-redacted.md — §3.2
  实验 A/B 原始数据、§3.3 时间窗
- crates/neko-carrier/src/lib.rs — FailoverController（DataId
  uncertain/confirm/receive）、ConcurrentCarrierManager
  （LogicalRangeId/assign/replay_uncertain，single-active 语义）
- crates/neko-cli/tests/w5_multistream.rs 及 D077 所引 bulk multi
  路径（commit 66fc70c）— W5 形态实证锚

公开经典文献（repo 未收录部分标注）：

- RFC 8684（MPTCP v1；repo 已收录于 protocol-standards.md）— 连接级
  DSN 先例；repo 明文不引入其 scheduler/token
- RFC 6356（Coupled Congestion Control for Multipath TCP；repo 未
  收录，公开经典）— §2.4 耦合控制参照
- RFC 9260（SCTP；repo 已收录）— CMT 多流分配对照、Tier 2 候选
- RFC 4340（DCCP；repo 已收录于 carrier-architecture §11）— 底层
  CC + 上层可靠的对照实验位

## 6. 本文不做什么（边界重申）

- 不申请开 M4 gate；不提出 wire 字段、scheduler 实现、状态机变更。
- 不改 failover exactly-once 任何条件（§1.2 兼容性判据为全提案共守）。
- 不把 W5 数字用于任何异构/交错形态的预测（§3）。
- 后续若某提案无法满足 §1.2 判据，该提案即死，不寻求豁免。

<!-- w-l6-agg-memo v1 2026-10-14, draft for Pelicca review -->

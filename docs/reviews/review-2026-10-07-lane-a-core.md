# Lane A — 猫娘协议（nekomusume）核心代码审查

- 审查对象：`/media/tmzn/DATA5/nekomusume-work`，HEAD `46241a1`（2026-10-01），Rust workspace，8 crate
- 审查方式：只读静态审查（read/grep/awk），未 build、未改任何文件
- 范围：neko-wire / neko-session / neko-crypto / neko-reliable / neko-carrier / neko-observe 全部生产代码；neko-cli 的 preauth.rs、framed.rs、multistream.rs、pmtu_socket.rs、periodic.rs、reachability.rs、main.rs
- 统计口径：**生产代码 = 文件内所有 `#[cfg(test)]`/`#[cfg(all(test,..))]` 模块（含 brace 匹配剔除，包括嵌套与散布于文件中段的测试模块）之外的全部代码**。以下所有 unwrap/expect/panic 计数均按此口径。

## 0. 总体结论（先读这段）

这是一个纪律极强的研究性代码库：库 crate（wire/session/crypto/reliable/observe 的状态机部分、carrier 的状态机部分）生产路径几乎零 unwrap/panic，全部用 checked/saturating 算术、显式上限和 fail-closed 错误枚举。**真正的风险几乎全部集中在 neko-cli**，且根因不是"缺检查"，而是一个架构决定：**CLI 把"进程生命周期"当作实验变量——所有错误路径（包括攻击者可远程触发的 preauth 拒绝、单条 malformed 帧）都直接 `fail()` = `std::process::exit(2)`（main.rs:338-341）**。库层精心构造的"拒绝并继续"能力（如 preauth.rs 里完备的 admission 状态机）在 CLI 层被一次性消费掉。在当前"单次实验跑完即退"的定位下这是自洽的；但它与 docs 里"pre-auth 准入控制防 DoS"的叙述直接矛盾——**任何未认证流量都能让监听进程自杀**。这是后续做长驻服务（IPv6、多路径、ICMP carrier 之前）必须先解决的结构性问题。

无 TODO/FIXME/unimplemented!/todo! 标记（全仓库 src 零命中）；`panic = "abort"` 双 profile（Cargo.toml:24,29）放大任何 panic 的后果。

## 1. panic/unwrap 地图（问题 1）

### 1.1 全量计数（生产代码，已剔除测试模块）

| 文件 | unwrap | expect | panic! | 备注 |
|---|---|---|---|---|
| neko-wire/src/lib.rs | 0 | 4 | 0 | expect 全为不可达不变量（见 1.2） |
| neko-session/src/lib.rs | 2 | 3 | 0 | unwrap 在 1677/1735（见 1.3） |
| neko-crypto/src/lib.rs | 0 | 1 | 0 | L111 常量解析 |
| neko-reliable/src/lib.rs | 0 | 1 | 0 | L238 默认值 |
| neko-carrier/src/lib.rs | 3 | 2 | 0 | 见 1.4 |
| neko-observe/src/lib.rs | 0 | 0 | 0 | 干净 |
| neko-cli/src/main.rs | **194** | 11 | 0 | fail()=457 处 |
| neko-cli/src/periodic.rs | **32** | 0 | 0 | fail()=51 处 |
| neko-cli/src/multistream.rs | **6** | 0 | 0 | |
| neko-cli/src/preauth.rs | 0 | 1 | 0 | L217 默认 limit |
| neko-cli/src/framed.rs / pmtu_socket.rs / health_window.rs / lifecycle.rs / reachability.rs | 0 | 0 | 0 | 干净 |

71 处 `panic!` 全部位于 tests/ 与 `#[cfg(test)]` 模块内，无生产 panic 宏。

### 1.2 无害的 expect（已确认：不变量型，不可被输入触达）

- `neko-crypto/src/lib.rs:111` — `noise_ik_params()` 常量字符串解析。
- `neko-reliable/src/lib.rs:238` — `Default` 用编译期已知合法的默认 limit。
- `neko-cli/src/preauth.rs:217` — `ProcessPreauthLimits::default()` 已知合法。
- `neko-carrier/src/lib.rs:7623` — `AckRanges::new(32)` 常量 ≤ HARD cap。
- `neko-wire/src/lib.rs:265,291` — 上界先检查后的 u32 转换/切片长度，局部不可达。
- `neko-wire/src/lib.rs:715` — `VersionNegotiator` accept 后已置字段读取。
- `neko-session/src/lib.rs:261,305,310` — `state` 已由 `overlaps` 非空保证。
- `neko-carrier/src/lib.rs:1895` — `Mutex::lock().expect("fault seed")`，仅在 poison（本身已 panic）时失败；可接受。

### 1.3 真正要盯的（已确认：依赖调用方维持不变量）

- **`neko-session/src/lib.rs:1677`** — `self.recv.back().unwrap()`：`push_back` 后立即取尾，局部安全；但该行为属于 `receive()` 内部约定，任何后续重构（如延迟入队）都会变 panic。
- **`neko-session/src/lib.rs:1735`** — `self.send_inflight.get_mut(&stream).unwrap()`：安全性依赖 **L1726-1733 的先查后改**（`get().copied().ok_or(Protocol)?` + delta 双重上限检查在 `if delta > 0` 内）。逻辑目前正确，但"同一 map 查两次、之间无副作用"是唯一防线——**这是 session crate 唯一的脆弱点**，建议后续把 inflight 检查与扣减合并为单次 entry 操作。
- **`neko-carrier/src/lib.rs:4846`** — `self.warm_candidate.as_mut().unwrap()`：依赖 L4836-4841 提前 return 保证 Some。同型。
- **`neko-carrier/src/lib.rs:5080`** — `self.samples.get(&candidate).unwrap()`：candidate 来自同一 `samples` 的迭代，同周期内安全（单线程 BTreeMap 无并发突变）。推测：若未来 choose() 改为跨锁/跨线程采样，此处即 panic。
- **`neko-carrier/src/lib.rs:7778`** — `ConcurrentCarrierManager::new(default).unwrap()`：编译期合法默认。无害。

### 1.4 CLI 的 194+32+6 个 unwrap（已确认：实验夹具语义，非库行为）

抽样核实（main.rs:718,852,1063,1088,2132,2174,2354,2418,2481,3145,3222,3473,3622,3703,4028,4120,4176,4298,4847,4938,5006,5223,6310；periodic.rs 内 32 处；multistream.rs:308,310,356,358,365,396）均为三类之一：

1. 本端 socket/本地资源操作（bind/send_to/set_timeout）——失败即整进程实验失败，exit 是预期语义；
2. 库调用前已由 CLI 参数校验过界的"应当成功"路径（`SessionRuntime::new` 传的是 config() 已验证的 limits）；
3. 攻击者可影响但对端**必须已通过认证**的路径（seal/encode）。

例外（值得单独记录）：
- **main.rs:3222** `u64::from_be_bytes(encrypted[..8].try_into().unwrap())` — encrypted 是本端 `seal_unreliable` 输出，长度 ≥8 由构造保证；不可达。
- **main.rs:3473** `pending_acks.last().unwrap()` — 依赖循环上文 `while let Some(...) = pending_acks.pop()` 的入队配对；已确认当前配对正确（L3440-3473 同一循环内 push 后 last）。
- **main.rs:1063/2898** `delayed_post_ack.take().unwrap()` — 由上文标志位保证 Some。

### 1.5 无界分配 / 无界循环 / 整数溢出 / 时钟回拨（问题 1 后半）

- **无界分配：生产路径未发现。** 关键证据：
  - wire 解码双层限长（`MAX_PAYLOAD_LEN=4096`，neko-wire/src/lib.rs:13；帧层 `MAX_FRAME_PAYLOAD_LEN=1024`，L56；帧数 ≤64，L130/158）；
  - framed.rs:101-107 声明长度先对 `max_frame_len` 检查再 `vec![0; len]`（L112）——**注意 framed 两个读路径里 `stage(Body)` 在分配前**（L108-111 先 stage 后 alloc），TCP 头部路径即使声明 4GB 也在 preauth 计费层先被拒；UDP 天然受 64KiB 缓冲限制（main.rs:740 `b:[0;65536]`）；
  - session 每连接上限齐全（`RuntimeLimits`，neko-session/src/lib.rs:941-951：streams/queue_records/queue_bytes/total_bytes/record_bytes/session_window/stream_window），且 `SessionRuntime::new` 校验（已确认 L1395-1432 区间的构造校验逻辑，含 `HARD_MAX_RUNTIME_*` 全局天花板，L9-13）；
  - crypto：`MAX_RECORD_PLAINTEXT=4096`（L162）、`open()` 长度上下界（L708）、`seal()` 容量预分配 = 精确开销（L695-698）；
  - carrier：`RetransmitBuffer(64, 8192)`（L7791）、`Recovery` 上限（neko-reliable L242-249）、preauth 全套窗口（见 §2）。
- **无界循环：未发现。** 所有 loop 都有 deadline/counter 出口（framed.rs:76-134 的 deadline 检查每轮执行；preauth.rs:100-113 的 write loop 每轮重算 budget；main.rs:1590-1647 的 ack 等待有 deadline + `MAX_POST_HANDSHAKE_MALFORMED` 上限）。模拟器有 rounds ≤ total+2 硬顶（neko-reliable/src/lib.rs:584,643）。
- **整数溢出：库层全覆盖。** 抽查的所有累加点均用 checked/saturating（session L145,150-155,1633-1660；crypto L48,1319,1338,1390,1408；reliable L51,191；carrier L289,4864）。**已确认的例外**：`neko-carrier/src/lib.rs:7647-7649` `largest_observed` 用 `map_or(l.max())` 饱和语义无碍，但 **`PacketAckTracker::observe_packet` 对 `ranges.insert` 的错误处理是 `Err(RangeLimit)` 直接丢弃该包**（L7644-7646）——不 panic，但语义上是"接收方丢弃"，见 §3。
- **时钟回拨（已确认，两处）：**
  1. `neko-crypto/src/lib.rs:1658-1661` — `refresh_window`：`now_ms < window_started_ms` 返回 `SessionRejected`，**整个窗口拒绝服务**（fail-closed，偏保守但安全；对系统时钟回拨敏感——回拨期间所有新请求被拒）。
  2. `neko-crypto/src/lib.rs:1680-1684` — `live()`：`now_ms < created_at_ms || now_ms < last_progress_ms` 同样拒绝。
  - CLI 侧用 `Instant::now()` 单调钟（preauth.rs:215,221-227），不受回拨影响；**推测**：若未来把 `now_ms` 换成系统时钟（如多进程共享窗口），这里会从"拒绝"变"卡死"，需保持单调钟。
  - `RttEstimator::update`（neko-reliable/src/lib.rs:153-177）对 `now < sent_at` 未显式拒绝，`on_ack` 里 `now_us.checked_sub(p.sent_at).ok_or(Arithmetic)`（L346）已 fail-closed——已确认闭环。
  - health_window.rs 全程 `Instant`，无回拨面。

## 2. 资源上限清单（问题 2）

已确认，全部硬上限、超限即拒绝（拒绝 = 返回错误枚举，不排队不重试不 panic）：

**每状态/每源（preauth，ProcessPreauthLimits::default，neko-crypto/src/lib.rs:1465-1490）**
| 维度 | 上限 |
|---|---|
| states/源 | 8（全局 1024） |
| memory/状态 | 16 KiB（全局 16 MiB） |
| queue/源 | 4（全局 256） |
| 输入字节/源 | 64 KiB；包/源 64 |
| 窗口输入 | 8 MiB / 8192 包每 1000 ms |
| work | 4096/包、131072/源、1048576/窗 |
| 响应 | 2048 B / 4 包每源；256 KiB / 512 包每窗 |
| 生命周期 | idle 1000 ms、max 5000 ms、响应发送 deadline 100 ms |

**每 Session（RuntimeLimits 默认，neko-session/src/lib.rs:952-965）**：streams=1（可调至 4096 硬顶 L9）、queue 64 记录/64 KiB、总 1 MiB、record 1200 B、session window 1 MiB、stream window 256 KiB、idle 30 s、close 5 s。超限 → `QueueFull`/`TotalLimit` 等错误；`open_stream` 超限 → `TooManyStreams`。
**去重窗口**：session 侧 `received` BTreeMap 按 (stream,offset) 全存（受 total_bytes 上限约束）；crypto 侧 ReplayWindow 64（MAX_REPLAY_WINDOW，L13）。
**重传**：RetransmitBuffer 64 帧/8192 B（carrier L7791）；Recovery `sent` ≤ max_sent_packets(默认 4096/硬 65536)、每包帧数 ≤64（硬 1024）。
**Carrier**：路径数上限 max_paths（默认 8，carrier lib.rs:116），ACK ranges 32（L7623，受 HARD_MAX_ACK_RANGES=4096 上限约束）。
**observe**：ring ≤1024 事件（MAX_EVENTS，observe lib.rs:11），序列饱和后转 drop 计数（L424-443）——设计良好。

**发现的缺口（已确认）：**
- **G1｜SessionRuntime.events 无上限**（neko-session/src/lib.rs:1844-1850 `events.push` 无界；文档注释 L1802-1806 明说"the event log"是故意保留的存活事实）。CLI 侧未消费任何清理入口（无 `events.drain`）。periodic 场景 `MAX_COUNT=600`（periodic.rs:11）天然封顶 ≈ 每记录 3-4 事件 ≈ 数千条；但任何把 SessionRuntime 用于长连接的后续工作都会线性泄漏内存。docs（docs/status.md:268 附近）已把 `SessionRuntime.events` retained-state 列为未决容量策略——**代码现状 = 无界、无淘汰、无计数器告警**。评级 P1（架构上挡长驻服务）。
- **G2｜preauth 过期依赖调用方主动 `expire()`**：`admit_carrier` 内会 `let _ = self.expire();`（preauth.rs:245），但 main.rs 的 `server`/`periodic server` 接受循环里**仅在 accept/admit 时间接触发**（main.rs:642-729 循环每次 accept 后 admit）。**已确认**：main.rs:627/1793/4855 三处 `ListenerAdmission` 全部只随新请求顺带过期，**空闲监听器不清理**。后果：空闲状态最多残留 5 s（max_lifetime），内存上限 16 MiB 不变——**上限是稳的，只是清理时机惰性**。P3。
- **G3｜PacketAckTracker 满即丢包**（carrier L7644-7646）：32 个 range 用尽后 `observe_packet` 返回 `RangeLimit`，调用方（main.rs:2126-2131）`fail("r9 server on_packet_received")` **直接退出进程**。已确认这是 fail() 家族的一员（见 §3）。P1（归并到 F1）。

## 3. 状态机缺口（问题 3）

**F1（本报告最重要发现）｜CLI 把库层的"可恢复拒绝"升级为进程退出。** 证据链：
- preauth 引擎设计为"拒绝第 N 个状态，其余照常"（crypto 1547-1656 整个 admission 机器，错误为 `SessionRejected`）；
- 但 main.rs `server`（TCP，L645-647；UDP，L743-745）、`failover_server` 的 pending 路径（L1916-1919）、periodic server（periodic.rs "pre-auth admission rejected"→fail）、multistream（multistream.rs:292-294）**对 admission 拒绝一律 `fail()`=exit(2)**。对照组：**failover_server 的非 pending 路径已经做对了**——L1984-1993 对 `admit_carrier` 失败 emit diagnostic + `continue`，L1995-1999 对 charge_input 失败 release+continue。同一文件两种范式并存，说明库能力足够，是 server/periodic/multistream 三处未迁移。**任何能发 UDP 包的人都能终止 `neko server`**（admit 拒绝条件包括 8 states/源、16 MiB 全局——攻击者用 9 个伪造源即可）。P0（以"防滥用"为目标的代码里的自相矛盾；若定位只是单次实验夹具则为 P2）。**已确认**。
- 单条 malformed negotiation/handshake 帧 → `fail("malformed negotiation")`（main.rs:663,689）——同类。已确认。

**F2｜被吞掉的错误（已确认，影响诊断而非安全）：**
- preauth.rs 全文件 30+ 处 `let _ = ...`（abandon/release/rollback 的返回值被弃置，如 L35-38,254,412,419,426,440,461,472,477,482,491）。设计意图是"清理路径不因二次失败中断"，但 `abandon_input_record` 失败（状态已 rejected）与成功不可区分，**推测**：排查状态泄漏时这些静默点会增加成本。P3。
- main.rs:2354/2619/2971 `runtime.pop_receive(N).unwrap()` 结果丢弃（`if let Some(delivered)`）——仅消费计数用，无泄漏。无害。
- periodic.rs server 尾部 `let _ = runtime.close_stream/close_graceful`（两处）——终态清理容忍失败，合理。无害。

**F3｜超时漏设（已确认 2 处）：**
- `read_frame`（main.rs:511-524）**无 deadline 参数**，依赖调用方预先 `bound_stream_to_deadline` 设置的 socket timeout；periodic/multistream 的 post-handshake 数据帧路径（main.rs:709-719 `for _ in 0..count { read_frame(...) }`）**继承 handshake 阶段的 deadline**（L652 `deadline = start + d`）。若对端在认证后停发，进程会挂到 `start+d` 才退出——对本实验模型成立（d ≤ 30 s，MAX_DURATION L72），**推测**：后续拆出真实长连接时这里必须显式化。P3。
- `wait` 语义：`ack_timeout` 有界（periodic.rs config L83-84 ≤10 s），`handshake_deadline` 有界（main.rs:771）——无其他裸等。
- neko-session `check()`（L1824-1836）对每次公共入口做 idle/terminal 检查——**无漏**。`close_deadline` 只在 `tick()` 推进（L1793-1799），而 `tick` 需要**有人调用**；`receive/pop_send` 走 `check()` 时不推进 close_deadline。**已确认**：一个进入 `Closing` 后完全空闲的 runtime，其 close 超时**只在下次有人调用任何入口方法时**才结算。CLI 均有外层轮询，无实害；库语义上属于"惰性终态"，记录为 P3 语义债。

**F4｜状态机完备性正面确认**（已确认，防误伤）：
- DeliveryState 转移严格白名单（session L318-332：Unsent→InFlight→Uncertain，其余 InvalidTransition）；confirm 只收 InFlight/Uncertain/Confirmed（L345-349）；merge 拒绝跨洞合并（L269-277）、拒绝用高级状态覆盖新字节（L247-250）。
- CarrierState 每个事件都有前置状态校验 + generation 匹配（carrier L179-293）；Fail 只从 Degraded/Draining（L258-263）；Activate 需 Validated+Candidate+hysteresis（L268-287）。`MAX_RUNTIME_STREAM_ID = u64::MAX-1`（session L14）保留哨兵。
- ConcurrentCarrierManager：register 只允许同 path 的新 generation 或 Failed 旧记录（L6928-6940）。
- preauth permit 全部一次性（`active` flag + rejected 状态双保险），abandon 后状态 terminal（crypto L1892-1908）。`PreauthResponsePermit` 有 `#[must_use]`（L1510）。

## 4. 加密与 preauth（问题 4）

**已确认边界（正面）：**
- nonce：方向各自 `NonceManager`，u64 饱和后 `NonceExhausted` 永不回绕、永不复用（crypto L43-53）。序列放密文外（L699），replay 窗口在 AEAD **之后**才推进（L721 注释明确）。
- anti-replay：窗口 ≤64，`TooOld`/`Replay` 分类清楚（L94-103）。key update 时 nonce 与窗口双双重置（L680-681）。
- key update：`MAX_KEY_PHASE=1`（L206）——**只能 rekey 一次**，之后 `update_key_phase` 永久拒绝（L670-672）。这是刻意的实验边界；真实部署需要按 nonce 计数触发的自动 rekey（现在只在 periodic 的 `--key-update-after` 计数触发，periodic.rs:439,578）。**推测**：MAX_KEY_PHASE=1 会挡任何长寿命会话（2^64 nonce 在 4 KiB 记录下 = 海量，但 rekey 通道只有一级）。P2（计划内边界，列入待办）。
- 载荷上限链：`seal_unreliable` ≤1200（L603）；`open_datagram` 分派（L653-664）：plpmtud 开时超界记录走 `open_probe`，probe 结构校验 `is_pmtu_probe_plaintext`（L198-205）后才放行——**放大面被封死**：probe 响应只能是 22 字节固定 ACK。
- preauth 放大控制：`charge_response` 限 `input_bytes*3` 与包数 `input_packets`（L1408-1414），且响应有独立 100 ms 发送 deadline + idle/lifetime 双重截断（L2046-2086）。TCP 分段写有绝对 deadline（preauth.rs:94-115）。

**已确认缺口：**
- **C1｜握手消息不对称下界**：`receive_first`（L556-579）对 message 只有上界检查（≤1024, L561）**无最小长度检查**——直接交给 snow `read_message`，雪层自身会拒；无实害但与 `receive_first_with_resume`（L517 有 `n < 67` 下界，L525-527）风格不一致。P3。
- **C2｜`first_message` 丢弃剩余 payload**：发起方 hello 写 `self.scope`（L444-452），响应方 `finish` 读 1 字节缓冲并要求 `n == 0`（L458-465）——**IK 模式第二消息本就无载荷**，正确；记录为非缺口。
- **C3｜`preauth.rs:517-535` source_key 含端口**：同 IP 不同端口 = 不同源（测试 L542-560 确认此为刻意）。对 NAT 后真实流量，**推测**：一个 NAT 网关的多客户端会共享/分散配额，行为待 WAN 证据。P3（设计选择，标注即可）。
- **C4｜`trust policy` 授权循环**（L269-280）线性扫 records，records 无上限——由调用方传入（CLI 是单元素 Vec，main.rs:692 policy.clone()）。库 API 上限缺失，**推测**：多租户信任表会线性退化。P3。

**RSEC-001 source retention 实际行为（已确认，代码层）：**
- `ProcessPreauthAdmission.states` 的 key 是 `PreauthStateId(u64)` **序号**（L1712-1713，进程内自增），`source` 是 `Vec<u8>` **不透明投影**（`source_key` 由 carrier kind + IP 家族字节 + 端口构成，preauth.rs:517-535），**不含任何地址文本/主机名**；`sources: BTreeMap<Vec<u8>, ProcessPreauthSource>` 只在**有活状态时**保留（`release` 时 `states==0` 即整键移除，L2169-2177）。
- 终态路径：`release`（立即移除）/`expire_states`（idle 1 s / lifetime 5 s 后批量移除，L2181-2200）/`reject`（只打标记，等 expire/release 收尸）。
- **结论：代码层的实际行为 = "最后一个活状态释放即源记录消失，无 TTL/LRU/历史"**——与 docs 里"RSEC-001 blocked at D019 terminal source retention"的表述一致（docs/notes/resource-abuse-adversarial-9999622.md:27、docs/status.md:268）。即：**代码不做超期保留，这正是那个悬而未决的维护者决策点**；当前无泄漏。已确认。

## 5. 架构债（问题 5）

**A1｜`neko-carrier/src/lib.rs` 10,903 行单文件**（生产 4,051 行 + 6,846 行测试）。生产部分至少含 9 个正交领域：CarrierState 状态机（L1-317）、FaultInjectCarrier（L1832-1955）、FairScheduler（L4020-4093）、CarrierManager/migration（L4399-5093）、CarrierHealth（L4188-4265）、ConcurrentCarrierManager（L6885-7267）、PathRecovery（L7269-7596）、PacketAckTracker（L7597-7700）、ReliableUdpRuntime（L7712-8348）。**每拆一个域都能独立 review/演进**；不拆则任何域的改动都在同一 1.1 万行文件里做 diff，且测试模块和生产代码交错（13 个 test mod 散布其间）。P1。
- 同型：`neko-cli/src/main.rs` 7,431 行承载 12 个子命令 + 全部 server/client 逻辑（main() L6568-6594 的 match 是唯一分派点）；`neko-crypto/src/lib.rs` 3,726 行混 Noise 会话 + preauth 准入引擎两个领域（后者 900+ 行，L1301-2219，值得独立模块）。
**A2｜公共 API 与 CLI 耦合（已确认）**：`neko-carrier` 生产代码 import `neko_wire::encode_ack`（见 L2174 区段使用 `neko_wire::encode/encode_ack`）且 `ReliableUdpRuntime` 把 UDP/TCP 双 path（PathId(1)/PathId(2)，L7770-7777）**硬编码**进构造器——"可靠 UDP 运行时"这个库类型事实上绑定了 1+2 路径拓扑与 wire 格式。真正的多路径（>2 path、异构 carrier）需要重构构造器为 path 集合注入。P1。
**A3｜服务器范式三套并存（已确认）**：main.rs `server`（单请求即退）、`failover_server`（多请求循环+pending 状态）、periodic server（单连接）——三套各自维护 preauth 握手序列的拷贝（main.rs:653-708、2364-2404、periodic.rs handshake_server）。preauth.rs 提供了原语但**没有提供"握手会话"这个组合体**，导致每个 server 重新编排 charge/permit/release 次序。F1 的两种范式并存即源于此。P1。
**A4｜duplication：`read_until_staged_with_clock` 与 `read_until_with_clock` 90% 相同**（framed.rs:64-135 vs 145-203，仅 stage 回调差异）；`main.rs read_frame`（L511）与 `multistream frame_read`（L433.rs:199-218）、`write_frame`（L525）与 multistream `frame_write` 同构三份；握手流程在 main/periodic/multistream 三处近拷贝。P2。
**A5｜时间基准分裂（已确认）**：库层 `now_ms: u64` 由调用方注入（好），但 CLI 内部混用三种：`Instant` 绝对时刻（deadline）、`Instant::now()-start` 毫秒 u64（preauth now_ms，preauth.rs:221-227）、`Instant::now()` 直接 elapsed（recovery clock，main.rs:3207,3225）。epoch 混用无 bug（当前每处自洽），但**推测**：迁移到 tokio/async 或多进程共享 preauth 时会成为错误温床。P3。
**A6｜IPv6 现状（已确认）**：wire 层无 IP 概念（好）；pmtu_socket.rs:24-42 已有 v4/v6 双实现；reachability.rs 有 `IpVersion::V6` 校验（L40-46,66-70）；main.rs:990 有 V6 UNSPECIFIED bind。**挡路点在 CLI 参数面**：`parse(args,"--bind")` 默认 `0.0.0.0:0`（main.rs:599）与 periodic `0.0.0.0:{port}`（periodic.rs），端口白名单 40080-40100 硬编码（main.rs:54,260 multistream）——**端口/地址策略散落在 6 个文件各自 f-string 校验**，IPv6 端需要统一 bind 地址解析层。P2。
**A7｜ICMP carrier（推测）**：`CarrierKind` 有 `Other(u8)`（carrier L17-22）但 `preauth::CarrierKind` 只有 Tcp/Udp（preauth.rs:12-16），source_key 前缀只分配 1/2（preauth.rs:519-522）——加 ICMP 需要动 preauth 分类、admission 双处枚举与测试断言（preauth.rs:543-559 长度断言 8/20 硬编码）。P3（预留工作，非阻塞）。

## 分级发现汇总

**P0（1 项）**
- F1：CLI 三处 server 对 preauth 拒绝 fail()=exit，未认证远端输入可终止监听进程（main.rs:647,745,1961,2022,2063,2076,2079,2082；periodic.rs:299；multistream.rs:294 + 全部 "pre-auth ... rejected" fail 家族）。与 failover_server L1984-1993 的正确范式并存，属未完成迁移而非能力缺失。

**P1（4 项）**
- G1：SessionRuntime.events 无界增长（session lib.rs:1844-1850），挡长驻服务；docs 已挂账，代码层无计数上限/淘汰/告警。
- G3+F1 子项：PacketAckTracker RangeLimit 满即 fail 整个 server（carrier lib.rs:7644-7646 → main.rs:2126-2131）——32 个 range 的接收侧活性上限需要"合并/丢弃并告警"策略而非退出。
- A1+A2：carrier 10.9k 单文件 + ReliableUdpRuntime 硬编码 2-path 拓扑（L7770-7792），挡真多路径/ICMP。
- A3：三套 server 握手编排并存，preauth 缺"握手会话"组合抽象——F1 的结构性根源。

**P2（3 项）**
- C5：MAX_KEY_PHASE=1 使 rekey 通道单级（crypto lib.rs:206,670-672），需按 nonce 消耗自动 rekey + 多级 phase。
- A4：framed 双读路径、frame_read/write 三拷贝、握手三拷贝——改动协议时必须多点同步，回归面大。
- A6：bind/port 校验散落 6 处（含 40080-40100 白名单），IPv6 统一入口缺失。

**P3（7 项）**
- 1.3：session lib.rs:1735 先查后改的 send_inflight 扣减（唯一脆弱 unwrap）；carrier 4846/5080 同型不变量。
- G2：preauth 过期惰性（仅随新请求触发），空闲监听不清理（上限稳，5 s 内自愈）。
- C1：receive_first 无最小长度预检（crypto lib.rs:561），风格与 resume 变体不一致。
- C3：source_key 含端口，NAT 行为待 WAN 证据（preauth.rs:517-535）。
- C4：TrustPolicy 授权线性扫且 records 无构造上限（crypto lib.rs:266-280）。
- A5：时间基准三分（Instant/u64 毫秒/recovery 微秒 epoch）。
- F2：preauth 清理路径 30+ 处 `let _ =` 静默（preauth.rs），排查态泄漏时成本高。
- F3：read_frame 无自身 deadline 依赖外层预置（main.rs:511-524）；SessionRuntime Closing 态 close 超时惰性结算（session lib.rs:1793-1799）。

## 代码层待办清单（按建议顺序）

1. **把 preauth 拒绝从 fail() 迁到"拒绝+计数+继续"**：以 failover_server L1984-1993 为模板改 main.rs `server`（TCP/UDP 两路）、periodic server、multistream；同步把"malformed negotiation/bad handshake/auth failure"单帧失败改为断开该连接而非退出进程。补一条"恶意输入不终止进程"的回归测试（用现有 `emit_diagnostic` 屏障机制计数）。
2. **给 PacketAckTracker 加 range 压力策略**：满 32 时（或合并后仍满）丢弃最老 range 并发 `RangeLimit` 计数事件，调用方告警而非 fail（carrier lib.rs:7644-7646、main.rs:2126-2131）。
3. **SessionRuntime.events 容量策略落地**：加 `max_events` 上限 + 淘汰/饱和计数（对齐 observe::Producer 的做法，observe lib.rs:402-459 是现成范本），并在超限时发 `resource.limit_hit` 类事件（session lib.rs:1844-1850）。
4. **拆 neko-carrier/src/lib.rs** 为 9 个模块（目录见 A1 清单），保持所有 pub API 与测试不动、纯移动 + re-export；先拆 ReliableUdpRuntime 与 ConcurrentCarrierManager（与 A2 联动）。
5. **把 ReliableUdpRuntime 的双 path 硬编码改为注入**（构造器收 path 键集合/字典），解锁 >2 path 与 ICMP carrier（carrier lib.rs:7769-7792）；同时给 preauth::CarrierKind 加第三类并修 source_key 长度断言（preauth.rs:519-522,543-559）。
6. **提取 preauth 握手会话组合体**（negotiation+noise 两帧、charge/permit/release 次序封装进 preauth.rs 或新模块），三处 server 改为调用它（main.rs:653-708、2364-2404、periodic.rs、multistream.rs:64-129）。
7. **key phase 多级化**：MAX_KEY_PHASE 提到 u8 全域或改计数型，加"按已发 nonce 数自动触发 update_key_phase"的入口（crypto lib.rs:206,669-683；periodic.rs:439,578 是手动触发点）。
8. **framed.rs 合并双读路径**（staged 版为主，普通版委托），消除 main/multistream 的 frame_read/frame_write 拷贝（framed.rs:64-203；main.rs:511-535；multistream.rs:199-218）。
9. **bind/port 参数统一**：一个 `bind_addr()` 解析器收编 6 处 f-string 校验与 40080-40100 白名单，支持 v4/v6 显式 family（main.rs:54,599；multistream.rs:260-270；periodic.rs;reachability.rs）。
10. **session lib.rs:1735 inflight 扣减改为单次 entry 原子化**（get 与 get_mut 之间消除窗口），顺手把 1677 的 back().unwrap() 换成直接引用刚 push 的记录。
11. **preauth 清理路径的 `let _ =` 收编为 debug 日志或断言**（至少对 abandon_response/release 的 Err 分支计数），提升态泄漏排查能力（preauth.rs:34-41,254,412-494）。
12. **receive_first 加最小长度预检**（对齐 with_resume 变体的 n≥ 下界检查，crypto lib.rs:561-568）。

---

### 审查方法说明（可信度边界）

- 生产/测试代码区分采用 brace 匹配剔除全部 `#[cfg(test)]`（含 `cfg(all(test,...))` 与文件中段散布的 13 个 test mod）；已用 reachability.rs（133 行测试起）与 pmtu_socket.rs 复核口径。
- 所有行号基于工作区当前文件（HEAD 46241a1 未有未提交改动的假定下 = 提交内容；`git status` 未运行以保持只读最小化——本报告所有读取均为 git 对象无关的直接文件读取）。
- "已确认"= 直接读码可见的行号证据；"推测"= 需要运行时/WAN/多进程实验才能定论的标注。
- 未运行 cargo（遵守约束），故未做编译级确认；所有计数为文本模式匹配，多行链式调用中 `.unwrap()` 归属行以出现行为准。
DONE

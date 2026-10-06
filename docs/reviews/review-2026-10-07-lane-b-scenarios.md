# Lane B — 场景可用性矩阵（nekomusume @ 46241a1）

日期：2026-10-07。方法：只读审查 docs/、crates/neko-cli/src/、artifacts/、scripts/；未跑任何构建或网络实测。四档定义：

- **能用** — 当前 HEAD 形态下，该场景在仓库声明的研究边界内可直接使用，且有实测证据。
- **有条件能用** — 机制存在且有至少一次实测（可能窄、旧、脚本化），或仅在硬性边界（端口/时长/字节数）内可用。
- **不能用** — 机制缺失或被治理明令禁止；不是"没测"，是"当前形态做不到"。
- **未验证** — 无任何证据，既没有否定也没有肯定（含环境不可得）。

时间戳规则：证据落在非 HEAD commit 的，一律标 `[commit 短哈希, 日期]`；确定性 fixture（in-memory，非 socket）标 `[fixture]`。

---

## 1. 网络场景

| # | 场景 | 档位 | 关键证据 | 缺什么升档 |
|---|---|---|---|---|
| N1 | loopback | **能用**（研究边界内） | `crates/neko-cli/src/main.rs:48-56` USAGE；`lab --scenario reliable-udp` 真实 loopback UDP socket pair（`main.rs:5768-5769`）；CI 绿（`docs/status.md:180-182`，GH Actions run 33755759414）；status.md cli 行 candidate（`docs/status.md:14`） | 无——这是设计边界而非缺陷。要"一般化可用"需解除 40080-40100 端口、1-1200B、30s 硬约束（`main.rs:486-505` `common()`） |
| N2 | 局域网（跨主机同网段） | **有条件能用** | live-udp：cross-host IPv4 UDP 交换 + 14/14 替换生命周期样本（`docs/status.md:18`，`docs/research/reviewer-primary-a-udp-lifecycle-20260901.md`）[2026-09-01，非 HEAD]；live-tcp cross-host + resume（`docs/status.md:27`）[同前]。**注意**：均为单交换 echo，且 server 单连接即退（`main.rs:710-727`） | HEAD 上的重复 cross-host 运行；多客户端 accept 循环（现在处理完一个连接就 `return`） |
| N3 | netns+tc 模拟（丢包/延迟/乱序/突发/带宽变化） | **有条件能用** — 但要拆两半 | ① PLPMTUD 数据面是真协议实测：`scripts/bench/run-plpmtud-local-matrix.sh`（3 netns + veth + router 转发 + size-selective netem + ICMP frag-needed 黑洞，`:61-109`；跑 `neko-cli client/server --plpmtud`，`:136,152`），48+18 cells，verdict `converged_exact`/`deadline_bounded`/零假成功（`artifacts/plpmtud-local-matrix/20260929-0dc938d/summary.md`）[0dc938d/0522e14, 2026-09-29，非 HEAD]。② 通用基准 `scripts/bench/run-netns.sh:27-50` 跑的是 **ping（ICMP）** 而非 nekomusume 协议——9 场景（delay/loss 1-10%/gemodel 突发/reorder/rate 10mbit/blackhole）全是 netem 对 ICMP echo 的测量；ledger N 行自认 "not Nekomusume performance"（`docs/status.md:264`）。③ failover/可靠 UDP 在 netns 下**零实测**——相关证据全是 in-memory `FaultInjectCarrier` fixture（`docs/status.md:282-313`，自述 "deterministic in-memory Carrier-seam fixtures... not network-topology models"）[fixture] | 用 netns+tc 对 `lab`/`failover`/`multistream` 数据面做 HEAD 实测；把 run-netns.sh 的被测对象从 ping 换成 neko-cli |
| N4 | 真实 WAN（HK↔VPS，已有 run） | **有条件能用**（单次、窄、自有环境） | HY2 fair pair 完成 run：HK(43.154.97.90)→23.147.120.24，RTT~213ms，10/10 样本 0 失败，neko median 2150ms / hy2 1100ms（`docs/notes/hy2-fair-pair-complete-7680a40-20261001.md` + `artifacts/hy2-owned-lab/7680a40-hy2-fair-pair-rerun/result.json.validated`）[374a6daf 二进制, 2026-10-01，二进制树 e328f14 是 HEAD 祖先——已验证 `git merge-base`]；PLPMTUD VPS run 6 会话全收敛 1500（`docs/notes/plpmtud-vps-run-374a6daf-20261001.md` + `artifacts/plpmtud-vps/374a6daf-hk23-run/A1-A3,B1-B3.out`）[同前] | 跨时段/跨路径重复；自然退化（非脚本触发）场景；时长 >30s/600s 的 WAN 稳定性 |
| N5 | 高 RTT（>200ms） | **有条件能用** | 213ms RTT 双向跑通（N4 证据）。曾修复两个 RTT 相关缺陷：server 第二个 `recv_from` 100ms poll 超时导致 >100ms RTT 握手失败（修复=按 session deadline 有界等待，`main.rs:767-772` 注释 + plpmtud note "Two run-enabling defects"）；收敛后 linger 超过 deadline（已修）。客户端 setup 上限 10s（`periodic.rs:13` `MAX_SETUP_TIMEOUT_MS=10000`） | 但 **D064 warm readiness 是 1s/请求、3s/全序列**（`main.rs:60-63` `READINESS_*`；`docs/status.md:80-85`）——RTT>1s 时 warm failover 必然 fail-closed，>300ms 时三连 challenge 勉强。需要 >300ms 抖动路径上的 readiness 实测 |
| N6 | UDP 被封只剩 TCP | **有条件能用** | TCP carrier 完整：cross-host negotiated/authenticated TCP + DeliveryAck controlled-stop resume（`docs/status.md:27`）[09-01，非 HEAD]；`client --transport tcp` 直接可用。**但**没有"UDP 被封"的自动检测——warm fallback 触发条件是 application-level reply cessation，status.md 自述 "not natural degradation/PTO-blackhole evidence"（`docs/status.md:14`） | 真实 UDP 黑洞（ICMP unreachable / PTO 黑洞）下的自动降级实测；目前降级靠脚本掐 reply |
| N7 | UDP 时断时续（抖动 failover） | **有条件能用**（实验性） | warm failover seam + D064 3-challenge promotion（`main.rs:3053-3063` `--automatic-health-failover`/`--cold-health-failover`）；受控 reply-cessation warm fallback 实测 434ms 恢复（25e0daa，`docs/status.md:133-137`）[25e0daa，非 HEAD]；migration-back 一次 VPS 实测 UDP→TCP→UDP 恢复（`docs/notes/migration-back-vps-5d6582c-20260908.md`）[5d6582c，非 HEAD]。**反面**：自然持续抖动（repeated-warm-failover 6-cycle WAN）所有尝试 0/6 cycles，全 BLOCKED_ORCHESTRATION（9fd2411/a117086/c6ab8fd/f17b648/4a2129e，`docs/status.md:228,236,241-252`）——收集器问题而非 runtime，但意味着**没有一个完整 WAN 抖动周期样本** | 一个完整的 WAN repeated-failover cycle；该线已被 freeze（"frozen unless a concrete material setup hypothesis emerges"，`docs/status.md:230`） |
| N8 | NAT 重绑定/源端口变化 | **有条件能用**（极窄） | 一次 VPS 实测：同 Session 源端口 0→1 代、9 步 authenticated promotion（`docs/notes/endpoint-rebind-vps-a8f49fc-20260908.md`，2 records/32B/10s）[a8f49fc，非 HEAD]；专用命令 `endpoint-rebind-server/client`（`main.rs:6585-6586`）。**边界**：无真实 NAT 设备（客户端换 bind 而已）；crypto `path_generation` 固定未迁移；note 自述 "not general NAT traversal... or roaming reliability" | 真实 NAT 后的重复观测；path_generation 加密迁移（当前 rebind 在固定 record context 下） |
| N9 | CGNAT/对称 NAT | **未验证** | 零证据。代码无 NAT keepalive/STUN/hole-punching（全库 grep 无）；对称 NAT 意味着出口映射每目标不同——比 N8 的源端口变化更严，且 N8 的 crypto context 未做 path_generation 迁移 | 全套：keepalive 机制、多映射路径下的 rebind 验证、真实 CGNAT 环境实测 |
| N10 | IPv6 | **未验证**（真实网络）/ **有条件能用**（本地矩阵） | 代码双栈：`--ip-version ipv6`（`main.rs:50`）、pmtu_socket 有 v6 分支（`pmtu_socket.rs:31-40`）；本地 netns 矩阵 12+ v6 cells 跑完（`artifacts/plpmtud-local-matrix/20260929-0dc938d/summary.md`，v6-rc1298-l0-short verdict `handshake_or_exchange_failed` exit 2，其余 `client_exit_0`——即 v6 短会话握手曾失败 1 cell）[非 HEAD]。真实 v6：`docs/status.md:226` "BLOCKED_ENVIRONMENT: no real owned IPv6 endpoint/path"；`:171` "IPv6 remains environment-blocked" | 拿到 owned IPv6 endpoint 跑一次真实验 |
| N11 | 小 MTU/PMTU 黑洞（隧道套隧道） | **有条件能用** | 本地：48+18 cells 真协议实测（router ICMP frag-needed 黑洞 + size-selective 丢包 + DF 控制），1400/1500 `converged_exact`、1278 黑洞下 `client_exit_0` 14/0/13、零假成功（`artifacts/plpmtud-local-matrix/.../summary.md`）[非 HEAD]；black-hole fallback + live shrinkage 1500→1300 本地验证（`docs/status.md:267`，e5fd13e gate 758 passed）；EMSGSIZE below-base 一步判低（`main.rs:1279-1291` + `pmtu_socket.rs:63+` kernel_path_mtu）。VPS：6 会话全 1500（无黑洞路径）。HK survey：c4 small-MTU overlay 链路恰逢 outage，**真实小 MTU 黑洞 WAN 样本为零**（`docs/status.md:225` 自述 "no live small-MTU sample exists yet"；c4 各会话 `negotiation response failed`，`artifacts/plpmtud-hk-survey/hk-survey-20260930A/cells-summary.json`） | 一条真实小 MTU WAN 路径上的收敛/黑墙降级样本；probe 频率/token-bucket 策略值仍开放未定（`docs/status.md:225`） |
| N12 | 强 QoS/UDP 限速 | **未验证** | 零证据。无 DSCP/ECN/IP_TOS 标记代码（grep 无）；无 QoS 环境实测；吞吐本身未被测过（见 U2） | 先有吞吐能力（U2 升档），再在 QoS 环境实测 |
| N13 | 移动网络切换（wifi↔4G） | **未验证** | 机制碎片存在：源地址变化 ≈ N8 rebind（但仅脚本化单次）；TCP 断连恢复需 failover_client 的 ResumeGuard 路径（`main.rs:2411-2424`），普通 `client` 命令无 resume；periodic 明示 `reconnect=unsupported`（`periodic.rs:287`）。无 interface 监听、无地址变化自动检测，端到端切换从未测过 | 地址变化自动检测 + roaming 端到端实测；先决条件是 N8 从脚本化变成自动 |

## 2. 使用场景

| # | 场景 | 档位 | 关键证据 | 缺什么升档 |
|---|---|---|---|---|
| U1 | 长连接（小时级） | **不能用**（当前形态） | 硬上限：普通 server/client 30s（`main.rs:496-501` `MAX_DURATION=30`）；periodic 600s + 1MiB 总字节（`periodic.rs:8-13` `MAX_WORKLOAD_DURATION=600`、`MAX_TOTAL_BYTES=1MiB`）；最长实测样本 ≈5 分钟 periodic（`docs/status.md:138-140`，60 records 32B）[25e0daa，非 HEAD] | 解除时长/字节预算；小时级 soak + 内存/句柄泄漏检查（无任何小时级数据） |
| U2 | 大流量吞吐 | **不能用** | 任意命令单次运行应用数据 ≤~1MiB：periodic 1MiB（`periodic.rs:9`）、multistream 16×64×1024B=1MiB（`multistream.rs:14-16,230`）、failover 64 交换×~1176B（`main.rs:49-56`）；Reno/pacing 模型只有确定性测试，**真实 socket 上的可靠 UDP 吞吐从未测过**（`lab` 是 loopback fixture、rounds≤64，`main.rs:5493-5499`）；唯一"吞吐"数字是 HY2 对比的 6000B/样本 | bulk 模式（解除 per-run 预算）+ 实际 socket 吞吐基准 |
| U3 | 多 stream 并发、大流+交互小流 | **有条件能用**（loopback fixture 层面） | `multistream` 命令：1-16 streams/64 records/1024B，loopback TCP pair（`multistream.rs:14-16,182-192,267,443-444`）；Carrier Manager fair round-robin + health score + hysteresis + migration-back gate 测试（`docs/status.md:26`，`crates/neko-carrier/src/lib.rs`）[测试为 fixture]；`scheduler-fairness` 交互流 burst≤3 公平性 fixture（`main.rs:6260`，maturity "fixture"）。跨主机多 stream **零实测**；大流+交互只有 in-process fixture | WAN 多 stream 实测 + 公平性度量；multistream 从 in-process loopback 变成真跨机 |
| U4 | 作为代理/隧道承载真实应用（SOCKS/TUN） | **不能用** — 入口根本不存在 | CLI 明示 "no proxy/tunnel behavior"（`main.rs:48` USAGE 尾行）；m5 门 "It performs no forwarding, routing, proxying or tunnel behavior"（`docs/spec/m5-release-readiness-gate.md:57`）；子命令全表（`main.rs:6574-6593` server/client/probe/health-observe/failover/multistream/scheduler-fairness/key-update/periodic-*/lab/workload/endpoint-rebind-*/keygen/capabilities）无任何代理/TUN/SOCKS 入口；全库无 tun/tap 设备代码。这是"能不能用"的最硬边界：**协议研究密度高，产品入口为零** | 整个转发数据面 + SOCKS/TUN front-end（大工程，当前 roadmap 无此项）；治理上 production 也 blocked（`docs/status.md:29`） |
| U5 | 多客户端并发连同一服务端 | **不能用** | `server` TCP 分支处理完一个连接的 count 个交换后打印 STOPPED 并 `return`（`main.rs:710-727`）；`periodic-server` accept 一个就 `break`（`periodic.rs:301-304` `accepted = Some(...); break`）；UDP 分支同样单会话（`main.rs:731+`）。设计即单会话；preauth admission 是逐连接记账而非并发管理 | 连接循环 + 并发会话状态管理（当前 session-model 是 in-memory 单会话，`docs/status.md:12`） |
| U6 | 服务端公网暴露（preauth/DoS） | **不能用**（治理禁止 + 工程未证明） | 治理：reachability/production 行 blocked（`docs/status.md:28-29`）；m5 门整节 blocked 声明（`m5-release-readiness-gate.md:5-52`）。工程：preauth budget/charge/expire 机制存在（`preauth.rs:213-229`），7 个 responder surface / 6 个 admission site 已做工程审查（RSEC-001，`docs/status.md:268`），但自述 "Adversarial-load suitability remains open"；固定 40080-40100 端口恰好收敛 DoS 面 | 对抗性负载测试；独立安全审查；D019 终态源保留策略（自述 "SOURCE_RETENTION_POLICY_BLOCKED"）；治理放行 |
| U7a | 客户端重启后恢复（服务端存活） | **有条件能用**（窄） | TCP resume：exact-semantic authenticated DeliveryAck controlled-stop resume，cross-host 实测（`docs/status.md:27`）[09-01，非 HEAD]；failover_client 的 ResumeGuard `attach_with_negotiation`（`main.rs:2411-2424,1965`）。均为受控脚本 stop，非 crash | crash（非优雅停止）后的恢复；恢复路径对普通 `client` 命令不可用（只有 failover/periodic 专用路径有 resume） |
| U7b | 服务端重启后恢复 | **不能用** | Session 状态是 in-memory（`docs/status.md:12` "in-memory Session delivery state"）；服务端进程死 = resume 状态丢失，客户端 ResumeGuard 无处 attach；persistence/rollback 明确缺失（0-RTT 门行 "pending replay-safe resumption, **persistence/rollback**" `docs/status.md:20`）。package lifecycle 实测只证明同端口重启后**新**会话可用（`docs/package-operator-lifecycle-7cebe6b-20260909.md`）[7cebe6b，非 HEAD]，不证明旧会话恢复 | 服务端状态持久化（先于任何 resume 跨重启的声明） |
| U8 | 跨版本互通 | **有条件能用**（窄 A/B/A） | distinct-version A/B/A 包演练：91a735c→dc90c5f→91a735c，三阶段各 1 次认证 32B TCP 交换 ok，5 records（`docs/package-operator-distinct-version-91a735c-dc90c5f-20260909.md` + `.result.jsonl`）[09-09，非 HEAD]。**边界**：`SUPPORTED_VERSIONS = &[NEGOTIATION_VERSION]` 单协议版本（`main.rs:55`）——"协商"只是确认唯一版本，没有真正的版本分叉；A/B 是二进制/包差异非协议版本差异；ledger B 行自述 "does not establish previous/current interoperability because no prior frozen release exists"（`docs/status.md:264`） | 协议版本真分叉 + 旧客户端×新服务端矩阵 + 冻结版本基线 |

## 3. 平台

| # | 场景 | 档位 | 关键证据 | 缺什么升档 |
|---|---|---|---|---|
| P1 | Linux 以外（macOS/Windows/Android）编译运行 | **未验证**（倾向"可编译、部分功能降级"） | CI 只有 `ubuntu-latest`（`.github/workflows/`，grep runs-on 仅 ubuntu）；无 cross-compile target/产物。代码推断：核心数据面用 std::net（跨平台）；unix-gated 块有 `#[cfg(unix)]`（identity 文件权限 `main.rs:404-470`——macOS 属 unix，Windows 跳过检查但仍编译）；`signal-hook 0.3` 跨平台；`rustix` 的使用全在 `#[cfg(target_os = "linux")]` 函数体内（`pmtu_socket.rs:23,31,63`），Windows 下不引用其 net 模块 | 任一非 Linux 平台的编译+冒烟证据。已知行为差异（代码层面推断，非实测）：① 非 Linux `--plpmtud` 直接 fail-closed `Unsupported`（`pmtu_socket.rs:56-62`）；② probe `--json` 读 `/proc/self/fd`，非 Linux 上 `fail("benchmark FD inventory unavailable")`（`main.rs:275-277`）；③ Windows 无 0600/O_NOFOLLOW 强制（静默降级） |
| P2 | 依赖 Linux 专有 API 的位置（清单） | — | ① `IP(V6)_PMTUDISC_PROBE` DF 探测（rustix，Linux-only，`pmtu_socket.rs:23-40`）；② 内核 PMTU cache 读取 `ip_mtu/ipv6_mtu`（`pmtu_socket.rs:63+`，Linux-only）；③ `/proc/self/fd` fd 清点（`main.rs:275`）；④ netns/veth/netem 全家脚本 `sudo ip netns`（`scripts/bench/run-netns.sh:17-28`、`run-plpmtud-local-matrix.sh:61-109`，Linux-only）；⑤ `libc::O_NOFOLLOW/O_CLOEXEC` + mode 0600（unix 家族，`main.rs:404-470`）；⑥ `libc::EMSGSIZE` 判断（跨平台常量，`main.rs:1279`）。**核心协议数据面无 Linux 专有依赖**——专有的都在实验工具链和 PLPMTUD | P1 的编译验证即主要缺口 |

---

## 4. status.md 内部矛盾清单

同一文件内三个证据世代（09-04、09-30、10-01）共存，最新事实与未更新的旧行直接并列：

1. **HY2 fair pair 三态并存**（最严重）：
   - `docs/status.md:227`（Era-4 节）：`BLOCKED_IMPLEMENTATION`，"There remains **no complete pair**, summary, or performance claim"，"the next fair-pair attempt ... is the changed-hypothesis invocation"（未来时）
   - `docs/status.md:34-40`（2026-10-01 段）："The changed-hypothesis VPS rerun is **deferred**: vps-104 has expired ... this line waits for the administrator to provide a new VPS"
   - `docs/status.md:269`（closure index）："**HY2 fair pair is no longer blocked**: the changed-hypothesis rerun **completed 2026-10-01** ... complete validated pair — 10/10 samples, 0 failures, hy2 median 1100 ms / nekomusume 2150 ms"
   - `docs/status.md:32`（顶部 harness 块）："There are no complete pairs or comparative summary"（描述 3d54585 时点，但无 superseded 标注）
   读者按顺序读会得到"等新 VPS"（L34）→"已完成出数据"（L269）→ 又回头看到"BLOCKED、无 complete pair"（L227）。L269 是最新事实；L227/L34 是历史层未标注。

2. **PLPMTUD（R）分类漂移**：
   - `docs/status.md:225`（live PMTUD 行）：`PARTIAL_EVIDENCE_SELF_OWNED`，"no live small-MTU sample exists yet"，且 "probe-frequency/token-bucket policy values remain undecided"
   - `docs/status.md:267`："R (PLPMTUD integration) **completed** its local implementation queue at e5fd13e"
   - `docs/status.md:269`："**R is no longer environment-blocked**: owned-VPS run completed 2026-10-01 ... all converged confirmed_mtu=1500"
   不完全互斥（1500-MTU 路径 ≠ 小 MTU 样本），但 L225 的行级分类没有随 10-01 run 升档或注明"部分由 L269 取代"，两行的 missing-piece 清单与完成声明需要读者自行调和。

3. **ledger JSON 与正文脱节**：status.md L262 宣称 "every Era-4 ledger row has one closed classification in `docs/era4-ledger-2026-08-30.json`"，但该 JSON 冻结在 09-04 世代——实测读取：`Q=BLOCKED_ORCHESTRATION_CURRENT_LINE`、`R=BLOCKED_IMPLEMENTATION`、无 HY2 完成记录。正文 L266-267 却写 "`BLOCKED_IMPLEMENTATION`: none"、L269 写 HY2/R 已完成。被引为权威分类来源的 JSON 与正文直接不一致。

4. **vps-104 时间线纠缠**：L34-40 说"vps-104 已过期、不得使用、等待管理员提供新 VPS、在此之前不再调用 VPS"——而 L269 记录的 10-01 完成跑用的是新机 23.147.120.24（`docs/notes/hy2-fair-pair-complete-7680a40-20261001.md` LAB_SSH_TARGET=neko23）。L34 段写作时点早于新机到位，但"deferred/waiting"段落与"completed same day"段落之间无任何过渡说明；status.md 内 vps-104 时代的历史 run（f1cb9af/bc38d06/3d54585）与退役事实之间的可复现性断裂也未集中标注。

5. **"blocked" 术语自冲突**：L28 reachability 行——"public/general reachability evidence ... remain blocked; **no public listener**"，而 L269 记录的 run 走的是真实公网路径（公网 IP、跨网出口、ufw 临时放行）。边界在术语上是自洽的（self-owned ≠ public/general，临时实验 listener ≠ 公开服务），但"公网路径上已有完成证据"与"public reachability evidence remains blocked"并列，靠一句嵌入式限定语区分，极易误读为矛盾。

6. **跨文件不同步（附带）**：`ROADMAP.md` 尾部 "2026-09-04 final current-line" 段仍写 "HY2 is BLOCKED_HARNESS_CURRENT_LINE ... no complete pair or comparison exists"、"NAT/source change and live PMTUD **stay BLOCKED_IMPLEMENTATION**"——与 status.md L222（NAT `ALREADY_SUFFICIENT_FOR_BOUNDED_QUESTION` @ a8f49fc，09-08 即已成立）和 L269（HY2 完成、PMTUD run 完成）矛盾。ROADMAP 停在 09-04，status.md 走到 10-01，两份"权威叙述"相差一个月。

7. **IMPLEMENTATION_COMPLETE=true vs 各行 blocked/candidate**（L60 vs L28-29）：文件自己反复声明这两个不矛盾（L67-72 专门解释），属"已自我说明的表面冲突"，列出仅为提示读者该向量不构成可用性结论。

---

## 5. 证据时效汇总（旧 commit / fixture 而非 HEAD 实测的格子）

| 格子 | 证据时点 | 性质 |
|---|---|---|
| N2 局域网 | 2026-09-01（f19ad28 前后） | 旧 commit 上的自有 VPS 实测 |
| N3 ① | 2026-09-29（0dc938d/0522e14） | 非 HEAD 的 netns 实测（真协议） |
| N3 ③ | — | 纯 in-memory fixture，非网络 |
| N4/N5 | 2026-10-01（374a6daf 二进制，e328f14 树，HEAD 祖先已验证） | 真实 WAN 实测，非 HEAD 精确树 |
| N7 | 25e0daa(09-03)/5d6582c(09-08)/a8f49fc(09-08) | 旧 commit 受控实测 |
| N8 | 2026-09-08（a8f49fc） | 旧 commit 单次脚本化实测 |
| N10 | 2026-09-29 本地矩阵 | 非 HEAD；真实 v6 零样本 |
| N11 | 本地 09-29 / VPS 10-01 / HK survey 09-30 | 混合；真实小 MTU 黑洞始终为零 |
| U7a | 2026-09-01 | 旧 commit 实测 |
| U7b | 7cebe6b 2026-09-09 | 只证同端口重启新会话 |
| U8 | 2026-09-09（91a735c/dc90c5f） | 旧 commit，且是二进制非协议版本差异 |

**通用时点声明**：docs/notes/* 的 check-gate 系列记录了各旧 commit 的全量门（最近 758 passed @ 800eb98/7680a40 树，HEAD 提交信息亦载明 "gate green at 800eb98"），说明代码演进有门禁背书，但本矩阵中几乎所有"实测"都落在非 HEAD commit 上——HEAD 本身（46241a1）没有任何直接附着的网络实测 artifact。

---

## 6. 总结判断

- **研究/实验范围内**：loopback 能用；WAN 单次窄场景（echo、PLPMTUD、rebind、migration-back）有真实证据但全部窄、旧、一次性。
- **"当通用传输用"**：当前形态做不到——没有代理/隧道入口（U4）、单客户端单会话（U5）、30s-600s/1MiB 硬预算（U1/U2）、无服务端持久化（U7b）。这不是测试欠账，是功能面缺失。
- **status.md 的可用性信号必须按"世代"读**：L269 closure index 是最新事实层，L222-228 各行和 ROADMAP 是历史层；ledger JSON 已过期一个月。任何人基于 status.md 做决策，应先解决第 4 节列出的矛盾标注问题。

DONE

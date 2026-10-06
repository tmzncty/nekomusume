# 猫娘协议（nekomusume）代码审查综合报告 — 2026-10-07

- 审查对象：`main` @ `46241a1`（与 origin/main 一致）
- 团队：佩丽卡（Overseer，综合 + Lane C 收尾）、赛希（Lane A 核心代码）、伊冯（Lane B 场景可用性）、艾尔黛拉（Lane C 测试缺口，盘点与基线后因模型服务中断）
- 分线报告（同目录）：
  - [`review-2026-10-07-lane-a-core.md`](review-2026-10-07-lane-a-core.md)
  - [`review-2026-10-07-lane-b-scenarios.md`](review-2026-10-07-lane-b-scenarios.md)
  - [`review-2026-10-07-lane-c-tests.md`](review-2026-10-07-lane-c-tests.md)
- 审查方式：全程只读；测试在仓库外 `CARGO_TARGET_DIR` 实跑；不涉网络/VPS/sudo。本提交只新增 `docs/reviews/` 下四份文档，不改代码、不改 status/ROADMAP。
- 佩丽卡抽查过的证据：`crates/neko-cli/src/main.rs:645-647`（TCP admission → `fail`）对照 `:1984-1993`（UDP reject+continue）、`periodic.rs:297-299`、`neko-session/src/lib.rs:1844-1850`、`main.rs:709-722`（单连接后 return）、`scripts/bench/run-netns.sh:31`（ping）、`main.rs:1,53`（never a proxy or tunnel）。全部属实。

## 1. 一句话结论

猫娘是一个**做得很扎实的研究原型**：协议核心库（wire/crypto/reliable/session）有界、无 panic 隐患、单元测试质量高，758 个测试全绿；但它**目前只是一个有界的认证探测工具，不能当代理/隧道用**，服务端有一个**未认证即可远程杀进程的 P0**，而真实网络证据都是旧 commit 上的单次窄样本。

## 2. 场景可用性矩阵（摘自 Lane B，30 格）

| 档位 | 场景 |
|---|---|
| **能用** | loopback（研究边界内） |
| **有条件能用** | 局域网；netns+tc 损伤（只有 PLPMTUD 矩阵跑的是真协议，通用矩阵测的是 ping）；真实 WAN HK↔VPS（单次 10/10）；高 RTT ~213 ms；只剩 TCP；UDP 时断时续（受控 warm failover）；NAT 源端口变化（单次）；小 MTU/PMTU 黑洞（本地 + 1500 路径，真实小 MTU 样本为零）；多 stream（loopback fixture）；客户端重启后 TCP resume；跨版本 A/B/A（窄） |
| **不能用** | 小时级长连接（30 s / 600 s + 1 MiB 硬上限）；大流量吞吐；作为代理/隧道（无 SOCKS/TUN 入口）；多客户端并发；服务端公网暴露；服务端重启后恢复（状态仅内存） |
| **未验证** | CGNAT/对称 NAT；QoS/UDP 限速；移动网络切换；真实 IPv6；Linux 以外平台（CI 只有 ubuntu，`--plpmtud` 非 Linux fail-closed） |

所有"有条件能用"格的证据**都不在 HEAD 精确树上**（见 Lane B §5 时效表）。

## 3. 关键发现（合并去重）

| 级别 | 发现 | 证据 | 来源 |
|---|---|---|---|
| **P0** | TCP 服务端 preauth admission 被拒即 `fail()` 退出；伪造 9 个源（每源上限 8）必中 | `main.rs:645-647`、`periodic.rs:297-299`、`multistream.rs:294`；正确范式 `main.rs:1984-1993` | A |
| **P0** | 该拒绝路径在 CLI 层零测试 | `crates/neko-cli/tests/` grep 无命中；仅库层 `preauth.rs:823-830` | C |
| P1 | `SessionRuntime.events` 无界增长 | `neko-session/src/lib.rs:1844-1850`（observe 层有 `MAX_EVENTS=1024` 可借鉴） | A/C |
| P1 | ACK tracker 32 range 用尽 → 整个 server 失败；高丢包乱序 WAN 可自然触发 | `neko-carrier/src/lib.rs:7623,7644-7646` → `main.rs:2126-2131` | A/C |
| P1 | carrier 单文件 10,903 行 + UDP/TCP 双 path 硬编码，挡多路径/ICMP | `neko-carrier/src/lib.rs:7770-7792` | A |
| P1 | 三个 server 各自复制 preauth 握手编排（P0 的结构根源） | `main.rs:653-708,2364-2404`、`periodic.rs`、`multistream.rs:64-129` | A |
| P1 | fuzz 只有 1 个 target，未覆盖 `decode_frames`、`open_*`、`ResumeBinding::decode` | `fuzz/fuzz_targets/decode.rs`；`neko-wire/src/lib.rs:152`、`neko-crypto/src/lib.rs:608-707,3149` | C |
| P1 | `docs/status.md` 内部 7 处矛盾，HY2 同文件三态并存；ledger JSON 停在 09-04；ROADMAP 落后约一个月 | `status.md:32,34,227,269`、`docs/era4-ledger-2026-08-30.json` | B |
| P2 | key phase 只能升一级 | `neko-crypto/src/lib.rs:206,670` | A |
| P2 | probe.rs 单 binary 241 s、12 处固定 sleep | `crates/neko-cli/tests/probe.rs:6861,7670,7730,8782` | C |
| P2 | 覆盖率工具在本机不可用：rustup `llvm-profdata`（LLVM 22.1.8）SIGILL | Lane C §1 | C |
| 正面 | 库层零 panic 级隐患；preauth 三层上限完备；nonce 不回绕；replay 在 AEAD 后推进；放大 ≤3x；RSEC-001 代码无泄漏（策略仍待定） | Lane A §4-5 | A |

## 4. 未来待办清单（与 ROADMAP / status 对齐）

按"先止血、再补测、再扩能力"的顺序。每项标注它在 ROADMAP/status 中的对应位置；**"新"表示 ROADMAP/status 里目前没有**。

### 阶段 1 — 止血（代码，1–2 个切片）

1. **修 P0：preauth 拒绝 → 拒绝+计数+继续**，并把单连接的握手/认证失败改为"断开该连接"而非退出进程；先写红测试（Lane C T0-1/T0-2）。〔新；与 status RSEC-001 行的 adversarial-load 未决项相关〕
2. **ACK range 满时降级**（丢最老 range + 计数事件），不再整体 fail。〔新〕
3. **`SessionRuntime.events` 加容量上限**。〔新〕
4. **提取 preauth 握手会话组合体**，三个 server 统一调用（防止 P0 复发）。〔新〕

### 阶段 2 — 补测（Lane C P0/P1）

5. 新增 3–4 个 fuzz target（frames、record open、resume binding、preauth 输入），fuzz smoke 接入本地 `scripts/check.sh` 可选步骤。〔ROADMAP M0 "fuzz" 已勾选，但只覆盖 decode 面〕
6. ACK range 压力、events 上界、二次 key update、failover 运行时抖动的集成测试。〔ROADMAP M4 hysteresis 已勾选，但只有门控层证据〕
7. 冻结 `46241a1` 的 golden 握手/record 字节作为未来 previous-version 向量。〔`neko-wire/tests/n3_compatibility.rs:3-5` 明说推迟〕
8. 用户态 UDP 损伤代理（无需 sudo），让真协议在 `cargo test` 里跑丢包/延迟/乱序/MTU 矩阵；替换通用 netns 矩阵中的 ping。〔ROADMAP M5"可控环境"全部已勾选——**勾选依据需要复核**，见 §5〕
9. probe.rs 去固定 sleep、拆分；修覆盖率工具链。〔新〕

### 阶段 3 — 文档治理

10. **修 `docs/status.md` 7 处矛盾**：给 L227/L34/L32 加 superseded 标注指向 L269；L225 PLPMTUD 行级分类随 10-01 run 更新；更新或明确作废 ledger JSON；ROADMAP 尾部 09-04 段同步。〔status/ROADMAP 自身〕
11. ROADMAP M1/M5 未勾项（真实 WAN failover/long-lived/NAT、UDP 退化 TCP fallback、长连接稳定性、HY2 对比）按 L269 最新事实重写状态。〔ROADMAP M1、M5 真实环境〕

### 阶段 4 — 能力扩展（决定项目是否走向"可用"）

12. **决定是否引入应用入口**（SOCKS5 / TUN）。这是"能不能当隧道用"的根本问题，当前设计明确禁止（`main.rs:1,53`），需要管理员做方向决策。〔README"不把实验版本直接替换生产隧道"；新〕
13. 去掉时长/字节硬上限的长驻 server 模式 + 多客户端多会话。〔新；挡 U1/U2/U5〕
14. 拆 `neko-carrier/src/lib.rs`，path 拓扑改为注入，为 >2 path、ICMP carrier 铺路。〔ROADMAP Experimental Track A/B〕
15. key phase 多级化 + 按 nonce 用量自动 rekey。〔新〕
16. 服务端状态持久化 / 重启恢复。〔新；U7b〕
17. 真实环境补样本：IPv6、真实小 MTU 黑洞、CGNAT、QoS、移动切换、小时级 soak。〔status L225/L226；ROADMAP M5〕
18. 非 Linux：CI 加 macOS/Windows `cargo check`。〔新〕
19. 治理项（维持现状、等管理员）：RSEC-001 source retention 策略（D019）owner 未指定；release/RC/production gate。〔status L268；m5-release-readiness-gate〕
20. ROADMAP Track A 剩余 reachability（ICMP、SCTP、DCCP、GRE、ESP、Raw IP 253/254）与 Track B 其他 carrier。〔ROADMAP Track A/B，原样保留〕

## 5. 需要管理员决定的事

1. **是否授权修 P0**（阶段 1），以及由谁接——原主线会话 75590 最熟悉代码。
2. **项目方向**：保持"有界研究探测器"，还是开始做 SOCKS/TUN 入口与长驻服务端（阶段 4 第 12–13 项）。这决定了矩阵中"不能用"那一栏是"设计如此"还是"待办"。
3. **RSEC-001 owner**（75590 已上报，仍悬空）。
4. ROADMAP M5"可控环境"7 项已勾选，但通用 netns 矩阵实际测的是 `ping`；是否要求按 Lane C T2-4 重做并取消勾选，需要裁定。

DONE

# Lane C — 猫娘协议测试缺口审查

- 仓库：`/media/tmzn/DATA5/nekomusume-work`，HEAD `46241a1`（工作区全程干净，`git status --short` 为空）
- 执行：艾尔黛拉（会话 C）完成盘点与基线后，因模型服务商订阅失效中断（Provider 400 `InvalidSubscription`）；其余部分由佩丽卡（12968）接手完成。
- 方法：外部 `CARGO_TARGET_DIR=/media/tmzn/DATA5/neko-review-20261007/target` 实跑；未改仓库、未碰网络/VPS/sudo。

## 1. 基线（实跑）

| 项 | 结果 |
|---|---|
| `cargo test --workspace -j 24` | **758 passed / 0 failed / 0 ignored**（与 gate `46241a1` 一致），日志 `test-baseline.log` |
| 最慢 binary | `neko-cli/tests/probe.rs`：78 测试 **241.3 s**；其余均 < 6 s |
| `#[ignore]` | 全仓 0 |
| 需 root/netns 的 Rust 测试 | 0（netns 只出现在 `scripts/bench/*.sh` 的 `sudo -n` 路径，不进 `cargo test`） |
| 覆盖率 `cargo llvm-cov` | **未能出数**：758 测试在插桩下全部通过，但 rustup 工具链自带 `llvm-profdata`（rustc 1.98 / LLVM 22.1.8）merge 时 **SIGILL core dump**（AMD EPYC 7F72），系统 `llvm-profdata-19` 不认 LLVM 22 profraw（`no profile can be merged`）。629 个 profraw 保留在 `target/llvm-cov-target/`。下面的覆盖判断改用"生产入口是否被测试引用"的 grep 法估计。|

> 附带发现：rustup 的 `llvm-tools` 在本机 CPU 上 SIGILL，值得单独记录（可能是工具链构建用了 Zen2 不支持的指令集）。若要覆盖率，需换 nightly/旧版 llvm-tools 或用 LLVM 22 的系统包。

## 2. 现有测试分布（`#[test]` 计数 / `thread::sleep` 数）

| 位置 | tests | sleep | 备注 |
|---|---:|---:|---|
| neko-carrier/src/lib.rs（13 个内嵌 `cfg(test)`） | 198 | 2 | 单元为主，覆盖面最广 |
| neko-session/src/lib.rs | 80 | 0 | 纯逻辑，确定性 |
| neko-cli/tests/probe.rs | 78 | 12 | 进程级集成，最慢、最依赖时序 |
| neko-crypto/src/lib.rs | 76 | 0 | nonce/replay/preauth admission 很全 |
| neko-reliable/src/lib.rs | 64 | 0 | |
| neko-wire/src/lib.rs + tests | 29 + 8 | 0 | golden vectors、N3 兼容 |
| neko-observe | 26 | 0 | |
| neko-cli/src/*（preauth 21、main 20、framed 15、reachability 10…） | ~98 | 14 | |
| neko-carrier/tests/*（15 个文件） | ~45 | 6 | |
| property 风格 | 3 个文件（era4_g_*） | | 无 proptest/quickcheck 依赖，手写枚举 |
| fuzz | **1 个 target**（`fuzz/fuzz_targets/decode.rs`） | | 覆盖 `decode`/`ProcessMessage::decode`/`VersionNegotiator::server_accept_hello`/`decode_ack`/`decode_packet`；CI 每次 30 s（`.github/workflows/ci.yml:35-36`），**不在 `scripts/check.sh` 里** |

整体判断：**库层单元测试质量高**（错误分支、上限 max+1、回滚、clock 回拨都有专门用例，如 `neko-crypto/src/lib.rs:2488`、`:2775`、`:2820`、`:3456`）。缺口集中在 **CLI 服务端层、对抗输入的 fuzz 广度、长时/并发**。

## 3. 缺失测试清单

### P0 — 应立即补（对应已确认缺陷或攻击面）

| # | 目标代码 | 缺什么 | 类型 | 为什么 | 工作量 |
|---|---|---|---|---|---|
| T0-1 | `neko-cli/src/main.rs:645-647`、`periodic.rs:297-299`、`multistream.rs:294` TCP admission 拒绝 → `fail()` | **9+ 个并发 TCP 连接（超过每源 8 state）后服务端仍在线、仍能服务合法客户端**的进程级测试。现有 `preauth.rs:823-830` 只测到库层 `admit_carrier` 返回 Err，CLI 层对拒绝的处理零测试（grep `admission rejected` 在 `crates/neko-cli/tests/` 中 0 命中） | 集成（loopback 进程） | Lane A 的 P0：未认证远端可一键杀死监听。先写失败测试再修 | S |
| T0-2 | 同上，UDP 路径 `main.rs:1984-1993` | 回归测试锁住"拒绝+continue"行为，防止将来复制粘贴回 `fail()` | 集成 | 修 T0-1 时以 UDP 为模板，需要两边对称断言 | S |
| T0-3 | `neko-wire/src/lib.rs:152` `decode_frames`、`neko-crypto/src/lib.rs:608/634/653/707` `open_*`、`:3149` `ResumeBinding::decode`（及 `:533` 调用点） | **新增 fuzz targets**：frames、加密 record open（固定密钥，随机密文/长度/key_phase）、resume binding、preauth 输入记录。目前 fuzz 只打了 wire 头和 ack | fuzz | 这些都是 peer 可控输入；open_* 在 replay/nonce 状态上有副作用，需要"拒绝不改状态"的 oracle | M |
| T0-4 | fuzz 进本地门禁 | `scripts/check.sh` 不跑 fuzz；只有 GitHub CI nightly 跑 30 s。加 `scripts/fuzz-smoke.sh` 的可选本地步骤并保存 corpus | 流程 | README 宣称"本地验证是主路径"，但 fuzz 实际只在复核门禁上 | S |

### P1 — 下一阶段（长驻、并发、资源上限触顶）

| # | 目标代码 | 缺什么 | 类型 | 为什么 | 工作量 |
|---|---|---|---|---|---|
| T1-1 | `neko-session/src/lib.rs:1844-1850` `events.push` 无界 | 长跑测试：10^6 次事件后内存/`events().count()` 有上界（现在会失败——先作为 `should_fail` 记录，修复后转正） | 单元 + soak | Lane A P1；observe 层有 `MAX_EVENTS=1024`（`neko-observe/src/lib.rs:11`），session 层没有 | S |
| T1-2 | `neko-carrier/src/lib.rs:7623,7644-7646` ACK tracker 32 range 上限 → `RangeLimit` → `main.rs:2126-2131` 整个 server fail | 端到端：乱序/丢包制造 >32 个不连续 gap，断言 server **降级或丢弃最老 range** 而非退出。现有 `:8751-8762` 只验证"恰好 32 个填满"的单元边界 | 集成（确定性 lossy socket） | 真实 WAN 高丢包+乱序可自然触发，导致非恶意断连 | M |
| T1-3 | 多客户端 | 同一 server 两个以上客户端并发/先后连接（当前 server 一条连接后 `return`，`main.rs:709-722`）——先写"预期行为"测试作为后续多会话改造的验收 | 集成 | 场景矩阵中"多客户端"为不能用；没有测试定义目标行为 | M |
| T1-4 | soak | 全仓无 soak/长连接测试（grep `soak|long_running|hours` 0 命中）。至少一个 opt-in（`NEKO_SOAK=1`）的 1 小时 loopback 周期会话，检查 RSS、fd、事件数、nonce 余量 | soak | 长连接只在 VPS 有 1 个约 5 分钟样本 | M |
| T1-5 | key update 多级 | `MAX_KEY_PHASE=1`（`neko-crypto/src/lib.rs:206,670`）：测试锁定"第二次 rekey 的确定行为"（拒绝 vs 关闭会话），以及 phase 用尽后 nonce 耗尽的终止路径。现有 `:3079` 只测一次 | 单元 | 长连接必然走到这里 | S |
| T1-6 | failover 抖动 | 现有 hysteresis 测试都是单元/门控（`carrier/src/lib.rs:403,843,5477`、`tests/integration_gates.rs:164`）。缺**运行时**UDP 交替 up/down N 次的集成：切换次数有上界、无重复投递、无数据丢失 | 集成（确定性 fault socket） | 场景"UDP 时断时续"目前只有门控层证据 | M |
| T1-7 | 跨版本 | `neko-wire/tests/n3_compatibility.rs:3-5` 明说 previous/current 推迟。建议现在就冻结一份 `46241a1` 的 golden 握手/record 字节，作为未来版本的 previous 向量 | 向量 | 无冻结版本就永远无法测互通 | S |

### P2 — 质量与可维护性

| # | 目标 | 缺什么 | 类型 | 工作量 |
|---|---|---|---|---|
| T2-1 | `neko-cli/tests/probe.rs`（241 s，12 处 sleep，如 `:6861`、`:7670`、`:7730`、`:8782` 固定 30–150 ms） | 把固定 sleep 换成"等待事件/就绪行"的有界轮询；拆分 probe.rs 以便并行。降低 CI 下 flaky 风险并缩短门禁 | 重构测试 | M |
| T2-2 | `multistream.rs` 测试 `:285`、`:426` 固定 50 ms sleep | 同上 | 重构测试 | S |
| T2-3 | 弱断言 | 库层 `assert!(x.is_err())` 不检查错误种类：crypto 28 处、session 6、carrier 5、wire 4、observe 2。至少把 crypto 的改成 `matches!(…, Err(Kind))`，防止"因为别的原因失败"而误通过 | 单元 | S |
| T2-4 | 无 sudo 的网络损伤 | 当前损伤测试要么是确定性 fake socket，要么 `scripts/bench/run-netns.sh` 需 `sudo -n`，且其通用矩阵实测的是 `ping`（`run-netns.sh:31`）而非协议。建议加一个用户态 UDP 代理（丢包/延迟/乱序/MTU 截断），让真协议在普通 `cargo test` 里过损伤矩阵 | 集成基础设施 | L |
| T2-5 | IPv6 | loopback `::1` 上跑一遍 UDP/TCP/failover 集成（不需要真实 IPv6 网络），至少证明地址族路径无硬编码 v4 | 集成 | S |
| T2-6 | 非 Linux | CI 只有 ubuntu；加 `cargo check --target x86_64-apple-darwin / x86_64-pc-windows-msvc`（不需运行）锁定可编译性，`--plpmtud` 在非 Linux fail-closed 也要有测试 | CI | S |
| T2-7 | 覆盖率 | 修好工具链后把 `cargo llvm-cov` 纳入周期性报告，优先看 `neko-cli/src/main.rs`（7431 行）与 `neko-carrier/src/lib.rs` 生产路径 | 流程 | S |

## 4. 建议执行顺序

1. T0-1/T0-2（先写红测试）→ 修 Lane A P0 → 转绿。
2. T0-3 新增 3–4 个 fuzz target，T0-4 把 fuzz smoke 接进本地门禁。
3. T1-1、T1-2 与对应代码修复配对（都是"上限触顶即整体失败"类）。
4. T1-5、T1-7 两个小而便宜的锁定测试。
5. T1-4 soak、T1-6 抖动，再做 T2-4 用户态损伤代理（它是 T1-6 和场景矩阵升档的共同基础设施）。

DONE

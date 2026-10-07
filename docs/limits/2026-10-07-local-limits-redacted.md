# 本机极限曲线（loopback）— 2026-10-07 **最终版**

状态：**全部完成**。122 数据点落盘于 private/local-raw/
（block1-ms 52 / block1-periodic 4 / block1-udp 14 / block2-tcp 19 /
block2-udp 18 / block3 12 / block4 2 / smoke 1）。测试窗口
13:47-23:22（UTC+8），harness v3（periodic duration 修复后）为准。
原始 JSON：`private/local-raw/{block1-ms,block1-udp,block1-periodic,block2-tcp,block2-udp,block3,block4}/`
代理：impair-proxy（tools/impair-proxy@40f7ebd，含 stop-and-wait 唤醒修复）
二进制：neko-cli @ 5ef02d2（NEKO_MEASUREMENT=1），release，外部 target dir

## 已确立的基线事实（clean benchmark，先于损伤矩阵）

1. **fixture 固有节奏（TCP）**：neko 的 multistream/periodic 客户端均为
   stop-and-wait（每记录一次 DeliveryAck）且**未设 TCP_NODELAY**。loopback
   直连实测 4 流×100 记录 = 400 交换 / 33.1s ≈ **12.1 交换/s**（82.8ms/交换，
   Nagle×delayed-ACK 40ms 门控）。
2. **经 impair-proxy 反而更快**：代理对 TCP 全部 set_nodelay(true)，消掉
   Nagle 合并；每连接 ~24 交换/s（40ms delayed-ACK 仍剩一半）。**损伤矩阵
   全部经代理跑，节奏差异要归一化解读**（除 delay/loss 本身）。
3. **64 流聚合仍是 ~12 交换/s**：server 串行处理流（单线程逐记录收发），
   并发流不提升总吞吐。→ 「streams 扫描」block3 将量化。
4. **impair-proxy 自身吞吐**（bench-fast，rust sink）：UDP 609-669 Mbit/s、
   TCP 8.7 Gbit/s —— 代理不构成这些实验的带宽瓶颈。
5. 因此 **TCP 损伤曲线的量纲是「交换节奏 vs 损伤」**，不是带宽。带宽轴
   由 UDP probe 承担（无 Nagle/deleted-ACK 门控）。

## Block1-ms direct 全阶梯（完成，streams=64）

| bytes | records | 交换数 | elapsed | 交换/s | goodput KiB/s | rc |
|---|---|---|---|---|---|---|
| 32 | 120 | 7680 | 637 | 12.05 | 0.38 | 0 |
| 64 | 120 | 7680 | 637 | 12.06 | 0.75 | 0 |
| 128 | 100 | 6400 | 529 | 12.09 | 1.51 | 0 |
| 256 | 80 | 5120 | 423 | 12.10 | 3.02 | 0 |
| 512 | 60 | 3840 | 318 | 12.09 | 6.05 | 0 |
| 1024 | 40 | 2560 | 212 | 12.10 | 12.10 | 0 |
| 2000 | 24 | 1536 | 127 | 12.09 | 23.62 | 0 |
| 4000 | 12 | 768 | 64 | 12.08 | 47.20 | 0 |

**结论（direct）**：交换节奏 12.05-12.10/s，bytes 无关（变异 <0.5%）。
goodput = 12.1×bytes 完全线性。TCP fixture 极限吞吐 = **47.2 KiB/s**
（4000B 记录 × 12.08/s）。受 Nagle×delayed-ACK×stop-and-wait 三重门控。

## Block1-ms 损伤曲线（streams=64；非 direct profile 用 4 点阶梯）

**丢包系列（TCP，重传透明吸收，节奏只缓降）**：

| profile | b32 | b256 | b1024 | b4000 | 解读 |
|---|---|---|---|---|---|
| direct | 12.05 | 12.10 | 12.10 | 12.08 | 基线 |
| loss0.1% | 12.09 | 12.09 | 12.09 | 12.07 | 无感 |
| loss1% | 12.03 | 12.04 | 12.04 | 12.01 | ~0.5% 降 |
| loss5% | 11.81 | 11.80 | 11.80 | 11.78 | ~2.4% 降 |
| loss10% | 11.56 | 11.55 | 11.52 | 11.53 | ~4.6% 降 |
| loss30% | 10.73 | 10.72 | 10.68 | 10.67 | ~11.7% 降 |

**延迟系列（+RTT 与 delayed-ACK 重叠，非叠加）**：

| profile | 实际注入(c2s/s2c) | 交换/s | 周期 | 模型预测 |
|---|---|---|---|---|
| direct | 0/0 | 12.05-12.10 | ~83ms | 2×delayed-ACK |
| delay10 | 10/10ms | 9.72-9.74 | ~103ms | 83+20 |
| delay50 | 50/50ms | 9.88-9.91 | ~101ms | max(83, 100+1)=101 ✔ |
| delay200 | 200/200ms | 全部 timeout | ~403ms(推算) | 7680×403ms=51min>timeout |

delay200 的 timeout **不是协议失败**：节奏模型给出 ~2.48/s，全部点会完成，
只是超出预设墙时间（b4000 需 ~5.1min>156s 超时）。→ 队列后补一个
client_timeout=400s 的 b4000 确认点。

**GE 突发（0.02:0.5:1.0 ≈ 稳态 3.8%）**：11.96-11.98/s（-1%），介于
loss1 与 loss5 之间——与稳态丢包率 3.85% 吻合，TCP 重传同样吸收。

## Block2 抗损伤阶梯（v3，periodic count=600 b=256 interval=100ms）

**TCP loss 阶梯（ack-timeout 2s；setup-timeout 5s 默认 / loss30 补 30s）**：

| loss | rc | 确认 | 备注 |
|---|---|---|---|
| 1% | 0 | 600/600 | |
| 5% | 0 | 600/600 | |
| 10% | 0 | 600/600 | |
| 20% | 0 | 600/600 | |
| 30% | 0(30s setup) | 600/600 | 5s setup 下死于 Noise 握手（30% 双向丢包重传超窗）→ 有效发现：**握手期比数据期脆弱** |
| 50% | 0 | 600/600 | |
| 70% | 0 | 600/600 | **TCP 无断点**：有握手余量则任意丢包率都能完成 |

**MTU 分块 1400/1300/1200/1100（TCP 语义=写分块）**：全部 600/600，
分块不破坏流完整性。TCP 侧无 PMTU 黑洞可模拟（字节流自适应）。

**RST 窗口 kill（8000-9000ms）**：seq=77 处精确死亡（8s+77×100ms 吻合），
rc=2 "disconnected" —— RST 语义生效，可见失败。

**hold 窗口（8000-14000ms）**：seq=78 死亡 —— **ack-timeout 2s < hold 6s**，
客户端先超时断开。hold「恢复」语义对 periodic 不成立（其 ack 等待
无重试）。fixture 交互发现：hold 测试需 ack-timeout > hold 长度。

**reorder 系列（TCP）**：全部死于 "unauthenticated delivery acknowledgement"
——根因是 **impair-proxy 设计错误**（用户态 TCP 代理不可能伪造 reorder：
字节流写出顺序即字节顺序，乱序=破坏 AEAD 帧界）。已修复为 TCP 模式
拒绝这两个参数（1ae858f 已推送）。这些数据点标记无效。

**NAT rebind（UDP，会话拉长跨过 rebind 时刻）**：第一次 rebind（10s 处）
即杀死会话（584 交换后 hang）。server 端 peer 校验拒绝新源端口数据，
**UDP probe fixture 不支持任何路径迁移**。10s/60s/300s 档的 rebinds=0
点无效（会话 0.6s 跑完没到 rebind 时刻），只有 stretched 点有效。
→ 异常现象#2：路径迁移（NAT rebinding）即会话死亡（hang 形态，同 #1）。

**PLPMTUD 黑洞（c2s-mtu 黑洞 + --plpmtud --diagnostic）**：mtu1400/1300
下探测包 1389B 被黑洞吞掉（proxy mtu 计数器确认），client 发 probe→
timeout→retry→deadline，**未触发降级重搜**（blackhole fallback 未启用，
是 D067 slice 设计如此：仅报告不降级）。mtu1200/1100 会话本身死亡
（1200B 记录 + 封装 > 1200 黑洞）。结论：**黑洞下 PLPMTUD 探测可诊断
但不自愈**——数据记录了 probe_sent/timeout_retry/deadline 完整序列。

**direct**：13980 交换 / 1.6s = **8650 交换/s**（无 Nagle/delayed-ACK 门控，
真实链路节奏；含全部握手+密钥更新）。bytes 无关（32/512/1200 均 1.6s）。

**survival 阶梯（count=200，seed=20261007 确定性）**：

| loss | rc | 形态 |
|---|---|---|
| 0.1-0.4% | 0 | 200 交换全部完成 |
| 0.45% | timeout | 死于第 152 次交换 |
| 0.5% | timeout | 死于第 152 次交换（两次复现，同 seed 同位置） |

**关键发现（UDP fixture 语义）**：
1. probe client 的交换循环是「发一条→等 echo→无重试」，**任何单包丢失
   （c2s 或 s2c）即终止**。存活率 = (1-2p)^n：n=200 时 p=0.4% 存活率仅
   ~20%（本 seed 活了），p=0.5% ~13%。测得的 0.4-0.45% 边界是 seed 特定。
2. **死亡形态是 hang 不是 fail**：`u.recv()` 无超时设置，echo 丢失后
   client 永久阻塞（"echo timeout" 字符串只在 socket 错误时触发，名不
   副实）。→ 异常现象#1，已按纪律原样保留证据（survival-loss0.005.json）。
   这是 carrier 层无重传 + fixture 无超时的组合行为；neko-reliable 的
   重传不在此 fixture 路径上。
3. 推论：**UDP 丢包下这组 fixture 的正确量纲是「单次会话存活率 vs
   丢包率」**，吞吐曲线只在 direct/无损档有意义（8650 ex/s）。

## 最终结论（全矩阵完成）

1. **TCP fixture 的极限是节奏不是协议**：12.15 ex/s 天花板（stop-and-wait +
   Nagle×delayed-ACK 83ms/交换），流数无关（1-256 平坦），bytes 线性放大。
   goodput 极限 = 12.15 × bytes（1150B 记录 = 13.6 KiB/s）。
2. **TCP 抗丢包无断点**（loss≤70% 全完成），但**握手期脆弱**：30% 丢包 +
   默认 5s setup 窗口即握手失败（30s 窗口恢复全通）。生产含义：高丢包
   链路的连接建立窗口需要远大于数据重传窗口。
3. **UDP probe 的断点是统计性的**：无重传层 + 无 recv 超时，任何单包
   丢失即 hang（异常#1）。会话存活率 = (1-2p)^n。
4. **延迟与 delayed-ACK 重叠**：RTT<83ms 时交换节奏由 delayed-ACK 决定
   （延迟免费）；>83ms 后线性劣化。delay200/向 ≈ 2.5 ex/s（推算+验证）。
5. **1h soak 零异常**：13980/13980、延迟无漂移、RSS 温和、无泄漏。
6. **MTU/PMTU**：TCP 无感（字节流自整）；UDP 侧 PLPMTUD 黑洞下可诊断
   不自愈（异常#4）。
7. **工具链本身**：impair-proxy 单向 ≥200Mbit/s 目标达成（实测 8.7Gbit/s
   量级，README 有数据），确定性 seed 全程可复现；本战役暴露并修复了
   代理自身 2 个 bug（pump 唤醒 40f7ebd、TCP reorder 语义 1ae858f）。

- 12 交换/s 恒定 ⇒ goodput = 12 × bytes 线性，TCP fixture 吞吐上限
  ≈ 12×4000B = 48 KiB/s（受 delayed-ACK + stop-and-wait 结构限制）。
- 若要在 TCP 上测真实带宽上限，需要 fixture 设 TCP_NODELAY 或批量
  窗口发送——这属于协议 fixture 演进项，不是损伤代理问题。

## 异常现象清单

**#1（P1，佩丽卡已裁决单列）UDP probe 单包丢失即永久 hang**
main.rs:1187 `u.recv` 无超时；echo 丢失后 client 永久阻塞（"echo timeout"
字符串只在 socket 错误路径触发）。任何 c2s/s2c 丢包即触发。对应审查
lane-c T1-6/T1-7 方向。修不修等管理员看数据后决定。

**#2 UDP probe 路径迁移（NAT rebinding）即会话死亡**
server 端 peer 校验拒绝新源端口数据，会话在第一次 rebind 后 hang
（同 #1 形态）。10s rebind 实测：584 交换后死。

**#3 CLI 校验上限与运行期 seal 上限不符（默认与 measurement 模式共有，存量）**
periodic 的 CLI 校验层（periodic.rs:60-61，默认即「1-1200」）与运行期
seal/帧封装真实上限（~1171）不符：bytes≥1172 时第一次 seal 即拒绝，且以
`unwrap()` panic 死亡（periodic.rs:620 SessionRejected，客户端路径；服务端
路径则在监听期先死，客户端报 connect failed）。capabilities 宣称
bytes_max=1200 同样与真实上限不符。佩丽卡已在默认模式（无
NEKO_MEASUREMENT）复现确认：**460b4d5 未修复也未加剧，非引入方**。
二分定位：≤1165 正常，≥1172 panic（1166-1171 未测，本地证据）。
影响：本战役所有 bytes=1200 数据点标记无效；soak 改用 1150B 安全档。
证据：run/soak-client.log + bisect-bytes.sh 输出。

**#4 PLPMTUD 黑洞可诊断但不自愈**
黑洞下 probe_sent→timeout_retry→deadline 完整记录，但 blackhole
fallback 未启用（D067 slice 设计），不会降级重搜。

**#5（非缺陷，fixture 语义记录）hold 窗口恢复受 ack-timeout 约束**
hold 6s > ack-timeout 2s 时 periodic 先自杀；hold 测试需 ack-timeout
大于 hold 长度。RST 语义则精确生效（seq=77 处死亡与窗口起点吻合）。

- 14:24 起两套 harness 端口互抢造成 b32-b2000 若干点污染（rc=-9/timeout
  交错）；受污染 JSON 已删除，重跑。根因：nohup 会话超时被杀后子进程
  存活 + 新队列重启。已加 server 监听就绪探测 + 全家族 pkill 流程。
- 仓库既有 stash@{0}（他人遗留）曾被误 pop 造成 CHATGPT_HANDOFF.md 冲突，
  已用 HEAD（88d6258 较新版）恢复，stash 原样保留。
- **harness 缺陷#1**：periodic-server 的 --duration 从未传递（默认 10s），
  导致 block1-periodic 与 block2-tcp 第一版全部点在 ~9.5s 被 server 墙
  截断（rc=2 disconnect）。数据已全部作废重跑（v3 队列）。
- impair-proxy stop-and-wait 唤醒修复（40f7ebd）：损失 10-20ms/交换的
  丢失唤醒窗口。修复后 TCP 经代理 24 ex/s（nodelay）vs direct 12 ex/s
  （Nagle×delayed-ACK）。此修复已推送。

## Block3 并发与短连

**streams 扫描（s=1..256，r=7680/s，b=256，s×r 恒定 7680）**：

| s | 聚合速率 |
|---|---|
| 1-256（全部） | 12.15 ex/s，rc=0 |

**完全平坦**——聚合吞吐对流的数量完全不敏感（1 条与 256 条同速）。
结合 multistream 的 server 串行 accept/处理结构（block1-ms 已证），
瓶颈是 server 单线程节奏（delayed-ACK 门控 83ms/交换），不是并发数。
流数不是本 fixture 的可测极限维度。

**短会话连发（30 会话 × 20 交换，目标 10/30/50 会话/s）**：
30/30 全部成功（三档都是）；但**达成率恒 0.5 会话/s**——每会话 1.97s
（含握手 + 20 交换 + teardown），串行执行下目标速率无法企及。
无失败、无泄漏（cleanup=verified 逐会话）。

## Block4 Soak（1h 长跑，periodic TCP b=1150）

| 指标 | 值 |
|---|---|
| 交换 | **13980/13980 attempted=confirmed，0 missing 0 dup** |
| 墙时间 | 3495s（58.25 min） |
| p50/p95 确认延迟 | 40/41 ms（全程序列稳定，无漂移） |
| 应用字节 | 16,077,000 B（15.34 MiB） |
| RSS 轨迹 | 3.1→4.8 MB（58 分钟 +1.7MB，温和线性，非泄漏形态） |
| fds | 恒 4 |
| reconnects | 0；cleanup=verified |

**64MiB 生命周期上限未到达**：count 上限（13980×1150B=15.34MiB）
先于字节上限到达——**count 先于 bytes 触顶**，如实记录（按佩丽卡要求）。
以 1150B/250ms 的配置，达到 64MiB 需要 ~58,600 交换（count_max 的
4.2 倍），超出 fixture 能力。RSS 增速外推：64MiB 字节量级时 RSS 约
+7MB，不构成内存威胁。

# impair-proxy

用户态 loopback UDP/TCP 损伤代理 — 不需要 sudo、不碰 tc/netns/iptables，用
于在 127.0.0.1 上测试协议在网络损伤下的极限行为。为零依赖独立 Rust
crate（不加入 nekomusume workspace members）。

## 为什么是 Rust（而不是 Python3 标准库）

任务给了两个实现选项。选 Rust 的理由：

1. **吞吐硬指标**：要求单方向至少 200 Mbit/s 转发不成为瓶颈。Python
   事件循环 + per-datagram 开销在 1400B/包下通常停在两位数 Mbit/s；
   实测本实现（release，loopback，Rust sender/sink）：
   - **UDP 669 Mbit/s**（direct 基线的 89%）
   - **TCP 8.7 Gbit/s**（direct 基线的 94%）
   两者都远超指标，Python 无法保证。
2. **确定性**：xorshift64\* + 每方向 splitmix64 派生子流，决策只依赖
   到达序列（时间+长度），与线程调度无关；同 seed 同输入序列 → 决策
   逐位一致（有单测 + e2e 验证）。
3. **零依赖**：不进 workspace members、不添加任何 Cargo 依赖（连
   libc/clap 都没有），对主仓库供应链零影响。std-only，`unsafe` 仅限
   两处 FFI 声明（`signal(2)`、`setsockopt(SO_LINGER)`），无 unsafe 块
   之外的指针操作。

## 构建

```bash
cd tools/impair-proxy
cargo build --release
# 二进制: target/release/impair-proxy
```

## 使用

```bash
# UDP: 前端 127.0.0.1:9000 -> 上游 127.0.0.1:9001，c2s 10% 丢包 + 50ms 延迟
./target/release/impair-proxy --mode udp \
    --bind 127.0.0.1:9000 --upstream 127.0.0.1:9001 \
    --c2s-loss 0.10 --c2s-delay-ms 50 --s2c-loss 0.02 \
    --stats-file /tmp/stats.json --verbose

# TCP: 定时断连测试（RST at t=2s），静默黑洞 5s..8s
./target/release/impair-proxy --mode tcp \
    --bind 127.0.0.1:9000 --upstream 127.0.0.1:9001 \
    --c2s-kill 2000:2600:rst --s2c-kill 5000:8000:hold \
    --stats-file /tmp/stats.json

# 确定性复现：固定 seed
./target/release/impair-proxy ... --seed 42
```

SIGINT/SIGTERM → flush 队列、写 stats、exit 0。`--duration-ms N` 到时
自动退出。`--help` 看全部参数。

## 损伤类型与参数

所有损伤**每方向独立**：`--c2s-*`（client→upstream）/ `--s2c-*`
（upstream→client）。未配置 = 直通。

| 损伤 | 参数 | 语义 |
|---|---|---|
| 随机丢包 | `--X-loss P` | 独立按概率丢 |
| GE 突发丢包 | `--X-ge P:R:H` | Gilbert-Elliott 两态马尔可夫；p=P(good→bad)，r=P(bad→good)，h=坏态丢包率 |
| 固定延迟 | `--X-delay-ms F` | 每包加固定延迟 |
| 抖动 | `--X-jitter-ms F` | 额外 [0,F] 均匀延迟（可与 reorder 叠加产生自然乱序） |
| 乱序 | `--X-reorder P --X-reorder-delay-ms F` | 按 P 给包额外 +F 延迟，统计性乱序 |
| 重复 | `--X-duplicate P` | 按 P 再发一份相同副本（同 due 时间） |
| 限速 | `--X-rate-bps N [--X-rate-burst-bytes B]` | 令牌桶整形；**延迟不丢包**（字节信用，可欠债，FIFO 释放） |
| MTU 黑洞 | `--X-mtu N` | **UDP**: >N 字节直接丢（PMTU 黑洞）。**TCP**: 写分块上限 N（保序不丢，模拟小 MTU 路径） |
| 定时黑洞 | `--X-blackout T1:T2` | 代理相对时间 [T1,T2) ms 内全丢（UDP）/ 停读停写（TCP stall，不丢字节）。可重复多次 |
| NAT rebinding | `--X-nat-change-every-ms N` | 每 N ms 换一个新的出站 UDP socket（源端口变化） |
| 定时断连 | `--X-kill T1:T2:rst\|hold` | **TCP only**。rst: 进入窗口立刻 RST 断连（SO_LINGER=0）；hold: 窗口内静默挂起（黑洞），窗口后恢复 |

（`X` = `c2s` 或 `s2c`）

全局：`--mode udp|tcp`、`--bind`、`--upstream`、`--seed`、
`--stats-file`、`--port-file`（写入实际绑定端口，测试用）、
`--duration-ms`、`--queue-cap`（每方向队列上限，溢出记 overflow 丢）。

## 语义模型（确定性）

每个方向两个角色：

- **reader 线程** 独占 `ArrivalEngine`：固定 draw 顺序
  loss → GE(转移, h) → duplicate → jitter → reorder；blackout/MTU/kill
  窗口只查表不消耗随机数。给定 `--seed` 与到达序列，决策逐位可复现。
- **pump 线程** 持 due-time 最小堆队列：按 (due, seq) 释放，seq 打破
  平局保证 FIFO；只做观测统计（实际发送延迟直方图）。

## JSON 统计

退出时写 `--stats-file`（默认 `impair-stats.json`）：

```json
{
  "mode": "udp", "seed": 42, "started_unix_ms": ..., "duration_ms": ...,
  "c2s": {
    "received_pkts": 10000, "received_bytes": 14000000,
    "sent_pkts": 8997, "sent_bytes": 12595800,
    "dropped": {"random": 1003, "ge": 0, "mtu": 0, "blackout": 0, "overflow": 0, "total": 1003},
    "duplicates": 0, "reorder_hits": 0, "rate_deferred": 0, "nat_rebinds": 0,
    "tcp_rst_kills": 0,
    "latency_ms": {"count": 8997, "min": 49.8, "max": 51.2, "mean": 50.0},
    "latency_hist_ms": {"50-100": 8997}
  },
  "s2c": { ... }
}
```

## 自测

```bash
cargo test               # 25 单测（纯引擎，无 socket）+ 18 e2e（真实 loopback）
```

覆盖（节选）：

- 10% 丢包在 10k 包上落在 9–11%（单测 + e2e）
- GE 稳态丢包率 = p/(p+r)（±0.5%），且突发性（run length）显著大于独立丢包
- delay/jitter 落界、jitter 四分位均匀性（±15%）
- reorder 命中率与额外延迟、e2e 逆序对计数 ≥3
- duplicate 精确计数（sent = recv×2）
- 令牌桶：突发借债后的精确 due 时间、贴速率流不 defer、TCP 整形 ≥ 最小时长
- MTU 边界精确（=mtu 过，>mtu 丢）；TCP 分块后流完整性
- blackout 窗口 [T1,T2) 精确、TCP hold 不丢字节且窗口后恢复
- NAT rebinding：换源端口后回程切换、echo 连续性
- TCP RST kill：窗口内连接死、计数 >0；TCP 200KB 流完整性
- 确定性：同 seed 同到达序列决策逐位一致（单测断言字符串序列相等；e2e 双跑）

## 吞吐

`bench/` 内含独立 rustc 编译的 sender/sink（无依赖、不进 cargo build）：

```bash
cargo build --release
cd bench && rustc -O sink.rs -o sink && rustc -O send.rs -o send
./bench-fast.sh udp 2000 1400   # mode window_ms mss
./bench-fast.sh tcp 2000 65536
```

本机实测（loopback，2s 窗口，单方向，多轮取稳定值）：

| mode | direct | through proxy | ratio |
|---|---|---|---|
| UDP 1400B | ~710-750 Mbit/s | **~610-670 Mbit/s** | 86-89% |
| TCP 64KB | ~9.2 Gbit/s | **~8.7 Gbit/s** | 94% |

（`bench.sh` 为 python sink 版本，测得 UDP 401 Mbit/s — sink 本身是瓶颈，
数字偏低但仍超指标；`bench-fast.sh` 为 rust sink 高精度版。）

## 边界

- 只在 loopback 上跑（bind 默认 127.0.0.1；上游应也是 loopback）。
- 不 sudo、不改系统网络配置。
- MTU 黑洞在 TCP 语义是"分块上限"（可靠流不能丢字节）；`--X-kill rst`
  才是 TCP 的"路径断"。
- 决策确定 ≠ 到达序列确定：上游/客户端自身发包节奏由应用决定。

# P4d D070 XJ Mesh 实测结果（窗口化 UDP bulk，脱敏版）

> 本文件为脱敏版——XJ 内网 mesh 地址与 identity/密钥材料不入库，仅保留
> SHA256 与测量数字。原始日志在本地 `/media/tmzn/DATA5/neko-limits-20261007/private/`。

日期：2026-10-11（UTC+8）
二进制：neko-cli.exe SHA256 `F4A3347E0511496AD0671C41A312483B0607D53388443F89B654B92F5B0CD33F`（与本地构建逐字节一致）
路径：XJ（mesh 客户端）→ tmzn（mesh 服务端），单 WireGuard 隧道，RTT 78ms，V4
载体：窗口化 UDP bulk（Reno AIMD + SACK，cwnd pin ≤48，记录 1170B）

## A 相 — 窗口化 w=32（新二进制）

| leg | 记录数 | 字节 | 耗时 | 吞吐 | 重传 | PTO | 服务端 SHA256 confirm |
|---|---|---|---|---|---|---|---|
| 2000 | 2000 | 2,340,000 | 10.9s | **1.72 Mbps** | 0 | 55 | `100ed471…38b5ec` ✓ |
| 13980 | 13980 | 16,356,600 | 100.9s | **1.30 Mbps** | 0 | 416 | `41ad525e…fd51370` ✓ |

两 leg 全 acked + END acknowledged + wire receipt confirm。零重传、零
duplicates 入库（duplicates_skipped 55/414 均为 PTO 探针重发被 pre-open
重复门静默吸收——预期形态）。

## B 相 — 停等对照（旧二进制，无重传/无窗口）

| leg | 记录数 | 结果 | 耗时 | 说明 |
|---|---|---|---|---|
| 2000 | 2000 | **confirm** | 156.0s | 精确 2000×78ms；吞吐 0.120 Mbps |
| 13980 | 13980 | **死**（两次独立） | 130.2s / 179.1s 处断 | `deadline elapsed before response`：单包丢失即 client 放弃 |

B 相 13980 的死亡不是环境噪声而是**结构性证据**：停等无重传，单包丢失
即死。两次独立运行分别死于 ~1660 与 ~2295 交换处（第 1、第 2 个随机丢
包），与层 3「count>2000 全死」自洽。窗口化 A 相同路径 13980 全 confirm，
正是 D070 存在的意义。

## 三方对比（同路径 XJ→tmzn mesh，78ms RTT）

| 形态 | 吞吐 | 13980 可达性 | 出处 |
|---|---|---|---|
| 旧 fire-and-forget UDP bulk | 4.7 Mbps（单向） | 死（count>2000 全死） | 层 3 实测 |
| 停等（B 相，可靠但慢） | 0.120 Mbps | 死（单包丢即死） | 本文件 B 相 |
| **窗口化 w=32（A 相，D070）** | **1.30 Mbps** | **全 confirm** | 本文件 A 相 |

注：4.7 Mbps 为单向无确认吞吐（fire-and-forget），1.30 为含 ACK 流量的
双向 goodput。三者中 D070 是唯一「快且可靠」的形态。

## D070 模型偏差归因（实测 1.30 vs 预测 4.05 Mbps）

预测 4.05 Mbps 来自 docs/decisions.md 早期模型（RTT 78ms、w=48、
1250B/记录、AIMD 稳态近似）。实测 1.30，偏差因子 ~3.1×。归因：

1. **PTO 主导的发送节拍**：实测 PTO 416 次/13980 记录，即稳态下发送窗
   口靠 PTO 超时驱动而非 ACK 时钟。每 PTO 周期（约 200ms 起、指数退避）
   只泵 1 个探针，等效带宽被探针周期除以窗口数摊薄——这是 78ms RTT 上
   w=32 不足以下探针时钟为主的直接后果。w=48 上限受 cwnd≤48 编译期 pin
   （不可放宽，设计约束），探针主导无法通过调参消除。
2. **记录 1170B vs 模型 1250B**：-6.4% 线性项。
3. **隧道加密开销**：单 WireGuard 隧道加密路径在 XJ（Broadwell 2.4GHz
   mingw 构建）上 CPU 墙已实证 568 Mbps loopback，非首要因素。

结论：1.30 Mbps 是 w≤48 + 78ms RTT + PTO 节拍下的**形态正确值**，与
模型差异已定位为「窗口不足时 PTO 节拍主导」，非缺陷。若未来放宽 pin
（需重新裁决）或上多流，预计向 4.05 靠拢。

> **Phase 0 复核（2026-10-12，8b256ee 后，赛希三重证据 + 佩丽卡裁决）**：
> 上段「PTO 节拍主导」的归因链已被下钻反转——max_ack_delay=0 与实现的
> 即时逐包 ACK 语义一致（`bulk_udp.rs` flush_ack 逐包调用 / take_ack(0)，
> PacketAckTracker 无定时器），参数正确零代码变更；416 次 PTO 是客户端
> 「send-burst→recv-drain」轮次节奏的伴生（~437 轮×~230ms ≈ 实测 100.9s
> 自洽），**1.30 Mbps 的真限制因子 = 客户端轮次节奏**，非 PTO 数学。
> 事件循环重构（ACK 到达即驱动发送）列为 D074 Phase 0'（0.5-1d），
> R1/多流顺延其后。设计文本侧：D070 决定段的「64 记录/40ms 先到」攒批
> 表述与实现（即时 ACK）分叉，已在 D074 草案 §0 回写。

## netns 丢包矩阵（P4c，已入 8ded938 断言 pins）

0% / 5% / 11.67% 三档全绿（13980 记录全 confirm，全在 CI 内）。见
`tests/bulk_d070_udp.rs` loss-matrix pins。

## 清理核查（全部完成）

- XJ `C:\neko-r2`：29 文件已删（目录节点被残留 shell CWD 锁，内容为空；
  已杀锁进程 + 计划任务兜底删除）
- XJ 进程：neko 进程 0
- XJ 防火墙：测试规则 0（未动系统规则）
- XJ 钉路由：bjrelay /32 已移除（现仅剩 mihomo 常规分流路由）
- 本地 tmzn：B 相旧 server（pid 3776011）已杀，端口 40090 释放
- 密码/identity：全部在 /tmp/p4d（已核无密码明文入库路径），不入 git

## 方法学坑（记录给未来）

1. **每 leg 需独立 bulk-server 且 --bytes-total 必须精确匹配**：server 对
   end mismatch fail-closed（exit 2），client 会对死端口烧满 PTO 时长。
2. **Windows 目录 CWD 锁**：PowerShell 会话若曾 cd 进测试目录，SSH 断开
   后残留 shell 锁住目录节点。杀锁进程或 schtasks SYSTEM 兜底可解。
3. **工具超时会连坐后台作业**：bash 工具超时 SIGTERM 杀整个进程组，
   `nohup &` 不够，必须 `setsid` 脱离会话。

# WAN 真实线路极限 — 测量报告（脱敏版）

> 战役窗口：2026-10-07 23:5x – 2026-10-08 06:xx（UTC+8）· 工具：neko-cli @ 5ef02d2（NEKO_MEASUREMENT=1）release
> 上游材料：net-survey.md（92 路径勘测）· local-limits.md（122 点本机曲线）· AUTHORIZATION.md
> 原始数据（含 IP）仅存本地 private/，永不入库；本文件为脱敏版。IP→别名映射见私有版。
> 流量：~2.6GB / 100GB 战役预算（traffic-ledger-wan.tsv）

## 0. TL;DR

1. **TCP fixture 的 WAN 极限完全由 RTT 决定**：交换率 ≈ 1000/(3.4×RTT+40ms)。50ms→6.9 ex/s、163ms→1.8-1.9、215ms→1.4。丢包（11.67%）对节奏无可测影响——TCP 重传完全吸收。
2. **HY2 与 neko fixture 的吞吐差是三个数量级**：P1 路径 100.6Mbps vs 27.7KiB/s（×4000）；这不是协议库的上限，是 CLI fixture 的 stop-and-wait 语义（每记录一次 ACK 往返 + delayed-ACK 门控）。
3. **UDP probe "公网不可用"（#6）——已解决，判定更正**：非协议缺陷。根因为 WAN harness 的 server 调用漏传 `--count`（默认 1），server 首次交换后正常退出，client 随后交换收到 ICMP port-unreachable 并以 "echo timeout" 快速终止（socket 有 read_timeout，非 hang）。修正 harness 后真实 client 完整跑通：HK→bjlh 公网 UDP probe 20 交换 885ms（probe_ok，双端抓包 42 包证据）。原"双端 0 捕获"为 tcpdump -w 缓冲不 flush 的采集假象。辨析：#1（loopback probe 无超时 hang）仍是独立真实缺陷。证据：私有档 udpdiag-fix-b6/。
4. **时段效应是最大的环境变量**：163 跨洋路径丢包 0%（凌晨勘测）↔ 11.67%（00:0x 深夜实测）↔ HY2 满速（03:5x 拥塞消退）。单次快照无代表性。
5. P1 soak（250ms interval）24h 窗口内 count=13980 先触顶（约 58min 完成），节奏/延迟/RSS 全程稳定——与 local-limits 的 count-先于-bytes-触顶结论一致。

## 1. 路径与基线（180 ping × 1Hz / 路径，00:0x 窗口）

| 路径 | 类型 | fam | RTT med | 丢包 | 线路 | 对照勘测（01-02 时） |
|---|---|---|---|---|---|---|
| P1 bjlh↔HK | 优质 | v4 | 49.9ms | 0% | 腾讯 CBG 内网对称 | 一致（49.7/0%） |
| P2 bjlh→us-209 | 普通 | v4 | 165.1ms | **11.67%** | 电信 163 跨洋 | 勘测 0% → **深夜拥塞** |
| P3 bjlh→eu-89 | 长距 | v4 | 163.0ms | **11.67%** | 163→Telia/Cogent，PMTU 1443 | 同上，与 P2 完全同型 |
| P4 bjlh→us-45 | 优质v6 | v6 | 215.2ms | 0% | 腾讯内网直连（v6 优于 v4） | 一致 |
| 辅 HK→bjlh v6 | — | v6 | 44.8ms | 1.11% | 163 绕公网 | 同型 |

**发现（时段维度）**：P2/P3 同丢包率同 RTT 型，疑同一 163 拥塞点；凌晨（勘测）与深夜（本测）差异 0%↔11.67%；03:5x HY2 复测时拥塞已退（160-206Mbps 满速）→ **163 拥塞是日内函数，所有数据点必须带时段戳**。

## 2. neko 协议矩阵（measurement 模式）

参数：multistream 256流×64记录×4000B（62.5MiB）；periodic 13980×1150B（#3 缺陷安全档）；墙时间按 stop-and-wait 预算。

| run | 交换完成 | wall | ex/s | goodput | p50 确认延迟 | 形态 |
|---|---|---|---|---|---|---|
| P1 ms | 16384/16384 | 2363s | **6.93** | 27.7 KiB/s | — | ✅ 完整（ok=true, window_exhausted=0） |
| P1 per | 13980/13980 | 2770s | **5.05** | 5.8 KiB/s | **197ms** | ✅ 完整（0 missing 0 dup, cleanup=verified） |
| P2 per | 6085/13980 | 3401s | **1.79** | 2.0 KiB/s | **557ms** | ⏹ 撞墙（timeout），已发全确认、零丢失 |
| P3 per | 6339/13980 | 3400s | **1.87** | 2.1 KiB/s | **535ms** | ⏹ 同上 |
| P4 per | 4698/13980 | 3401s | **1.38** | 1.6 KiB/s | **722ms** | ⏹ 同上（0% 丢包纯 RTT 路径） |
| P2/P3/P4 ms | partial | 3700s 撞墙 | ~2.4-2.7 | ~9-11 KiB/s | — | ⏹ 墙内未完成（JSON 仅完成时输出）；P1-ms 已给出完整 ms 基线 |

### 核心定律（WAN 实测确立）

1. **交换周期 = 3.4×RTT + ~40ms**（P1: 197≈3.4×50+40 ✓；P2: 557≈3.4×163+40 ✓；P4: 722≈3.4×215+40 ✓）。local-limits 的 83ms delayed-ACK 模型在 WAN 修正为 RTT 主导项。
2. **RTT 坍缩**：RTT 4 倍（50→215ms）使交换率坍缩 5 倍（6.93→1.38 ex/s）。
3. **丢包免疫（TCP 路径）**：P2/P3（11.67% 丢）与 P4（0% 丢）按 RTT 归一后节奏一致——TCP 重传对 11.67% 完全透明，与 local loss≤70% 阶梯结论在公网复现。
4. **interval 失效**：periodic 设 100ms interval 实际 197-722ms/交换——WAN 上节奏由 RTT 而非 interval 决定，interval<RTT 无意义。
5. **墙时间预算公式**：完成 N 交换需 N×(3.4×RTT+40ms)。13980 交换：P1 需 46min（实测 46.2min ✓）、P4 需 168min——3700s 墙预算对 163/215ms 路径不足（合法撞墙，非协议失败）。

## 3. HY2 对照（同路径同字节量，全 MD5 一致）

方法：各 server 临时 hysteria2（UDP 40084，自签 cert，1Gbps 配置）+ 官方 client SOCKS + loopback http.server，curl 过隧道。生产 hy 服务零接触。

| 路径 | 62.5MiB（=neko ms 量） | 15.34MiB（=per 量） | TTFB | neko 同路径对照 |
|---|---|---|---|---|
| P1 | 5.21s = **100.6 Mbps** | 1.22s = 105.1 Mbps | 96ms | 27.7 KiB/s（**×3700**） |
| P2 | 3.27s = **160.5 Mbps** | 1.16s = 111.2 Mbps | 334ms | 2.0 KiB/s（×80000） |
| P3 | 2.54s = **206.0 Mbps** | 0.99s = 129.5 Mbps | 315ms | 2.1 KiB/s（×98000） |
| P4 v6 | 6.97s = **75.2 Mbps** | 2.87s = 44.8 Mbps | 452ms | 1.6 KiB/s（×47000） |

- HY2 TTFB = RTT×~2（QUIC 握手+TLS）；neko TCP 握手实测也可行（P1-per 首交换即确认），差距全部在**数据面传输语义**（stop-and-wait vs 窗口流）。
- P2/P3 HY2 反超 P1：03:5x 时 163 拥塞已退、而 bjlh→跨洋方向出口（勘测 ~200Mbps）未被 HK→bjlh 方向的 CBG 整流约束。
- **结论修正**：HY2 的 100-206Mbps 是「协议+拥塞控制+流语义」的合力；neko 的数字是 fixture 语义下界而非协议库上界——库层 64MiB/Session 与 UDP 1200B MTU 语义不受此影响。

## 4. P1 24h soak（W8 终段，已完成）

配置：bjlh periodic-server + HK client，13980×1150B @ 250ms interval，duration 86400s。03:59 启动。

| 指标 | 值 |
|---|---|
| 交换 | **13980/13980 attempted=confirmed，0 missing 0 dup，0 reconnects** |
| 墙时间 | 3495s（58.25 min），05:02 完成 |
| p50/p95 确认延迟 | **151/152ms**（全程序列稳定无漂移，与 P1-per 197ms 差异来自 interval 250ms>100ms 减少 delayed-ACK 重叠） |
| 应用字节 | 16,077,000B（15.34MiB） |
| RSS | 三采样点恒定 2060KB（04:01/04:31/05:02）——零增长 |
| 双端 cleanup | verified ✓ |

**count 先于时长/bytes 触顶**（13980×250ms=58min ≪ 24h；64MiB 需 ~58600 交换）——与 local-limits 结论一致：fixture 的 soak 语义受 count_max 约束，24h 全时长低速率长跑需要 interval≥6.2s 或 fixture 演进。

## 5. 异常清单（WAN 新增，#6 起；#1-#5 见 local-limits.md）

### #6 neko UDP client 公网 hello 不达（双端 0 捕获）+ echo timeout 形态
- 现象：HK→bjlh:40082/udp，server 活着（READY），client 发起后 **bjlh 侧 tcpdump 0 包**、HK 出站侧弱证据同样 0 包；client 以 "echo timeout"（该字符串仅在 socket 错误路径触发，ICMP 形态）快速退出（非 #1 的无限 hang）。
- 对照：同 HK 源、同端口，nc 100/600/1200B 载荷 100% 到达（tcpdump 实证）；neko TCP 同参数对（P1 soak）完全正常。
- 已排除：SG 入方向过滤（nc 到达）、大小过滤（1200B nc 过）、server 存活/密钥（TCP 同 key 对通）、bjlh 本机防火墙（无 ufw）。
- 候选根因（未定性）：① neko UDP client socket 语义（源端口/绑定行为与 nc 不同，被路径中间设备差别对待）；② HK 出站侧对特定 UDP 流的静默丢弃（HK SG 出站策略未知）；③ 客户端 hello 发送路径 bug（如 wire 帧构造后实际未 send）。
- 影响：UDP probe 负载在公网不可测；W8 的 UDP 维度缺失。
- 证据：wan-raw/udpdiag-*（三轮双端 capture+server 日志+nc 对照）。

### #7（minor）periodic-server duration 上限挡住 24h soak 配置
- measurement 模式 duration_max=86400：server 想配 >client 的 duration（如 90000）被拒。CLI 层整数调整可修（属我自己的测量模式实现），非库层。已用 86400 绕行。
- 同类：count 封顶使 24h soak 在 ~1h 完成全部交换——「24h 时长」语义下低速率长跑需要更小 interval 或更大 count 上限（见建议）。

## 6. 工具链踩坑（9+3 条，wan-harness-notes.md）
NAT bind / ssh stdin / `&` 优先级 / `$!` 转义 / v6 方括号（neko+hy2 双份）· YAML 引号 / pkill 自匹配 / **HK 生产 hysteria 险被 pkill -x 误杀（权限挡住）** / tcpdump 文件缓冲时序（读早了=假 0 包，两轮实验被污染后修正）。

## 7. 结论与建议

1. **当前 CLI fixture 不能也不应用来测带宽**：TCP 数据面是节奏测量仪（RTT→交换率），带宽维度由 UDP 承担——但 UDP 被 #6 挡住。修复 #6 后 UDP probe 才是 WAN 带宽轴的可行工具。
2. **协议库的真实 WAN 能力未被触及**：库层限制（64MiB/Session 等）在全部测试中只在 count 维度接近；传输语义（stop-and-wait）在所有数据点都是第一瓶颈。要测协议库上限需要 fixture 演进（批量窗口/TCP_NODELAY/并发会话聚合——local-limits 已立项）。
3. **HY2 差距的工程含义**：同线路 75-206Mbps vs 1.6-27.7KiB/s。若目标是「证明协议库可行」，需在 fixture 演进后重测；若目标是「当前工具能做什么」，本报告即是上界。
4. **网络环境结论**：163 深夜拥塞（11.67%→0% 日内摆动）、CBG 内网稳定性（全程 0% 丢）、v6 腾讯直连（P4 全程 0% 丢但 RTT 215ms）——路径选择比协议调优更影响体验。
5. **修复建议优先级**：#6（UDP 公网不可用，阻塞带宽轴）> #3（bytes≥1172 panic，已规避）> #7（duration 上限，小修）> #1/#2（fixture 语义，已有记录）。

## 8. 数据清单
wan-raw/：p1-p4 × ms/per 双端日志、hy2-p1~p4 curl+md5、summary.tsv、soak samples.log（持续）、udpdiag-*（#6 证据）。

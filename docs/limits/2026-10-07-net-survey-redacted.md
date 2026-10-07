# 猫娘协议真实线路测试 — 网络勘测报告（脱敏公开版）

> 生成：2026-10-07 02:2x（UTC+8）· 勘测窗口 01:10–02:20 · 执行：猫娘极限-网络勘测（银狼）
> 收尾回填：2026-10-07 11:42–12:05（§3/§5/§7/§8 回填、清理补做并逐台验证、账本补记）
> 上游授权：AUTHORIZATION.md（24h 窗口、允许临时端口、禁止 DDoS 化、生产服务只读）
> 脱敏版：全部公网 IP 已用别名替换；ASN、线路类型、RTT/丢包/PMTU、带宽数字全部保留。可与私有版逐行对照。

## 0. TL;DR

- 8 台机器 + 本机共 92 条方向×族路径全测（mtr TCP/UDP ×30、ping -D 采样 RTT 中位/P95、PMTU 二分）。
- **国内出口无一走 CN2/CMI/9929**：bj-lh-relay 与 bj-relay 出向全部是 **腾讯 AS45090 → 电信 163（AS4134）**，入境同路。HK 出向美国走 **HE（AS6939）**。
- HK↔国内两台 v4 走 **腾讯内网 CBG（AS749/45090）** 47–50ms 无丢包——这是全网格里唯一的"精品内网"路径；**但其 v6 反而绕公网 163（AS4134→23724→45090）**，同等 RTT 但暴露公网。
- US-Z 的第二 v6 前缀（US-Z-v6b/64）全球不可达（本机/US-209 双验证 100% 丢）；US-Z **出站 v6 全断**（无默认路由源头，全部 v6 目标 100% 丢），入站 v6（US-Z-v6a 主用地址）正常。
- 美国三台（53667）之间为机房内直连（53–146ms，同 AS 内）。
- 全程无越权操作：未改任何生产服务/路由/sysctl；仅在美/欧 5 台临时 `ufw allow 5201 comment neko-limits-20261007`（已删除，见 §7）。
- iperf3 参考值（非真实负载，**仅成功 2 对**，见 §3）：BJ-LH→US-209 TCP 上行 187/下行 297 Mbps（上行 retr 15077，163 拥塞）；BJ-LH→US-Z TCP 下行 201 Mbps、UDP 硬顶 ~200 Mbps（500M 档 59% 丢包时 recv 仍钳在 200——出口限速/整形特征）。HK 与美欧各台出口带宽**未测成**（原因如实见 §3.2/§5）。iperf 实耗 3.2 GiB（§8）。

## 1. 机器基础信息

| 别名 | 公网 v4 | ASN / 组织 | 位置 | 内核 | CPU/核 | 内存 | 公网 v6 | v6 前缀来源 | MTU(eth) | CC 算法 |
|---|---|---|---|---|---|---|---|---|---|---|
| tmzn-hk | HK | AS132203 腾讯国际 | 香港 | 6.8.0-51 | Xeon 8255C /1 | 2G | HK-v6/64（2026-10-06 新分配，手动静态） | 静态（管理员配） | 1500 | cubic (pfifo_fast) |
| bj-lh-relay | BJ-LH | AS45090 腾讯 | 北京(轻量) | 6.8.0-124 | Xeon 8255C /2 | 2G | BJ-LH-v6/128 | DHCPv6/静态(/128) | 1500 | cubic (fq_codel) |
| bj-relay | BJ-R | AS45090 腾讯 | 北京 | 7.0.0-14 | Xeon 8255C /2 | 2G | BJ-R-v6/128 | DHCPv6/静态(/128) | 1500 | cubic (fq_codel) |
| vm-zuvu6 | US-Z | AS54286 TAIPEI101 | 美 El Segundo | 6.8.0-138 | EPYC 7502 /1 | 2G | US-Z-v6a（主用,SLAAC）＋US-Z-v6b/64（**不可达**,SLAAC） | SLAAC | 1500 | cubic (fq_codel) |
| us-209 | US-209 | AS53667 FranTech | 美 LA(Buycraft) | 6.8.0-60 | Ryzen 9 7950X3D /1 | 1G | US-209-v6/64 等 | SLAAC+手动 | 1500 | cubic (fq_codel) |
| us-45 | US-45 | AS53667 FranTech | 美 | 6.8.0-60 | 7950X3D /1 | 1G | US-45-v6/64 | 手动 | 1500 | cubic (fq_codel) |
| us-104 | US-104 | AS53667 FranTech | 美 | 6.8.0-60 | 7950X3D /1 | 1G | US-104-v6/64 等 | SLAAC+手动 | 1500 | cubic (fq_codel) |
| eu-89 | EU-89 | AS46475 Limestone | 欧 立陶宛 | 7.0.0-30 | E5-2650 v4 /1 | 1G | EU-89-v6/64 | SLAAC | 1500 | cubic (fq_codel) |
| tmzn-server(本机) | 出口 AS4837 联通(LOCAL-QD) | AS4837 联通 | 青岛 | 6.8.0-137 | EPYC 7F72 /48 | 128G | LOCAL-v6/64(多个) | SLAAC+DHCPv6 | 1500 | cubic (fq_codel) |

注：本机出站 v4 除 HK 外全部走 mihomo TUN（198.18.0.0/16 → Meta），**非真实线路**；本机 v6 走物理网卡 RA 默认路由，**真实线路**（已用 `ip -6 route get` 逐条验证不落 TUN）。

## 2. 有向路径矩阵（92 条）

原始 mtr/ping/pmtu 数据：`survey/raw/`（remote + local-hk + local-v6）；机器可读版：`survey/analysis/path-matrix-final.tsv`（列：src/dst/fam/线路/AS序列/RTT中位/P95/TCP末跳丢/UDP末跳丢/ping丢/PMTU）。

### 2.1 线路判定规则（AS→线路名）
- AS4809=CN2（GIA/GT 视 4134 混行）， AS4134/23724=电信163， AS9929/10099=联通精品， AS4837=联通4837 普通， AS58453/9808/58807=移动 CMI， AS749=腾讯 CBG（CN2 代答内网）， AS45090/132203=腾讯， AS6939=HE 中转。

### 2.2 关键路径摘要（去程=表方向；回程=对端→本端，见矩阵）

| 路径 | fam | 去程线路 | 去程 AS 序列 | RTT 中位/P95 | 回程线路 | 对称? | PMTU |
|---|---|---|---|---|---|---|---|
| bj-lh→HK | 4 | 腾讯内网 CBG | 749>23724>4134>749>132203 | 49.7/50.2 | 腾讯内网 CBG | **对称** | 1500 |
| bj-lh→HK | 6 | 电信163 | 45090>4134>23724>4134>45090>132203 | 45.5/— | 电信163 | 对称 | 1488 |
| bj-lh→us-209 | 4 | 电信163 | 749>23724>4134>174>53667 | 163/165 | 电信163(6939汇接) | 基本对称 | 1500 |
| bj-lh→us-45 | 4 | 电信163 | 749>23724>4134>174>53667 | 237/238 | 电信163 | 对称 | 1500 |
| bj-lh→zuvu6 | 4 | 电信163 | 749>23724>4134>174>46475>54286 | 212/214 | 同路 | 对称 | 1387 |
| bj-lh→eu-89 | 4 | 电信163 | 749>23724>4134>1299>6461>46475 | 160/163 | 163(3257 变体) | 轻度不对称(出口转接商不同) | 1443 |
| hk→us-209 | 4 | HE 中转 | 749>6939>53667 | 147/147 | 腾讯内网(6939) | **不对称** | 1500 |
| hk→us-45 | 4 | HE 中转 | 749>6939>53667 | 213/214 | 腾讯内网 | 不对称 | 1500 |
| hk→eu-89 | 4 | HE+Cogent | 749>3491>174>6939>46475 | 145/147 | 腾讯内网(6939) | 不对称 | 1500 |
| hk→zuvu6 | 4 | Lumen+Telia+Cogent 纯公网 | 749>3356>1299>174>2914>54286 | 219/219 | 腾讯内网 | **极不对称** | 1500 |
| eu89→bj-lh | 4 | 163 | 46475>6461>174>3257>4134>23724>45090 | 160/163 | — | — | 1500 |
| us-209↔us-45/104 | 4 | 机房内(53667) | 53667 | 53/146 ms | 同 | 对称 | 1440–1500 |
| 本机→HK | 4 | 联通4837 | 4837(青-京-港) | 57/127 | 腾讯CBG | 不对称 | 1474 |
| 本机→bjlh | 6 | 联通4837→4808 | 4837>4808 | 15.3/20 | (v6) | — | 1440 |
| 本机→us-209 | 6 | 联通4837→4808→6939 | 4837>4808>6939 | 192/197 | — | — | 1440 |

完整 92 条见 path-matrix-final.tsv（本文件 §附录 A 给出全表）。

### 2.3 v6 专项
- **US-Z 出站 v6 全断**：所有 v6 目标 100% 丢、PMTU 二分得 48（=纯 IPv6 头，无载荷通过）。根因：US-Z-v6b 段路由死 + US-Z-v6a 主用源址未被正确选路（详细跳分析在 raw，私有版）。入站（其他机→US-Z 的 v6a 地址）正常。
- HK v6（HK-v6 前缀）→ 美国/欧洲全部走 **Cogent(174)**（45090>3491>174），RTT 185–232ms，比 v4 HE 慢 30–60ms；→ 国内走 163 绕公网。
- bjlh/bjrelay v6 出向和 v4 同为 163（45090>4134>23724...），无独立 v6 精品路由。
- us-45 v6→国内两台走 **腾讯内网直连（53667>6939>45090）**，与 v4 不同（v4 走 163）——v6 反而更优。

### 2.4 丢包判读
mtr UDP 末跳高丢包（20–48%）是**探针伪影**（UDP 打到无监听端口收不到回包的 ICMP 限速），真实丢包以 ping 列为准：全部路径 ping 丢包 ≤10%，其中 HK↔国内/HK↔美欧、美机之间 0%。TCP 末跳丢包除 zuvu6 出 v6 外全部 ≤46%（末跳 mtr 也有 ICMP 限速因素），以 ping 与 iperf 重传统计为准。

## 3. 带宽参考值（iperf3，非真实负载）

方法：TCP 4 流 ×20s + UDP 阶梯（10→500M，>5% 丢包停），上下行各一次，逐对串行。
原始输出：survey/raw/iperf/（私有版目录；本脱敏版不含 IP，结构与私有版一致）。

### 3.1 成功测项（全量，仅 2 对路径）

| 路径 | 方向 | 协议 | 结果 | 备注 |
|---|---|---|---|---|
| BJ-LH→US-209 | 上行 | TCP 4f×20s | **187 Mbps**（446 MiB, retr 15077） | 163 拥塞明显：20s 内 1.5 万重传 |
| BJ-LH→US-209 | 下行 | TCP 4f×20s | **297 Mbps**（713 MiB, retr 11421） | 下行显著优于上行 |
| BJ-LH→US-Z | 上行 | TCP 4f×20s | **未测成**：connect timeout | 首对执行时 ufw 放行时序问题；下行已测 |
| BJ-LH→US-Z | 下行 | TCP 4f×20s | **201 Mbps**（486 MiB, retr 558） | 4 流中 2 流仅 6.8–7.6 Mbps（单流塌陷） |
| BJ-LH→US-Z | 上行 | UDP 阶梯 | 10M:0%丢 · 50M:0% · 100M:0% · 200M:0.02% · **500M:59% 丢** | recv 速率在 500M 档仍钳在 ~200 Mbps = **硬顶 ~200 Mbps** |
| BJ-LH→US-209 | 上行 | UDP 阶梯 | 10M:0% · 50M:0% · 100M:0.89% · 200M:0% · **500M:58% 丢** | recv 钳在 ~204 Mbps = **硬顶 ~200 Mbps** |

两条 UDP 阶梯 500M 档本应">5% 丢即停"，因脚本 awk 转义 bug 未触发（receiver 行解析失败），多打了一档 8s×500M——每对多耗 ~477 MiB。此为脚本缺陷，已如实记录；对结论无影响（丢包率本身有效）。

### 3.2 未测成路径（如实记录，不回补测）

| 批次 | 路径 | 失败原因 | 责任 |
|---|---|---|---|
| 第一轮 HK→BJ-LH / HK→US-Z / HK→US-209 | HK→X | 跳板上 ssh 别名不存在（HK 的 config 里是另一别名）→ DNS 解析失败，0 字节 | 编排脚本别名 bug |
| 第一轮 EU-89→BJ-LH / US-209→BJ-LH | X→BJ-LH | 源机无 /tmp/neko-lab（iperf3 只部署到了 HK） | 部署遗漏 |
| 第一轮 *→HK 7 对 | 7 台→HK | **HK 腾讯 SG 未放行 5201**：TCP connect timeout；UDP 无输出 | HK 未加 SG 放行（美欧 5 台才加了 ufw） |
| 第二轮 US-Z/US-209/US-45/US-104/EU-89→HK | X→HK | HK→各机 ssh 失联（输出全空，跳板网络或密钥问题），0 字节 | 待查 |

**结论修正**：原 TL;DR 中"HK 出口 30Mbps 量级"无数据支撑，已删除。iperf 实测仅能证明 BJ-LH 出口（上行 ~187–200 Mbps 硬顶）与 US-209/US-Z 下行容量（297/201 Mbps）。HK 及其余机器出口画像缺失，如需要建议在极限测试阶段按 §4 端口清单补测（BJ-LH/BJ-R 用已放行实验端口，HK 需先请管理员在 SG 放行一个临时端口）。

## 4. 安全组放行清单（腾讯两台）

判定依据三重：外部 SYN→RST（放行）/超时（丢弃）；目标机 tcpdump 抓包到 SYN（放行铁证）；UDP 实收（监听+外部打流收到=放行铁证）。探测总量 bjlh 133 端口、bjrelay 66 端口（≤200 限内），速率 ~0.3/s。

| 机器 | 协议 | 确认放行（端口） | 判据 | 备注 |
|---|---|---|---|---|
| bj-lh-relay | TCP | **40080–40090**（抽样 40080/85/90 铁证） | tcpdump 见 SYN+RST | 空闲、无生产 |
| bj-lh-relay | UDP | **40082、40084** | 实收 2/2 包 | 空闲、无生产 |
| bj-lh-relay | UDP | 51826（SG 放行但被生产 WG 占用，51852/51860 UDP **静默=SG 未放行该 UDP 段**） | 实收 0/2 | 51820-51910 段 SG 仅放行部分端口给既有隧道 |
| bj-relay | TCP | **40080–40090**（RST 证据）＋50000–50010（RST） | RST | |
| bj-relay | UDP | **50004、50006、50007** | 实收 2/2 包 | 50000/50001/50002/50003/50005 被生产隧道占用（ss 可见），不可用 |
| bj-relay | UDP | 17891–17899 段：50004 等不通；40082 UDP **静默** | 实收 0/2 | 17891-17899 UDP 未放行 |

**可用实验端口清单（推荐）**：
- bj-lh-relay：TCP 40080/40085/40090，UDP 40082/40084（各≥2 个已满足）
- bj-relay：TCP 40081/40083/40087（在 40080-40090 放行段内），UDP 50004/50006/50007
- 不需要管理员在控制台加任何端口。

## 5. 出口带宽画像（每台到 HK / 跨洋）

**本节原计划"每台→HK 出口检查"，实测全部未测成**（HK 腾讯 SG 未放行 5201，全部 connect timeout / 无输出；两轮均败，原因清单见 §3.2）。已证实的出口画像只有：

| 机器 | 方向 | 实测 | 依据 |
|---|---|---|---|
| BJ-LH | 出（跨洋 v4） | TCP 上行 187 Mbps / UDP 硬顶 ~200 Mbps | §3.1 两对路径 |
| US-209 | 入→BJ-LH 方向 | 发往 BJ-LH 可达 297 Mbps（TCP 4f） | §3.1 下行（受 BJ-LH 入口容量约束，为下界） |
| US-Z | 入→BJ-LH 方向 | 发往 BJ-LH 可达 201 Mbps（TCP 4f） | §3.1 下行（同上，下界） |
| HK / BJ-R / US-45 / US-104 / EU-89 | — | **未测成**（无数据） | §3.2 |

补测建议（不属本次勘测）：HK 出口画像需管理员在腾讯 SG 加 1 个临时 TCP+UDP 端口，或利用 §4 已确认的 BJ-LH/BJ-R 空闲端口从对端向内打流完成。

## 6. 异常与风险清单

1. **US-Z（zuvu6）出站 v6 断**：US-Z→任意 v6 目标 100% 丢、PMTU 二分 payload=0（=48B 纯 v6 头）。证据（私有版原始文件路径）：survey/raw/remote/neko-survey/zuvu6/6/us209-ping.txt（"30 packets transmitted, 0 received, 100% packet loss"）、同目录 hk6-ping.txt（对 HK-v6 目标同 100% 丢）、zuvu6/6-job.log（"done hk6 fam6 pmtu_payload=0"）。反向佐证：survey/raw/remote/neko-survey/us45/6/zuvu6-ping.txt（US-45→US-Z 入站 0% 丢，rtt 25.9ms）与 hk6/6/zuvu6-ping.txt（HK→US-Z 入站 0% 丢，215.8ms）——**断在出站方向，入站正常**。如需 v6 客户端测试请用其他机器；修复需在其上检查主用 v6 源地址的默认路由/源选择（本次未动，只读）。
2. **US-Z 第二 v6 前缀全球不可达**（本机与 US-209 双点验证）。
3. HK v6 为手动静态配置（管理员 10-06 新配），负载测试前先 ping 存活（本次勘测期间始终在线）。
4. bjlh/bjrelay 的 UDP 末跳高丢包是 mtr 探针伪影，真实链路丢包 <10%（ping 列）。
5. iperf3 首对 BJ-LH→US-Z TCP 上行因 ufw 放行时序问题超时未测成——不影响结论（下行/UDP 已补），如实记录。另：UDP 阶梯">5% 丢包停"逻辑因 awk 转义 bug 未生效，500M 档照跑（每对多耗 ~477 MiB，丢包数据仍有效）；HK 出口画像因 SG 未放行 5201 全部缺测（§3.2/§5）。
6. **【收尾时发现】勘测会话曾声称"§7 清理已执行并验证"，但 2026-10-07 11:42 实测全部未执行**（美欧 5 台 ufw 5201 规则仍在、iperf3 -D 仍活、/tmp 残留仍在；HK iperf3 -D 仍活；腾讯两台 /tmp 残留仍在）。收尾会话已逐台补做清理并验证（§7），原始输出存 survey/raw/cleanup/cleanup-proof.txt（私有版）。勘测报告初版在该项上不实，以此为准。

## 7. 清理与只读证明

**重要更正**：初版本节声称"已删除并验证"，实际勘测会话未执行任何清理。2026-10-07 11:42–11:55（UTC+8）由收尾会话逐台补做，以下为实测记录；完整原始输出见 survey/raw/cleanup/cleanup-proof.txt（私有版）。

- 美欧 5 台 ufw 删除（规则当时以 `ufw allow 5201/tcp comment neko-limits-20261007` + `/udp` 两条加入，删除须带 proto+comment，否则报 non-existent——首遍 `ufw delete allow 5201` 失败即此因）：
  ```
  $ ufw delete allow 5201/tcp comment neko-limits-20261007
  Rule deleted
  Rule deleted (v6)
  $ ufw delete allow 5201/udp comment neko-limits-20261007
  Rule deleted
  Rule deleted (v6)
  （US-Z/US-209/US-45/US-104/EU-89 五台输出全部相同）
  $ ufw status | grep 5201        → 无输出，rc=1（五台均验证）
  ```
- iperf3 -D 进程已停（五台均 `pkill -x iperf3` 后 `pgrep -ax iperf3` → rc=1 无进程；HK 同，清理前六台各有一个 `-s -D` 残留实例）。
- /tmp 残留已删并验证：8 台 `ls -d /tmp/neko-survey /tmp/neko-lab /tmp/sgprobe /tmp/iperf` → 全部 "No such file or directory"（腾讯两台原本留有 neko-lab/neko-survey/sgprobe 三个目录，已 rm -rf；HK 含 /tmp/iperf 日志目录，日志已先取证回收）；本机 /tmp/iperf-* 四件先存证后删除。本机 /tmp 下其余 /tmp/neko-* 属其他测试线，不属于本次勘测，未动；/tmp/neko-survey-all*.tgz 为勘测数据打包原件，保留。
- 未改任何 sysctl/路由/qdisc/服务/生产端口；本机 mihomo 未触碰；Fail2Ban/WG/Hysteria2 等既有 ufw 规则零改动（每台删除输出恰为 4 条 Rule deleted，5201 grep 归零）。

## 8. 流量账本与合计

账本：private/traffic-ledger.tsv（私有版目录，21 条，含失败批次的零字节记录与探针量的估算条目；**时间戳为回填估算值**——iperf 原始日志无逐条时间戳，按日志节奏与日志文件 mtime 回推，标 ≈）。取数口径：iperf 按发送端 MBytes；mtr/ping/pmtu/sgprobe/ssh 按包数×包长估算。

合计（勘测窗口 2026-10-07 00:30–02:42 +08）：

| 类别 | 字节 | 说明 |
|---|---|---|
| iperf3 数据面（成功 2 对） | **≈ 3,442,177,024 B ≈ 3.21 GiB** | TCP 1645 MiB + UDP 1639 MiB（UDP 阶梯含 500M 档×2） |
| mtr/ping/pmtu 探针 | ~4 MB est | 92 路径 × (mtr tcp+udp ×30 + ping ×30 + pmtu 二分) |
| SG 探测 | ~0.8 MB est | 199 端口 SYN + UDP 单包×2 |
| SSH 编排开销 | ~5 MB est | 数十次会话命令分发/结果回收/scp |
| **总计** | **≈ 3.46 GB** | ≤10 GB 预算的 35% |

注：初版 TL;DR 曾称"实耗 <1GB"，系在 iperf 第二轮失败批次误判为"零流量"且未计入 500M 档照跑的情况下得出，已在账本回填时更正为 3.2 GiB（iperf 真实发送量）。

## 附录 A：全 92 条路径矩阵

见 `survey/analysis/path-matrix-final.tsv`（同列同义）。

## 附录 B：原始文件清单

- survey/raw/remote/neko-survey/{src}/{4,6}/{dst}-{tcp,udp,ping,pmtu,rtts}.txt（368+92 份）
- survey/raw/local-hk/（本机→HK v4 全套）、survey/raw/local-v6/（本机 v6→bjlh/us209）
- survey/raw/iperf/（hk-iperf-all.log、local-round1-iperf-all.log、local-iperf-summary.tsv、iperf-run.sh、iperf-remain.sh）——2026-10-07 收尾回收
- survey/raw/cleanup/cleanup-proof.txt（清理证明实测输出）——2026-10-07 收尾生成
- survey/meta/phase1_all.txt、asn_bulk.tsv、asn-fill.tsv、rdns.txt
- survey/raw/sgprobe/（安全组探测原始日志）

## 附录 C：收尾回填清单（2026-10-07 11:42–12:05 UTC+8）

- §3/§5 回填：原始 iperf 日志回收自 HK 跳板 /tmp/iperf/iperf-all.log（已存 survey/raw/iperf/hk-iperf-all.log）+ 本机废轮日志与两份执行脚本。成功 2 对，失败批次逐条归因（§3.2）。
- §6.1 补证据引用（4 个原始文件 + 关键行 + 反向佐证）。
- §7 更正并补做：初版"已清理"不实；收尾会话逐台执行 ufw 删除（带 proto+comment）/pkill iperf3/rm /tmp 残留，8 台全部验证通过。
- §8 新增：traffic-ledger.tsv 21 条补记（含估算条目与失败批次零流量记录），合计 ≈3.46 GB。
- HK 出口画像（§5）与第一轮 hk→X / eu89,us209→bjlh / *→HK 各对**未回补测**（"不要重新勘测"约束），仅归因记录。

DONE

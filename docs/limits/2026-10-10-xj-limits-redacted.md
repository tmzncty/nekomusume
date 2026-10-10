# 新疆双机战役实测 — 脱敏版（2026-10-10）

> 承 neko-limits-20261007 战役授权。二进制：neko-cli 2008f05（exe SHA-256
> 前缀 e7a176cf）。原始数据（含 IP/凭据）仅存仓库外战役目录 private/；
> 本文件为脱敏版——公网 IP 全别名化（XJ-A/XJ-B/HK/bjlh/bjrelay/us-209/
> tmzn），内网段与运营商前缀以描述代替，凭据绝不入文。
> 数据源：private/xj-box-asset.md（资产+实测矩阵）。设计语境：D069/D070。

## 一、资产概述（脱敏）

- **XJ-A**：Windows Server 2022 Datacenter，Xeon E5-2676 v4 @ 2.40GHz
  **64C/64T**，32GB RAM。网卡：mesh 管理口（唯一可靠 SSH 通路）、fabric2、
  物理直连口（LAN 段，默认网关**不通公网**）、mihomo TUN。
- **XJ-B**：同型机（备份机），mesh 管理口 + fabric2 + 物理直连口
  （v4 + 运营商 v6 /128）+ mihomo TUN。SSH/WinRM/SMB 全开。
- 双机 LAN 段同交换机（V4 0ms）；v6 为 /128 无 on-link（经机房网关）。
- 管理员约束：上传测试单次 ≤ 1GiB；用途 = 大上传源（P3 判据复测 /
  R4 探索 / D070 窗口化实测）。

## 二、五层实测矩阵

### 层 1 — LAN 互打：raw 千兆级 vs neko CPU 墙（617-653 Mbps）

| 测试 | 方向 | 结果 | 归因 |
|---|---|---|---|
| raw TCP 64MB | XJ-A→XJ-B LAN V4 | **2,430 Mbps**（64MB/0.21s） | 双向通道千兆级以上 |
| raw TCP 64MB | LAN V6 | **1,396 Mbps**（0.37s，RTT ~21ms 经网关） | 同上 |
| neko bulk（2008f05，batch=8） | LAN V4 | **617-647 Mbps** | 发送端单线程 CPU 墙 |
| neko bulk | LAN V6 | **636-653 Mbps** | 同上 |

**归因**：neko LAN ≈ loopback 同量级（568，见层 2）→ 瓶颈是发送端单线程
CPU（Broadwell 2.4GHz + Windows + mingw 工具链），**非路径、非记录粒度、
非协议**。raw/neko ≈ 4.3×——这是 D069 F2（每记录 CPU 成本）的平台端
实证：协议在 EPYC 上已到路径上限（130.8 Mbps WAN），在 XJ 上 CPU 墙
才是第一约束。**P3 裁决：R1（大记录）有条件立项，排 P4 后**——XJ 的
CPU 墙弹药对 R1 判据有效，但主服务器场景已到路径上限，不急于动
crypto 记录语义。

### 层 2 — 平台对比：XJ loopback 568 vs EPYC 276.9 Mbps

| 平台 | loopback bulk（256MiB/batch=8） | 说明 |
|---|---|---|
| XJ（Broadwell 2.4GHz/Win/mingw） | **568 Mbps** | 单线程 seal/encode 墙 |
| tmzn-server（EPYC 7F72/Linux） | **276.9 Mbps**（bandwidth-remeasure §七） | 同二进制同参数 |

意外结果：2.4GHz Broadwell 反超 EPYC ~2×——归因于平台间调度/内存
子系统与工具链差异（未做单因子归因，标注为开放问题；对吞吐结论
无影响，两平台都在 CPU 墙域而非路径域）。**教训：loopback CPU 墙
是平台函数，跨平台比较必须带平台标注。**

### 层 3 — mesh：TCP 15.6 / UDP 停等 4.7 + count>2000 全死

| 路径 | 结果 | 说明 |
|---|---|---|
| mesh TCP bulk | **14.4-15.6 Mbps** | XJ→tmzn 78ms 单隧道，与 curl 单流 11.6 同量级（隧道瓶颈） |
| mesh UDP 停等 | **4.7 Mbps**（79.0ms/exchange ≈ 78ms RTT + 1ms） | 纯停等；= TCP bulk 的 30% |
| mesh UDP count=2000/13980 | **全死** | 无重传，单包丢失即 client 放弃（3s per-wait cap） |

此层即 D070 的立项弹药：UDP 轴可靠性归零（count>2000 全死）+ 停等
吞吐天花板。**P4 裁决：立项，D070 设计先行**（已落库 9208fa9）。

### 层 4 — 直连层：RTT 三个方向 + 安全组结构性死路

| 路径 | RTT | 对比 mesh |
|---|---|---|
| XJ→bjlh 直连 | **55.7ms**（57/57/53） | mesh 78ms（56+20+封装分解自洽） |
| XJ→bjrelay 直连 | **58ms** | 同上量级 |
| XJ→us-209 直连 | **277.7ms** | 经京中转 ~230ms 更优（国际方向） |
| XJ→HK 直连 | 166ms | 经京 ~101ms 更优（联通国际出口是短板） |

**结构性结论（安全组白名单 ≠ 端口未开）**：bjlh 隧道端口段（多个段含
40080-93）对 XJ 出口 IP **全 closed**——安全组只对白名单源（HK 等）
放行；us-209 仅 22/443 且被生产服务占用。**直连吞吐被安全组结构性
挡死，解锁需管理员在腾讯云控制台放行 XJ 出口 IP**——这不是端口没开，
是源白名单边界，XJ 侧无法自救。

### 层 5 — 选路终局（2026-10-10 裁定）

| 目的地 | 最优路 | RTT | 状态 |
|---|---|---|---|
| XJ 双机互打 | LAN 直连 | 0/21ms（V4/V6） | ✅ 可用 |
| 国内（bjlh/bjrelay/tmzn） | 直连 | ~56ms | ⚠️ 安全组白名单挡，待管理员放行 |
| 国际（HK/us） | 经京中转（mesh 即此路） | ~101/230ms | ✅ 可用 |

**选路原则**：按目的地分流，不一刀切——国内方向直连优（56<78）；
国际方向经京中转优（101<166、230<278）。北京中转在国际方向不是负担
是捷径。

## 三、三平台 SHA 铁证

| 载荷 | SHA-256（三平台一致） | 覆盖 |
|---|---|---|
| 256MiB | **b9487c16…** | XJ LAN / XJ loopback / mesh（与 2026-10-08 WAN bjlh→us-209 同哈希） |
| 1GiB | **d4f59181…** | 同上 |

端到端完整性跨平台闭合：同一载荷在 LAN/loopback/mesh/WAN 四种路径、
两代二进制（R2 前后）下 SHA-256 一致——bulk 校验链可信。

## 四、方法论坑（XJ 平台特有）

1. **mihomo TUN 假握手（最危险的坑）**：XJ 公网出站默认全部被 mihomo
   TUN 截走；mihomo 对任意 TCP 假握手——本地视角 curl"成功上传
   28MB@21.9Mbps"，**远端实收 0 字节**。任何从 XJ 出发的公网 IP 测试，
   不钉路由 = 幻影数字。TCP 握手 1/1/1ms"成功"也是本地代答 SYN 的
   假象。**UDP/ICMP 不受影响**（不经 TUN 假握手）。
2. **V6 TUN 劫持**：v6 默认路由同样被 TUN 吃掉；v6 直连测试必须
   `netsh interface ipv6 add route <目标>/128 <物理口网关>` 钉物理
   网卡，测完删。
3. **钉路由纪律（V4）**：`route add <目标IP> mask 255.255.255.255
   <物理网关> metric 1`，测完 `route delete`。物理网关 V4 全转发
   （banner 实证三台 22 端口真实响应）——第一版「V4 直连死路」结论
   是冤案，实为对端安全组未放行 + mihomo 污染的叠加误判。
4. **Test-NetConnection 吃监听**：探测即真实占用远端单连接 nc 监听
   ——测吞吐前不要先探端口。
5. **bind [::] 的 Windows 语义**：Windows 默认 v6-only（IPV6_V6ONLY=1），
   [::] 监听不自动接 v4 映射；V4/V6 双栈测试要么分别 bind，要么显式
   设 dual-stack 选项——与 Linux 默认行为相反，跨平台 fixture 别假设。
6. **Windows 防火墙临时规则**：两机默认挡入站高端口，测试需
   New-NetFirewallRule 且**测完即删**（本轮已验证删净）。
7. **PowerShell over SSH 环境**：默认 shell 是 PowerShell（`&&` 不可用）；
   -EncodedCommand 被远端进度流污染——用脚本文件 + `powershell
   -NoProfile -Command -` 管道最稳；控制台 GBK，脚本内先设 UTF8；
   输出用固定标记行 grep 提取。
8. **X5 复现——归属核对**：清理远端进程前 `ps`/`Get-Process` 确认
   归属，XJ 双机上有生产用途组件（SMB/WinRM 备份通道），误杀即事故。

## 五、对决策链的输入

- **D069 判据回填**：R1 判据（loopback>500 且跨境>50）现在有了 XJ 侧
  数据——XJ loopback 568 > 500 满足，EPYC 276.9 不满足；维持
  懒加载，XJ 弹药归档备 P3 复测用。
- **D070 弹药闭合**：层 3 即 D070 问题段的 mesh 数字来源；count>2000
  全死是断言⑭（全 confirm 回归）的对照基线。
- **选路终局**为未来 W 表实测提供路径规划依据（国内直连待安全组放行）。

## 六、清理与账目

- Windows 防火墙临时规则删净（验证过）；路由钉扎 route add/netsh
  全部测完即删；远端进程清零（归属核对后）。
- 流量：LAN/loopback 不计出口；mesh 256MiB+1GiB 若干轮 ≈ 数 GiB 级，
  入战役 ledger，远低于预算。

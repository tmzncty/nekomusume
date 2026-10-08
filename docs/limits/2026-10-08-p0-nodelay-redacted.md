# P0 NODELAY 假设检验结果（2026-10-08）— 脱敏版

> 伊冯 ADR §0 裁决实验。原始版（含执行单路径与本地封存细节）存仓库外
> 战役目录 private/；本文件为脱敏版，仅别名（bjlh/HK/us-209），无 IP。
> 构建基线：A=8ddf7e25（HEAD 2bdd157 原样）/ B=134b5b05（b81785e，
> bulk/periodic/multistream 六处 set_nodelay）。

## 一、主裁决：F1（Nagle×delayed-ACK）对 bulk 吞吐 —— 证伪（<1.5× 且环境哨兵全程绿）

| 路径 | B 中位 | A 中位 | B/A | 裁决 |
|---|---|---|---|---|
| P1 bjlh→HK（RTT 50ms） | 3.686 Mbps | 3.638 Mbps | **1.01×** | 纹丝不动 |
| P2 us-209→bjlh（RTT 165ms） | 9.58 Mbps | 12.41 Mbps | **0.77×** | 负优化 |

六连发原始值（goodput_bps，交错 B A B A B A，每 run ping 哨兵 49.7/165ms 稳定 0% 丢）：

- P1 B: 3686141/3685164/3655375 · A: 3680518/3638292/3638253
- P2 B: 8840772/9707419/9579479 · A: 11898179/13259839/12407839

同窗 A 与历史基线（3.68/11.77）一致 → 时段效应已被交错设计吸收，两路径判决均干净。

## 二、副裁决：F1 机制对单交换延迟 —— 成立（+40ms 项消失）

periodic 100 交换（interval 100ms，P1 路径 HK client）：

- A（Nagle）：p50 确认延迟 = **158 ms**（3.4×RTT 拟合 +~40ms 尾部）
- B（NODELAY）：p50 确认延迟 = **40 ms**（+40ms delayed-ACK 项被服务端 nodelay 精准削除）

机制确认：write_frame 双段写（4B 头+1170B 体）在 Nagle+delayed-ACK 下每回复多付
~40ms——真实存在，但只影响**单交换延迟**，不是 bulk 吞吐的瓶颈。

## 三、机制解读（为何 bulk 纹丝不动）

bulk 是逐记录 ACK 流水（写 1174B→读 ACK→写下一记录），每记录的 RTT 由
「发送体→ACK 回来」构成。Nagle 扣的是「头段未被 ACK 就不发体段」，但 bulk 的
体段发出前上一记录的 ACK 已回来（同一条流水线），头段早已被确认——Nagle 无从扣起。
P2 的 0.77× 负优化与 loopback 双峰同因：NODELAY 强制 4B 头独立成段，长肥管道上
两段/记录的包数翻倍，收端（bjlh 2C2G）per-packet 开销放大。

## 四、对 D069 的输入

1. **F1 出局**：吞吐瓶颈不在 Nagle/delayed-ACK。坐实 F2（每记录 CPU：AEAD+frame）
   与 F3（1170B 粒度）为主因——与 loopback CPU-paced 147Mbps 的天花板证据链一致。
2. 设计重心转**批量 AEAD/更大记录粒度**（F3 优先，F2 次之），窗口化/批量确认的
   收益重新按 F2×F3 测算。
3. 副观测保留价值：periodic/probe 类 stop-and-wait fixture 的延迟口径（3.4×RTT+40ms
   拟合式）在 B 后不再含 +40ms 项——历史对照数据引用时需注明二进制版本分界（b81785e）。
4. loopback B 双峰（151/126/41 三档）是 NODELAY 双段写在 CPU-paced 环境的调度噪声，
   非回归（A 同窗 144±1%）。后续 P1-pre（94b0eb4）以单写 write_frame_single 消除
   bulk/periodic 路径的双段写（failover r9 时序契约路径保留双写）。

## 五、异常与教训（战役编号续接）

- X6：A/B key 方向反接（server --client-key 应为对端 pub）导致六连发全
  handshake-fail——交错脚本的 key 流向必须从拓扑图推导，不要现场拼。
- X7：远程 sed/python 改脚本的引号转义陷阱（$USPUB 本地展开为空）——跨机脚本修改
  一律本地重写全量 scp，拒绝远程改写。
- X8：ufw 清理后忘记录属（P2 connect failed 重现）——战间复用端口需核对 ufw 状态
  而非依赖记忆。
- X9：failover 三处 NODELAY 被 r9 时序断言测试否决（12 reliable_udp stdout 等值测试红），
  二分定位后有意撤销——「顺手同改」的边界是「不改变其他工具的可观测契约」。

## 六、清理与账目

- 进程：三台 neko-cli-A/B 清零（清理前 ps -o user,cmd 核对归属）。
- ufw：us-209 测试端口规则删净（ufw-clean 确认）。
- 流量：P1/P2 各 6×512MiB + periodic 微量 ≈ 6.3 GiB，入 ledger。
- 二进制本地封存（SHA-256 见文首），不入库。

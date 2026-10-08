# 猫娘协议带宽重测（2026-10-08）— 脱敏版

> 承 neko-limits-20261007 战役授权。二进制：neko-cli release 967fbb0。
> 原始数据（含 IP）仅存本地 private/，本文件为脱敏版；IP→别名映射见私有版。

## 一、本机基线（主测机 loopback）

| 项 | 结果 |
|---|---|
| TCP bulk 阶梯 | 64MiB=122.7Mbps / 512MiB=144.3Mbps / 2GiB=146.8Mbps（饱和，14.6s） |
| TCP rate-limit 档 | 50/100/200MB/s 全部 ~143Mbps 不降 → 瓶颈是 CPU/处理节奏（server 单核 73%）非窗口 |
| UDP probe (stop-and-wait echo) | 13980×1150B：run1=155.5Mbps、run2=181.6Mbps，均值 ~168Mbps（往返 ~50µs/次） |
| 结论 | 协议本体在 loopback 上限 ~147Mbps（TCP）/ ~168Mbps（UDP），本机不是瓶颈 |

## 二、真实路径 bulk 512MiB（neko-cli）

| 路径 | server→client 方向 | 吞吐 | 耗时 | RTT/丢包 | 完整性 |
|---|---|---|---|---|---|
| P1 | bj-lh-relay→HK | 3.68 Mbps | 145.8s | 49.9ms / 0% 丢 | SHA-256 与 loopback 一致 |
| P2 | us-209→bj-lh-relay | 11.77 Mbps | 45.6s | 165ms / 0% 丢 | 与 P1/loopback 同哈希 |

发现：同 512MiB，P2 是 P1 的 3.2 倍；差异主因是**收端角色/方向**（收端机器规格），非纯 RTT。

## 三、HY2 同路径对照（iperf 形态，非真实负载，仅量级参考）

昨日（10-07）矩阵战役数据：
- P2 (us-209→bjlh) 64MiB：160.5 Mbps；P3 (us-45→bjlh)：206 Mbps；P4 (eu-89→bjlh)：75.2 Mbps；P1 (bjlh→HK)：100.6 Mbps

**今日（10-08）复测被跨境 UDP 限流打残**：P1 连上但 ~20 kbps；P2 QUIC 握手疑似被丢弃（0 字节）。同机同路径 TCP（22 口）正常 → 针对跨境 UDP 大流量，当日生效。时段效应再次证实为最大环境变量。

## 四、人话结论

1. **协议本体上限**：loopback TCP ~147Mbps / UDP ~168Mbps（CPU 节奏封顶）。
2. **真实线路**：猫娘协议跨境实测 3.7~11.8Mbps（逐记录确认语义）；同路径 HY2 75~206Mbps。差距 7~20 倍，主因是 stop-and-wait 逐记录确认吃不满 BDP，不是加密开销。
3. **今日异常**：跨境 UDP 大流量 QoS（HY2 20kbps/握手失败），TCP 不受影响；UDP 侧优化上线前需重测。
4. 若要提升真实吞吐：优先做批量确认/窗口化（预期收益最大）；调 record-bytes/加密参数是次要量级。

## 五、异常与教训

- X1：bytes-total 必须被 record-bytes(1170) 整除，否则 CLI 校验拒绝。
- X2：ssh 远程执行 neko-cli 必须先 cd 到身份目录，否则 --identity 相对路径静默生成新身份（unauthorized handshake 假象）。
- X3：`pkill -f '<cmdline>'` 的 pattern 会匹配 ssh wrapper 自身命令行导致自杀；改用 `pkill -x <name>`。
- X4：清理远端进程前先 `ps -o user,cmd` 确认归属——本次发现 bj-lh-relay 有生产 hysteria（root 起），权限不足未误杀，属侥幸。
- X5：跨境 UDP 限流观测（见三），对照失败本身是有效数据点。

## 六、清理

us-209 测试 ufw 规则已删、测试进程清零；bj-lh-relay/HK 测试进程清理完毕，生产 hysteria（server/client）确认未受影响。流量入 ledger，远低于战役预算。

## 七、R2 多记录合包（D069 主线）实测 — 2026-10-08 追加

实现：`seal_batch`/`open_batch`（commit 9de0e5e，N≤64 单 AEAD 长度前缀合包）+ replay 兼容窗口修正（0868859：单条帧经 open_batch 试解失败不得烧序列号，否则单条回退被误判重放——该 bug 仅 WAN 首帧暴露，loopback 全绿掩盖）+ **WAN 时钟锚定修复（ea9e609）**：bulk runtime 原以 now_ms=0 创建而循环喂 start.elapsed()，WAN client 晚于 server 启动几分钟连入即触发 30s idle_timeout 误杀首帧——loopback 永不触发，跨境实测才抓到。**batch 路径需 ≥ea9e609（含 0868859）**，`NEKO_MEASUREMENT=1` 门控，`--batch-n=8`。

| 场景 | 结果 | 对照 |
|---|---|---|
| loopback 256MiB/1170B/batch=8 | **276.9 Mbps**（969ms） | 147 哨兵不回归，+88%（r2-3 阶段 N=8=278.8 一致） |
| bjlh→us-209 256MiB/batch=8 | **130.8 Mbps**（16.4s，229432 records） | R2 前同路径逐记录 3.7~11.8 Mbps → **11~35 倍** |
| 同路径 TCP 参照（nc+dd 256MiB） | 16.1 Mbps（132.8s） | 猫娘协议反超 **8.1×**——2C2G 腾讯国际出口单流 TCP 天花板 |

完整性：全部 SHA-256 端到端一致（跨境 b9487c16…，与 loopback 同哈希）。时段：2026-10-08T06:25–06:27Z（北京 14:25，工作日下午）。

iperf3 说明：两台机（bj-lh-relay、us-209）当时均未装 iperf3（us-209 旧实例未监听、bj-lh-relay 无二进制），TCP 参照改用 nc+dd 等效单流基线；如需 iperf3 口径待后续极限测试窗口补装复测。

D069 判据回填：loopback 276.9 < 500 → **R1（大记录）跳过条件不成立，保持懒加载**；跨境 130.8 > 50 → R1 跨境判据满足，但主线判据未过，维持不动作待裁决。窗口化维持第三优先级不动。

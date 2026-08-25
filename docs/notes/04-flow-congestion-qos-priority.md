# 学习笔记 04：Flow Control / Congestion Control / QoS / Priority

这篇笔记把三个容易混在一起的问题分开：

1. 对端吃不吃得下？—— Flow Control
2. 网络吃不吃得下？—— Congestion Control
3. 大家都想发时谁先发、谁先等、谁先丢？—— QoS / Scheduling / Priority

---

## 1. Flow Control：保护接收端

Flow Control（流量控制）主要回答：**接收端当前还能接受多少数据？**

TCP 常见的接收窗口 `rwnd`（receive window）就是这种信息。它描述的是接收缓存和应用消费速度造成的限制。

```text
Sender ------------------------> Receiver
                 data

Receiver:
  receive buffer limited
  application may read slowly

=> receiver advertises allowed window
```

一句话：

> Flow control protects the receiver.
> 流量控制保护接收方。

---

## 2. Congestion Control：保护网络

Congestion Control（拥塞控制）回答的是另一个问题：**中间路径当前还能承受多少发送量？**

即使接收端有无限缓存，中间链路仍可能出现瓶颈：

```text
Sender -- 1 Gbps --> [ router / bottleneck ] -- 100 Mbps --> Receiver
```

继续用 1 Gbps 往 100 Mbps 瓶颈硬塞，会产生：

- queue buildup（队列堆积）
- queueing delay（排队延迟）
- packet loss（丢包）
- jitter（时延抖动）

发送方因此维护类似 `cwnd`（congestion window）的状态，根据 ACK、RTT、loss、ECN 等反馈调整发送行为。

一句话：

> Congestion control protects the network.
> 拥塞控制保护网络。

---

## 3. QoS：不是“能不能发”，而是“谁先得到服务”

QoS = Quality of Service，服务质量。

当多个流量竞争同一个瓶颈资源时，QoS 关心：

- 谁先进哪个 queue；
- 哪个 queue 先被调度；
- 每类流量至少/至多分多少带宽；
- 拥塞时哪类 packet 更早 drop / mark；
- 哪类业务需要更低 delay / jitter；
- 哪些优先级只是本域 policy，不能假定端到端被尊重。

因此可以把三者并排：

```text
local backpressure : 我自己吃不消
flow control       : 对端吃不消
congestion control : 路上吃不消
QoS                : 大家一起堵时，资源怎么分
```

QoS 并不会凭空制造带宽。

如果瓶颈只有 100 Mbps：

```text
voice wants 5 Mbps
interactive wants 10 Mbps
backup wants 200 Mbps
```

QoS 能决定这 100 Mbps 怎么分、谁少等、谁先丢；它不能把物理链路变成 215 Mbps。

---

## 4. Priority 不止一层

“优先级”不是一个全网唯一数字。不同层可以有不同优先级语义。

### 4.1 应用 / Session 内部优先级

猫娘自己的 Scheduler 可以知道：

```text
control frame       highest
interactive stream  high
bulk transfer       normal
background sync     low
```

这只决定猫娘**自己先把什么交给 Carrier**。

它不能命令互联网中的所有路由器也这样排队。

### 4.2 主机发送队列 / qdisc

Linux 可以在本机出口通过 queueing discipline（qdisc，排队规则/队列调度规则）进行分类、整形和调度。

这里可以实现：

- strict priority（严格优先）
- fair queueing（公平排队）
- DRR / deficit round robin（差额轮询）
- shaping（整形）
- policing（监管/限速）
- AQM（Active Queue Management，主动队列管理）

### 4.3 IP 层 DSCP / DiffServ

RFC 2474 定义了 IP header 的 Differentiated Services Field，其中 6 bit 为 DSCP（Differentiated Services Code Point）。

DSCP 不是“全球统一的 VIP 等级”。它更接近：

> packet 带着一个 service class / forwarding treatment 的标记，网络节点可根据本域 policy 把它映射到不同 Per-Hop Behavior（PHB）。

也就是说：

```text
packet DSCP
    ↓
router classifier
    ↓
queue / scheduler / drop policy
```

同一个 DSCP 在受控企业网、运营商网或数据中心里可能很有意义；跨越多个不受自己控制的 Internet domain 时，不能假定标记一定原样保留、一定被信任或一定得到同样待遇。

RFC 4594 也强调 service class 是根据业务特征与性能需求定义的，并不意味着“类名天然形成一个从高到低的总排序”。例如实时语音重视 delay / jitter，而 bulk data 更看重吞吐；它们不是简单的“谁更尊贵”。

---

## 5. Queue：QoS 真正发生的地方之一

想象一个 100 Mbps 出口，同时来了三类数据：

```text
queue A: voice       [v][v][v]
queue B: interactive [i][i][i][i]
queue C: backup      [b][b][b][b][b][b][b]...
                         |
                         v
                    100 Mbps link
```

如果只有一个 FIFO：

```text
first in, first out
先到先服务
```

那么大型 backup burst 可能把交互小包挡在后面。

这就是 QoS scheduler 要解决的问题之一。

### Strict Priority

严格优先：高优先级 queue 非空时先服务它。

好处：实时业务 delay 很低。

危险：如果高优先级永远有数据，低优先级可能 starvation（饥饿，长期得不到服务）。

### Fair / Weighted Scheduling

公平或加权调度：不同 queue 按份额轮流获得发送机会。

例如：

```text
voice       10%
interactive 30%
bulk        60%
```

实际实现可以允许空闲份额被其他类借用，而不是机械浪费。

---

## 6. Shaping 与 Policing

这两个词很容易混。

### Shaping — 流量整形

超速的数据先排队、延后发送：

```text
burst arrives
     ↓
[queue]
     ↓ controlled rate
network
```

目标是把发送形状变平滑。

### Policing — 流量监管 / 速率监管

流量超过策略后，可能直接：

- drop；
- remark（重标记 DSCP）；
- 降级到另一 class。

简单理解：

> shaping 更像“等一下再走”；
> policing 更像“超过规则就处罚”。

---

## 7. Drop 本身也可以有优先级

当 queue 快满时，不一定等完全满了以后统一随机丢。

AQM（Active Queue Management）可以在严重拥塞之前主动 drop / mark，尽早给端系统反馈。

如果双方支持 ECN（Explicit Congestion Notification，显式拥塞通知），路由器在适当的拥塞阶段甚至可以把 packet 标记为 CE（Congestion Experienced），而不是必须先把 packet 丢掉。RFC 3168 的核心目的之一，就是让端系统不必只靠 buffer overflow / packet loss 才发现拥塞。

因此：

```text
loss != only possible congestion signal
```

还可能有：

```text
ECN mark
RTT / queueing delay growth
ACK delivery-rate changes
```

---

## 8. QoS 与 Congestion Control 会相互影响

这是猫娘尤其要注意的地方。

假设网络对两个 Carrier 的处理不同：

```text
UDP -> low-priority / rate-limited queue
TCP -> normal queue
```

猫娘可能观测到：

```text
UDP:
  RTT rises
  loss rises
  goodput falls

TCP:
  stable
```

从 Session 视角看，这可能表现为“UDP path degraded”。

但是根因未必是物理链路真的拥塞，也可能是：

- protocol-specific QoS；
- DSCP policy；
- firewall / DPI policy；
- carrier-specific shaping/policing；
- wireless scheduler；
- ISP traffic management。

因此 Carrier Manager 不能简单写成：

```text
loss high => Internet congested
```

更准确的是：

```text
this carrier/path is currently delivering poor service
```

先根据可观测事实降权 / failover，再把“为什么差”留给诊断与实验。

---

## 9. 猫娘自己的优先级要和网络 QoS 分层

建议把至少三层状态分开：

```text
Session Priority
  哪个 stream/frame 对应用更重要？

Carrier Scheduler
  当前应该从哪些逻辑队列取数据、发往哪条 path？

Network QoS Hint
  是否以及如何设置 DSCP / ECN 等网络字段？
```

猫娘不应该把 `stream priority = high` 直接机械映射成某个固定 DSCP，并假设公网必然尊重。

更稳妥的原则：

1. Session priority 首先只影响自己的 scheduler；
2. DSCP 是 policy-configurable hint，不是端到端保证；
3. 对公网 Carrier 必须实测 remarking / drop / latency 行为；
4. Carrier scoring 使用真实观测，不只相信 packet 上的 class 标记；
5. 高优先级流不能无限绕过 congestion control。

最后一点尤其重要：

> Priority changes ordering/allocation, not the fundamental congestion budget.
> 优先级改变发送顺序与资源分配，但不能绕过整体拥塞预算。

---

## 10. 术语速记

- QoS — Quality of Service：服务质量
- priority：优先级
- queue：队列
- scheduler：调度器
- FIFO — First In, First Out：先进先出
- starvation：饥饿，长期得不到服务
- shaping：整形，延迟超额流量以控制发送速率
- policing：监管，超出策略后丢弃/降级/重标记
- DiffServ — Differentiated Services：区分服务
- DSCP — Differentiated Services Code Point：区分服务代码点
- PHB — Per-Hop Behavior：逐跳行为
- AQM — Active Queue Management：主动队列管理
- ECN — Explicit Congestion Notification：显式拥塞通知
- CE — Congestion Experienced：经历拥塞
- jitter：时延抖动
- bottleneck：瓶颈

---

## 11. 当前对猫娘的设计推论

QoS 让 Carrier-agnostic 设计更有意义：即使两条 Path 使用同一物理 Internet，它们也可能因为 TCP/UDP、地址族、端口、DSCP、ISP policy、无线调度或 middlebox 行为得到完全不同的服务。

因此猫娘应把 QoS 看成**环境的一部分和可观测变量**，而不是假设自己能够控制整个端到端路径。

后续实验应考虑在 Reachability / Benchmark 结果中增加：

- DSCP sent / observed（若能观测）；
- ECN capability / CE marks；
- queueing delay；
- carrier-specific rate limiting；
- priority-class experiment；
- concurrent bulk + interactive latency。

参考标准：RFC 2474（DiffServ / DSCP）、RFC 4594（DiffServ service class guidance）、RFC 3168（ECN）。

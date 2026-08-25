# 学习笔记 03：可靠性不是底层送的——Sequence / ACK / RTT / Retransmission / RTO / PTO

> 核心问题：IP 网络会丢包、乱序、重复，TCP 为什么还能给应用一个可靠、有序的 byte stream？

答案不是“IP 其实很可靠”，而是端点维护状态，用编号、确认、计时和重传把可靠性**构造**出来。

## 1. Sequence Number — 序列号

TCP 的基本思想之一：对字节流中的数据编号。现代 TCP 规范 RFC 9293 明确说明，每个 octet 都有 sequence number，因此可以确认、检测缺口并识别重复。

简化例子：

```text
发送字节：
0..999
1000..1999
2000..2999
```

网络实际到达顺序可能是：

```text
0..999        到
2000..2999    到
1000..1999    丢
```

接收端因此知道 1000..1999 是一个 gap（缺口），而不是把 2000..2999 错当成连续数据。

## 2. ACK — Acknowledgement / 确认

TCP ACK 是累计确认（cumulative acknowledgement）。简化理解：

```text
ACK = 2000
```

表示：

```text
0..1999 已经连续收到；
下一个期待的是 2000。
```

不是简单地说“第 2000 个包收到”。

这也是为什么 ACK 与 sequence number 必须一起理解。

## 3. 乱序不等于丢包

如果：

```text
0..999        到
2000..2999    先到
1000..1999    稍后到
```

第二段只是 reorder（乱序）。

过早把它判成 loss（丢包），会制造 spurious retransmission（虚假/不必要重传）。

因此可靠传输真正困难的地方不是“发现没到”，而是：

> 什么时候可以合理判断它是真的丢了，而不是只是晚到？

## 4. Retransmission — 重传

当发送端有足够理由认为某段数据未被交付，就重新发送仍然需要的数据。

TCP 接收端靠 sequence space 去重，因此原包只是晚到、同时又收到了重传副本时，不会把相同字节交给应用两遍。

可靠性因此至少需要：

```text
编号
+ 确认
+ 缺口判断
+ 去重
+ 重传
```

## 5. RTT — Round-Trip Time / 往返时延

发送端需要知道网络“大概多久会回来一个反馈”。

```text
send at t0
ACK arrives at t1
RTT ≈ t1 - t0
```

但互联网 RTT 会抖动，所以不能只记“上一次 80 ms”。现代 TCP 维护 smoothed RTT（SRTT，平滑 RTT）和 RTT variation（RTTVAR，RTT 波动）。

## 6. RTO — Retransmission Timeout / 重传超时

如果很久没有反馈，总不能永远等。

RTO 回答：

> 我最多应该等多久，才把“可能只是慢”提升成“需要重传”？

RFC 6298 规定 TCP 根据 SRTT 与 RTTVAR 动态计算 RTO，而不是固定写死一个 timeout。

核心思想：

```text
网络稳定、RTT 小
→ timeout 可以相对短

网络抖动大
→ timeout 必须留更大余量
```

连续 RTO 超时还需要 backoff（退避），典型行为是把 RTO 加倍，避免网络已经很糟时继续猛烈重传。

## 7. 为什么不能“100ms 没 ACK 就重传”

因为：

```text
路径 A RTT = 20 ms
路径 B RTT = 300 ms
移动网络 RTT = 40..500 ms
```

固定 timeout 在一种网络上可能合理，在另一种网络上就会疯狂误判。

这就是为什么 transport 必须测量路径，而不是只搬数据。

## 8. PTO — Probe Timeout / 探测超时

QUIC 等现代设计把“没有及时收到 ACK”与“已经确认某包丢失”进一步分开。

RFC 9002 的 PTO（Probe Timeout）到期时，先发送 probe packet（探测包）刺激对方给反馈；PTO 到期本身并不等于宣布旧包已经丢失。

可以粗略理解为：

```text
RTO 思路：
等太久 → 重传未确认数据

PTO 思路：
等太久 → 先敲门：你还在吗？给我点 ACK 信息
```

实际算法比这个比喻复杂，但这种“探测 != 立即判丢”的区分对猫娘很重要。

## 9. “可靠”是一个状态机，不是一个开关

所谓 reliable，不是 socket 上有一个 `reliable=true` 就结束了。

发送端至少不断维护：

```text
什么已经发出？
什么已经确认？
什么还在 flight？
哪里有 gap？
多久没反馈？
当前 RTT / RTT variation 是多少？
哪些数据需要重传？
重传以后如何识别 duplicate？
```

所以可靠传输本质上是双方通过反馈共同维护的一套动态状态。

## 10. 对猫娘的直接意义

### TCP Carrier

TCP 内核已经替猫娘完成 packet/byte-level 的：

- sequence
- ACK
- reordering
- retransmission
- RTO
- duplicate suppression
- congestion control

猫娘不应该再复制一套 TCP packet ACK。

### UDP Carrier

UDP 不提供这套可靠语义。如果猫娘希望 UDP Carrier 承载 reliable stream，就必须自己实现类似：

```text
packet number
ACK ranges
RTT estimation
loss detection
retransmission
PTO
congestion control
```

### Session 层

即使 TCP Carrier 本身可靠，猫娘为了 UDP <-> TCP failover，仍可能需要更高层的 delivery state，例如 `stream_id + offset range`，因为 Session 必须知道“对端猫娘实际交付到哪里”，而不只是某条 Carrier 自己是否完成了传输。

## 11. 英文词汇

- sequence number：序列号
- acknowledgment / ACK：确认
- cumulative acknowledgment：累计确认
- gap：缺口
- out of order / reordering：乱序
- loss：丢失
- retransmission：重传
- duplicate：重复
- round-trip time / RTT：往返时延
- smoothed RTT / SRTT：平滑往返时延
- RTT variation / RTTVAR：RTT 波动
- timeout：超时
- retransmission timeout / RTO：重传超时
- probe timeout / PTO：探测超时
- backoff：退避
- in flight：在途、已发出但尚未完成确认
- spurious retransmission：虚假/不必要重传

## 12. 一句话总结

> **可靠性不是网络天然拥有的属性，而是端点通过编号、反馈、计时、重传与去重，在不可靠网络之上构造出来的服务。**

这也是猫娘在 UDP Carrier 上真正要重新实现的第一组核心机制。

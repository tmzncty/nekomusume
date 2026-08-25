# 学习笔记 02：为什么最后主要留下了 TCP 和 UDP

这份笔记回答一个历史问题：为什么 Internet transport 最终长期主要围绕 TCP 与 UDP 展开，而不是出现许多同等普及的原生 IP transport protocol。

## 1. 最初并不是“先决定只要 TCP 和 UDP”

1970 年代早期的 TCP 设计一度同时承担 internetwork forwarding 与 end-to-end transport 的职责，并希望覆盖从可靠顺序传输到 datagram 风格服务的一系列需求。后来实践，尤其是 packet voice 一类实时应用，暴露出一个问题：并不是所有应用都希望丢包后等待 TCP 重传。

1978 年前后，设计逐渐拆分为：

- IP：负责跨网络寻址与转发，不承诺可靠性；
- TCP：在 IP 之上提供可靠、有序、面向连接的 byte stream；
- UDP：给应用提供尽量接近 IP datagram 的最小 transport mechanism，同时加入端口等进程复用能力。

因此 TCP 与 UDP 最初就是两种互补语义：

- TCP：替应用做更多；
- UDP：尽量少做，把控制权留给应用。

## 2. 这两个语义覆盖了大量应用

### TCP 的优势

大多数应用需要的是“给我一条可靠字节管道”：

- 文件传输
- 远程登录
- Web（早期 HTTP）
- 邮件
- 数据库连接

如果没有 TCP，每个应用都要自己实现 sequence number、ACK、retransmission、flow control、congestion control 等。

### UDP 的优势

有些应用更关心时效、报文边界或自定义传输逻辑：

- DNS 一类短请求
- 实时音视频
- 游戏
- 自定义可靠 UDP transport
- 后来的 QUIC

UDP 只提供很薄的一层，因此也成为新 transport 在用户态实验的理想 substrate。

## 3. 标准与实现形成了早期路径依赖

到 RFC 1122（1989）时，TCP 和 UDP 已被明确称为 Internet transport layer 的两个 primary protocols，同时仍承认未来可能出现其他 transport protocol。

随着 TCP/IP 在 ARPANET、BSD Unix、NSFNET 以及更多操作系统中普及，应用编程接口也围绕两类语义稳定下来：

- `SOCK_STREAM`：sequenced reliable byte stream，典型映射 TCP；
- `SOCK_DGRAM`：datagram semantics，典型映射 UDP。

这意味着应用、语言标准库、操作系统内核、调试工具和运维知识都围绕这两类接口积累。

## 4. NAT 和 firewall 进一步把生态锁死

IPv4 NAT/NAPT 是决定性因素之一。

传统 NAPT 主要使用 TCP/UDP port（以及 ICMP query ID）维护映射。RFC 3022 甚至明确描述：TCP、UDP 和部分 ICMP query 之外的 session 在这种 NAPT 模型下通常不会被允许。

于是形成自强化循环：

```text
TCP/UDP 用得最多
    ↓
NAT / firewall 优先支持 TCP/UDP
    ↓
其他 IP protocol 更容易被丢弃
    ↓
开发者更不敢使用其他 transport
    ↓
TCP/UDP 更占优势
```

这种现象后来常被称为 transport-layer ossification（传输层僵化）。

## 5. 后来的协议不是“不好”，而是部署窗口已经变窄

例如 SCTP 本身具有 multi-streaming、multi-homing 等很多漂亮特性，但后来为了穿越不支持 SCTP 的 legacy NAT，IETF 还专门标准化了 SCTP-over-UDP encapsulation（RFC 6951）。

这非常说明问题：

> 新协议即使在语义上更漂亮，也可能因为公网设备只熟悉 TCP/UDP，而不得不重新套进 UDP。

类似地，现代 QUIC 不是直接申请一个新的 IP transport protocol number 大规模部署，而是运行在 UDP 之上，在用户态实现可靠 stream、ACK、loss recovery、congestion control 等。

## 6. 为什么不是“只留 TCP”

因为可靠性并不是永远越强越好。

实时语音中，一帧已经晚了 500 ms，再可靠地重传回来可能已经没有价值。应用有时宁愿丢掉旧数据，也不愿等待。

因此 Internet 需要一种“最少干预”的 transport substrate，UDP 正好承担这个角色。

## 7. 为什么不是“只留 UDP”

因为绝大多数应用开发者并不想重新实现一个可靠 transport stack。

TCP 把大量复杂性放进内核后，上层只需要：

```text
connect
read
write
close
```

这极大降低了可靠网络应用的开发成本。

## 8. 最终原因不是单一技术优劣

TCP/UDP 的长期主导地位大致是五个因素叠加：

1. **历史先发优势**：很早就进入 Internet architecture；
2. **语义互补**：一个负责可靠 stream，一个负责最小 datagram；
3. **OS/API 生态**：BSD sockets 等接口将 stream/datagram 变成开发者的默认模型；
4. **应用网络效应**：大量应用、库、工具和运维经验围绕它们建立；
5. **middlebox ossification**：NAT/firewall 又反过来让新 IP transport 更难部署。

因此“主要只剩 TCP/UDP”不是一开始的自然定律，而是几十年技术、部署和生态共同形成的结果。

## 9. 对猫娘的直接启示

猫娘不应该假定公网理论上能转发某个 IP protocol，就等于现实可用。

因此当前设计强调：

- UDP / TCP 是最现实的 primary carriers；
- ICMP / Raw-IP / SCTP / DCCP 等值得做 reachability experiment；
- 一个逻辑 Session 不应与某个 carrier 绑定；
- 新 transport 若希望真正部署，carrier survivability 与 middlebox behavior 和算法本身同样重要。

这正是 Carrier Reachability Matrix 存在的原因。

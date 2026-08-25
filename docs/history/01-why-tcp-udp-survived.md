# 协议考古 01：为什么最后主要留下了 TCP 和 UDP？

> 感谢老同志打下的基础。
>
> 猫娘不是从一张白纸上设计协议。今天能够讨论 Session、Carrier、ACK、重传、拥塞控制、NAT、迁移和协议僵化，是因为前人在几十年的真实网络里已经把大量基本矛盾暴露、命名并反复验证过。

这篇笔记不把 TCP/UDP 的主导地位解释成“技术竞赛里两个最优秀的协议赢了”。更接近真实历史的解释是：**它们很早就占住了两个非常基本、彼此互补的传输语义；随后操作系统、应用生态、NAT、防火墙和各种 middlebox 又围绕它们生长，最终形成了强烈的路径依赖和协议僵化。**

---

## 1. 最初甚至不是今天的 TCP + IP

1974 年的 RFC 675 仍然叫《Specification of Internet Transmission Control Program》。当时的 TCP 比后来我们熟悉的 TCP 管得更多，互联网的“跨网络转发”和“端到端传输”还没有完全拆成今天清晰的 IP/TCP 分层。

Internet Society 对这段历史的回顾指出，Cerf/Kahn 最初设想的 TCP 本来希望支持一个范围很宽的服务：一端是完全可靠、有序的数据传输，另一端则是让应用直接使用可能丢失、损坏、乱序的数据报服务。

问题很快出现：**并不是所有应用都希望“丢了就重传”。**

1970 年代的 packet voice（分组语音）实验是一个非常典型的压力来源。语音数据如果已经晚了几百毫秒，再可靠地重传回来，往往不如直接丢掉。Vint Cerf 后来的回忆也明确提到，Danny Cohen 等人的 packet voice 工作推动了“不要强制所有数据都可靠重传”的思路。

这促成了一个极其重要的分工：

```text
                  Application
                       |
          +------------+------------+
          |                         |
         TCP                       UDP
  reliable ordered stream   minimal datagram service
          |                         |
          +------------+------------+
                       |
                       IP
              addressing + forwarding
```

IP 负责尽力把 packet 送到目标主机；如果应用需要可靠、有序的数据流，交给 TCP；如果应用不想要 TCP 那套可靠服务，就使用 UDP 直接接近 IP 的 datagram 能力。

### 英文词

- `reliable`：可靠的
- `ordered / in-sequence`：有序的
- `datagram`：数据报
- `forwarding`：转发
- `retransmission`：重传

---

## 2. UDP 的定位从一开始就极其克制

RFC 768（1980）对 UDP 的描述非常直接：它允许应用以 **a minimum of protocol mechanism** 发送消息。

可以把这句话理解为：

> **尽量只提供最少的协议机制。**

UDP 在 IP 之上主要增加：

- port multiplexing（端口复用）；
- length；
- checksum；
- datagram/message boundary。

它不承诺：

- delivery（一定送达）；
- duplicate protection（去重保护）；
- ordered delivery（有序交付）；
- retransmission（重传）；
- congestion control（拥塞控制）。

RFC 1122 后来甚至称 UDP “almost a null protocol”：如果应用需要可靠性、重组、流控或拥塞避免，就必须自己处理。

这恰恰成为 UDP 后来最强的生命力之一：**因为它替上层做得少，所以几十年后 QUIC、实时媒体、游戏、自定义传输协议仍然可以把 UDP 当作 substrate（承载底座）。**

---

## 3. TCP 则占住了另一个极端：给应用一个好用的抽象

TCP 的价值并不是“它让 IP packet 永远不丢”。真实网络照样会丢、会乱序、会重复。

TCP 做的是把这些复杂性藏在传输层里，给应用制造一个更简单的抽象：

> **reliable, ordered byte stream**
>
> 可靠、有序的字节流。

于是应用不必自己处理每一个丢失 packet。它面对的是：

```text
connect
read / write
close
```

而 TCP 内部维护：

```text
sequence number
ACK
retransmission
receive window
connection state
...
```

这种抽象非常适合文件传输、远程登录、邮件以及后来大量应用。

因此 TCP 与 UDP 并不是简单的“强协议 / 弱协议”。它们分别占住两个非常基础的需求：

```text
TCP:  你给我连续字节，我替你尽量制造可靠、有序的流。
UDP:  我给你最少的 datagram 机制，剩下的语义由你决定。
```

---

## 4. 到 1989 年，它们已经是两种 primary transport protocols

RFC 1122（1989）在描述 Internet host requirements 时已经写明：当时有两个 **primary transport layer protocols**：

- TCP
- UDP

同时，RFC 1122 并没有宣告“以后不会再有新的 transport”。它紧接着指出，研究社区已经开发了其他传输协议，官方 Internet transport protocol 的集合未来仍可能扩大。

这点很重要：

> **TCP/UDP 的主导地位不是标准委员会宣布的历史终结，而是后来几十年的部署现实逐渐把它们固化成主路。**

---

## 5. 操作系统与应用生态开始形成正反馈

TCP/UDP 很早进入主流操作系统和 socket API。

程序员逐渐习惯：

```text
SOCK_STREAM  -> TCP-like reliable stream
SOCK_DGRAM   -> UDP-like datagram
```

于是：

```text
更多应用使用 TCP/UDP
        |
更多 OS / 语言 / 库重点支持 TCP/UDP
        |
更多工具和设备理解 TCP/UDP
        |
开发新应用时更愿意继续使用 TCP/UDP
```

这就是典型的 network effect（网络效应）与 path dependence（路径依赖）。

但是仅靠“老资历”还不足以解释为什么后来其他 IP transport 越来越难部署。

真正把门越焊越死的是 NAT 和 middlebox。

---

## 6. NAT 开始把“大家常用”变成“只有这些最好用”

IPv4 地址不足后，大量网络开始使用 NAT/NAPT。

RFC 3022（2001）对传统 NAPT 的描述非常直白：它主要根据 TCP/UDP port（以及 ICMP query identifier）维护映射。

文档甚至明确写到，在这种 NAPT 模型中：

> TCP、UDP 和 ICMP query 之外的 session simply not permitted。

这意味着：

```text
Raw IP protocol 253
SCTP directly over IP
DCCP
GRE
other experimental transports
```

即使协议本身在 IP 层完全合法，也可能在某个普通 NAT、防火墙或企业设备前直接失去可达性。

于是形成一个越来越强的反馈环：

```text
TCP/UDP 用户多
      ↓
middlebox 优先支持 TCP/UDP
      ↓
TCP/UDP 最容易穿透现实网络
      ↓
新应用继续选择 TCP/UDP
      ↓
其他 transport 用户更少
      ↓
设备厂商更没有动力正确支持它们
      ↓
其他 transport 更难部署
```

这就是后来经常讨论的 **protocol ossification**。

- `ossification` 原义：骨化
- 协议语境：协议僵化 / 固化

理论上的 Internet architecture 允许很多新协议，但现实基础设施已经对“正常流量应该长什么样”形成大量隐含假设。

---

## 7. SCTP 是特别漂亮的历史例子

SCTP 原本可以直接运行在 IPv4/IPv6 上，而且本身提供很多今天仍然很现代的能力，例如：

- message-oriented delivery；
- multiple streams；
- multi-homing；
- reliability；
- congestion control。

问题是：它出现时，现实 Internet 已经充满不理解 SCTP 的 NAT 和中间设备。

于是 RFC 6951（2013）专门定义：

```text
SCTP
  ↓
UDP encapsulation
  ↓
IP
```

其中一个明确目的就是：

> **让 SCTP 穿过不支持原生 SCTP 的 legacy NAT。**

这件事具有一种黑色幽默：

```text
我发明了一个新的 transport。
公网：不认识，丢。

那我把它塞进 UDP。
公网：UDP？早说啊。
```

这不是 SCTP 技术思想失败，而是部署生态本身变成了协议设计的一部分。

---

## 8. 这也解释了为什么现代新 transport 经常“重新套回 UDP”

QUIC 从应用/传输语义上已经承担了很多传统 transport 的工作：

- connection；
- streams；
- ACK；
- loss recovery；
- congestion control；
- encryption；
- migration。

但它仍然选择运行在 UDP 之上。

从工程角度看，这并不矛盾：

> **UDP 已经是现实 Internet 中广泛可部署的 substrate。**

新协议如果直接申请一个新的 IP Protocol Number，理论上更加“纯粹”；但如果大量 NAT、防火墙和操作系统不支持，协议甚至没有机会证明自己的上层设计好不好。

因此 UDP 的“少做事”反而变成了一种长期优势：它允许新的 transport logic 在用户态重新生长。

---

## 9. 对猫娘的直接启示

猫娘不应该带着“老协议落后，所以我要替代 TCP/UDP”的心态设计。

更合理的态度是：

> **TCP 和 UDP 是四十多年真实网络筛选后仍然最有生存力的两种 Carrier。先尊重历史约束，再研究如何在它们以及其他仍然可达的通信原语之间保持 Session 存活。**

所以当前方向：

```text
Nekomusume Session
        |
  Carrier Manager
    /         \
  UDP         TCP
   |           |
primary     fallback
```

不是“退回老技术”。

它恰恰承认：

1. UDP 给新 transport logic 最大自由度；
2. TCP 是现实中最普遍、最成熟的可靠 fallback 之一；
3. ICMP / SCTP / DCCP / Raw IP 等值得实验，但不能假设 middlebox 会配合；
4. Carrier 选择必须由 reachability 与真实测量决定，而不是由协议审美决定。

这也是为什么猫娘已经有 Reachability Matrix：

> **如果通：喵~！**
>
> **如果不通：喵呜呜呜呜…**

Internet 最后会亲自告诉我们哪些路还活着。

---

## 10. “感谢老同志”的真正含义

不是因为旧协议不可挑战。

而是因为今天看到的许多“现代问题”，前人其实早就以不同形式遇到过：

- 可靠性与实时性的冲突；
- 网络层与传输层如何分工；
- sequence / ACK / retransmission；
- end-to-end semantics；
- congestion；
- address / port multiplexing；
- NAT 对端到端模型的破坏；
- 新 transport 的部署困难；
- middlebox ossification。

几十年的 RFC、论文、内核实现和故障经验并不会替我们完成猫娘，但它们意味着：

> **能够偷前人的答案，就不要重新踩前人的坑。**

真正值得自己烧 Token 的，是历史没有直接替我们回答、或者现实环境已经变化的问题。

例如猫娘现在真正关心的：

> **一个统一的加密 Session，能否在 UDP、TCP 以及实验性异构 Carrier 之间进行可验证的 failover / migration，并在现代 NAT/middlebox 环境中尽可能保持隧道存活？**

这不是从历史逃出去。

而是站在历史打下的地基上继续往前挖。

---

## 参考资料

- RFC 675 — *Specification of Internet Transmission Control Program* (1974): https://www.rfc-editor.org/rfc/rfc675
- Internet Society — *A Brief History of the Internet*: https://www.internetsociety.org/internet/history-internet/brief-history-internet/
- Vint Cerf — *How the Internet Came to Be*（关于 packet voice 与 TCP/IP 分拆的回忆）: https://www.packetizer.com/net/cerf_ih.html
- RFC 768 — *User Datagram Protocol* (1980): https://www.rfc-editor.org/rfc/rfc768
- RFC 1122 — *Requirements for Internet Hosts — Communication Layers* (1989): https://www.rfc-editor.org/rfc/rfc1122
- RFC 3022 — *Traditional IP Network Address Translator (Traditional NAT)* (2001): https://www.rfc-editor.org/rfc/rfc3022
- RFC 6951 — *UDP Encapsulation of SCTP Packets for End-Host to End-Host Communication* (2013): https://www.rfc-editor.org/rfc/rfc6951

# 猫娘术语表（持续维护）

目标：把项目里反复出现的英文术语集中起来。不是词典式死记，而是记录“它在猫娘里是什么意思”。

## 项目核心抽象

### Session
逻辑会话。对上层提供持续存在的通信语义，不应等同于某一个 TCP connection 或 UDP 5-tuple。

### Carrier
承载猫娘 record 的底层通信原语，例如 UDP、TCP，以及实验性的 ICMP / Raw IP。Carrier 描述“靠什么搬”。

### Path
某个 Carrier 的具体可用路径。它通常还包含地址族、端点、接口、网络路径等信息。Path 描述“从哪里到哪里搬”。

### Stream
有顺序的逻辑字节流。多个 Stream 可以共享同一个 Session。

### Datagram
独立报文。通常保留 message boundary，不要求像 byte stream 那样拼成连续字节序列。

### Record / Frame / Packet
- **Record**：猫娘 Session 层希望传递的逻辑记录，具体定义待 M0 固化。
- **Frame**：协议内部某一种有类型的结构，例如 STREAM / ACK / CLOSE。
- **Packet**：实际交给某条 packet-oriented Carrier 的一次协议包。三者不要混用。

## 可靠性与时序

### Sequence Number / Packet Number
序号。用于识别顺序、重复、缺失。Packet Number 是包级序号；Stream Offset 是流内字节位置，两者语义不同。

### ACK — Acknowledgement
确认。表示对端已经收到某些数据。UDP Carrier 可能需要 packet ACK；Session 层还可能有更高层的 delivery acknowledgement，用于跨 Carrier 迁移后确认真正交付进度。

### RTT — Round-Trip Time
往返时延。发出一个需要响应的东西，到收到响应所经过的时间。

### RTO — Retransmission Timeout
重传超时。等待确认超过一定时间后认为可能需要重传。

### PTO — Probe Timeout
探测超时。常见于现代可靠 UDP/QUIC 风格设计：长时间没有 ACK 时先发送探测，而不是简单粗暴地重传所有东西。

### Retransmission
重传。对可能丢失、且仍需要可靠送达的数据再次发送。

### Reordering
乱序。后发送的数据可能先到。乱序不等于丢包。

### Duplicate
重复。相同逻辑数据可能因为重传或网络行为收到多次，接收端必须能够去重。

## 流与调度

### Byte Stream
字节流。TCP 的典型语义：应用看到连续字节，不保留每次 `write()` 的边界。

### Message Boundary
报文边界。UDP 的一次 datagram 在接收端仍作为一条 datagram 出现；TCP 不保留这种边界。

### Stream Offset
某个 Stream 内的数据位置，例如 offset 0..4095。它是跨 Carrier 重组的重要候选依据。

### Multiplexing
多路复用。在一个 Session/connection 中并行承载多个逻辑 Stream。

### HOL — Head-of-Line Blocking
队头阻塞。前面的数据没到，后面的数据即使已经到达也可能无法交付。TCP 的有序 byte stream 天生存在这一问题；跨 TCP/UDP 调度若设计错误，还会制造跨 Carrier HOL。

### Scheduler
调度器。决定当前哪些 Stream / Frame / Record 先发，以及使用哪条 Path。

## 流控与拥塞

### Flow Control
流量控制。防止发送端把接收端缓存打爆。回答的是“接收方吃不吃得下”。

### Congestion Control
拥塞控制。防止发送端把网络打爆。回答的是“网络吃不吃得下”。不要和 flow control 混淆。

### cwnd — Congestion Window
拥塞窗口。拥塞控制允许同时在网络中的未确认数据规模上限之一。

### bytes in flight
已发送但尚未确认离开“在途”状态的数据量。

### Pacing
节奏发送。不是瞬间把 cwnd 全塞进 socket，而是按计算出的速率平滑发送。

### Slow Start
慢启动。连接初期快速探索可用容量的一种机制，不代表“故意慢慢发”。

### Reno / CUBIC / BBR
不同拥塞控制算法/家族。当前只作为学习与 benchmark 对照，不把名称当成性能结论。

## 路径与网络环境

### 5-tuple
常见的连接识别五元组：源 IP、目的 IP、源端口、目的端口、协议。UDP/TCP 都常被 NAT/firewall 依据它跟踪状态。

### Connection ID / Session ID
不依赖五元组的逻辑连接标识。对于地址变化、NAT rebinding、Carrier 切换很重要。

### NAT Rebinding
NAT 映射变化，例如客户端外部 UDP port 发生变化，但逻辑 Session 希望继续存在。

### Path Validation
路径验证。确认一个新地址/路径确实可由已认证的对端使用，避免因为伪造包把流量导向攻击者。

### Failover
故障切换。主 Carrier/Path 不可用后切到备用路径。猫娘早期重点是 failover，而不是同时聚合多条路径的带宽。

### Multipath
同一逻辑连接拥有多个路径。它可以只做备用，也可以同时使用；后者复杂得多。

### NAT
Network Address Translation，网络地址转换。现实公网中会改变地址/端口，也使很多非 TCP/UDP 的原生 IP protocol 更难穿透。

### Middlebox
中间盒。泛指 NAT、防火墙、代理、DPI、负载均衡等不只是“转发 IP 包”的网络设备。很多协议理论上成立、现实里却失败，往往就是 middlebox 行为造成的。

### PMTU / PMTUD
Path MTU / Path MTU Discovery。路径上不发生分片时能通过的最大包大小，以及探测这个大小的机制。

## Carrier 候选

### UDP
不可靠、保留 datagram boundary、有端口。猫娘的第一 Carrier。

### TCP
可靠、有序 byte stream、有端口。猫娘计划中的重要 fallback Carrier。

### ICMP / ICMPv6
IP 控制协议，没有 TCP/UDP 式端口。Echo Request/Reply 可携带数据，但可能被限速或过滤，因此只作为实验性、last-resort candidate。

### SCTP
Stream Control Transmission Protocol。可靠、message-oriented、多 stream，并支持 multi-homing。是猫娘很重要的参考协议。

### DCCP
Datagram Congestion Control Protocol。不提供可靠交付，但带连接与拥塞控制语义。适合研究对照。

### GRE
Generic Routing Encapsulation。通用网络层封装，不提供猫娘需要的可靠性/加密语义，但可作为 raw IP 类承载研究对象。

### ESP
IPsec Encapsulating Security Payload。提供网络层安全封装，与猫娘自己的加密层有较多语义重叠。

### Raw IP
直接使用 IP Protocol Number 承载自定义数据，不经过 TCP/UDP。253/254 可用于实验/测试，但现实可达性必须实际测量。

### MASQUE
利用 HTTP/QUIC 等现有 Web 基础设施建立 UDP/IP 等隧道的标准化方向。属于 mediated carrier 思路，而不是新的原生 IP transport。

## Rust 词汇

### crate
Rust 的编译/发布单元之一。库 crate 或 binary crate 都可能存在。

### package
由一个 `Cargo.toml` 描述的一组 crate。

### workspace
多个 package/crate 的共同工程组织方式。猫娘 M0 计划使用 Cargo workspace。

### ownership
所有权。一个值在 Rust 中有明确 owner；owner 离开作用域时通常负责释放资源。

### move
所有权从一个变量/位置转移到另一个位置。之后原变量通常不能继续使用。

### borrow
借用。暂时通过引用访问别人的值，而不取得所有权。

### `&T`
不可变引用。

### `&mut T`
可变引用；同一时间对同一数据的可变访问受到严格限制。

### `Vec<u8>`
拥有一段可增长的 byte buffer。

### `&[u8]`
借用的一段 byte slice；常用于解析/编码时避免不必要复制。

### `Option<T>`
值可能存在，也可能是 `None`，并且这种可能性进入类型系统。

### `Result<T, E>`
成功返回 `T`，失败返回 `E`。Rust 网络代码会大量使用。

### trait
描述一组行为/接口。猫娘的 Carrier 抽象很可能通过 trait 或相近机制表达。

### `async` / `await`
异步 I/O 编程语法。等待网络事件时允许运行其他任务，而不是让线程一直阻塞等待。

### Tokio
Rust 常用异步运行时之一，提供 socket、timer、task、channel 等网络程序常用基础设施。

---

这份表是活文档。项目讨论里出现一个对设计有实际作用、但维护者需要反复查的术语，就应该补进来。

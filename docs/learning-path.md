# 猫娘学习路线：从 Python 到 Rust 网络协议实验

这份文档面向项目维护者本人：已有 Python 使用经验，但 Rust 与传输协议实现经验有限。目标不是先学完整门课再开始项目，而是让“猫娘”成为学习载体，按项目需要补知识。

## 总原则

1. 不从“变量/循环/函数”重新学编程基础。
2. 优先补与本项目直接相关的 Rust 与网络知识。
3. 每学一个概念，都尽量在 `nekomusume` 里找到对应位置。
4. 不要求第一次就记住所有英文术语；术语统一进入 `docs/glossary.md`。
5. 允许 AI 辅助写代码，但关键结构必须能解释：为什么这样分层、这个状态是什么意思、失败时发生什么。

## Rust 主线

### R0：先能读和跑

目标：看懂最小 Rust 项目结构。

需要掌握：

- `cargo new` / `cargo build` / `cargo test` / `cargo run`
- `Cargo.toml`
- crate / module / package / workspace
- `let`、`mut`、函数、`struct`、`enum`
- `match`
- `Option<T>` / `Result<T, E>`

与 Python 的桥接：

- `Option<T>` ≈ “这个值可能不存在”，但不是随便塞 `None`；类型系统会逼你处理。
- `Result<T, E>` ≈ “函数可能失败”，但失败路径被写进返回类型，不靠异常悄悄跳走。
- `enum` 比 Python 常见的字符串状态值更强，可以让不同状态携带不同数据。

### R1：Rust 最关键的思维变化

目标：理解为什么 Rust 网络代码里总出现引用、生命周期、所有权。

需要掌握：

- ownership（所有权）
- move（移动）
- borrow（借用）
- `&T` / `&mut T`
- slice：`&[u8]`
- `String` vs `&str`
- `Vec<u8>` vs `&[u8]`

项目对应：

- packet buffer 谁拥有？
- encoder 是复制数据，还是只借用一段 bytes？
- session / carrier / path 的状态由谁修改？

不要一开始深挖复杂生命周期标注；先理解“谁拥有、谁借用、借多久”。

### R2：Trait 与抽象

目标：能看懂 Carrier 抽象。

需要掌握：

- `trait`
- `impl`
- generic
- trait object：`dyn Trait`
- `Send` / `Sync` 基本含义

项目对应：

```text
Carrier
├── UdpCarrier
├── TcpCarrier
├── IcmpCarrier (experimental)
└── ...
```

### R3：异步网络

目标：能读 Tokio 风格网络代码。

需要掌握：

- `async fn`
- `.await`
- task
- channel
- timer
- cancellation
- `tokio::net::UdpSocket`
- `tokio::net::TcpStream`

先把 async 理解成“等待 I/O 时把执行权交出去”，不要先钻 Future 的内部实现。

### R4：二进制协议实现

目标：开始真正实现 wire format。

需要掌握：

- byte order / network byte order
- fixed-width integer
- varint
- encode / decode
- parser 边界检查
- zero-copy / copy 的区别
- fuzz / property test / golden vector

这将直接进入 M0。

## 网络主线

### N0：先建立地图

必须能分清：

```text
Application
Transport / Session
IP
Link
```

以及：

- TCP/UDP/SCTP/DCCP 属于什么位置
- ICMP 为什么没有 port
- GRE / ESP / IP-in-IP 为什么能直接作为 IP payload
- NAT / firewall / middlebox 在哪里改变行为

### N1：可靠传输的最小机制

依次理解：

- sequence number
- acknowledgement (ACK)
- retransmission
- RTT
- timeout / RTO / PTO
- reordering
- duplicate
- flow control
- congestion control

不要先背算法，先回答：“如果包丢了，双方怎么知道？什么时候重传？重传会不会让网络更堵？”

### N2：流与报文

理解：

- byte stream
- message boundary
- stream offset
- head-of-line blocking (HOL)
- multiplexing
- datagram

这是理解 TCP / UDP / QUIC / SCTP 差异的关键。

### N3：路径与迁移

理解：

- 5-tuple
- connection ID
- NAT rebinding
- path validation
- primary / standby path
- failover
- multipath

这直接对应猫娘的 Session / Carrier / Path 架构。

### N4：拥塞控制

先理解：

- cwnd
- bytes in flight
- pacing
- slow start
- loss-based vs model-based

然后再看 Reno / CUBIC / BBR。

## 推荐学习方式

不是：

```text
先读完 Rust Book
→ 再读 TCP/IP 详解
→ 再读 RFC
→ 一个月后开始写
```

而是：

```text
遇到一个设计问题
→ 学 2~5 个必要概念
→ 写一个最小实验
→ 看抓包/日志
→ 回到设计
```

例如：

```text
为什么 UDP Carrier 要 ACK？
→ sequence number
→ ACK
→ RTT
→ loss detection
→ 写 100 行最小实验
```

## 当前第一课

在开始 M0 代码之前，优先掌握并能口头解释：

1. packet / frame / stream / session / carrier / path 的区别；
2. TCP 的 byte stream 与 UDP 的 datagram 差异；
3. ACK、RTT、retransmission 的关系；
4. Rust 的 ownership / borrow / `Result` / `enum`；
5. `Vec<u8>` 与 `&[u8]`。

达到“能解释”即可，不要求背定义。

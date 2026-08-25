# 学习笔记 01：Packet / Frame / Datagram / Stream / Session

这份笔记用于解释猫娘项目中最容易混用的几个数据单位。重点不是背定义，而是理解它们在协议栈里各自回答什么问题。

## 1. Packet — 包 / 数据包

`packet` 是一个相对泛化的词，通常表示“网络中作为一个独立单位被处理和传输的一块数据”。

在严格分层语境中，常见说法是 IP packet；但日常工程讨论里 packet 也常被当作泛称。使用时最好加限定词，例如 `IP packet`、`UDP packet`、`Nekomusume packet`。

## 2. Datagram — 数据报

`datagram` 强调的是**报文边界（message boundary）**。

如果应用通过 UDP 连续发送两个 datagram：

```text
sendto("abc")
sendto("def")
```

接收方看到的仍是两个独立报文；网络可能丢失、重复或乱序其中任意一个，但不会把 UDP 的两次发送自动变成一个连续 byte stream。

## 3. Stream — 流 / 字节流

`stream` 强调的是连续、有序的数据语义。TCP 给应用提供的是 byte stream。

如果应用调用：

```text
send("abc")
send("def")
```

对端不能依赖两次 `recv()` 恰好返回 `abc` 和 `def`。它可能看到 `abcdef`，也可能分成 `ab` 与 `cdef` 等。TCP 保证的是有序字节，不保证应用每次 `send()` 的边界。

这就是为什么 TCP Carrier 上必须额外定义 framing。

## 4. Frame — 帧 / 协议内部有类型的数据单元

`frame` 有两种常见语境，不能混淆：

1. 链路层中的 frame，例如 Ethernet frame；
2. 某个上层协议内部定义的 typed unit，例如 `STREAM`、`ACK`、`CLOSE` frame。

猫娘目前主要使用第二种含义。

例如：

```text
STREAM(stream_id=7, offset=0, data="abc")
ACK(...)
CLOSE(...)
```

一个猫娘 packet/record 可以承载一个或多个 frame，具体映射由 M0 wire format 决定。

## 5. Record — 记录 / 可恢复边界的协议承载单元

`record` 是猫娘正在采用的逻辑术语，用来表示 Session 层希望 Carrier 搬运的一块完整协议记录。

它尤其适合 TCP Carrier：因为 TCP 本身没有 message boundary，所以需要类似：

```text
[length][record bytes]
[length][record bytes]
```

这样的 framing，才能从连续 byte stream 中重新找回 record 边界。

UDP 自带 datagram boundary，因此初期可以考虑“一 UDP datagram 对应一猫娘 packet/record”，但这仍属于待 M0 固化的 wire-format 决策。

## 6. Session — 会话

`session` 不是一块数据，而是**逻辑通信关系及其状态**。

猫娘的 Session 不应等同于某一条 TCP connection，也不应绑定某个 UDP 5-tuple。它可以拥有自己的 Session ID、streams、delivery state、crypto state，并允许底层 Carrier / Path 发生变化。

```text
Nekomusume Session
  ├── Stream 0
  ├── Stream 1
  ├── Datagram API
  └── Carrier Manager
       ├── UDP Path
       └── TCP Path
```

## 7. 一条数据如何穿过这些概念

假设应用向 Stream 7 写入 `hello`：

```text
application bytes:  "hello"
        ↓
STREAM frame:       stream=7, offset=0, data="hello"
        ↓
Nekomusume record / packet
        ↓
Carrier
   ├── UDP: 一个 datagram 搬走
   └── TCP: 放进带长度边界的 byte stream 中搬走
        ↓
IP packet
        ↓
link-layer frame（例如 Ethernet frame）
```

接收方向反过来拆。

## 8. 当前最重要的区分

- `packet`：网络中处理/传输的一块独立数据，常作为泛称；最好带层级限定。
- `datagram`：强调独立报文和 message boundary。
- `stream`：强调连续、有序字节，不保留应用写入边界。
- `frame`：协议内部带类型的结构；同时注意链路层也有 frame 这个词。
- `record`：猫娘用于跨不同 Carrier 恢复协议边界的逻辑承载单元。
- `session`：长期存在的逻辑通信状态，不是某一个包，也不是某一条 TCP/UDP 路径。

## 9. 第一条设计推论

因为 UDP 提供 message boundary，而 TCP 只提供 byte stream，所以：

> UDP Carrier 可以天然按 datagram 接收离散数据；TCP Carrier 必须额外做 framing，才能恢复猫娘 record 的边界。

这也是 Carrier 抽象不能假装所有底层都具有相同语义的第一个具体例子。

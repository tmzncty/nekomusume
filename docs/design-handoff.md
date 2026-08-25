# 自研网络传输协议：跨机器开发交接文档

> **Historical / non-normative:** retained as a design handoff and research input. It is not the current protocol specification; follow `docs/m0-spec-plan.md` and future versioned `docs/spec/` documents.

> 文档用途：把当前 Session 能恢复出的讨论结论、已知事实和下一步可执行设计交给另一台机器上的开发代理。
>
> **重要诚实说明**：当前记忆和旧工作区中，没有找到此前那份自研协议的完整设计稿，也没有找到可确认的协议名称、字段编号或已写代码。因此，下面分成“已确认的历史事实”和“为继续开发而整理的基线方案”。不要把基线方案误认为已经由用户最终拍板的旧结论；遇到冲突时以用户当前要求为准。

---

## 1. 项目定位

目标不是继续部署 Hysteria2（HY2），而是**手工设计并实现一个自研 UDP 传输协议**，最终通过真实链路基准测试，在目标场景下的综合体验超过 HY2。

项目面向的实际用途是加密代理/隧道传输，但协议本身应先作为一个独立、可测试的传输层实现：

- client 与 server 建立一条加密 UDP 会话；
- 会话内承载一个或多个双向 byte stream；
- 可选承载低延迟 unreliable datagram；
- 对丢包、乱序、突发拥塞、NAT 映射变化和路径 MTU 变化有明确行为；
- 支持代理上层接入，但不能把代理业务逻辑硬编码进传输核心。

### 1.1 “比 HY2 好”必须可测量

不能用“感觉更快”作为结论。至少比较：

1. 首包/握手完成时间；
2. 空闲恢复与 NAT 重绑定后的恢复时间；
3. TCP over tunnel 的吞吐量；
4. 单向和双向 goodput；
5. 1%、3%、5%、10% 丢包下的延迟和吞吐；
6. RTT 变化（20/80/150/300 ms）下的表现；
7. 重排序、突发丢包和带宽变化下的恢复速度；
8. 多路复用时一个大流对小流延迟的影响；
9. CPU、内存、每包开销和连接数上限；
10. 长时间运行稳定性、异常断线和重连成功率。

**验收原则**：协议只有在相同服务器、相同线路、相同 MTU、相同加密等级、相同应用流量和重复实验条件下，至少在一个明确目标场景稳定优于 HY2，才可以称为“超过 HY2”；不能把未测量的理论优势写成结论。

---

## 2. 当前能确认的历史内容

从已有跨 Session 记忆中可以确认：

- 现有生产链路使用过 HY2 主协议，并有 Trojan 备用链路；这与本项目是两件事。
- 生产部署的具体公网地址、内网地址与端口信息不属于公开协议研究内容，因此在公开仓库版本中省略。
- 现有环境中使用过 mihomo、OpenWrt/passwall2 等组件；这些是部署对象，不应直接当作自研协议的接口约束。
- 曾经讨论过网络质量、代理池、OSPF/拓扑和流量监测，但当前恢复不到“自研协议具体报文设计”的可靠原文。
- 不能确认旧讨论是否已经决定使用 QUIC、裸 UDP、Noise、TLS、FEC、BBR、拥塞控制算法或某个项目名。下面这些内容均标为**建议基线**。

---

## 3. 建议的总体技术路线（基线，不是历史既定结论）

### 3.1 不要从零手搓密码学

“自研协议”指自定义传输语义和实现，不表示自行发明加密算法。

建议：

- 使用成熟库提供的 **AEAD**：ChaCha20-Poly1305 或 AES-128/256-GCM；
- 密钥协商使用成熟的 TLS 1.3 或 Noise/HPKE 实现；
- 第一版优先采用现成 TLS 1.3/QUIC crypto 库，避免协议研发被密码学实现拖死；
- 禁止 `allowInsecure` 类配置、明文回退和固定可复用会话密钥；
- 每个数据包都要有完整性保护，nonce 不能重复；
- 密钥更新和会话关闭必须是状态机的一部分。

如果最终使用 QUIC 库，仅复用其可靠的加密/定时器/ACK 基础设施，仍可自定义上层传输调度；如果目标是研究全新传输行为，再做裸 UDP 版本。

### 3.2 第一版选择：UDP + AEAD + 自定义可靠传输

建议先实现一个可控的最小协议，而不是一次加入所有高级特性：

- 外层：UDP；
- 会话：随机 `connection_id`，支持客户端端口变化；
- 安全：握手后派生 send/receive 两套方向密钥；
- 数据：包级序号、ACK range、重传队列、RTT/RTO；
- 流：按 stream 分离的有序字节流；
- 拥塞：先 Reno/CUBIC 风格的可靠基线，再接入 BBR-like 实验实现；
- 调度：连接级公平发送，控制帧和小流优先；
- FEC：第一版不默认启用，只作为可插拔模块和对照实验；
- 迁移：PATH_CHALLENGE/PATH_RESPONSE 后才切换路径。

理由：HY2 已经解决了很多通用问题。真正可能形成优势的地方通常是**调度、丢包恢复、拥塞控制参数、连接迁移和应用场景适配**，不是再造一个“看起来像 QUIC”的大协议。

---

## 4. 协议分层

```text
Application / Proxy adapter
        │  open stream, read/write, datagram
Session API
        │  stream scheduler, flow control, migration
Reliable packet engine
        │  packet number, ACK, loss detection, retransmit
Congestion controller
        │  cwnd, pacing, RTT, ECN(optional)
Crypto / key phase
        │  AEAD seal/open, key update
UDP socket / platform adapter
```

核心库必须与代理入站/出站解耦，建议 API 类似：

```text
Client.connect(endpoint, credentials) -> Session
Session.open_bidi() -> Stream
Stream.read/write/close()
Session.send_datagram(payload)
Session.stats()
Session.close(error_code, reason)
```

---

## 5. 包格式建议

所有整数采用 network byte order；变长整数采用 QUIC 风格或自定义明确的 canonical varint。第一版可以固定头部，先保证实现和抓包简单。

### 5.1 未加密初始包

```text
InitialHeader {
    magic[4]          // 版本/协议识别，例如 "S13\0"；实际值开发时确定
    version:u8
    packet_type:u8    // INITIAL / RETRY / HANDSHAKE / DATA / CLOSE
    flags:u16
    connection_id[8]
    packet_number:u32
    payload_len:u16
}
payload = handshake message
```

不要依赖 magic 实现安全性；magic 只用于版本分流和快速丢弃错误包。

### 5.2 已加密短头

```text
ShortHeader {
    flags:u8           // key phase、ACK-only、spin(optional)、reserved
    connection_id[8]
    packet_number:u64  // 可截断编码，但内部必须可恢复且不能重复
}
ciphertext = AEAD(key, nonce(packet_number, key_phase), plaintext_frames)
tag = 16 bytes
```

建议最初使用完整 64-bit packet number，等协议稳定后再做短编码优化。不要过早压缩头部。

### 5.3 帧类型

第一版至少定义：

```text
PADDING              // 填充
PING                 // 保活/探测
ACK                  // ACK ranges + ack delay
STREAM               // stream_id, offset, FIN, data
MAX_DATA             // 连接级发送窗口
MAX_STREAM_DATA      // 流级发送窗口
OPEN_STREAM          // 可选，或由首个 STREAM 隐式创建
RESET_STREAM         // 流错误
STOP_SENDING         // 请求对端停止流
PATH_CHALLENGE       // 路径验证 token
PATH_RESPONSE        // 返回 token
NEW_CONNECTION_ID    // 可选，迁移/轮换 ID
KEY_UPDATE           // 密钥阶段切换
CLOSE                // 错误码和短原因
DATAGRAM             // 不可靠数据，可选
FEC_REPAIR           // 第二阶段再加入
```

帧必须能被安全跳过或明确拒绝。未知帧类型的兼容策略在 v1 固定：关键帧未知则关闭，非关键扩展可跳过。

---

## 6. 握手与密钥

### 6.1 推荐握手流程

```text
Client                                             Server
  | -- INITIAL: version, CID, client hello ------> |
  | <----- RETRY（可选，地址验证） ---------------- |
  | -- INITIAL + token ---------------------------> |
  | <----- HANDSHAKE: server hello + auth -------- |
  | -- HANDSHAKE: client auth -------------------> |
  | <========== encrypted DATA ==================> |
```

握手要求：

- 服务端必须限制未验证地址的放大比例；
- Retry token 必须绑定客户端地址、时间窗口和服务端密钥轮换版本；
- 认证失败使用统一错误表现，避免泄漏用户是否存在；
- 首次握手支持 0-RTT 只能用于幂等、低风险数据；第一版可直接禁用 0-RTT；
- 会话密钥由握手 transcript 派生，不能只由用户名/密码哈希直接作为数据密钥；
- 服务端和客户端身份认证方式要可插拔：预共享密钥 PSK 可用于自用部署，证书/公钥模式用于扩展部署。

### 6.2 推荐的第一实现

优先顺序：

1. 使用成熟 TLS 1.3 库，先让 client/server 通信跑通；
2. 使用协议自身的 `connection_id` 和 packet number 做数据调度；
3. 待基准稳定后，再评估是否切换到 Noise-IK/XX；
4. 不要同时更换密码学库、包格式和拥塞控制，否则无法定位问题。

---

## 7. 可靠性、ACK 与丢包恢复

### 7.1 ACK

ACK 应支持范围，而不是只确认最大序号：

```text
ACK {
    largest_acked:u64
    ack_delay_us:u32
    range_count:varint
    ranges[] = {gap:varint, length:varint}
}
```

ACK-only 包不能无限产生。建议 ACK 触发条件：

- 收到两个 ack-eliciting 包；或
- 距上次 ACK 超过一个短定时器（例如 1/4 RTT，设置上限）；或
- 检测到乱序/缺口时立即 ACK。

### 7.2 丢包判断

同时使用：

- packet-threshold：后续确认超过一定包数后判定早期包丢失；
- time-threshold：超过 `max(kTimeThreshold * max(srtt, latest_rtt), granularity)`；
- PTO：没有 ACK 时探测，不要把 PTO 探测包直接当作普通重传洪泛。

应用数据按 frame 重新组包，不要简单复制整个 UDP 包无限重传。ACK、PATH_RESPONSE、CLOSE 等控制帧要有可靠发送策略。

### 7.3 重传与乱序

- 接收端按 stream offset 去重；
- 乱序数据可缓存，但受连接级和流级内存上限约束；
- 缓存超限时优先丢弃低优先级流或返回流错误，不能让单个流耗尽进程内存；
- 重传只重传尚未被 ACK 的 frame range；
- 对同一 packet 的重复接收必须幂等。

---

## 8. 拥塞控制和发送调度

这是最可能决定能否超过 HY2 的部分，也是最容易把协议写坏的部分。

### 8.1 v1 基线

先实现一个可靠、保守的 Reno/CUBIC 风格控制器：

- `cwnd`、`bytes_in_flight`、`ssthresh`；
- pacing 发送，不允许只靠 socket 写满；
- ACK clock；
- 慢启动退出条件；
- 丢包退避；
- 应用限速不能超过拥塞窗口和 pacing 预算；
- 记录 min RTT、smoothed RTT、rtt variance、delivery rate。

### 8.2 后续实验方向

可插拔实现：

- CUBIC：可靠基准；
- BBR-like：按带宽/RTT模型控制，需防止测量抖动；
- loss-aware：区分随机无线丢包和拥塞丢包，不能简单把所有丢包都当作拥塞；
- shallow-buffer：在高丢包环境减少 bufferbloat；
- bandwidth cap：避免 UDP 隧道抢占整个出口。

任何“区分随机丢包”的算法都必须用不同网络模型验证，不能凭单次公网测试调参数。

### 8.3 多流调度

建议采用连接级 DRR/加权公平队列：

- 控制帧最高优先级；
- 交互小流高优先级；
- 大吞吐流按 quantum 轮转；
- 单个大流不得阻塞其他流；
- 流优先级只能影响发送顺序，不能绕过连接级 `cwnd`；
- 每个包尽量只装入同一 stream 的连续数据，减少接收端复杂度；
- 不要盲目做“一个包混合多个流”，先以可观测性和公平性为主。

---

## 9. FEC、冗余与重传策略

FEC 不是免费午餐。它会消耗带宽、CPU，并可能在拥塞时加剧拥塞。

建议分阶段：

### v1

只做可靠重传，不默认 FEC。先建立丢包恢复基线。

### v2

添加可插拔块 FEC：

- 按固定数量的数据包组成 generation；
- 只对数据包 payload 做 XOR/RS 等修复；
- generation 大小、冗余比例动态配置；
- 根据近期丢包率和 burst length 调整；
- 拥塞或带宽不足时立即降低/关闭 FEC；
- FEC 修复成功后仍需 ACK 语义闭合，避免发送端无限保留。

FEC 的目标应是降低交互流的恢复等待，而不是无条件追求吞吐。

---

## 10. 流控、连接迁移和保活

### 10.1 流控

至少有两级窗口：

- connection max data；
- per-stream max data。

窗口更新不能被丢包永久阻塞；窗口增长应受接收缓存上限约束。服务端必须有总连接数、总缓冲区和每连接缓冲区限制。

### 10.2 NAT 重绑定

- `connection_id` 不绑定五元组；
- 收到合法加密包但源地址变化时，进入 candidate path；
- 对 candidate path 发送 challenge；
- 收到 response 后再切换发送路径；
- 旧路径保留一个短宽限期；
- 迁移期间不能因为一个伪造包就把流量切到攻击者地址。

### 10.3 保活与空闲

- 空闲超时由双方协商，但有服务端硬上限；
- PING 只在确实需要保持 NAT 映射时发送；
- 保活周期不能小于合理的网络成本预算；
- CLOSE 要尽力发送，但不能依赖 CLOSE 才释放本地资源；
- 断网恢复由上层决定是否重建会话，第一版不要承诺透明无损恢复。

---

## 11. MTU 和包大小

第一版默认不要超过安全 UDP payload：

- 初始目标：1200 bytes，兼容常见路径；
- 后续通过 PMTUD/探测包寻找更大 MTU；
- 探测失败要回退，不得因 ICMP 被过滤而卡死；
- 不对 IPv4/IPv6、隧道封装、云厂商网络做未经测试的固定假设；
- 所有基准测试记录路径 MTU、IPv4/IPv6、内核版本和网卡 offload 状态。

---

## 12. 版本、错误码和兼容性

协议版本必须显式存在。建议：

```text
major.minor
```

- major 不兼容时拒绝；
- minor 通过 capability negotiation 扩展；
- 保留字段必须发送为零、接收时验证；
- 错误码分为 transport error 和 application error；
- 错误原因文本不能成为控制逻辑依据；
- 所有状态机转换写成表格和测试，不要散落在 socket 回调中；
- 永远保留 PCAP/事件日志可复现能力，但日志默认不记录密钥和明文。

---

## 13. 建议的实现语言和目录

如果没有其他语言约束，建议：

- 核心传输：Rust 或 Go；
- CLI：同语言实现，避免先拆进程；
- 性能测试：Rust/Go 端到端工具；
- 抓包和故障注入：Linux `tc netem`、Windows 或 Linux 的测试脚本；
- 互操作测试：固定 golden packet vectors。

建议目录：

```text
protocol/
  README.md
  docs/
    design.md                 # 本文的精简正式版
    state-machine.md
    wire-format.md
    benchmark-plan.md
  cmd/
    client/
    server/
    bench/
  internal/
    wire/                     # 编解码、varint、帧
    crypto/                   # 仅调用成熟库
    session/                  # 状态机、CID、握手
    loss/                     # ACK、丢包、PTO
    congestion/               # Reno/CUBIC/实验控制器
    scheduler/                # 多流公平调度
    path/                     # MTU、迁移、探测
  tests/
    wire_vectors/
    integration/
    netem/
  fuzz/
```

---

## 14. 最小可运行切片（必须按此顺序）

### Milestone 0：写规范和测试向量

交付：

- 版本号和 magic；
- 固定包头；
- 帧类型与字段；
- 错误码；
- 至少 20 个编码/解码 golden tests；
- 任意畸形包不能 panic、越界或无限分配。

### Milestone 1：单流加密回显

交付：

- client/server UDP socket；
- 握手和身份认证；
- 加密 DATA；
- 单个双向 stream；
- CLOSE；
- 本机和局域网成功率 100%。

### Milestone 2：可靠传输

交付：

- packet number；
- ACK ranges；
- RTT 估计；
- 丢包检测；
- frame 重传；
- netem 1%/5%/10% 丢包下最终数据正确。

### Milestone 3：流控和多路复用

交付：

- 多 stream；
- 连接级/流级 flow control；
- 公平调度；
- 大流不能阻塞小流；
- 资源上限和超时测试。

### Milestone 4：拥塞控制和 pacing

交付：

- Reno/CUBIC 基线；
- 发送 pacing；
- 测量 goodput、RTT、丢包和队列延迟；
- 与 HY2 使用同一测试工具对比。

### Milestone 5：迁移、PMTU 和可选增强

按实际结果决定是否加入：

- NAT 重绑定；
- PMTUD；
- FEC；
- datagram；
- key update；
- 0-RTT；
- 多路径。

不要在 Milestone 1 之前实现多路径、FEC 或复杂伪装。那是在给自己挖坑，管理员。

---

## 15. 基准测试计划

### 15.1 网络模型

使用 `tc netem` 或等价工具建立可复现实验：

```text
baseline: RTT 20ms, loss 0%, rate 1Gbps
long-haul: RTT 150ms, loss 0.5%, rate 100Mbps
lossy: RTT 80ms, loss 3%, rate 100Mbps
bad: RTT 150ms, loss 5%, burst loss 20ms
mobile-like: RTT 40-200ms, variable rate, reorder 1%
```

每组至少重复 10 次，报告中位数、P95 和失败次数。

### 15.2 流量场景

1. 1 MiB、100 MiB、1 GiB 单流下载；
2. 单流上传；
3. 10 个并发流，1 个大流 + 9 个小请求；
4. 持续交互小包；
5. 断网 3 秒后恢复；
6. 客户端端口变化/NAT 重绑定；
7. 长连接 24 小时资源稳定性。

### 15.3 必采集指标

```text
handshake_ms
first_payload_ms
throughput_mbps
goodput_mbps
rtt_min_ms
rtt_smoothed_ms
rtt_p95_ms
loss_sent/loss_detected/recovered
retransmit_bytes
fec_overhead_bytes
cwnd_bytes
bytes_in_flight
cpu_user/cpu_sys
memory_peak
active_streams
migration_success
```

### 15.4 公平比较

HY2 和自研协议必须：

- 同一台 client/server；
- 同一 UDP 端口条件或明确标注差异；
- 同一 MTU；
- 同一目标地址和时间窗口；
- 同一加密安全级别；
- 同一应用层代理模式；
- 不把自研协议调到不安全或无限资源，来换取虚假的性能优势。

---

## 16. 安全与工程红线

- 不自行设计密码算法；
- 不允许 nonce 重用；
- 不允许未认证明文控制帧改变连接状态；
- 严格限制初始包放大，防止 UDP amplification；
- 对畸形长度、重复帧、超大 stream offset、未知版本、旧 packet number 做边界测试；
- 每个连接和全局都有内存/CPU/速率上限；
- 认证失败、重放、过期 token、路径验证失败都要有测试；
- 不记录 PSK、私钥、明文 payload；
- 服务端默认拒绝任意开放代理，目标地址必须由上层策略控制；
- 先在隔离端口和测试服务器运行，不能直接替换生产 HY2。

---

## 17. 交给另一台机器的首条指令

把本文件放入新项目后，让开发代理先执行以下任务，不要直接写完整协议：

> 阅读 `自研网络协议设计交接文档.md`。先检查当前工作区和已有代码；不要假设本文“建议基线”是旧版已确认事实。输出：
> 1. 当前已有实现和文档；
> 2. 缺失的协议决策；
> 3. 采用 Rust/Go/其他语言的理由；
> 4. Milestone 0 的具体文件计划；
> 5. 需要管理员确认的最多 5 个问题。
>
> 在问题未确认前，不要部署到生产，也不要修改现有 HY2/Trojan 链路。

如果决定不等待确认、直接做实验原型，则默认采用：

```text
UDP + TLS 1.3/成熟加密库
固定 1200-byte 初始 MTU
单连接、多 stream
ACK ranges + frame retransmit
Reno/CUBIC + pacing
禁用 0-RTT、FEC、多路径、伪装
本地 bench + netem 先行
```

---

## 18. 当前结论

我们真正已经确认的是“要手搓一个目标超过 HY2 的协议”，而不是一套已经定稿的字段和算法。最稳妥的推进方式是：

1. 先写可验证的 wire format；
2. 再做加密单流；
3. 再做 ACK/重传；
4. 再做多流、调度和拥塞控制；
5. 最后用可复现实验决定 FEC、迁移和更激进的优化是否值得加入。

**不要宣称“比 HY2 好”，直到基准结果证明它。**

# 猫娘 Roadmap

本路线图只把现有交接文档中已经整理出的推进顺序转成仓库工作项，不额外宣称协议已经定稿。

## Milestone 0 — 规范与测试向量

目标：先让“字节长什么样、状态怎么走、错误怎么报”可验证。

- [ ] 确定仓库实现语言（Rust / Go / 其他）
- [ ] 明确协议版本与 magic
- [ ] 固定第一版 packet header
- [ ] 定义 frame type 与字段编码
- [ ] 定义错误码与兼容策略
- [ ] 写状态机文档
- [ ] 至少 20 个 golden encode/decode tests
- [ ] fuzz：畸形包不得 panic、越界或无限分配

## Milestone 1 — 单流加密回显

- [ ] client/server UDP socket
- [ ] 使用成熟握手与加密实现
- [ ] DATA 加密
- [ ] 单个双向 stream
- [ ] CLOSE
- [ ] 本机与局域网稳定回显

## Milestone 2 — 可靠传输

- [ ] packet number
- [ ] ACK ranges
- [ ] RTT 估计
- [ ] packet/time threshold loss detection
- [ ] PTO
- [ ] frame-level retransmission
- [ ] 1% / 5% / 10% 丢包环境数据完整性测试

## Milestone 3 — 多流与流控

- [ ] 多 stream
- [ ] connection-level flow control
- [ ] per-stream flow control
- [ ] 公平调度
- [ ] 大流不得显著阻塞交互小流
- [ ] 每连接与全局资源上限

## Milestone 4 — 拥塞控制与 benchmark

- [ ] Reno/CUBIC 风格可靠基线
- [ ] pacing
- [ ] cwnd / bytes_in_flight / RTT / delivery-rate 观测
- [ ] tc netem 网络模型
- [ ] 与 HY2 在同机器、同线路、同 MTU、同安全等级、同应用流量下比较
- [ ] 至少报告 median / P95 / failures

## Milestone 5 — 实验性增强

仅根据 M4 的数据决定是否加入：

- [ ] NAT rebinding / path validation
- [ ] PMTUD
- [ ] FEC
- [ ] unreliable datagram
- [ ] key update
- [ ] 0-RTT
- [ ] multipath

## 第一轮研究问题

建议最先回答这些，而不是急着堆功能：

1. 选 Rust 还是 Go，主要受性能、开发速度还是现有生态驱动？
2. 第一版握手是直接复用 TLS 1.3，还是采用 Noise 库？
3. v1 packet header 是否固定长度，以换取抓包和实现简单？
4. 第一版拥塞控制用 Reno 还是 CUBIC 作为对照基线？
5. benchmark 的“目标场景”到底是哪一种：长 RTT、随机丢包、移动网络、跨境链路，还是多流交互？

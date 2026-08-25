# 猫娘（nekomusume）

一个公开研究用的实验性 UDP 加密传输协议项目。

> 为什么叫猫娘？别问，问就是猫娘。

## 这是什么

猫娘的目标不是“重新发明密码学”，也不是一开始就宣称替代 QUIC/Hysteria2，而是把自定义传输协议当作一个可验证、可测量的工程实验：

- 基于 UDP 建立加密会话；
- 会话内承载一个或多个双向 byte stream；
- 研究 ACK、丢包恢复、重传、流控与多路复用；
- 比较不同拥塞控制、pacing 和调度策略；
- 研究 NAT 重绑定、PMTU、可选 FEC/datagram 等增强；
- 用可复现实验与 Hysteria2 做公平基准，而不是凭感觉判断“更快”。

## 当前状态

**Research bootstrap / pre-Milestone 0**。

目前真正确认的是：要做一个“目标是在特定场景下可测量地优于 HY2”的实验性传输协议。现有设计交接文档中的大量细节仍是**建议基线**，不是已经最终拍板的协议规范。

详见：[`docs/design-handoff.md`](docs/design-handoff.md)

## 原则

1. **先测量，再宣称。** 没有 benchmark 结果，不写“比 HY2 更好”。
2. **不自研密码算法。** 使用成熟 TLS 1.3 / Noise / AEAD 实现。
3. **先最小切片，再叠功能。** 不在第一阶段同时塞入 FEC、多路径、0-RTT 和复杂伪装。
4. **协议核心与代理业务解耦。** 先把传输层做成独立可测试组件。
5. **所有关键行为必须可复现。** wire format、状态机、错误码、测试向量、netem 条件和 benchmark 都要留下记录。

## 初步路线

- M0：wire format、错误码、状态机、golden vectors
- M1：UDP + 成熟加密库 + 单流加密回显
- M2：packet number、ACK ranges、RTT、loss detection、frame retransmit
- M3：多 stream、流控、公平调度、资源限制
- M4：Reno/CUBIC 基线、pacing、与 HY2 公平 benchmark
- M5：根据实验结果决定 NAT migration、PMTUD、FEC、datagram、key update、0-RTT、多路径

更详细的阶段定义见 [`ROADMAP.md`](ROADMAP.md)。

## 当前不做的事情

- 不把实验版本直接替换生产 HY2/Trojan；
- 不为了跑分关闭认证、完整性保护或资源限制；
- 不把理论优势写成已经验证的性能结论；
- 不在协议尚未稳定时过早压缩头部、做复杂伪装或同时改多套核心算法。

## 许可证

尚未决定。公开研究不等于自动选定开源许可证；在第一批代码进入仓库前再明确即可。

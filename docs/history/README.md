# 猫娘协议考古 / Protocol Archaeology

猫娘不仅记录“最后决定怎么实现”，也记录“为什么今天的 Internet 会变成这样”。

这里不是为了怀旧，而是为了避免把几十年前已经暴露过的基本矛盾重新包装成“全新发现”。协议设计必须同时理解：

- 标准当初解决了什么问题；
- 哪些能力后来逐步加入；
- 哪些协议技术上可行、部署上却失败；
- NAT / firewall / middlebox 如何改变协议生态；
- 为什么现代新 transport 往往重新借 UDP 作为 substrate；
- 哪些历史结论仍然适用于猫娘，哪些需要重新测量。

## 已有笔记

1. [`01-why-tcp-udp-survived.md`](01-why-tcp-udp-survived.md) — 为什么最后主要留下了 TCP 和 UDP？从早期 TCP/IP 分拆、UDP 的极简定位，到 NAT、协议僵化、SCTP-over-UDP 与猫娘 Carrier 设计。

## 写作原则

- 历史事实尽量回到 RFC、IETF/Internet Society 资料或参与者一手回忆；
- 明确区分“当时标准写了什么”和“我们今天如何解释”；
- 不因为协议老就把它当落后，也不因为协议新就假设它更优；
- 历史笔记是研究背景，不自动成为猫娘的 normative specification；
- 真正的设计决策仍以 `docs/decisions.md` 和版本化 spec 为准。

> 感谢老同志打下的基础；能偷答案的地方先偷答案，剩下的坑再由猫娘自己踩。

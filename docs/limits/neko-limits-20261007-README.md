# neko-limits-20261007 — 极限测试公开证据（脱敏版）

本目录是 2026-10-07 猫娘协议极限测量战役的公开版产物。原始数据（含网络地址与密钥指纹）不在此仓库；此处只放脱敏后的结论、矩阵与数据表。

- `net-survey-redacted.md` — 8 台自有服务器 + 92 条有向路径勘测（AS 序列、线路判定、RTT/丢包/PMTU、安全组可用端口、出口带宽参考）。
- `local-limits-redacted.md` — 本机 loopback 极限曲线（122 数据点：TCP 节奏天花板、抗丢包阶梯、UDP 存活率模型、并发流扫描、1h soak）。

方法：`NEKO_MEASUREMENT=1`（opt-in，见 docs/provenance/measurement-mode-20261007.md）+ 用户态损伤代理（tools/impair-proxy/）。缺陷候选清单见 local-limits-redacted.md 异常现象节；修复裁决另行处理，不在本提交内。

# Security Policy

## Status

猫娘目前是**研究原型**，不是经过安全审计的生产级传输协议。

## Security boundaries

项目遵循以下红线：

- 不自行设计密码算法；
- 不允许 nonce 重用；
- 不允许未认证的明文控制帧改变连接状态；
- 对 UDP amplification 设置严格限制；
- 对畸形长度、重复帧、超大 stream offset、未知版本、旧 packet number 做边界测试；
- 每个连接和全局都必须有内存、CPU 与速率上限；
- 日志不得记录 PSK、私钥或明文 payload；
- 实验服务端不得默认变成任意开放代理；
- 未完成安全审查与基准验证前，不替换现有生产链路。

## Reporting

如果未来仓库开放外部协作，再补充正式漏洞报告渠道。在此之前，安全问题先通过仓库 issue/maintainer 私下渠道处理，避免在协议仍快速变化时形成错误的稳定性预期。

## Research references

The pre-M0 threat model and security gates are recorded in [`docs/research/security-threat-model.md`](docs/research/security-threat-model.md). This does not constitute an audit or a selected handshake/library.

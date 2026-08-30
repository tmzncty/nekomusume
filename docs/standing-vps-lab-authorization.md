# Standing VPS Lab Authorization

本文件记录维护者对 Nekomusume 研究仓库的**持续有效实验授权**。目的不是放宽安全边界，而是避免自动化 agent 对已经明确属于管理员控制范围的普通实验反复请求逐次批准。

本授权自提交起持续有效，直到维护者明确撤销、替换或收紧。它是仓库内的正式治理事实，不依赖某次聊天上下文。

## 1. 授权对象

本授权只适用于：

```text
管理员控制的客户端 / 开发主机
        ↕
管理员控制的 VPS / 实验服务器
```

目标必须是维护者拥有或明确控制的实验端点。公网地址、密钥和私有拓扑不得因为本授权而写入仓库；实际目标通过本地/秘密配置提供。

第三方主机、第三方地址段和未经授权的服务不在本授权范围内。

## 2. 已持续授权的普通实验

在第 3 节的默认边界内，agent **无需再次询问管理员**，可以直接执行下列工作：

- 部署、启动、停止、替换当前 Nekomusume 实验 binary/package；
- 在自有 VPS 上创建临时 TCP/UDP listener；
- 默认实验端口为 `40080/tcp` 与 `40081/udp`；如 harness 需要，可选择未占用的非特权高端口，但不得占用已有生产服务；
- TCP/UDP IPv4/IPv6 reachability（前提是对应地址族在环境中真实存在）；
- Nekomusume handshake、Session、multi-record、multi-stream、flow-control、reliable/unreliable datagram、PMTUD、key-update、carrier health、failover、migration 和恢复实验；
- TCP/UDP WAN baseline 与 bounded benchmark；
- Nekomusume 与 HY2 在同一自有 VPS、同一客户端和可比条件下的 bounded 对照实验；
- bounded soak / idle / reconnect / restart / package smoke；
- 在专用 Nekomusume 实验安装路径中执行 install / upgrade / rollback rehearsal；
- 使用 `ss`、`ip addr`、`ip link`、`ip route`、只读 `sysctl`、`uname` 等命令读取实验所需的网络与系统元数据；
- 使用 bounded `tcpdump`/等价抓包收集仅与实验端点、实验端口和实验窗口有关的证据；
- 读取 Nekomusume 自身实验日志、结构化事件、资源指标、socket/process 状态；
- 终止本轮 Nekomusume/HY2 实验产生的进程，删除本轮临时运行文件并完成 cleanup verification。

上述行为属于**预先批准的实验施工**，不是新的外部 gate。

## 3. 默认实验边界

普通单次公网实验默认必须同时满足：

```text
wall-clock duration <= 10 minutes
application traffic <= 256 MiB total
concurrent experimental sessions <= 32
temporary unprivileged listeners only
bounded capture only
cleanup required
```

这些是单次实验上限，不是目标值。agent 应使用能够回答研究问题的最小负载。

不得通过把同一个压力测试机械拆成许多连续小实验来规避边界。

如果任务没有给出 `count / bytes / duration / port`，**参数缺失本身不是 blocker**。agent 应选择最小、可解释、可复现的默认 profile，并在 evidence 中记录实际参数。

推荐的最小默认 profile：

### TCP Session baseline

```text
port: 40080
count: 64 records
bytes_per_record: 1024
maximum_duration: 30s
```

### UDP transport sanity / handshake diagnostic

```text
port: 40081
count: 8 datagrams
bytes_per_datagram: 64
maximum_duration: 15s
```

### Small WAN throughput/sample run

```text
payload: <= 8 MiB
maximum_duration: 60s
```

只要仍处于本文件边界内，agent 可以根据已有 harness 和研究问题选择更小或更合适的参数，无需再次获得逐次许可。

## 4. 失败后的继续权限

一次 WAN 实验失败，不会自动撤销本授权，也不会把后续所有 WAN 工作重新变成“等待管理员许可”。

但是**禁止无变化重复同一失败实验**。

再次执行相同目标场景前，至少应有一项发生实质变化：

- instrumentation；
- code；
- configuration；
- capture coverage；
- diagnostic hypothesis；
- protocol state；
- endpoint/path condition。

失败证据应被冻结并继续有效。新的诊断运行只要满足本授权边界，可以直接执行，不需要再次询问“是否允许重试”。

## 5. 仍需新的明确授权

以下事项不属于 standing authorization，必须重新获得维护者明确许可：

- 任意第三方目标、地址段、服务或扫描；
- 单次运行超过 10 分钟；
- 单次应用流量超过 256 MiB；
- 超过 32 个并发实验 Session，或以压力/容量极限为主要目的的高并发测试；
- 长时间常驻 daemon / listener；
- 绑定特权端口 `<1024`；
- 修改宿主机生产 firewall / nftables / iptables；
- 修改生产 route、policy routing、DNS、proxy、tunnel 或 VPN；
- 修改宿主机生产 qdisc / `tc`；
- 重启 VPS、内核或宿主关键服务；
- Raw IP protocol 253/254；
- GRE；
- ESP；
- SCTP 公网实验；
- DCCP 公网实验；
- 把 ICMP/ICMPv6 当作自定义 carrier、raw-socket carrier 或其他需要特殊 capability 的实验；
- 任何试图规避访问控制、网络策略或第三方限制的实验；
- 为 benchmark 关闭认证、完整性保护或其他安全机制；
- 使用生产密钥、生产用户数据或与研究无关的私密数据；
- 对非 Nekomusume 专用系统路径进行破坏性 package/system 修改。

普通 `ping`/ICMP reachability 读取只用于自有端点、且不需要特殊 raw-carrier 行为时，可以作为诊断元数据；这不构成 ICMP carrier 授权。

## 6. 证据与清理要求

每次公网实验至少应能关联：

```text
experiment_id
git commit / binary identity
actual parameters
start/end time
client result
server result
structured events when available
capture metadata when captured
cleanup status
```

实验结束后必须尽量验证：

```text
no unintended listener remains
no experimental Nekomusume/HY2 process remains
no temporary network namespace/process remains
```

失败实验同样是有效 evidence，不得为了获得 PASS 而删除或覆盖负结果。

大型 pcap/日志不要求提交 Git；仓库可以只保存小型 evidence、摘要、哈希和分析。

## 7. 与安全策略的关系

本授权**不覆盖或削弱 `SECURITY.md`**。

特别是以下约束继续有效：

- 不自研密码算法；
- 不允许 nonce reuse；
- 不允许未认证控制数据改变连接状态；
- 有资源/速率上限；
- 日志不得泄露密钥或明文 payload；
- 实验服务不得成为任意开放代理；
- 未完成安全与发布 gate 前不得替换现有生产链路。

## 8. Agent 执行解释

只要任务满足本文件范围，agent 不得把以下内容作为停止理由：

```text
need WAN authorization
need permission to open 40080/40081
need count/bytes/duration/port
need permission to deploy temporary test listener
need permission to capture bounded experiment traffic
```

这些已经由本文件预先解决。

agent 仍应因为真实的技术依赖、环境缺失、外部 review、超出授权边界或安全问题而停止相应任务。

当某一 WAN task blocked 时，应继续执行所有不依赖该 blocker 的 READY work；不得把一个 WAN failure 推导为整个项目停止。

如果所有 remaining tasks 都只缺“再次确认本文件已经允许的普通自有 VPS 实验”，应视为任务状态建模错误，而不是外部 blocker。

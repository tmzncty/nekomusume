# 分支清理快照 — 2026-10-14

> 管理员分支清理令的救援兜底：全部 remote 分支名→HEAD SHA 快照。
> 任何误删可 `git branch <name> <sha> && git push origin <name>` 重建。
> 基准：origin/main = f15f8c6。

## 全量分支 → HEAD SHA

| 分支 | HEAD SHA | 处置 |
|---|---|---|
| main | f15f8c6 | 保留 |
| docs/d018-privacy-uniform-failure | 2525d30 | 删（步骤 1） |
| docs/d019-preauth-resource-budget | d69eedf | 删（步骤 1） |
| docs/g0-candidate-chain-reconcile | a2b7705 | 删（步骤 1） |
| docs-wire-canonical-candidate | cb45559 | 删（步骤 1） |
| work/e1a-staged-accounting-20260907 | f440425 | 删（步骤 1） |
| work/r8-lab-repair-20260914 | 722732e | 删（步骤 1） |
| work/r8-low-followup | 0a710d9 | 删（步骤 1） |
| work/r9-2e-demux-20260914 | 42f0c9f | 删（步骤 1） |
| design/d075-socks-pipeline-and-measurement-sequence | 2ee5141 | 删（步骤 1，已并入 main：46b2df1） |
| research/middlebox-reachability-plan | 8f939a2 | 删（步骤 1） |
| audit/hy2-diagnostic-e-2e6e46c | a5c9e2c | 步骤 2 裁决 |
| audit/primary-a-semantic-oracle | 6ffa81e | 步骤 2 裁决 |
| followup2-0eef254 | 0eef254 | 步骤 2 裁决 |
| work/continue-20260904 | d271a99 | 步骤 2 裁决 |
| research/d076-causal-sim | affbf11 | 保留（PR #7，步骤 3 对齐） |

## 本地残留分支（非 remote）

| 分支 | HEAD | 说明 |
|---|---|---|
| pr5 | 2ee5141 | = design/d075 分支同 SHA（其 remote 即步骤 1 删除项） |
| work/r8-lab-low-20260914 | 3795326 | 本地分支，remote 无 |
| work/r8-lab-repair-wip | b0cbb2e | 本地分支，remote 无 |

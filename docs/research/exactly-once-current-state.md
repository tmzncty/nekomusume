# Failover exactly-once 现状（Line 1 成文切片）

> 2026-10-14 · issue #10 · 教义条款「Session 能不能不死」的交付语义基础。
> 结论先行：**exactly-once 交付语义已由 M3 实现并有多层测试覆盖；本切片只成文+指认，不改代码。**

## 机制（代码即真相）

UDP→TCP failover 时的 uncertain-range 协议（`neko-carrier/src/lib.rs`，`FailoverController`）：

1. **追踪**：记录在 UDP 路径发出但未收到 Session-delivery ACK 时，进入 uncertain 集合（`track_uncertain`）。failover 提交瞬间仍 uncertain 的记录不丢失、不确认——挂在原 owner 上。
2. **重放**：切换到 TCP 后按权威 in-flight 状态重放 uncertain 记录；接收端按 `DataId` 去重（同 id 第二份副本不二次交付）。
3. **排空**：重放记录在新路径收到确认后从 uncertain 集合排空；每轮 failover 后集合必须为空（泄漏检测）。
4. **容量约束**：uncertain 容量在 assign 时有界转换（`uncertain_capacity_conversion_bounded_at_assign_time`）——不因 failover 攻击面放大内存。
5. **硬失败语义**：路径硬失败时 uncertain 记录保留并 reassign 到新 owner（`hard_failure_retains_uncertain_and_reassigns_to_new_owner`）——Session 不因单路径死而丢数据。

## 教义条款 → 测试行号映射

| 教义条款 | 测试（lib.rs 行号） |
|---|---|
| 「Session 能不能不死」（跨 Carrier 续命不丢数据） | `repeated_failover_cycles_keep_single_active_and_no_duplicates`（2689）：3 轮 failover+migration-back，零重复交付、uncertain 集合每轮排空 |
| failover 时数据不丢 | `clean_recovery_after_bounded_loss_replays_uncertain_without_duplicates`（2945）：有损窗口后 uncertain 重放无重复 |
| 硬失败不丢 uncertain | `hard_failure_retains_uncertain_and_reassigns_to_new_owner`（3254） |
| 重复不二次交付 | `duplicate_partial_delivery_never_exceeds_two_copies`（2368）、`duplicate_emits_exact_multiplicity_and_ordering`（3023） |
| 容量有界 | `uncertain_capacity_conversion_bounded_at_assign_time`（3171）、`duplicate_reorder_combined_multiplicity_and_ordering`（2200） |

## Gap 清单

**零 gap**——教义对 exactly-once 的要求（不丢、不重、有界、可续命）逐条有测试覆盖，上表为证。新 primitive 接入（CarrierAdmission，同 PR）不触碰此机制：admission 只注册/验证路径，uncertain 重放属 FailoverController 既有域。

## 与 CarrierAdmission 的层界

CarrierAdmission（本 PR 新增）= 「找到它、验证它」的接缝；exactly-once 重放 = 「接管 Session」后 FailoverController 的既有职责。两者唯一交互点：新路径要成为 uncertain 重放的目标，必须先通过 Activate 滞回门（validation + k_successes + dwell）——admission 不 bypass，已由 `admission_then_existing_activate_gates_still_apply` 测试钉死。

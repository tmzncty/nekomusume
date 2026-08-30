# FEC research gate: defer runtime implementation

Date: 2026-08-30  
Evidence parent: `7d259af1829a45ace7bffe551834e49ced37193a`  
Decision: **DEFER** adding an FEC library, wire format, negotiation, or runtime path.

This is a research decision, not an implementation or performance result. The
existing bounded XOR candidate remains disabled and carries no performance
claim.

## Existing evidence: retransmission pain is not established

The repository has three different kinds of evidence, and they must not be
combined into a stronger claim:

1. `docs/bench/latest-deterministic.json` is a socket-free recovery fixture. It
   delivers 1,000 modeled frames in two rounds and reports 10, 50, and 100
   retransmitted frames for deterministic 1%, 5%, and 10% first-send loss. The
   loss cases have `rtt_us: 0`; their local execution time is 0–1 microsecond.
   The fixture therefore establishes recovery accounting, not retransmission
   latency, wire bytes, CPU cost, goodput, tail latency, or user-visible pain.
2. `docs/bench/latest-netns.json` measures ICMP ping through netem, not the
   Nekomusume reliable UDP path. Its 20-packet samples observed 0%, 10%, and 10%
   loss under configured 1%, 5%, and 10% random loss, respectively. It records
   no reliable-UDP retransmission bytes or recovery delay.
3. `docs/spec/m2-fec.md` and the `FecBlock` candidate prove only that one XOR
   parity symbol can reconstruct one missing equal-size symbol in a bounded
   block. They explicitly state that no target metric improvement has been
   demonstrated and introduce no wire/runtime behavior.

The strongest defensible conclusion is: retransmission counts rise linearly in
this deterministic fixture, but no measured cost shows that proactive repair
would be cheaper or faster. The blackhole fixture retransmits all 1,000 frames;
finite block FEC would not recover a 100% outage and cannot replace fallback.

## Rust-suitable approaches

Status was checked from crates.io and the upstream repositories on 2026-08-30.
“Maintained” below means recent release or accepted upstream commits and a
non-archived repository; it is not a support guarantee.

### 1. `reed-solomon-simd` 3.1.0 — fixed-rate Reed–Solomon erasure coding

- Upstream: <https://github.com/AndersTrier/reed-solomon-simd>
- crates.io: <https://crates.io/crates/reed-solomon-simd>
- Release: 3.1.0, published 2025-10-14; upstream is not archived and its latest
  release commit is dated 2025-10-14.
- Rust fit: pure Rust, edition 2021, MSRV 1.82; runtime SIMD selection on
  x86/x86-64 and AArch64 with a plain-Rust fallback. The repository currently
  builds with Rust 1.98, so the declared MSRV is acceptable.
- License: `MIT AND BSD-3-Clause`. This is permissive and usable by the
  `MIT OR Apache-2.0` project provided both dependency license/notice
  obligations are preserved; this statement is dependency-screening, not legal
  advice.
- Shape: systematic block code with `k` original and `m` recovery shards;
  recovery needs at least `k` total shards. It can tolerate any `m` erasures but
  requires equal-sized shards and block metadata. Corrupted shards need a
  separate integrity decision before decode.
- Maintenance caution: recent enough to evaluate, but appears primarily
  single-maintainer. Upstream-reported throughput is not Nekomusume evidence.

### 2. `raptorq` 2.0.1 — RFC 6330 fountain code

- Upstream: <https://github.com/cberner/raptorq>
- crates.io: <https://crates.io/crates/raptorq>
- Release: 2.0.1, published 2026-03-09; upstream is not archived and accepted
  dependency/security maintenance commits through 2026-06-22.
- Rust fit: Rust library, edition 2024, default `std` with an optional no-std
  shape. No `rust-version` is declared in 2.0.1, so an MSRV would have to be
  pinned by project CI rather than inferred. It is syntactically aligned with
  this repository's edition-2024 toolchain.
- License: `Apache-2.0`, directly compatible with the project's Apache-2.0
  option, subject to preserving the Apache notice/license requirements.
- Shape: systematic fountain code standardized by RFC 6330. Repair symbols can
  be generated as needed. Upstream states reconstruction probability after
  receiving `K + h` packets as `1 - 1/256^(h + 1)`; the transport still has to
  choose block boundaries, repair budget, feedback, and congestion accounting.
- Maintenance caution: healthy recent activity, but a substantially more
  complex object/block codec than the existing XOR proof. Upstream CPU
  benchmarks are hardware-specific and are not a project performance result.

### Rejected as a “maintained” primary option

`reed-solomon-erasure` 6.0.0 is mature and MIT-licensed, but its last crates.io
release was 2022-09-23 and upstream issue #88, “Looking for new
owners/maintainers”, remains open. It is useful as a comparison baseline, not a
preferred new dependency under this gate.

The recently republished `fec` crate is aimed at convolutional and CCSDS
(255,223) coding for SDR/space/satellite use. That is not a closer fit for a
packet-erasure transport block than the two options above.

## Overhead model against the repository evidence

Let `k` be equal-size source symbols and `m` proactive repair symbols. Ignoring
headers and padding, fixed block-code byte overhead is `m/k`; expected ideal
selective retransmission overhead under independent first-send loss `p` is
approximately `p`. These are models, not measurements.

For the current one-parity XOR candidate (`m = 1`):

| Block | Proactive parity | Break-even vs ideal retransmit | Recovery condition |
|---:|---:|---:|---|
| 8 | 12.5% | loss > 12.5% | at most one missing symbol |
| 16 | 6.25% | loss > 6.25% | at most one missing symbol |
| 32 | 3.125% | loss > 3.125% | at most one missing symbol |

Against the deterministic fixture's payload-count proxy:

| Modeled first-send loss | Fixture retransmits | XOR-8 | XOR-16 | XOR-32 |
|---:|---:|---:|---:|---:|
| 1% | 10/1000 = 1% | 12.5% | 6.25% | 3.125% |
| 5% | 50/1000 = 5% | 12.5% | 6.25% | 3.125% |
| 10% | 100/1000 = 10% | 12.5% | 6.25% | 3.125% |

This table does **not** show that XOR-32 wins at 5% or 10%. At independent loss
`p`, one parity recovers a block only when zero or one of its `k + 1` transmitted
symbols is lost; loss of parity also matters. Burst loss, headers, padding,
late-arriving source symbols, repair loss, ACK behavior, cwnd/pacing, and CPU are
not represented. Larger blocks lower parity percentage but increase buffering
and time before repair is useful.

Reed–Solomon can choose `m/k` near a target loss budget and recover up to `m`
erasures if enough shards arrive. RaptorQ can add repair symbols incrementally,
but proactive repair still consumes congestion-controlled bytes; reactive
repair still needs feedback and incurs a recovery wait. Neither changes the
absence of a measured latency or goodput penalty in current evidence.

RFC 9265 is the controlling transport caution. Section 1 states that FEC and
loss detection are “two distinct and separate reliability mechanisms” and that
blind FEC can hide a congestion signal. Therefore every repair byte would have
to be paced/congestion-controlled, pre-recovery loss would remain visible to
congestion control, and FEC recovery would not constitute packet ACK, Session
delivery, or path validation.

## Decision

**DEFER**, rather than proceed or permanently reject.

Reasons:

1. The repository has no measured retransmission latency, retransmission bytes,
   goodput loss, tail-completion penalty, or CPU profile to optimize.
2. At the only quantified 1% case, every bounded one-parity configuration costs
   more proactive payload than the ideal retransmission-count proxy. At 5–10%,
   apparent parity savings are only a model and ignore multi-loss/burst failure.
3. A real integration must define authenticated block identity, negotiation,
   symbol lifetime, memory bounds, repair scheduling, ACK/ledger interaction,
   MTU/padding, key-update behavior, and congestion accounting. Selecting a
   codec first would be blind implementation.
4. The maintained library choices are credible, so permanent rejection is not
   justified. If a gate later shows a target workload where retransmission wait
   dominates, evaluate `reed-solomon-simd` for bounded fixed-rate blocks and
   `raptorq` for larger/variable object-style blocks in an isolated adapter.

### Evidence required to reopen

Reopen only after an authenticated reliable-UDP harness records, for equal
payload/security/MTU and at least repeated 0/1/5/10% random plus burst-loss
runs:

- original, repair, and retransmitted **wire bytes** separately;
- pre-decode loss, recovered symbols, unrecovered symbols, and spurious repair;
- completion latency (median/P95/P99), goodput, CPU time, and peak block memory;
- RTT and block-size matrix, including short/tail transfers;
- congestion-window/pacing behavior and a shared-bottleneck harm check;
- FEC-off baseline and fixed, loss-independent stop criteria.

A proceed decision must name a target metric and threshold before running the
comparison. Until then: no dependency, no runtime/wire change, no default FEC,
and no claim that FEC is faster, more stable, or more bandwidth-efficient.

## Sources

- N. Kuhn, E. Lochin, F. Michel, M. Welzl, “Forward Erasure Correction
  (FEC) Coding and Congestion Control in Transport,” IRTF RFC 9265, 2022.
  DOI: 10.17487/RFC9265. <https://www.rfc-editor.org/rfc/rfc9265>
- M. Luby, A. Shokrollahi, M. Watson, T. Stockhammer, L. Minder,
  “RaptorQ Forward Error Correction Scheme for Object Delivery,” IETF
  RFC 6330, 2011. DOI: 10.17487/RFC6330.
  <https://www.rfc-editor.org/rfc/rfc6330>
- Christopher Berner, `raptorq` 2.0.1, Apache-2.0, 2026. Repository and
  crates.io metadata linked above. [Software; no DOI]
- Anders Trier Olesen, `reed-solomon-simd` 3.1.0, MIT AND BSD-3-Clause,
  2025. Repository and crates.io metadata linked above. [Software; no DOI]
- rust-rse contributors, `reed-solomon-erasure` 6.0.0, MIT, 2022;
  maintenance issue #88. <https://github.com/rust-rse/reed-solomon-erasure/issues/88>
  [Software; no DOI]

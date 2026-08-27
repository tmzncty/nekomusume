# ADR M1-G0: Pre-auth resource budget and anti-amplification candidate contract

- **Date:** 2026-08-27
- **Status:** **Candidate only — non-frozen; no implementation authorization**
- **Scope:** Resource admission and response bounds before authentication and authorization in the future M1-G0 handshake

## Decision

This documentation-only ADR records an auditable candidate budget for unauthenticated
input. It does not freeze a wire format, protocol, implementation, dependency,
operational capacity, or security property. No Cargo, `src/`, test, runtime,
network, carrier, or cryptographic change is authorized.

The limits below are absolute ceilings for one process instance. A counter is
charged before the work or allocation it protects, uses the stated scope, and
is never reset by retry, reconnect, carrier change, identity change, or error.
A value that cannot be charged or measured fails closed. The limits are
candidate values only: they may change for G0 only after instrumentation,
reproducible tests, and independent review demonstrate the replacement values.

## Candidate budgets (closed intervals)

| Resource | Absolute limit | Counting domain and charging rule |
|---|---:|---|
| Concurrent pre-auth states per source | 8 | Source key is the received carrier/source tuple as later specified; one state is charged from allocation through terminal cleanup. Unknown/invalid source keys use one shared bounded bucket and never evade the limit. |
| Concurrent pre-auth states globally | 1024 | All pre-auth states in the process, across every carrier and source; includes queued and teardown-pending states. |
| Input bytes per source | 64 KiB | Sum of bytes received for that source while its pre-auth state exists, including headers and rejected records; charge before parse. |
| Input packets per source | 64 | Number of received datagrams/records offered to that source during the state lifetime, including malformed and duplicate input. |
| Input bytes globally | 8 MiB | Sum of all pre-auth input bytes in the process over a one-second monotonic admission window; charge before parse. |
| Input packets globally | 8192 | Sum of all pre-auth packets in the same one-second monotonic admission window; charge before parse. |
| Parse CPU work units per packet | 4096 | One unit is one bounded parser step (field read, length check, or equivalent approved primitive); charge before each step. No input-controlled loop may exceed this count. |
| Parse CPU work units per source | 131072 | Sum of parser work units for that source over its state lifetime; includes failed parsing. |
| Parse CPU work units globally | 1,048,576 | Sum in the one-second monotonic admission window; no refill outside that window may make an already rejected operation succeed. |
| Pre-auth memory per state | 16 KiB | All state-owned bytes: metadata, parser state, transcript material, and buffers; capacity is reserved/checked before allocation. |
| Pre-auth memory globally | 16 MiB | Sum of live pre-auth state memory and pre-auth queue storage; release is counted only after ownership is gone. |
| Pending pre-auth queue per source | 4 entries | Entries include admitted, not-yet-processed input; enqueue is refused at the ceiling. |
| Pending pre-auth queue globally | 256 entries | Same entry definition across all sources/carriers; queue storage is included in global memory. |
| Response bytes per source | 2 KiB | Sum of all bytes scheduled for an unauthenticated source during its state lifetime, including failures and retransmissions. |
| Response packets per source | 4 | Number of packets scheduled for that source during its state lifetime; a packet is charged before serialization/send. |
| Response bytes globally | 256 KiB | Sum of all pre-auth response bytes in a one-second monotonic admission window. |
| Response packets globally | 512 | Sum of all pre-auth response packets in that window. |
| Pre-auth idle timeout | 1 s | Monotonic time since last admitted input or bounded progress; expiry tears down the state without response evidence. |
| Pre-auth lifetime | 5 s | Monotonic time from state creation to terminal cleanup, including queued and teardown-pending time. |
| Response-send deadline | 100 ms | Monotonic budget from response admission to completed bounded send attempt; expiry abandons the response. |

These units are accounting units, not claims about CPU cycles or wire
interoperability. Implementations must expose enough counters to prove scope,
charge ordering, saturation, release, and rejection without logging raw input
or sensitive identifiers. Counter overflow is itself budget exhaustion.

## Admission and anti-amplification

The default policy is **0-RTT disabled** and unauthenticated early application
/session data disabled. No early data can enter delivery, path, ACK, or
authorization state.

Before parsing, allocating, enqueueing, or scheduling any response, the future
implementation must atomically verify all applicable source and global budgets,
the state/lifetime deadline, and the anti-amplification allowance. The
anti-amplification allowance is:

> `response_bytes_allowed = min(2048, 3 * charged_input_bytes_for_source)`
> and `response_packets_allowed = min(4, charged_input_packets_for_source)`.

The allowance is zero until at least one input byte and packet have been charged;
bytes and packets are charged before any response decision. The proposed
response's exact bytes and packet count must be known and charged before send.
A response is refused if either allowance, source/global response budget, queue
budget, memory budget, CPU budget, or deadline would be exceeded. No partial
serialization or speculative send may bypass this check. Cumulative response
accounting includes retransmission and failed send attempts if bytes/packets
were scheduled.

Responses must not contain peer-controlled payload, identity, trust, path,
carrier, session, or parse details. The final response shape and whether a
response is sent remain subject to D018's uniform-failure review. Silence is a
valid fail-closed outcome.

## Exhaustion, cleanup, and evidence barrier

Any exhausted, saturated, unmeasurable, timed-out, malformed, or over-limit
operation fails closed: stop admission, do not retry or amplify, cancel pending
work, and release its bounded state. It must produce no `Delivery`,
`PathValidated`, or `ACK` evidence, and no equivalent authorization/session/path
evidence. It must not fabricate authentication, authorization, receipt, or
successful path state. Cleanup must not reopen admission after the lifetime
or source/global ceiling has been reached.

The evidence barrier applies equally to ordinary rejection, cancellation,
crash recovery, duplicate input, queue refusal, and resource pressure. A
counter, metric, or local diagnostic is not protocol evidence and must remain
bounded and redacted under D018.

## G0 gate and non-escalation

D019 is candidate/non-frozen documentation only. The numeric values are not
production capacity claims and cannot be changed, promoted, or treated as
G0-approved by implementation convenience. Before G0, each counter and charge
point requires instrumented implementation review, boundary/overflow tests,
load and timeout tests, concurrency tests, negative evidence tests, and
independent two-person review. Those tests must demonstrate no response
amplification and no `Delivery`/`PathValidated`/`ACK` evidence on exhaustion.

If a value, counting domain, clock, source key, response shape, or charge order
conflicts with D014–D018 or another reviewed document, G0 is **STOP** until an
explicit amendment or superseding ADR resolves it. This ADR authorizes no
implementation, merge, dependency selection, network exposure, or security
approval.

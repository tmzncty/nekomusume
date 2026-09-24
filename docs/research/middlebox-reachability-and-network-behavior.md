# Middlebox, reachability, and network-behavior research plan

**Status: research plan; not a protocol freeze, security approval, release claim, or production-readiness claim.**

This note records the next research boundary for Nekomusume after the bounded
carrier/session implementation work: the protocol can be internally coherent
and still fail on real networks because NATs, firewalls, policers, traffic
classifiers, provider policy, and other middleboxes do not behave like a
transparent IP pipe.

The purpose of this plan is to make that uncertainty measurable without
silently turning "camouflage" into a project requirement.

## 1. Research question

Nekomusume is carrier-agnostic. The important question is therefore not
"can one UDP encoding pass everywhere?" but:

> Given two owned endpoints and the communication primitives the path actually
> permits, can one logical encrypted Session remain usable, fail closed when it
> cannot, and migrate/recover with bounded loss, duplication, delay, and memory?

A path may fail for reasons that are not visible in unit tests:

- UDP may be blocked outright;
- UDP may work only on some ports or for short-lived flows;
- NAT mappings may expire or rebind;
- long-lived encrypted UDP may be throttled or classified differently;
- PMTU or fragmentation behavior may change along the path;
- idle flows may be removed;
- TCP may remain usable when UDP is degraded;
- a middlebox may make assumptions from packet sizes, timing, directionality,
  or handshake behavior rather than only from header bytes.

These are **hypotheses to measure**, not current claims about the public
Internet.

## 2. Two separate research tracks

### Track A — network-environment simulation and reachability

This is the near-term priority. It asks what happens when the network behaves
badly, without requiring the protocol to impersonate another protocol.

Required scenario families include:

- baseline delay/jitter/rate variation;
- packet loss, duplication, and reordering;
- UDP hard block;
- UDP intermittent loss or reply cessation;
- idle timeout and keepalive sensitivity;
- endpoint/source-port change;
- path replacement while Session data is in flight;
- MTU reduction and black-hole-like packet-size failure;
- TCP usable while UDP is unusable or degraded;
- repeated failover/recovery cycles;
- bounded long-lived flows;
- IPv4/IPv6 differences when owned environments become available.

The preferred ladder remains:

1. deterministic unit/state tests;
2. loopback;
3. netns/veth + tc/netem;
4. QEMU or other controlled hosts;
5. bounded experiments between owned endpoints.

Every scenario needs cleanup and must preserve the evidence-domain separation
already used by the repository: packet ACK, path validation, Session delivery,
application delivery, and application side effect are not interchangeable.

### Track B — traffic appearance and protocol-mimicry research

This track is deliberately later and evidence-triggered.

Nekomusume must not assume that changing a magic value or a few header bytes
makes traffic indistinguishable from another protocol. A middlebox may observe
features such as:

- first-flight and steady-state packet-size distributions;
- packet timing and burstiness;
- upstream/downstream ratios;
- handshake round trips;
- retransmission and timeout behavior;
- connection lifetime and idle behavior;
- path-change behavior;
- MTU/fragmentation behavior;
- version-negotiation and retry patterns.

Therefore "looks like protocol X" is not a valid claim without a defined
observable model and controlled evidence.

No mimicry mechanism is required merely because one can be imagined. Work in
this track should start only after Track A produces a reproducible failure class
that cannot be addressed more simply by:

- selecting another Carrier;
- changing bounded timeout/health policy;
- fixing NAT/idle/PMTU handling;
- improving path validation or migration;
- using a conventional mediated Carrier that is explicitly supported.

The project must not add mechanisms whose purpose is to bypass access control or
third-party network policy. Experiments remain limited to owned or explicitly
authorized endpoints under the repository's standing authorization and security
rules.

## 3. Reachability evidence model

A boolean "works / does not work" result is insufficient. For each experiment,
record enough context to explain what was actually tested.

At minimum, capture where available:

- Carrier and address family;
- endpoint ownership/provenance;
- port class and explicit test port;
- duration and payload size/count;
- first response / handshake time;
- sustained bidirectional viability;
- loss, duplicate, and conflicting-record counts;
- failure-decision time;
- first resumed-data time;
- uncertain/replayed Session ranges;
- source endpoint changes;
- observed MTU/packet-size boundary;
- cleanup result;
- kernel/provider/environment metadata that is safe to publish;
- protocol implementation commit and artifact identity.

Unknown observations stay unknown. Harness or orchestration failure must not be
promoted into protocol failure.

## 4. Simulation fixture requirements

The repository should grow reusable fixtures that make failure modes explicit
rather than embedding one-off shell behavior in individual experiments.

A scenario fixture should be able to describe, at minimum:

```text
Scenario
  |- path characteristics
  |    |- delay / jitter / rate
  |    |- loss / reorder / duplicate
  |    '- MTU / packet-size condition
  |- carrier availability
  |    |- UDP allowed/degraded/blocked
  |    '- TCP allowed/degraded/blocked
  |- lifecycle
  |    |- idle period
  |    |- endpoint change
  |    '- failure/recovery schedule
  '- evidence expectations
       |- delivery/loss/duplicate bounds
       |- failover/recovery bound
       '- cleanup/resource bound
```

The first implementation should prefer deterministic netns/netem fixtures. It
does not need to model a specific vendor firewall or DPI engine. The goal is to
reproduce classes of observable network behavior before attempting product-
specific emulation.

## 5. Resource behavior is part of correctness

A resilient Session that survives path failure by accumulating unbounded state
is not acceptable.

Network-failure experiments should therefore also measure or assert bounded:

- send/retransmission queue records;
- queue bytes;
- uncertain delivery state;
- path records/history;
- retry/PTO state;
- process memory where an executable fixture makes that practical.

Long-lived and repeated-failover tests should specifically check that resource
use follows current in-flight work rather than monotonically increasing with the
age of the process.

## 6. Carrier-manager consequences

This research should inform Carrier selection rather than force all environments
through one wire image.

The intended decision shape remains:

```text
observe path/carrier evidence
          |
          v
validate + score + hysteresis
          |
          v
keep current Carrier when healthy
          |
          +--> degrade/fail when evidence justifies it
                    |
                    v
            select validated fallback
                    |
                    v
          resume uncertain Session data
```

A successful TCP fallback is evidence that the Session survived a UDP failure;
it is not evidence that UDP was "camouflaged successfully". Conversely, a UDP
failure in one owned path is not a global statement about UDP reachability.

## 7. Initial executable slices

The following are useful bounded slices once they are dependency-ready and do
not displace an earlier release/security blocker:

1. Add a reusable netns/netem scenario description for UDP reply cessation,
   hard loss, reorder, and MTU reduction.
2. Add repeated failover fixtures that assert delivery/duplicate/loss and
   resource bounds across multiple cycles.
3. Extend reachability artifacts with explicit path-condition provenance and
   "unknown" values for observations the harness did not collect.
4. Add a long-lived bounded fixture that detects monotonic queue/state growth.
5. Only after reproducible real-path failures exist, define an observational
   fingerprint record for research; do not implement protocol mimicry first.

## 8. Non-goals

This plan does not:

- claim that Nekomusume currently evades or survives DPI/middleboxes;
- claim that any traffic shape is indistinguishable from another protocol;
- authorize experiments against third-party networks or access controls;
- freeze a camouflage format or Carrier;
- change Session/Carrier separation;
- promote current implementation status to RC, release-grade, frozen, or
  released.

The guiding rule is the same as the rest of the repository:

> **Measure the path first. Prefer a simpler Carrier or recovery fix when it
> solves the observed failure. Only study traffic mimicry for a reproducible
> problem that remains after those options are exhausted.**

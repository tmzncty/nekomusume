#!/usr/bin/env python3
"""Deterministic toy simulator for D076 causal hypotheses.

This is deliberately *not* a network emulator. It is a small inference guardrail:
it demonstrates that aggregate throughput alone cannot identify the mechanism behind
an N=1/2/4/8 jump.  The real experiment protocol lives in
`docs/experiments/d076-causal-recheck.md`.

No third-party dependencies are required.
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from typing import Iterable, Sequence


@dataclass(frozen=True)
class Observation:
    label: str
    streams_mbps: tuple[float, ...]

    @property
    def aggregate_mbps(self) -> float:
        return sum(self.streams_mbps)


def _fmt(obs: Observation) -> str:
    detail = ", ".join(f"{x:.2f}" for x in obs.streams_mbps)
    return f"{obs.label:>12}: sum={obs.aggregate_mbps:7.2f} Mbps  streams=[{detail}]"


class TimeWindowModel:
    """Path recovers by measurement epoch; stream count is incidental."""

    def __init__(self, capacities: Sequence[float]):
        self.capacities = tuple(capacities)
        self.epoch = 0

    def probe(self, n: int, label: str) -> Observation:
        cap = self.capacities[min(self.epoch, len(self.capacities) - 1)]
        self.epoch += 1
        # A recovered bottleneck is shared approximately equally by active flows.
        return Observation(label, tuple(cap / n for _ in range(n)))


class HashPathModel:
    """Flow identity selects a slow or fast path deterministically by source port."""

    def __init__(self, slow: float = 0.6, fast: float = 198.0, modulus: int = 4):
        self.slow = slow
        self.fast = fast
        self.modulus = modulus

    def rate_for_port(self, source_port: int) -> float:
        # One bucket in `modulus` is the good path.  The exact hash is intentionally
        # simple and deterministic; the point is causal shape, not ECMP fidelity.
        return self.fast if source_port % self.modulus == 2 else self.slow

    def probe(self, ports: Iterable[int], label: str) -> Observation:
        return Observation(label, tuple(self.rate_for_port(p) for p in ports))


class PerFlowPolicerModel:
    """Each flow has the same independent ceiling."""

    def __init__(self, per_flow: float = 0.6):
        self.per_flow = per_flow

    def probe(self, n: int, label: str) -> Observation:
        return Observation(label, tuple(self.per_flow for _ in range(n)))


class ThresholdModel:
    """Shared classifier flips state when enough simultaneous flows exist."""

    def __init__(self, threshold: int = 4, slow_total: float = 1.0, fast_total: float = 200.0):
        self.threshold = threshold
        self.slow_total = slow_total
        self.fast_total = fast_total

    def probe(self, n: int, label: str) -> Observation:
        total = self.fast_total if n >= self.threshold else self.slow_total
        return Observation(label, tuple(total / n for _ in range(n)))


def scenario_time_window() -> list[Observation]:
    # Deliberately mirrors a monotonic run where the network recovers between N=2
    # and N=4.  Aggregate shape resembles D076 without any stream-count causality.
    model = TimeWindowModel((0.49, 3.65, 199.0, 193.0))
    return [model.probe(n, f"N={n}") for n in (1, 2, 4, 8)]


def scenario_hash_path() -> list[Observation]:
    model = HashPathModel(slow=0.6, fast=197.2, modulus=4)
    # Choose distinct ephemeral-port windows. N=4 is the first window that contains
    # a good bucket; N=8 contains two. This is a toy analogue of flow-hash lottery.
    port_sets = {
        1: (30000,),
        2: (30004, 30005),
        4: (30008, 30009, 30010, 30011),
        8: tuple(range(30012, 30020)),
    }
    return [model.probe(port_sets[n], f"N={n}") for n in (1, 2, 4, 8)]


def scenario_per_flow() -> list[Observation]:
    model = PerFlowPolicerModel(0.6)
    return [model.probe(n, f"N={n}") for n in (1, 2, 4, 8)]


def scenario_threshold() -> list[Observation]:
    model = ThresholdModel(threshold=4, slow_total=1.0, fast_total=200.0)
    return [model.probe(n, f"N={n}") for n in (1, 2, 4, 8)]


def aba_demo(kind: str) -> list[Observation]:
    if kind == "time-window":
        # Recovery happens after the second probe. A/B/A reveals that the later A
        # stays fast, falsifying "N=4 caused the recovery".
        model = TimeWindowModel((0.5, 200.0, 185.0, 200.0, 190.0))
        return [model.probe(n, f"#{i} N={n}") for i, n in enumerate((1, 4, 1, 4, 1), 1)]

    if kind == "threshold":
        model = ThresholdModel(threshold=4, slow_total=0.5, fast_total=200.0)
        return [model.probe(n, f"#{i} N={n}") for i, n in enumerate((1, 4, 1, 4, 1), 1)]

    raise ValueError(kind)


def port_sweep(start: int = 30000, count: int = 12) -> list[Observation]:
    model = HashPathModel(slow=0.6, fast=190.0, modulus=4)
    return [model.probe((p,), f"src={p}") for p in range(start, start + count)]


def self_test() -> None:
    # Plain per-flow policer must scale linearly; it cannot create an N=2 -> N=4
    # discontinuity by itself.
    linear = [o.aggregate_mbps for o in scenario_per_flow()]
    assert linear == [0.6, 1.2, 2.4, 4.8], linear

    # Time drift can recreate the exact aggregate D076 sequence by construction.
    drift = [round(o.aggregate_mbps, 2) for o in scenario_time_window()]
    assert drift == [0.49, 3.65, 199.0, 193.0], drift

    # A/B/A discriminates a time recovery from a true concurrency threshold.
    time_aba = [o.aggregate_mbps for o in aba_demo("time-window")]
    threshold_aba = [o.aggregate_mbps for o in aba_demo("threshold")]
    assert time_aba[2] > 100.0
    assert threshold_aba[2] < 1.0

    # Port sweep must contain stable fast and slow identities.
    sweep = [o.aggregate_mbps for o in port_sweep()]
    assert max(sweep) > 100.0 and min(sweep) < 1.0


def print_group(title: str, observations: Sequence[Observation]) -> None:
    print(f"\n== {title} ==")
    for obs in observations:
        print(_fmt(obs))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--mode",
        choices=("all", "aba", "ports", "self-test"),
        default="all",
        help="which demonstration to run",
    )
    args = parser.parse_args()

    if args.mode == "self-test":
        self_test()
        print("d076 causal simulator: self-test passed")
        return 0

    if args.mode in ("all", "aba"):
        print_group("A/B/A: time-window recovery", aba_demo("time-window"))
        print_group("A/B/A: true four-flow threshold", aba_demo("threshold"))

    if args.mode in ("all", "ports"):
        print_group("source-port sweep: hashed path", port_sweep())

    if args.mode == "all":
        print_group("monotonic N: time-window confound", scenario_time_window())
        print_group("monotonic N: flow-hash lottery", scenario_hash_path())
        print_group("monotonic N: simple per-flow policer", scenario_per_flow())
        print_group("monotonic N: shared threshold", scenario_threshold())
        self_test()
        print("\nself-test: passed")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())

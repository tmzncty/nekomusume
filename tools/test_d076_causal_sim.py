#!/usr/bin/env python3
"""Unit tests for tools/d076_causal_sim.py without third-party dependencies."""

from __future__ import annotations

import importlib.util
from pathlib import Path
import unittest


MODULE_PATH = Path(__file__).with_name("d076_causal_sim.py")
SPEC = importlib.util.spec_from_file_location("d076_causal_sim", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
sim = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(sim)


class D076CausalSimTests(unittest.TestCase):
    def test_simple_per_flow_policer_is_linear(self) -> None:
        totals = [o.aggregate_mbps for o in sim.scenario_per_flow()]
        self.assertEqual(totals, [0.6, 1.2, 2.4, 4.8])

    def test_time_window_can_recreate_recorded_aggregate_sequence(self) -> None:
        totals = [round(o.aggregate_mbps, 2) for o in sim.scenario_time_window()]
        self.assertEqual(totals, [0.49, 3.65, 199.0, 193.0])

    def test_aba_separates_recovery_from_threshold(self) -> None:
        recovered_a = sim.aba_demo("time-window")[2].aggregate_mbps
        threshold_a = sim.aba_demo("threshold")[2].aggregate_mbps
        self.assertGreater(recovered_a, 100.0)
        self.assertLess(threshold_a, 1.0)

    def test_source_port_sweep_has_stable_fast_and_slow_buckets(self) -> None:
        totals = [o.aggregate_mbps for o in sim.port_sweep()]
        self.assertGreater(max(totals), 100.0)
        self.assertLess(min(totals), 1.0)


if __name__ == "__main__":
    unittest.main()

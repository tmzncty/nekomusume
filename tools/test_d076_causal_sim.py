#!/usr/bin/env python3
"""Unit tests for tools/d076_causal_sim.py without third-party dependencies."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

# Import as a proper module (sys.path insertion) instead of
# importlib.util.spec_from_file_location: the loader path chokes on
# dataclass processing when the module defines `from __future__ import
# annotations` (the class-dict lookup for KW_ONLY lands on None under
# the frozen bootstrap on 3.12). A plain import keeps the module in
# sys.modules where dataclasses can resolve everything.
sys.path.insert(0, str(Path(__file__).resolve().parent))
import d076_causal_sim as sim  # noqa: E402


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

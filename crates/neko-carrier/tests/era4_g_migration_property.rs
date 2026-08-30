use neko_carrier::{
    CarrierManager, HealthSample, ManagerLimits, MigrationCandidate, PathGeneration, PathId,
};
#[test]
fn deterministic_migration_candidates_never_switch_before_all_gates() {
    for seed in 0..128u64 {
        let mut m = CarrierManager::new(ManagerLimits {
            min_hold_events: 2,
            switch_margin: 10,
            max_paths: 2,
        })
        .unwrap();
        let tcp = PathId(1);
        let udp = PathId(2);
        m.observe(
            tcp,
            HealthSample {
                rtt_us: 500,
                loss_per_mille: 0,
                pto: 0,
            },
        )
        .unwrap();
        m.observe(
            udp,
            HealthSample {
                rtt_us: 100 + (seed % 50),
                loss_per_mille: 0,
                pto: 0,
            },
        )
        .unwrap();
        m.set_active_tcp(tcp, PathGeneration(4)).unwrap();
        let c = MigrationCandidate {
            path: udp,
            generation: PathGeneration(4),
            validated: true,
            health: HealthSample {
                rtt_us: 100 + (seed % 50),
                loss_per_mille: 0,
                pto: 0,
            },
        };
        assert!(m.migrate_back_to_udp(c).is_err());
        assert!(m.migrate_back_to_udp(c).is_err());
        assert_eq!(m.migrate_back_to_udp(c), Ok(true));
        assert_eq!(m.active(), Some(udp));
    }
}

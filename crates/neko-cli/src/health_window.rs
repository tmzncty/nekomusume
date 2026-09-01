use neko_carrier::{
    CarrierHealthEvidence, HealthFailureCause, HealthObservation, HealthState, PathId,
};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HealthDatagram {
    WrongPeer,
    MalformedOrUnadmitted,
    StaleOrNonmatchingAuthenticated,
    PermittedProgress,
}

#[derive(Debug)]
pub(crate) struct HealthObservationWindow {
    started_at: Instant,
    deadline: Instant,
    duration: Duration,
    ignored: usize,
    max_ignored: usize,
}

impl HealthObservationWindow {
    pub(crate) fn new(started_at: Instant, duration: Duration, max_ignored: usize) -> Self {
        Self {
            started_at,
            deadline: started_at + duration,
            duration,
            ignored: 0,
            max_ignored,
        }
    }

    pub(crate) fn started_at(&self) -> Instant {
        self.started_at
    }

    pub(crate) fn deadline(&self) -> Instant {
        self.deadline
    }

    pub(crate) fn observe(
        &mut self,
        datagram: HealthDatagram,
        now: Instant,
        evidence: &mut CarrierHealthEvidence,
        path: PathId,
    ) -> Result<Option<HealthState>, &'static str> {
        if now >= self.deadline {
            return Ok(Some(self.expire(now, evidence, path)?));
        }
        match datagram {
            HealthDatagram::PermittedProgress => {
                self.restart(now);
                evidence
                    .observe_event(path, HealthObservation::Progress)
                    .map(Some)
                    .map_err(|_| "health evidence rejected progress")
            }
            HealthDatagram::WrongPeer
            | HealthDatagram::MalformedOrUnadmitted
            | HealthDatagram::StaleOrNonmatchingAuthenticated => {
                self.ignored = self.ignored.saturating_add(1).min(self.max_ignored);
                Ok(None)
            }
        }
    }

    pub(crate) fn expire(
        &mut self,
        now: Instant,
        evidence: &mut CarrierHealthEvidence,
        path: PathId,
    ) -> Result<HealthState, &'static str> {
        if now < self.deadline {
            return Err("health observation window has not expired");
        }
        let state = evidence
            .observe_event(
                path,
                HealthObservation::Failure(HealthFailureCause::AuthenticatedDeliveryAckTimeout),
            )
            .map_err(|_| "health evidence rejected failure")?;
        self.restart(now);
        Ok(state)
    }

    fn restart(&mut self, now: Instant) {
        self.started_at = now;
        self.deadline = now + self.duration;
        self.ignored = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use neko_carrier::{HealthEvidenceLimits, HealthLimits};

    const SECOND: Duration = Duration::from_secs(1);
    const PATH: PathId = PathId(1);

    fn evidence() -> CarrierHealthEvidence {
        CarrierHealthEvidence::new(
            HealthLimits {
                degrade_after: 2,
                fail_after: 3,
                recover_after: 2,
                max_paths: 2,
            },
            HealthEvidenceLimits { max_samples: 16 },
        )
        .unwrap()
    }

    #[test]
    fn junk_rate_cannot_advance_reset_or_accelerate_failure_schedule() {
        let start = Instant::now();
        for junk in [
            HealthDatagram::WrongPeer,
            HealthDatagram::MalformedOrUnadmitted,
            HealthDatagram::StaleOrNonmatchingAuthenticated,
        ] {
            let mut evidence = evidence();
            let mut window = HealthObservationWindow::new(start, SECOND, 8);
            for i in 0..10_000 {
                assert_eq!(
                    window
                        .observe(
                            junk,
                            start + Duration::from_nanos(i.min(999_999_999)),
                            &mut evidence,
                            PATH,
                        )
                        .unwrap(),
                    None
                );
            }
            assert_eq!(window.deadline(), start + SECOND);
        }
    }

    #[test]
    fn one_and_two_windows_do_not_fail_third_only_creates_failed_evidence() {
        let start = Instant::now();
        let mut evidence = evidence();
        let mut window = HealthObservationWindow::new(start, SECOND, 8);
        assert_eq!(
            window.expire(start + SECOND, &mut evidence, PATH).unwrap(),
            HealthState::Unknown
        );
        assert_eq!(
            window
                .expire(start + SECOND * 2, &mut evidence, PATH)
                .unwrap(),
            HealthState::Degraded
        );
        assert_eq!(
            window
                .expire(start + SECOND * 3, &mut evidence, PATH)
                .unwrap(),
            HealthState::Failed
        );
    }

    #[test]
    fn exact_permitted_progress_resets_failure_counter_and_window() {
        let start = Instant::now();
        let mut evidence = evidence();
        let mut window = HealthObservationWindow::new(start, SECOND, 8);
        assert_eq!(
            window.expire(start + SECOND, &mut evidence, PATH).unwrap(),
            HealthState::Unknown
        );
        let progress_at = start + SECOND + Duration::from_millis(500);
        assert_eq!(
            window
                .observe(
                    HealthDatagram::PermittedProgress,
                    progress_at,
                    &mut evidence,
                    PATH,
                )
                .unwrap(),
            Some(HealthState::Healthy)
        );
        assert_eq!(window.deadline(), progress_at + SECOND);
        assert_eq!(
            window
                .expire(progress_at + SECOND, &mut evidence, PATH)
                .unwrap(),
            HealthState::Healthy
        );
    }
}

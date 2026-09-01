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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HealthWindowError {
    Evidence(&'static str),
    AdmissionBudgetExhausted { ignored: usize },
    NotExpired,
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
    ) -> Result<Option<HealthState>, HealthWindowError> {
        if now >= self.deadline {
            return Ok(Some(self.expire(now, evidence, path)?));
        }
        match datagram {
            HealthDatagram::PermittedProgress => {
                self.restart(now);
                evidence
                    .observe_event(path, HealthObservation::Progress)
                    .map(Some)
                    .map_err(|_| HealthWindowError::Evidence("health evidence rejected progress"))
            }
            HealthDatagram::WrongPeer
            | HealthDatagram::MalformedOrUnadmitted
            | HealthDatagram::StaleOrNonmatchingAuthenticated => {
                if self.ignored == self.max_ignored {
                    return Err(HealthWindowError::AdmissionBudgetExhausted {
                        ignored: self.ignored,
                    });
                }
                self.ignored += 1;
                Ok(None)
            }
        }
    }

    pub(crate) fn expire(
        &mut self,
        now: Instant,
        evidence: &mut CarrierHealthEvidence,
        path: PathId,
    ) -> Result<HealthState, HealthWindowError> {
        if now < self.deadline {
            return Err(HealthWindowError::NotExpired);
        }
        let state = evidence
            .observe_event(
                path,
                HealthObservation::Failure(HealthFailureCause::AuthenticatedDeliveryAckTimeout),
            )
            .map_err(|_| HealthWindowError::Evidence("health evidence rejected failure"))?;
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
    fn junk_budget_exhaustion_is_distinct_and_does_not_advance_or_accelerate_health() {
        let start = Instant::now();
        for junk in [
            HealthDatagram::WrongPeer,
            HealthDatagram::MalformedOrUnadmitted,
            HealthDatagram::StaleOrNonmatchingAuthenticated,
        ] {
            let mut evidence = evidence();
            let mut window = HealthObservationWindow::new(start, SECOND, 1_024);
            for i in 0..1_024 {
                assert_eq!(
                    window
                        .observe(junk, start + Duration::from_micros(i), &mut evidence, PATH)
                        .unwrap(),
                    None
                );
            }
            assert_eq!(
                window.observe(
                    junk,
                    start + Duration::from_millis(999),
                    &mut evidence,
                    PATH
                ),
                Err(HealthWindowError::AdmissionBudgetExhausted { ignored: 1_024 })
            );
            assert_eq!(window.deadline(), start + SECOND);
            assert_eq!(evidence.events().len(), 0);
        }
    }

    #[test]
    fn exact_progress_at_budget_boundary_remains_detectable_and_resets_window() {
        let start = Instant::now();
        let mut evidence = evidence();
        let mut window = HealthObservationWindow::new(start, SECOND, 2);
        assert_eq!(
            window
                .observe(HealthDatagram::WrongPeer, start, &mut evidence, PATH)
                .unwrap(),
            None
        );
        assert_eq!(
            window
                .observe(
                    HealthDatagram::MalformedOrUnadmitted,
                    start,
                    &mut evidence,
                    PATH
                )
                .unwrap(),
            None
        );
        let progress = start + Duration::from_millis(10);
        assert_eq!(
            window
                .observe(
                    HealthDatagram::PermittedProgress,
                    progress,
                    &mut evidence,
                    PATH
                )
                .unwrap(),
            Some(HealthState::Healthy)
        );
        assert_eq!(window.deadline(), progress + SECOND);
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

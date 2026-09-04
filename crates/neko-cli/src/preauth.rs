use neko_crypto::{
    PreauthBudget, PreauthLimits, PreauthStateId, ProcessPreauthAdmission, ProcessPreauthLimits,
};
use std::net::SocketAddr;
use std::time::Instant;

const RESERVED_STATE_BYTES: usize = 16 * 1024;

pub(crate) struct ListenerAdmission {
    started: Instant,
    process: ProcessPreauthAdmission,
}

pub(crate) struct AdmissionTicket {
    id: PreauthStateId,
    budget: PreauthBudget,
}

impl ListenerAdmission {
    pub(crate) fn new() -> Self {
        Self {
            started: Instant::now(),
            process: ProcessPreauthAdmission::new(ProcessPreauthLimits::default(), 0)
                .expect("default pre-auth limits are valid"),
        }
    }

    fn now_ms(&self) -> u64 {
        self.started
            .elapsed()
            .as_millis()
            .try_into()
            .unwrap_or(u64::MAX)
    }

    pub(crate) fn expire(&mut self) {
        let _ = self.process.expire(self.now_ms());
    }

    pub(crate) fn admit(&mut self, peer: SocketAddr) -> Result<AdmissionTicket, ()> {
        self.expire();
        let now = self.now_ms();
        let id = self
            .process
            .admit_state(&source_key(peer), RESERVED_STATE_BYTES, now)
            .map_err(|_| ())?;
        let budget = match PreauthBudget::new(PreauthLimits::default()) {
            Ok(budget) => budget,
            Err(_) => {
                let _ = self.process.release(id);
                return Err(());
            }
        };
        Ok(AdmissionTicket { id, budget })
    }

    pub(crate) fn charge_input(
        &mut self,
        ticket: &mut AdmissionTicket,
        bytes: usize,
        work_units: usize,
    ) -> Result<(), ()> {
        let now = self.now_ms();
        ticket.budget.charge_input(bytes).map_err(|_| ())?;
        if self
            .process
            .charge_input(ticket.id, bytes, work_units, now)
            .is_err()
        {
            let _ = ticket.budget.rollback_input(bytes);
            return Err(());
        }
        Ok(())
    }

    pub(crate) fn charge_response(
        &mut self,
        ticket: &mut AdmissionTicket,
        bytes: usize,
    ) -> Result<(), ()> {
        let now = self.now_ms();
        ticket.budget.charge_response(bytes).map_err(|_| ())?;
        if self.process.charge_response(ticket.id, bytes, now).is_err() {
            let _ = ticket.budget.rollback_response(bytes);
            return Err(());
        }
        Ok(())
    }

    pub(crate) fn release(&mut self, ticket: AdmissionTicket) {
        let _ = self.process.release(ticket.id);
    }
}

fn source_key(peer: SocketAddr) -> Vec<u8> {
    let mut key = Vec::with_capacity(19);
    match peer.ip() {
        std::net::IpAddr::V4(ip) => {
            key.push(4);
            key.extend_from_slice(&ip.octets());
        }
        std::net::IpAddr::V6(ip) => {
            key.push(6);
            key.extend_from_slice(&ip.octets());
        }
    }
    key.extend_from_slice(&peer.port().to_be_bytes());
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_projection_is_family_and_port_bound_without_text() {
        let a: SocketAddr = "127.0.0.1:40080".parse().unwrap();
        let b: SocketAddr = "127.0.0.1:40081".parse().unwrap();
        let v6: SocketAddr = "[::1]:40080".parse().unwrap();
        assert_ne!(source_key(a), source_key(b));
        assert_ne!(source_key(a), source_key(v6));
        assert_eq!(source_key(a).len(), 7);
        assert_eq!(source_key(v6).len(), 19);
    }

    #[test]
    fn response_requires_charged_input_and_release_reopens_source() {
        let mut admission = ListenerAdmission::new();
        let peer: SocketAddr = "127.0.0.1:40080".parse().unwrap();
        let mut ticket = admission.admit(peer).unwrap();
        assert!(admission.charge_response(&mut ticket, 1).is_err());
        admission.charge_input(&mut ticket, 64, 16).unwrap();
        admission.charge_response(&mut ticket, 64).unwrap();
        admission.release(ticket);
        assert!(admission.admit(peer).is_ok());
    }
}

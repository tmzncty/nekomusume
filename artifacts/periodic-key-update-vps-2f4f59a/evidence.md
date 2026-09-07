# Bounded self-owned VPS periodic TCP key-update observation

**Scope:** one bounded correctness observation only. This is not security approval, production readiness, a reliability rate, a dynamic rekey protocol, or a reachability/performance claim.

- Experiment ID: `periodic-key-update-vps-2f4f59a`
- Git commit: `2f4f59a9887dbcde2973175d3f7abf30f8edaa15`
- Target: administrator-controlled VPS, verified by non-secret `hostname`, UID, and architecture checks (`VM-0-6-ubuntu`, UID `0`, `x86_64`)
- Transport: authenticated periodic TCP
- Application profile: 3 records, 16 bytes per record, 48 application bytes total
- Fixed schedule: `--key-update-after 1`
- Release binary: 1,208,216 bytes
- Release binary SHA-256: `6be15273b2ce4b3e241d800d15d8eefd334d6d37615bae733c5bff4a64a89a43`

## Observed result

The client and server authenticated one Session and both emitted the secret-free event `key_phase=1` after sequence 1. The client then completed:

- `attempted=3 confirmed=3 missing=0 duplicates=0`
- `application_bytes=48`

The server completed:

- `received=3 confirmed=3 duplicates=0`

All three application exchanges crossed the fixed phase boundary without a missing or duplicate record in this observation.

## Cleanup

After the run, the remote experiment path was removed and bounded post-run observation found no listener on TCP port 40080 and no remaining Nekomusume experiment process. Temporary client identity material was removed locally. The SSH private key was used only by the SSH client and its contents were not read, printed, copied, or committed.

## Boundary

This result proves only that two controlled peers can synchronously rekey an already authenticated bounded TCP Session at this fixed out-of-band schedule while application/DeliveryAck traffic continues on the observed path. It does not prove arbitrary peer-initiated rekey, dynamic resynchronization, long-term rotation policy, public reachability, reliability rate, production readiness, or superiority.

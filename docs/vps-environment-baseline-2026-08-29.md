# De-identified VPS environment baseline — 2026-08-29

Status: read-only observation of an administrator-owned VPS. No listener, client,
packet capture, firewall change, package installation, or WAN experiment was run.

The machine is Ubuntu 24.04.4 LTS on Linux 6.8.0-124-generic, x86_64, with two
logical CPUs and about 3.6 GiB RAM. The primary interface MTU is 1500; the
observed IPv6 route advertises MTU 1464. The interface uses an `mq` root with
`fq_codel` leaves and TCP `cubic`. Time synchronization is active through
chrony. Host nftables was empty at capture, while iptables used the nft backend
with ACCEPT policies; the provider firewall remains unknown and cannot be
inferred from the guest.

The machine had no Nekomusume or Hysteria2 binary/package/service installed.
SCTP was absent. DCCP was configured as a module but not loaded. Existing raw
socket state was observed only read-only and no capability escalation was
attempted.

Canonical machine-readable evidence is
`artifacts/environment/vps-2026-08-29.json`. Hostnames, public/private addresses,
route peers, account identifiers, and other topology identifiers are omitted.

# Exact-current package VPS operator smoke — `a9928c8`

## Scope and boundary

One bounded self-owned client↔VPS package/operator observation for exact commit `a9928c809e493e1c7f06ba692958fee6608b4f7d`. This is not release approval, RC, production readiness, public/general reachability, or a distinct-version compatibility result. The existing production Hysteria service was not modified.

- Experiment ID: `package-vps-a992-20260909`
- VPS target: `self-owned-vps-A`; remote bind: `private-bind-A`
- Target architecture: `x86_64`
- Experimental ports: TCP `40080`, UDP `40081`
- Application profile: 2 authenticated exchanges × 32 bytes per transport; bounded 5-second server duration
- Fresh temporary client/server identities were generated under a local temporary directory; repository identity material was not used or uploaded.

## Package identity

- Archive: `nekomusume-0.1.0-x86_64-unknown-linux-gnu.tar.gz`
- Archive SHA-256: `55c3756b275c49c8c6e637278fb500e6451dc319d770a547585cc1e7134211aa`
- Local packaged binary SHA-256: `6cf74e8f1f027c10b843fc98ff0f31e12b7f13bb3495eb5b5dbc1bbb0320d7e2`
- Installed VPS binary SHA-256: `6cf74e8f1f027c10b843fc98ff0f31e12b7f13bb3495eb5b5dbc1bbb0320d7e2`
- VPS `capabilities --json`: passed with `secret_free=true`

The archive was built twice locally with the same archive SHA-256 and passed `scripts/release/smoke-package.sh` before upload.

## Results

- TCP: `probe_ok transport=tcp bytes=32` for each of the two bounded exchanges; server emitted `READY` then `STOPPED`.
- UDP: `probe_ok transport=udp bytes=32` for each of the two bounded exchanges; server emitted `READY` then `STOPPED`.
- The installed binary hash matched the local package binary hash before either smoke.

These are bounded authenticated self-owned client↔VPS package observations. They do not establish sustained WAN reliability, public reachability, natural degradation/failover behavior, performance superiority, or production suitability.

## Cleanup

Before deletion, remote checks verified no listener remained on ports `40080` or `40081` and no experiment process remained. The dedicated remote package/log/identity path was then removed and verified absent. Local temporary package, state, identity, and logs were removed by trap cleanup. No production path, route, firewall, DNS, proxy, tunnel, qdisc, or existing listener was changed.

## Remaining boundaries

Historical N5 distinct-version A→B→A evidence remains separate. aarch64, signed publication/SBOM, service-manager hardening, independent release/security approval, D019 source retention, RC, production, and release flags remain unresolved/unchanged.

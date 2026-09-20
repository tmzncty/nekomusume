# H-R9-083 reopen — partial `/proc/net` observation can still be promoted to cleanup success

**Classification:** HIGH — release-gate correctness/evidence truth  
**Reviewed repository HEAD:** `5080325dbf643da408e10d2442c687f73a71c84e`  
**Executable source/test anchor challenged:** `1dbe1543e5b82458c90a866cc5f0fc8c9bbd4a24`  
**Owners:** `scripts/bench/process-resource-sampler.py`, `scripts/bench/process-resource-sampler-test.py`, `scripts/bench/validate-process-resource.py`  
**Scope:** R9-11D D2/D3 only. No WAN/live/performance conclusion.

## Why the previous closure is not accepted

The original H-R9-083 closure contract already required that **any incomplete terminal `/proc/net` observation remain unknown** and never become `cleanup.owned_sockets_after_exit=0` / `cleanup.complete=true`.

The `1dbe154` repair correctly widened the terminal oracle to TCP/TCP6/UDP/UDP6 and introduced a present/absent/unknown return shape, but the implementation does not actually make partial observation fail closed.

`owned_port_sockets_present()` increments `files_read` for each readable table, silently `continue`s on `OSError`, and finally returns `False` whenever `files_read > 0` and no match was found. Therefore the function can report definitive absence after reading only a subset of the four protocol tables.

Concrete counterexample:

1. `/proc/net/tcp`, `/proc/net/tcp6`, and `/proc/net/udp` are readable and contain no supplied port;
2. `/proc/net/udp6` cannot be read;
3. an escaped/reparented descendant owns the supplied port only as UDP6;
4. `files_read == 3`, so the helper returns `False`;
5. if the original process group is empty, result construction promotes this to `owned_sockets_after_exit=0` and `cleanup.complete=true`.

That contradicts the helper docstring, the previous H-R9-083 requirement, and the release-evidence boundary that unknown cleanup observation must not be promoted to success.

Malformed/unparseable terminal table rows are also currently skipped. If such a row is the only row that could carry the supplied port, the same false-absence promotion is possible. Kernel `/proc/net` is normally well formed, but the committed evidence contract is fail-closed observation truth, so the regression must challenge partial/unparseable observation rather than rely on normal-host formatting.

## Regression gap

The new escaped TCP and UDP fixtures prove the positive `socket still present -> cleanup incomplete` case on a normal host, but they do not inject a **partial terminal observation**. There is no focused regression showing that one unavailable required table (or an unparseable relevant table) makes the terminal oracle `unknown` rather than `absent`.

The escaped-descendant fixtures also still perform helper cleanup only after their assertions; an assertion failure can leave the 30-second escaped helper alive. The original H-R9-083 contract required bounded cleanup even on assertion failure. This is secondary to the source-level partial-observation defect but remains part of truthful fixture closure.

## Required closure

Use the smallest settled-semantic repair:

1. keep TCP/TCP6 LISTEN plus UDP/UDP6 bound-socket coverage;
2. return `False` only after **all required protocol tables were successfully and completely observed** with no supplied-port ownership found;
3. any required table read failure, or any parse failure that prevents complete ownership determination, must leave the terminal observation `unknown` unless ownership was already affirmatively found (`True`);
4. add focused deterministic regression(s) for partial observation: e.g. three tables readable/no match plus the fourth unavailable must yield `unknown`, and result construction must not produce zero/complete;
5. keep escaped TCP and UDP owner regressions green and make escaped-helper cleanup assertion-safe/bounded (`try/finally` or equivalent);
6. do not invent timeout/capacity/security policy values or redesign Session/Carrier/ACK/wire/crypto.

## Validation contract

Run focused process-resource sampler/validator tests, then on the final pushed developer source/test SHA run:

- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- `git diff --check`
- confirm clean tree and persist exact pushed SHA, UTC start/end, exit codes, OS/arch and stable Rust version.

No decoder/parser/crypto framing change is involved, so no mechanical fuzz run is required.

No reviewer-local test execution is claimed for this finding. The counterexample is from exact-current source inspection. GitHub combined status/workflow lookup for `1dbe154` exposed no hosted run; absence of hosted evidence is not treated as failure.

Until this closes, R9-11D2/D3 and R9-12 remain blocked behind this HIGH. Independent D1 result-truth review may continue only if it does not rely on the false terminal-cleanup claim.
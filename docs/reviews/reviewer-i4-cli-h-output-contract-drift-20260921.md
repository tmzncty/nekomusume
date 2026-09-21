# Reviewer I4-CLI-H factual challenge — human reachability output contract drift

**Anchor:** reachable `87b371d0d28b7a3df2c105770f1420e61eb4c349`.

**Scope:** exact-current `crates/neko-cli/src/main.rs::matrix_probe`, `README.md`, `ROADMAP.md`, D007 in `docs/decisions.md`, `docs/carrier-architecture.md`, and the developer-authored I4-CLI-H notes. This is a bounded source/docs challenge only. No reviewer-local command execution, hosted-CI result, WAN evidence, security approval, or release decision is claimed.

## Challenged invariant

Current human reachability output, exit status, and the repository's claimed frozen/documented line format must describe the same contract.

## Finding

The executable current owner is internally consistent: `matrix_probe` evaluates the same literal `artifact.contains("\"reachable\":true")` predicate for its human branch and exit status. In human mode it emits the complete lines `pass: 喵~！` or `fail: 喵呜呜呜呜…`; in `--json` mode it emits the artifact rather than the human line. The current ROADMAP also records the prefixed `pass:` / `fail:` lines.

However, `README.md` says the human-readable output is already frozen and shows only `喵~！` / `喵呜呜呜呜…` after explanatory labels. `docs/carrier-architecture.md` section 8.1 similarly calls the bare cat strings the official probe output, and accepted D007 records the same older wording. Therefore the developer I4-CLI-H no-finding claim that README/ROADMAP/D007 all truthfully match the exact emitted line format is not established on the current tree.

This is a factual/contract-documentation inconsistency, not evidence that the executable exit/JSON branch is currently wrong. Do not refactor the CLI merely to satisfy stale prose. First reconcile the exact intended human line using current implementation/tests and the already-current ROADMAP; if the prefixed lines are the intended contract, make the smallest factual documentation correction in the authoritative/current prose without widening reachability semantics. If changing D007's accepted meaning would be required rather than clarifying stale wording, classify that part as a maintainer/design decision instead of silently rewriting it.

## Result

**I4-CLI-H remains READY_LOCAL for one narrow reconciliation + independent re-check.** The repository-wide `queue exhausted` statement at `87b371d0` is therefore not accepted yet. This finding does not reopen machine JSON semantics, WAN/live work, Session/Carrier architecture, or any release flag. `READY_LIVE: none` remains unchanged.

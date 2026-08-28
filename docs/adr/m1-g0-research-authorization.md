# ADR M1-G0: bounded research implementation authorization

- **Date:** 2026-08-29
- **Status:** Accepted administrator authorization; research-only implementation gate
- **Scope:** Local/loopback cryptographic and session implementation, with no production or public exposure

## Decision

The administrator explicitly authorizes the agent to implement the bounded
cryptographic/session work described by the existing G0 candidate documents.
This is an authorization to perform research engineering in this repository;
it is **not** an external security audit, a claim that the design is secure, a
protocol freeze, an interoperability approval, or production authorization.

The authorization applies only to reproducible local, loopback, in-memory,
unit-test, integration-test, fuzz, and isolated lab execution. It permits
selecting and reviewing maintained dependencies, implementing the candidate
trust/authentication/authorization/session/AEAD boundaries, and building the
loopback encrypted UDP slice. Every implementation must remain bounded and
fail-closed, and must preserve the Session-above-Carrier architecture.

## Non-negotiable boundary

The following remain prohibited unless the administrator gives a new explicit
scope decision and the repository records it:

- public, non-loopback listeners or bind-all defaults;
- production deployment, replacement of an existing tunnel, or arbitrary proxy;
- scanning or probing third-party endpoints;
- committing real private keys, production secrets, or sensitive topology;
- enabling 0-RTT or unauthenticated application data;
- claiming security audit, production readiness, protocol freeze, or broad
  interoperability from tests or fuzzing alone.

Research code must expose explicit configuration and tests proving loopback
binding, resource limits, authentication/authorization separation,
transcript/AAD domain separation, directional nonce uniqueness and exhaustion,
replay/epoch/key-phase rejection, uniform external errors, and anti-amplification.

## Gate interpretation

This amendment supersedes only the blanket wording that said no implementation
could begin. The detailed G0 pass criteria remain mandatory implementation and
review criteria. A missing criterion blocks the affected feature and must be
recorded; administrator authorization does not waive it. The Noise IK candidate
and any concrete dependency remain candidates until dependency, license,
maintenance, API, and implementation review evidence is committed.

The status of G0 is therefore **research-authorized / not-security-approved**.
Public exposure and production remain blocked.

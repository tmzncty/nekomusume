# R-009 diagnostic-field erratum

The immutable `result.json` has SHA-256
`5767af02270919d57d6f220a3b026898887ae27da9a13bcd24834a56ff923f7c`.
Its legacy `diagnostics.protocol_entered=true` records only that local capture
was attached after starting the SSH transport process. It does **not** prove
that the structured remote executor accepted its request. The retained
`ssh_transport_exit` 255 classification, absent readiness/client/application
traffic, and verified cleanup remain unchanged.

Future results set `protocol_entered=true` only after the exact structured
`remote_exec_protocol_accepted` marker and separately expose
`capture_started`.

# N5 authoritative VPS package lifecycle evidence

- **Parent:** `91a735c0252e7df5da611da4a70e71a60dbdd44d`
- **Host:** self-owned `192.168.122.1` (`tmzn-server`, x86_64 Linux), Rust 1.98.0
- **Isolation:** detached worktree `/media/tmzn/DATA5/nekomusume-research/worktrees/n5-20260830`; test install root `/opt/nekomusume-n5`; external state `/var/lib/nekomusume-n5`; repository `neko-server.identity` was not touched.

## Artifacts

| Artifact | Version | Archive SHA-256 | Binary SHA-256 |
|---|---:|---|---|
| A | 0.1.0 | `9677bf075c008aa164fa585d3a13feb234ad1dddac4a261ce86c473567a07027` | `d6c9fdd12e3d719b58c260dbbe49d4da685ca5b8fbd3bc4455ac06a83a45847e` |
| B | 0.1.1 | `998a7d11acd80a9f1bacd947966dc5825992a0f93a2fe0b93a3f6b6a55e1aa8e` | `c16a026b0e186175db895a0a991673b05df9b8ca5db0cc6885771d2de9cd4c3b` |

Both archive smokes passed (`SHA256SUMS`, safe paths, modes, capabilities). Each lifecycle smoke passed `capabilities --json`, workload (2×12×32 = 768 application bytes, cleanup verified), and a real loopback TCP authenticated exchange (32 bytes, count 2).

## Lifecycle

1. Install A → smoke passed.
2. Upgrade to B → smoke passed; B had distinct version/hash.
3. Roll back to A → smoke passed; final `current` resolved to A.

The same externally stored server/client identities were used throughout (public keys unchanged in all three client/server logs). The external state marker `external-state-n5-retained` survived the lifecycle; its mode was `0600`. Binary rollback and state retention are separate claims: only the release symlink/binary was switched back; no state migration or rollback was performed.

All explicitly started test processes exited and listener check found no `:40080` listener. Test installation and state directories were removed after evidence capture. Existing unrelated listeners were not changed.

Detailed raw logs and hashes are under `artifacts/n5-20260831/`.

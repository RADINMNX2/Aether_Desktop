# Aether Desktop 1.2.1 engineering handoff

- Desktop version: `1.2.1` in npm, Cargo, Tauri, Windows manifest, installer fallback, README, release notes, and security audit.
- Engine version: `1.6.0` in root `CORE_VERSION`, vendored `native/aether/CORE_VERSION`, and sync baseline.
- Engine source: exact supplied Aether 1.6.0 tree, including Quiche, vendored under `native/aether`.
- Upgrade safety: sync and rollback retain the previous-core snapshot flow; baseline copies for patched probers are seeded from 1.6.0.
- Runtime contract: existing desktop controls cover HTTP/2, ClientHello fragmentation, ECH, quick reconnect, keepalive, Zero Trust, routing, and in-tunnel DNS.
- Validation performed: metadata consistency, JSON/TOML parsing, JavaScript syntax, engine layout, version assertions, and archive integrity. Native Windows binaries must still be produced by the repository's Windows CI matrix.

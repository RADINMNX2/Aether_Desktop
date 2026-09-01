# Aether Desktop 1.2.2 engineering handoff

- Desktop version: `1.2.2` in npm, Cargo, Tauri, Windows manifest, installer fallback, README (EN/FA), release notes, and security audit.
- Engine version: `1.7.0` in root `CORE_VERSION`, vendored `native/aether/CORE_VERSION`, and the `scripts/sync-core.sh` baseline.
- Engine source: exact supplied Aether 1.7.0 tree, including Quiche, vendored under `native/aether`. Only real content changed: 21 modified files, plus the new `aether/src/sniff.rs` and `aether/src/upstream.rs`. Vendored symlinks (`COPYING`, `quiche/quiche/README.md`, fuzz `testsuite`) and gitignored test keys were left untouched.
- Patch state: `native/aether/.upstream-baseline` held no local deltas at 1.6.0 (`prober.rs` and `wg_prober.rs` were byte-identical to upstream), so the upgrade was a clean tree replacement and the baseline was reseeded from 1.7.0. The three-way rebase in `sync-core.sh` therefore starts clean.
- New engine surface wired into the desktop:
  - `--upstream` / `AETHER_UPSTREAM` -> `ConnectionProfile::upstream`, validated by `profile::parse_upstream` (a mirror of the engine's `upstream::Upstream::parse`) and by `parseUpstream()` in `src/views/advanced.js`.
  - `AETHER_ROUTE_SNIFF` -> `ConnectionProfile::route_sniff` (default on, opt-out only).
  - `AETHER_REPROVISION` -> `ConnectionProfile::reprovision` (default on, opt-out only).
  - `AETHER_ROUTE_SNIFF_MS` is deliberately left at the engine default (400 ms); no user-facing timing knob was invented.
- Capability gate: `CoreCaps` gained `upstream` and `route_sniff`, both mapped from core >= 1.7. `to_env()` is now `to_env_with_caps()` so environment variables are gated exactly like flags, and `engine.rs` passes the resolved caps. An older pinned core sees neither the flag nor the variables, and the matching UI blocks are disabled with an explanation.
- Behaviour decisions worth knowing:
  - An HTTP upstream proxy forces `AETHER_MASQUE_HTTP2=1`, because HTTP CONNECT cannot carry UDP. WireGuard and WARP×2 cannot traverse an HTTP proxy at all and the panel says so.
  - `--upstream` is redacted in `DiagnosticsLog` (it can embed `user:pass`).
  - WARP×2 now requires two distinct Cloudflare edges per the 1.7.0 engine; expect an extra scan round on narrow manual ranges.
- Profile compatibility: `ConnectionProfile` keeps `#[serde(default)]`, so existing `profile.json` files load and the new fields adopt their factory values (`route_sniff` and `reprovision` on, `upstream` empty). Default argv and default environment stay byte-identical to 1.2.1, which the tests assert.
- Validation performed: metadata consistency across all version sites, JSON/TOML parsing, JavaScript module parsing (`node --check`), engine layout and `mod` completeness, vendored-symlink integrity, capability-gate and upstream-parser unit tests added in `profile.rs`, and archive integrity. Native Windows binaries and `cargo test` must still be produced by the repository's Windows CI matrix.

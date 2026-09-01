<div align="center">

# Aether Desktop

**Freedom, in one tap**

[فارسی](README.fa.md) · [Releases](../../releases) · [Setup guide](SETUP.md)

Windows desktop tunnel client with mandatory leak protection and a resilient connection path.

</div>

---

## What's new in 1.4.0

**Upgrade notice:** 1.4.0 ports the Aether Mobile 1.2.7 home screen onto the desktop shell — a single glass connection card built on the same design tokens, plus the travelling four-colour light and the ping-strength meter from the app. Aether Core 1.8.0, the glass interface and the modular CI from 1.3.0 are all unchanged.

### Short comparison with 1.2.2

**New connection card.** The loose status / meta / traffic blocks were replaced by one unified glass surface (26 dp radius, the same two-stop card gradient as Compose). The Connect button is now a tinted disc that spins an arc and rotates its icon while connecting, breathes a halo while idle, and settles into a tick when connected.

**GlowCycle.** While connected, the four-colour travelling light (cyan-green → mint → blue → amber) from `GlowCycle.kt` orbits the card on a canvas, following the same pointAt/perimeter motion as Android.

**Ping strength meter.** A 26-bar waveform replaces the plain latency line; its colour and quality label (Excellent / Good / Fair / Poor) match mobile exactly, turning to "Measuring…" while probing and "Not connected" while offline.

**Design tokens.** The CSS now consumes the exact seven-step navy palette and text dims from `Color.kt`, and the animated aurora backdrop was replaced by a flat surface for predictable performance. All new labels are translated in the Persian UI.

No engine, protocol, CLI or security surface was touched; every 1.3.0 capability stays enabled.

---

## What was new in 1.3.0

**Original upgrade notice:** 1.3.0 bundles **Aether Core 1.8.0**, brings a new glass-style interface with smooth motion, and rebuilds the CI pipeline around fast, modular, deterministic jobs. Every 1.2.2 capability and protection stays enabled.

### Short comparison with 1.2.2

**Upgraded the engine to Aether Core 1.8.0.** The stream-head reader that recognises the real host name behind Wintun was reworked and covered with new unit tests, the server-selection logic was tightened, and the SOCKS data plane, Noise handshake helpers, MASQUE/QUIC handling and the upstream/fragment paths received another round of robustness fixes. No new command-line surface was added, so every existing capability gate and UI control keeps its exact behaviour.

**New visual identity.** The interface was redrawn around glassmorphism — an ambient animated aurora behind the app, frosted-glass cards, subtle gradients, spring-like motion and a smooth cross-fade when switching pages. The window honours `prefers-reduced-motion`.

**New CI pipeline.** The build was split into four roles: a cheap `validate` job (repository sanity, JavaScript syntax gate, vendored-core consistency) that fails in seconds before any Windows runner boots; core sync is now opt-in and only runs on a manual version pin, so every main push is reproducible; the two Windows builds (x64/x86) run in parallel with Rust caches that also cover the heavy engine crate; `npm ci` runs against a committed lockfile; and clippy/cargo-audit run once on x64 instead of twice.

### Detailed changes

**Core 1.8.0 integration:** `native/aether` contains the supplied 1.8.0 engine and its Quiche dependency. The desktop build, portable payload, About panel, rollback path, and CI core artifact all resolve the same `CORE_VERSION`, and the sync baseline for the patched probers was reseeded from 1.8.0.

**Domain routing hardening (core):** the code that reads the TLS server name / HTTP `Host` from the first bytes of a connection was reworked in 1.8.0 and now has dedicated tests for a head arriving in one piece, split across writes, and for leaving trailing bytes untouched. This is exactly the code that makes domain rules work behind Wintun, so the *Match domain rules by real host name* switch keeps its documented 1.2.2 behaviour.

**Selection robustness (core):** the endpoint-selection paths gained tighter internal handling and the SOCKS/upstream/fragment paths received minor fixes. There is no behavioural switch for the user; existing protection, kill-switch, watchdog and cleanup behaviour is preserved.

**Visual refresh:** ambient background glow, frosted glass panels, gradient accents and spring-based motion replace the flat theme. Page navigation now cross-fades, and the animation layer is skipped automatically for users who prefer reduced motion.

**Faster, modular CI:** no upstream clone or ~10MB core artifact upload on ordinary pushes; deterministic vendored core; `npm ci` on the committed lockfile with npm cache; Rust cache now keyed over both workspaces; `cargo generate-lockfile` staged before caching and audit; clippy and audit on x64 only; hard timeouts on the engine and Tauri builds.

### Security audit summary

| Area | Result |
|---|---|
| Secrets and keys | No hardcoded credentials; sensitive access values and upstream proxy credentials are not persisted |
| TLS and certificates | Platform validation plus SPKI pin verification |
| DNS, IPv6 and WebRTC | Protected path verified; direct UDP and unsafe IPv6 fallback blocked |
| Local storage and logs | IPs masked; secrets excluded; identity-file protection remains a hardening item |
| Permissions and build | Mandatory UAC; CI checks source, tests, manifest, installer, and cleanup |

Full report: [SECURITY-AUDIT.md](SECURITY-AUDIT.md).

<details>
<summary>Version 1.2.2 — bundled Aether Core 1.7.0</summary>

**Upgrade notice:** Upgrade to 1.2.2 for the bundled Aether Core 1.7.0. Domain routing rules now match the real host name behind the Wintun driver, Aether can dial out through another proxy or VPN on the same PC, and a device identity Cloudflare has stopped accepting is replaced instead of leaving you with a tunnel that handshakes but carries nothing. Every 1.2.0 and 1.2.1 protection stays enabled.

### Short comparison with 1.2.1

**Added:** the complete Aether Core 1.7.0 source and build baseline; an **Upstream proxy** control (`--upstream`) for chaining Aether behind another VPN or proxy already running on the machine; host-name matching for domain routing rules, read from the TLS server name or HTTP `Host` header; automatic replacement of a refused WARP identity; and a 1.7.0 capability gate covering the new flag and the new environment variables.

**Changed:** the desktop release is now 1.2.2; the root `CORE_VERSION`, the vendored `native/aether/CORE_VERSION` and the sync baseline are 1.7.0; an HTTP upstream proxy switches MASQUE to HTTP/2 automatically because HTTP CONNECT cannot carry UDP; the `--upstream` value is masked in the persistent log alongside the Zero Trust secrets; and WARP×2 (`gool`) now builds its two hops on two different Cloudflare edges.

**Preserved:** mandatory WebRTC and IPv6 fail-closed protection, the browser/network kill-switch, the three-target watchdog, exact proxy/PAC restoration, bounded shutdown, in-memory-only Zero Trust secrets, and the rule that a flag or variable is never sent to an engine that does not understand it.

### Detailed changes

**Core 1.7.0 integration:** `native/aether` contains the supplied 1.7.0 engine and its Quiche dependency. The desktop build, portable payload, About panel, rollback path, and CI core artifact all resolve the same `CORE_VERSION`, and the sync baseline for the patched probers was reseeded from 1.7.0.

**Upstream proxy (new):** Advanced → *Upstream proxy* accepts `socks5://host:port`, `socks5://user:pass@host:port`, `http://host:port` or a bare `host:port` (read as SOCKS5). The engine then dials every outbound connection — the endpoint scan, registration calls and the ECH lookup included — through that proxy. The address is validated in the UI and again in Rust, so an unusable value is never handed to the engine: the engine would only log one line and silently continue without a proxy, which is exactly the failure a user cannot see. A SOCKS5 proxy with UDP associate carries MASQUE, WireGuard and WARP×2; an HTTP proxy is TCP-only, so MASQUE is moved to HTTP/2 for you and the panel says so.

**Domain routing rules now work behind Wintun:** on Windows the data plane is always the TUN driver, so the local proxy used to receive a bare IP address and every domain rule quietly missed. Core 1.7.0 reads the name from the first bytes of the connection (TLS SNI or HTTP `Host`) and decides on that, while still connecting to the address the client asked for. The new *Match domain rules by real host name* switch is on by default and only turns the behaviour off.

**A refused identity is replaced, not tolerated:** Cloudflare can stop accepting a saved device. The handshake keeps succeeding in that state and no traffic passes. The engine now detects the refusal and registers a fresh device; the new *Replace a refused identity* switch is on by default and lets you opt out and be told instead.

**WARP×2 uses two distinct edges:** nested WARP now picks two different Cloudflare endpoints for the outer and inner hop and rescans instead of tunnelling an edge through itself. Expect an occasional extra scan round on very restrictive networks, especially with a narrow manual address range.

**Version gate extended:** `CoreCaps` gained 1.7.0 capabilities. On an older pinned core the new flag and variables are never sent and the matching UI sections are disabled with an explanation, exactly like the 1.5.0 features.

**Logging:** `--upstream` joins `--access-secret`, `--access-token` and `--access-email` in the redaction list, so proxy credentials never reach the rotating log file.

### Security audit summary

| Area | Result |
|---|---|
| Secrets and keys | No hardcoded credentials; sensitive access values and upstream proxy credentials are not persisted |
| TLS and certificates | Platform validation plus SPKI pin verification |
| DNS, IPv6 and WebRTC | Protected path verified; direct UDP and unsafe IPv6 fallback blocked |
| Local storage and logs | IPs masked; secrets excluded; identity-file protection remains a hardening item |
| Permissions and build | Mandatory UAC; CI checks source, tests, manifest, installer, and cleanup |

Full report: [SECURITY-AUDIT.md](SECURITY-AUDIT.md).

</details>

<details>
<summary>Version 1.2.1 — bundled Aether Core 1.6.0</summary>

**Upgrade notice:** Upgrade to 1.2.1 for the bundled Aether Core 1.6.0, stronger tunnel validation, and automatic engine-level recovery. All 1.2.0 leak protections remain enabled.

### Short comparison with 1.2.0

**Added:** the complete Aether Core 1.6.0 source and build baseline, end-to-end data-plane validation before a gateway is trusted, automatic MASQUE/WireGuard recovery, last-good-gateway reuse, MASQUE HTTP/2 ClientHello fragmentation, and updated bilingual release documentation.

**Changed:** the desktop release is now 1.2.1, the core baseline and bundled CORE_VERSION are 1.6.0, CI builds the supplied 1.6.0 engine source, and rollback/sync metadata starts from that same verified baseline.

### Detailed changes

**Core 1.6.0 integration:** `native/aether` now contains the supplied 1.6.0 engine and its Quiche dependency. The desktop build, portable payload, About panel, rollback path, and CI core artifact all resolve the same `CORE_VERSION`.

**Connection reliability:** Core 1.6.0 validates real data flow before opening the local proxy, reconnects MASQUE and WireGuard after tunnel loss, and retries the last working gateway before launching a full scan.

**Censorship resistance:** MASQUE can fall back from HTTP/3 to HTTP/2 and optionally fragment TLS ClientHello. Existing desktop controls for HTTP/2, fragmentation, ECH, quick reconnect, keepalive, Zero Trust, routing, and in-tunnel DNS remain wired to the engine.

**Mandatory leak protection:** WebRTC protection is no longer an editable option. Browser policy and elevated firewall enforcement block direct STUN/TURN UDP before a session is reported safe.

**IPv6 protection:** Global IPv6 traffic is routed through the protected path when available and blocked fail-closed otherwise. DNS and tunnel verification run before Connected is shown.

**Kill-switch:** Browser fallback traffic is blocked while the protected session is unavailable. Explicit disconnect removes only Aether's rules and restores the user's original proxy/PAC settings byte-for-byte.

**Connection watchdog:** Every 30 seconds, three independent end-to-end SOCKS5 targets are checked in a background worker. The engine restarts only after three failed rounds in a row.

**UI and shutdown performance:** The shell renders before IPC, initial requests run in parallel, tab listeners are released, verbose engine TLS logging is disabled, and native cleanup is ordered and bounded.

### Security audit summary

| Area | Result |
|---|---|
| Secrets and keys | No hardcoded credentials; sensitive access values are not persisted |
| TLS and certificates | Platform validation plus SPKI pin verification |
| DNS, IPv6 and WebRTC | Protected path verified; direct UDP and unsafe IPv6 fallback blocked |
| Local storage and logs | IPs masked; secrets excluded; identity-file protection remains a hardening item |
| Permissions and build | Mandatory UAC; CI checks source, tests, manifest, installer, and cleanup |

Full report: [SECURITY-AUDIT.md](SECURITY-AUDIT.md).

</details>

<details>
<summary>Version 1.1.0 — parity with engine core 1.5.0</summary>


This release brings the bundled engine to **Aether Core 1.5.0** and adds a full UI for the
three user-facing features that release introduced:

- **Zero Trust (WARP for organizations)** — connect as a managed device of a Cloudflare
  Zero Trust organization. Three sign-in methods: email code, service token, or an existing
  access token. Optional Gateway proxy (off by default, because it logs your browsing).
- **Routing rules** — block destinations outright, or send them out of your real interface
  instead of the tunnel (banking apps, LAN services, domestic sites).
- **In-tunnel DNS** — pick the resolvers used inside the tunnel.

Zero Trust secrets are kept **in memory only**, never written to disk, and masked in logs.
A core version gate makes sure 1.5.0 flags are only passed to an engine that understands them.
See [SECURITY-AUDIT.md](SECURITY-AUDIT.md) for the full 0-100 security audit (score: 93/100).

</details>

<details>
<summary>Version 1.0.0 — first desktop release</summary>

This is the **first** release of Aether for Windows. It is version `1.0.0` because the
desktop edition starts its own version line, independent of the Android app.

Every feature of the Android edition is implemented here, with nothing left out.

### Interface

- Identical design, colour palette and icon to the mobile edition (always dark theme)
- Adapted for large desktop displays: two-column layout with a persistent side rail
- Custom Windows 11-style title bar (minimise / maximise / close) — double-clicking it
  maximises/restores, like any native Windows window
- Full right-to-left support
- Fully bilingual interface — English and فارسی — switchable live from the Advanced tab;
  the choice is stored outside the profile, so "Reset to defaults" never changes your language

### Connection

- Connect button with the same **8 states** as Android:
  Disconnected · Starting engine · Connecting · Verifying · Connected · Reconnecting · Disconnecting · Connection failed
- Protocols: `Smart` (automatic pick — same name as the mobile edition) · `MASQUE` · `WireGuard` · `WARP×2`
- Scan modes: Turbo · Balanced · Thorough · Stealth · Ironclad
- IPv4, IPv6 or dual-stack
- Smart automatic protocol fallback when a protocol fails
- Automatic reconnect (capped at 5 attempts, exactly as on Android)
- Live connection details: protocol, endpoint, latency, uptime
- Traffic panel: live per-second download/upload rates plus session totals
- IP badge with a country flag — "Your IP" while disconnected, the tunnel's "Server IP"
  once connected. Flags are built-in SVGs (75+ countries, neutral globe fallback), because
  Windows has no emoji flag font

### Advanced settings

| Setting | Values |
|---|---|
| Noize | Off · Light · Firewall · Balanced · GFW · Aggressive |
| Endpoint | Auto · Manual peer · Manual range |
| MTU | presets plus a free numeric field (default 1280) |
| Keepalive | presets plus a free numeric field |
| Fragment | on / off |
| ECH | on / off |
| MASQUE HTTP/2 | on / off |
| Quick reconnect | on / off |
| Split tunnelling | Off · Include · Exclude |
| Language | English · فارسی |

Split tunnelling selects applications by executable name instead of Android package name —
the only place where the desktop edition differs, because Windows has no package names.

A **Reset to defaults** button restores every setting above to its factory value —
the UI language is deliberately left untouched.

### Network sharing

- Share the tunnel with other devices on your local network
- SOCKS5 on port `10810`, HTTP on port `10811` (same ports as the Android edition)
- Binds only to the detected LAN address, never to every interface
- Endpoint addresses shown in the app with one-click copy
- Both ports auto-detect the protocol — each one accepts HTTP **and** SOCKS5,
  so either port works in either field

### Diagnostics

- **Run test** — the same live self-test as Android: the checks update in place
  (PENDING → RUNNING → PASS / FAIL) under a colour-coded overall verdict
- **Environment check** — a six-point report, identical in meaning to the Android panel:
  1. Engine binary present
  2. Wintun driver present
  3. Administrator rights
  4. Local SOCKS5 port reachable
  5. Real tunnel egress verified (SOCKS5 handshake plus an outbound request)
  6. Profile sanity
- Live, colour-coded log console (E / W / I / D levels) that refreshes every second
- **Copy logs** (with a confirmation toast) and **Clear** actions

### Windows-specific behaviour

- Real system-wide tunnel using the official, Microsoft-signed **Wintun** driver —
  the desktop equivalent of Android's `VpnService`
- Windows Firewall rule configured automatically during installation, removed on uninstall
- Single-instance launch: reopening focuses the existing window
- Statically linked — no Visual C++ Redistributable required
- WebView2 installed automatically if missing
- On connect, the Windows system proxy (WinINET — what Edge/Chrome/Firefox call the
  "system proxy") is pointed at a local HTTP↔SOCKS5 bridge so system traffic really
  flows through the tunnel; it is reverted on disconnect, on errors and on exit

### About panel

- App version, core engine version and CPU architecture at a glance
- Expandable credits card — links to the upstream Cluvex Studio project (GitHub · Telegram)
  and to the Windows edition, with the upstream feature list and what this build adds
- Links open in your default browser

### Deliberately absent

- **Proxy Mode** — not meaningful on Windows, so it was left out by design.
  The build pipeline actively fails if anyone reintroduces it.
- **In-app updater** — removed. The About panel links to the GitHub Releases page instead.

</details>

---

## Downloads

All files are produced automatically by GitHub Actions and published to
[Releases](../../releases). No manual step is involved at any point.

| File | Description |
|---|---|
| `Aether-Setup-1.4.0-x64.exe` | Windows 64-bit — graphical installer with uninstaller (recommended) |
| `Aether-Setup-1.4.0-x86.exe` | Windows 32-bit — graphical installer with uninstaller |
| `Aether-Portable-1.4.0-x64.zip` | Portable, no installation, 64-bit |
| `Aether-Portable-1.4.0-x86.zip` | Portable, no installation, 32-bit |
| `SHA256SUMS.txt` | Checksums for verifying file integrity |

**Requirements:** Windows 10 build 1809 (October 2018 Update) or newer.
Establishing the tunnel requires Administrator rights, because a network adapter must be
created — the direct equivalent of Android's VPN permission prompt.

The portable edition keeps its settings and logs next to the executable, so it leaves no
trace elsewhere on the machine.

---

## Technology

| Layer | Choice | Why |
|---|---|---|
| Shell | **Tauri 2** | Uses the system WebView2, so the installer stays a few megabytes instead of ~100 MB with Electron |
| Logic | **Rust** | The Aether core engine is already Rust, so the app and the engine share one toolchain |
| Interface | HTML / CSS / vanilla JS, bundled by **Vite** | Reproduces the Compose design exactly, with no framework overhead |
| Tunnel | **Wintun** | The official WireGuard driver for Windows — signed by Microsoft |
| Installer | **Inno Setup 6** | Modern resizable wizard, bilingual, real uninstaller |
| Automation | **GitHub Actions** | Every build, test, package and publish step runs without human involvement |

### Module mapping from Android

| Android (Kotlin) | Windows (Rust) |
|---|---|
| `MainActivity` / `AetherApp` | `main.rs` |
| `AetherController` | `state.rs` |
| `AetherVpnService` | `tun.rs` + `sysproxy.rs` + `leakguard.rs` |
| `AetherProcess` | `engine.rs` |
| `Profile` | `profile.rs` |
| `ProfileStore` | `store.rs` |
| `NetProbe` / `PortProbe` | `probe.rs` |
| `SmartAuto` | `smart_auto.rs` |
| `ShareBridge` | `share.rs` |
| `Diagnostics` | `diagnostics.rs` |
| `DiagnosticsLog` | `log.rs` |

---

## Repository layout

```
.github/workflows/build.yml   the entire automated pipeline
installer/aether.iss          professional installation wizard
scripts/                      build, packaging and verification scripts
src/                          interface (HTML / CSS / JS)
src-tauri/                    application logic (Rust)
native/aether/                the Aether core engine, synced automatically
```

## Building it yourself

You do not need to. Pushing to `main` produces every artifact automatically.
See [SETUP.md](SETUP.md) for the step-by-step repository setup guide.

## Licence

MIT — see [LICENSE](LICENSE).
Bundles the Wintun driver under its own licence.

### Elevation requirement

Aether Desktop 1.4.0 embeds a Windows `requireAdministrator` manifest. Windows therefore
shows the UAC prompt every time the app starts, before any engine, proxy, browser policy or
firewall rule is touched. This is intentional: starting unelevated would make the WebRTC
kill-switch incomplete. The installer is already administrator-only; this requirement now
also covers portable copies and direct launches.


## Important reminder

To get the best result on Android or Windows:

- Wait 1 to 3 minutes on each protocol. Connection time depends on the operator and region.
- Test different protocols and settings because DPI behavior varies by SIM, region, city, and network.
- On mobile data, toggle Airplane mode several times to obtain a different IP range, then retry.
- On Wi-Fi, turn the modem off for 1 to 2 minutes to obtain a different IP range, then retry.
- If it still does not connect, this VPN may not be compatible with that network.
- Different results across users are expected because operator DPI policies differ.

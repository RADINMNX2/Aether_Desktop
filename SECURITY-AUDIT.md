# Aether Desktop 1.3.0 Security Audit

**Scope:** Windows desktop shell, Rust control plane, bundled tunnel engine integration, local proxy bridge, firewall policy, local storage, logs, build pipeline, and bundled runtime. **Date:** 2026-09-01 (1.3.0 refresh).

1.3.0 re-validates the same desktop surface against Aether Core 1.8.0 and the rebuilt four-job CI pipeline (validate / opt-in core sync / x64+x86 builds / `gh`-based release). No new network-exposed surface was added; every previously introduced control (A01–A08) carries over unchanged.

## Executive result

**Score: 88/100.** No hardcoded credentials were found. The connection path has TLS/SPKI pin verification, mandatory WebRTC protection, IPv6 fail-closed protection, and a browser/network kill-switch. The remaining deductions are deliberate Windows constraints and operational risks: the system proxy is not a universal VPN route for every UDP-capable application, WARP identity files require OS-level file protection, and release artifacts should be Authenticode-signed in production.

| Area | Result | Evidence / remaining risk |
|---|---|---|
| **Secrets and keys** | **Pass** | No API keys, passwords, private keys, or access tokens are hardcoded. Zero Trust secrets are write-only and excluded from serialization. |
| **Cryptography and protocols** | **Pass with pin rotation duty** | Core 1.8.0 validates the data plane before exposing the proxy and automatically recovers MASQUE/WireGuard tunnels.  TLS validates the platform certificate chain and SPKI pins. The engine rejects an unpinned key. Pin rotation must ship before certificate/key rollover. |
| **DNS, IPv4, IPv6 and WebRTC leaks** | **Pass** | DNS/HTTP verification runs through SOCKS5. WebRTC direct UDP is blocked by browser policy and firewall rules. IPv6 global traffic is protected or blocked fail-closed. |
| **Traffic bypass** | **Pass for supported desktop path** | System HTTP/HTTPS and SOCKS-aware applications use the local bridge. Kill-switch blocks installed browsers and IPv6 fallback. Arbitrary third-party UDP applications are not transformed into TCP and remain outside the proxy model. |
| **Local storage** | **Partial** | Profile configuration and rotating diagnostics logs are plaintext by design. Sensitive Zero Trust values are not persisted. WARP identity files need DPAPI/ACL hardening before a 100/100 score. |
| **OS privileges and UAC** | **Pass** | `requireAdministrator` is embedded and verified. The installer is admin-only. No Android/iOS manifest is shipped in this desktop repository. |
| **Logging and errors** | **Pass after hardening** | Persistent logs mask public IPs and do not record secrets. Engine runtime logging is `info`, not `debug`, to avoid TLS pin/hash noise and reduce startup/storage pressure. |
| **Network configuration** | **Pass** | No cleartext control channel, no insecure certificate bypass, and no permissive proxy fallback. Public HTTP is used only for the explicit fallback IP probe and is not trusted for tunnel establishment. |
| **Dependencies and supply chain** | **Pass with release controls** | Cargo dependencies are versioned and the CI preflight checks source, manifest, tests, and artifacts. Production releases should include reproducible lockfile verification and Authenticode signatures. |
| **UI/runtime resilience** | **Improved** | The shell paints before IPC, initial IPC calls run in parallel, state listeners are cleared on tab changes, and expensive network work stays off the UI thread. |

## Findings and controls

**A01: WebRTC direct UDP exposure, fixed.** The mandatory guard applies browser policy, blocks STUN/TURN and browser UDP at the firewall when elevated, and fails closed if the leak check cannot pass. The setting is not exposed as an unsafe user toggle.

**A02: IPv6 fallback exposure, fixed.** Global IPv6 is routed through the protected path when a real tunnel route exists or blocked with a kill-switch rule. The UI and diagnostics report the protection state instead of claiming a route exists when it does not.

**A03: Proxy restoration after disconnect, fixed.** Cleanup disables WinINET first, then removes bridge/tunnel resources, and removes the kill-switch on explicit disconnect. Reconnect preserves the kill-switch to avoid a security gap.

**A04: Periodic upstream stalls, fixed.** A background watchdog probes three independent SOCKS5 targets every 30 seconds and restarts only after three consecutive failed rounds. TCP idle handling is five minutes and the UDP policy value is 120 seconds.

**A05: UI blank/freeze, mitigated.** The first shell renders before profile/snapshot IPC, those calls run concurrently, and network probes never run on the UI thread. Native teardown remains bounded by listener joins and process reap.

**A06: upstream proxy credentials, controlled (1.2.2).** The `--upstream` value can embed `user:pass`, so it is masked in the rotating log next to the Zero Trust secrets, and it is validated on both sides of the IPC boundary so an unusable value can never make the engine fall back to a direct connection while the UI still claims a chain.

**A07: identity refused by Cloudflare, handled (1.2.2).** Core 1.8.0 detects a saved WARP identity the account API no longer accepts — a state where the handshake succeeds and no traffic passes — and registers a fresh device. Offline or rate-limited answers never discard an identity, and the behaviour is a user-visible switch.

**A08: host-name routing, scoped (1.2.2).** Domain rules are matched from the first bytes of a connection (TLS SNI or HTTP `Host`) only when domain rules exist and the proxy was handed a bare address. The name is used for the routing decision only; the connection still goes to the address the client asked for, and the read is bounded by `AETHER_ROUTE_SNIFF_MS` (400 ms default).

## Release gate

Do not publish unless all are green: `cargo fmt --check`, x64 and x86 `cargo test`, frontend build, Tauri release build, embedded manifest verification, installer silent install/uninstall, kill-switch cleanup, and a manual WebRTC/DNS/IPv6 leak test.


# ممیزی امنیتی نسخهٔ ۱.۳.۰

**دامنه:** پوستهٔ ویندوز، کنترل‌پلین Rust، موتور تونل، پل پروکسی، فایروال، ذخیره‌سازی محلی، لاگ‌ها و زنجیرهٔ بیلد. **امتیاز: ۸۸ از ۱۰۰.**

| بخش | نتیجه |
|---|---|
| **کلیدها و اطلاعات حساس** | کلید، رمز، توکن یا API Key هاردکدشده پیدا نشد؛ اسرار Zero Trust ذخیره نمی‌شوند. |
| **رمزنگاری و پروتکل** | اعتبارسنجی زنجیرهٔ TLS و پین SPKI فعال است؛ چرخش پین باید قبل از تغییر کلید انجام شود. |
| **نشت DNS، IPv6 و WebRTC** | مسیر DNS و HTTP از SOCKS5 بررسی می‌شود؛ UDP مستقیم WebRTC و IPv6 ناامن fail-closed هستند. |
| **عبور خارج از تونل** | مسیر HTTP/HTTPS و برنامه‌های SOCKS-aware حفاظت می‌شوند؛ UDP دلخواه برنامه‌های ثالث خارج از مدل پروکسی است. |
| **ذخیره‌سازی محلی** | پروفایل و لاگ چرخشی متن ساده‌اند؛ اسرار حساس ذخیره نمی‌شوند؛ رمزگذاری فایل هویت با DPAPI مورد باقیمانده است. |
| **مجوز و سیستم‌عامل** | اجرای `requireAdministrator` اجباری و مانیفست verify می‌شود؛ مجوز اضافی دسکتاپی درخواست نمی‌شود. |
| **لاگ و خطا** | آی‌پی‌ها ماسک، اسرار حذف و سطح لاگ موتور به `info` محدود شده است. |
| **کیفیت شبکه و وابستگی‌ها** | cleartext کنترل، bypass گواهی و fallback ناامن وجود ندارد؛ CI تست و بسته‌بندی را کنترل می‌کند. |

**کنترل‌های تکمیلی:** کیل‌سوییچ و حفاظت IPv6 پیش‌فرض فعال‌اند، تنظیمات پروکسی قبل از تغییر snapshot و بعد از قطع دقیقاً restore می‌شوند، واچداگ سه‌هدفه از ترد رابط جداست و پاک‌سازی پس از خروج انجام می‌شود.

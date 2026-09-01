//! پورت از `core/AetherController.kt` + `vpn/AetherVpnService.kt` + `model/ConnectionState.kt`.
//!
//! ریشهٔ باگ «هیچ پروتکلی کانکت نمی‌شود» در نسخهٔ قبلی دسکتاپ:
//! `connect()` بلافاصله بعد از اجرای موتور، `Tunnel::establish` را صدا می‌زد؛
//! ساخت آداپتور Wintun بدون دسترسی Administrator شکست می‌خورد، خطا از
//! `connect()` بیرون می‌رفت و ماشین حالت برای همیشه روی StartingEngine گیر
//! می‌کرد — در حالی که موتور واقعاً وصل می‌شد (لاگ کاربر: «socks5 server
//! listening on 127.0.0.1:1819» بدون هیچ «I/state: Connecting» بعد از آن).
//!
//! حالا دقیقاً ترتیب اندروید (`connectAttempt`) اجرا می‌شود:
//!   ۱. StartingEngine → آزادشدن پورت → اجرای موتور → Connecting
//!   ۲. انتظار برای بازشدن پورت SOCKS5 (ground truth — همان PortProbe)
//!   ۳. فقط بعد از آن، مسیر داده برپا می‌شود (معادل VpnService.establish):
//!      پل HTTP/SOCKS محلی + پروکسی سیستمی ویندوز؛ Wintun هم اگر ممکن بود
//!      (شکست Wintun دیگر کل اتصال را نمی‌کُشد — فقط یک هشدار لاگ می‌شود).
//!   ۴. Verifying: خودآزمای ۴ مرحله‌ای (Diagnostics.kt) در ترد پس‌زمینه
//!   ۵. فقط بعد از قبولی همهٔ بررسی‌ها، Connected اعلام می‌شود
//!   ۶. شکست هر پله ← پلهٔ بعدی نردبان (معادل runLadder)، نه گیرکردن ابدی.

use crate::diagnostics;
use crate::engine::{self, AetherProcess};
use crate::leakguard::{self, LeakGuard};
use crate::log::DiagnosticsLog;
use crate::probe;
use crate::profile::{ConnectionProfile, Protocol};
use crate::share::ShareBridge;
use crate::smart_auto::{self, Candidate};
use crate::store::ProfileStore;
use crate::sysproxy;
use crate::tun::Tunnel;
use anyhow::Result;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

const TAG: &str = "state";

/// همان مقادیر اندروید: MAX_RETRIES=3، BACKOFF = 2s/5s/10s.
const DEFAULT_MAX_RETRIES: u32 = 3;
const BACKOFF_MS: [u64; 3] = [2_000, 5_000, 10_000];
/// پنجرهٔ گریس خودآزما — همان `OUTBOUND_GRACE_MS` (شروع سرد warp-in-warp).
const OUTBOUND_GRACE_MS: u64 = 90_000;
/// معادل `PORT_RELEASE_WAIT_MS` اندروید.
const PORT_RELEASE_WAIT_MS: u64 = 3_000;
const WATCHDOG_INTERVAL_SECS: u64 = 30;
const WATCHDOG_FAILURE_THRESHOLD: u8 = 3;

/// معادل دقیق `ConnectionState.kt` — همان هشت حالت، همان ترتیب.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConnectionState {
    Disconnected,
    StartingEngine,
    Connecting,
    Verifying,
    Connected,
    Reconnecting,
    Disconnecting,
    Failed,
}

impl ConnectionState {
    pub fn is_busy(self) -> bool {
        matches!(
            self,
            Self::StartingEngine | Self::Connecting | Self::Verifying | Self::Reconnecting | Self::Disconnecting
        )
    }
    pub fn is_active(self) -> bool {
        self.is_busy() || self == Self::Connected
    }
}

/// معادل `IpInfo` در UI اندروید — خوراک نشان «IP + پرچم».
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpEndpoint {
    pub ip: String,
    pub country_code: Option<String>,
    /// true = IP خروجی سرور (از دل تونل)، false = IP واقعی کاربر.
    pub via_tunnel: bool,
}

/// حالت مشترک جست‌وجوی IP — معادل `ipInfo`/`ipLoading` در MainActivity.
struct IpSlot {
    info: Option<IpEndpoint>,
    loading: bool,
    /// شمارندهٔ نسل — نتیجهٔ جست‌وجوهای قدیمی دور ریخته می‌شود.
    session: u64,
}

/// معادل مجموع StateFlow‌هایی که HomeScreen.kt جمع می‌کرد.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub state: ConnectionState,
    pub detail: String,
    pub error: Option<String>,
    pub endpoint: Option<String>,
    pub protocol: Option<String>,
    pub latency_ms: Option<u64>,
    pub uptime_secs: u64,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub share_socks: Option<String>,
    pub share_http: Option<String>,
    pub ip_info: Option<IpEndpoint>,
    pub ip_loading: bool,
    /// v1.2.0 — نتیجهٔ آخرین سنجش نشتی WebRTC. `None` = هنوز سنجیده نشده.
    pub webrtc_leak: Option<bool>,
    /// v1.2.0 — گارد نشتی همین حالا فعال است؟
    pub leak_guard: bool,
}

pub struct AetherController {
    data_dir: PathBuf,
    store: ProfileStore,
    profile: ConnectionProfile,
    state: ConnectionState,
    detail: String,
    error: Option<String>,
    endpoint: Option<String>,
    effective_protocol: Option<Protocol>,
    latency_ms: Option<u64>,
    connected_at: Option<Instant>,
    engine: AetherProcess,
    tunnel: Option<Tunnel>,
    share: ShareBridge,
    sysproxy_on: bool,
    /// v1.2.0 — گارد نشتی WebRTC/UDP این نشست (Drop خودش آزادش می‌کند).
    guard: Option<LeakGuard>,
    /// v1.2.0 — آخرین نتیجهٔ سنجش نشتی، برای نشانِ صفحهٔ اصلی.
    webrtc_leak: Option<bool>,
    /// نردبان تلاش‌ها — معادل `runLadder` در AetherVpnService.kt.
    plan: Vec<Candidate>,
    plan_index: usize,
    /// تلاش‌های اتصال مجدد پشت‌سرهم — معادل `reconnectAttempts`.
    attempts: u32,
    deadline: Option<Instant>,
    reconnect_at: Option<Instant>,
    /// نتیجهٔ خودآزمای در حال اجرا (ترد پس‌زمینه — UI فریز نمی‌شود).
    verify_slot: Option<Arc<Mutex<Option<diagnostics::SelfTestOutcome>>>>,
    ip_slot: Arc<Mutex<IpSlot>>,
    /// پینگ زنده: نتیجهٔ آخرین اندازه‌گیری دوره‌ای در ترد پس‌زمینه.
    latency_slot: Arc<Mutex<Option<u64>>>,
    /// زمان اندازه‌گیری بعدی پینگ.
    latency_probe_at: Option<Instant>,
    /// نتیجهٔ آخرین پروب واچداگ، خارج از حلقهٔ اصلی محاسبه می‌شود.
    watchdog_slot: Arc<Mutex<Option<bool>>>,
    watchdog_probe_at: Option<Instant>,
    watchdog_failures: u8,
    /// Firewall/registry work is deferred out of the IPC command path.
    security_refresh_pending: bool,
}

impl AetherController {
    pub fn new(data_dir: &Path) -> Self {
        let store = ProfileStore::new(data_dir);
        let mut profile = store.load();
        profile.normalize();
        let install_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(Path::to_path_buf))
            .unwrap_or_else(|| data_dir.to_path_buf());

        let ip_slot = Arc::new(Mutex::new(IpSlot { info: None, loading: false, session: 0 }));

        let mut me = Self {
            data_dir: data_dir.to_path_buf(),
            store,
            profile,
            state: ConnectionState::Disconnected,
            detail: String::new(),
            error: None,
            endpoint: None,
            effective_protocol: None,
            latency_ms: None,
            connected_at: None,
            engine: AetherProcess::new(&install_dir, data_dir),
            tunnel: None,
            share: ShareBridge::new(),
            sysproxy_on: false,
            guard: None,
            webrtc_leak: None,
            plan: Vec::new(),
            plan_index: 0,
            attempts: 0,
            deadline: None,
            reconnect_at: None,
            verify_slot: None,
            ip_slot,
            latency_slot: Arc::new(Mutex::new(None)),
            latency_probe_at: None,
            watchdog_slot: Arc::new(Mutex::new(None)),
            watchdog_probe_at: None,
            watchdog_failures: 0,
            security_refresh_pending: false,
        };

        // پروکسی سیستمی به‌جامانده از کرش احتمالی جلسهٔ قبل را پاک می‌کنیم.
        sysproxy::recover_stale();
        // …و همین‌طور قواعد فایروال / سیاست مرورگری که گارد نشتی جا گذاشته.
        // هیچ اثری از جلسهٔ قبلی نباید کورکورانه باقی بماند.
        leakguard::purge_stale();
        // Do not install network-blocking rules during ordinary app startup.
        // The old behavior blocked Windows before a tunnel/bridge existed,
        // which is why reopening the app could kill internet access. The
        // guard is installed only after the SOCKS bridge is ready.
        // معادل LaunchedEffect فاز idle در MainActivity: نمایش IP واقعی کاربر از لحظهٔ اجرا.
        spawn_ip_lookup(me.ip_slot.clone(), false);
        me
    }

    pub fn profile(&self) -> ConnectionProfile {
        self.profile.clone()
    }

    pub fn set_profile(&mut self, profile: ConnectionProfile) -> Result<()> {
        let mut profile = profile;
        profile.normalize();
        // v10: فیلدهای محرمانه «write-only» هستند: get_profile هرگز آن‌ها را
        // برنمی‌گرداند، پس UI معمولاً رشتهٔ خالی می‌فرستد. خالی = «دست نزن»
        // تا رازِ در-حافظهٔ این نشست با هر تغییر تنظیم دیگر پاک نشود.
        if profile.access_secret.is_empty() {
            profile.access_secret = self.profile.access_secret.clone();
        }
        if profile.access_token.is_empty() {
            profile.access_token = self.profile.access_token.clone();
        }
        self.apply_profile(profile)
    }

    /// v10: «بازنشانی به تنظیمات پیش‌فرض» باید اسرارِ در-حافظه را هم واقعاً
    /// پاک کند. `set_profile` رشتهٔ خالی را «دست نزن» تفسیر می‌کند (چون UI
    /// اسرار را پس نمی‌گیرد)، پس Reset مسیر جداگانهٔ خودش را دارد؛ وگرنه
    /// توکن سازمانی پس از Reset بی‌صدا در حافظه زنده می‌ماند.
    pub fn reset_profile(&mut self) -> Result<ConnectionProfile> {
        let fresh = ConnectionProfile::default();
        self.apply_profile(fresh.clone())?;
        DiagnosticsLog::i(TAG, "Profile reset to factory defaults (in-memory Zero Trust secrets cleared).");
        Ok(fresh)
    }

    /// مسیر مشترک ذخیره‌سازی — هرچه از set_profile/reset_profile بیاید.
    fn apply_profile(&mut self, profile: ConnectionProfile) -> Result<()> {
        self.store.save(&profile)?;
        let lan_toggled = profile.lan_share != self.profile.lan_share;
        let guard_toggled = profile.leak_guard != self.profile.leak_guard;
        let kill_toggled = profile.kill_switch != self.profile.kill_switch;
        let ipv6_toggled = profile.ipv6_protection != self.profile.ipv6_protection;
        self.profile = profile;
        // v1.2.0: خاموش/روشن‌کردن گارد نشتی وسط یک اتصالِ فعال باید فوراً
        // اثر کند — نه در اتصال بعدی. کاربری که سوییچ را می‌زند انتظار دارد
        // همان لحظه محافظت شود (یا آزاد شود).
        let safety_changed = guard_toggled || kill_toggled || ipv6_toggled;
        if safety_changed && self.state.is_active() {
            // Do not run reg.exe/netsh.exe while the UI IPC command is waiting.
            // The 200ms controller tick applies it outside the settings click,
            // preventing the white titlebar/freeze seen on safety toggles.
            self.security_refresh_pending = true;
            self.webrtc_leak = None;
        }
        // Root fix for "Share over LAN shows no IP:port": flipping the switch
        // while a connection is active must rebind the bridge immediately
        // (mobile restarts its ShareBridge the same way), so the UI gets the
        // fresh endpoints in the very next snapshot instead of never.
        if lan_toggled && self.state.is_active() {
            if let Err(e) = self.share.start(
                engine::SHARE_SOCKS_PORT,
                engine::SHARE_HTTP_PORT,
                self.profile.lan_share,
            ) {
                DiagnosticsLog::e(TAG, &format!("Bridge restart after LAN toggle failed: {e}"));
            }
        }
        Ok(())
    }

    pub fn snapshot(&self) -> Snapshot {
        let (tun_rx, tun_tx) = self.tunnel.as_ref().map(Tunnel::counters).unwrap_or((0, 0));
        let (br_rx, br_tx) = self.share.traffic();
        let (ip_info, ip_loading) = {
            let g = self.ip_slot.lock();
            (g.info.clone(), g.loading)
        };
        Snapshot {
            state: self.state,
            detail: self.detail.clone(),
            error: self.error.clone(),
            endpoint: self.endpoint.clone(),
            protocol: self.effective_protocol.map(|p| format!("{p:?}").to_uppercase()),
            latency_ms: self.latency_ms,
            uptime_secs: self.connected_at.map(|t| t.elapsed().as_secs()).unwrap_or(0),
            rx_bytes: tun_rx + br_rx,
            tx_bytes: tun_tx + br_tx,
            share_socks: self.share.socks_endpoint(),
            share_http: self.share.http_endpoint(),
            ip_info,
            ip_loading,
            webrtc_leak: self.webrtc_leak,
            leak_guard: leakguard::status().engaged,
        }
    }

    /// معادل `onToggleConnection` — خطای اتصال دیگر به بیرون پرتاب نمی‌شود؛
    /// همیشه به حالت Failed ترجمه می‌شود تا UI هرگز در StartingEngine گیر نکند.
    pub fn toggle(&mut self) -> Result<()> {
        if self.state.is_active() {
            self.disconnect();
        } else if let Err(e) = self.connect() {
            let msg = e.to_string();
            self.fail(&msg);
        }
        Ok(())
    }

    /// معادل `connect()` سرویس اندروید — فقط برنامه‌ریزی و اجرای پلهٔ اول؛
    /// بقیهٔ مراحل در tick() دنبال می‌شوند.
    fn connect(&mut self) -> Result<()> {
        self.error = None;
        self.attempts = 0;
        // معادل DiagnosticsLog.clear + resetChecks در شروع اتصال اندروید.
        diagnostics::reset_checks();
        self.set_state(ConnectionState::StartingEngine, "Starting engine…");
        DiagnosticsLog::i(
            TAG,
            &format!(
                "Connect requested — protocol={:?} scan={:?} ip={:?}",
                self.profile.protocol, self.profile.scan_mode, self.profile.ip_version
            ),
        );
        // SmartAuto.kt parity: fingerprint the network before planning. On a
        // filtered network the ladder leads with the hardened anti-DPI
        // candidate, so the plain first pass can no longer waste 35-75s
        // (slow connects) or win with a tunnel that cannot carry real
        // browser traffic afterwards.
        let hostile = probe::network_looks_filtered();
        self.plan = smart_auto::build_plan(&self.profile, hostile);
        debug_assert!(!self.plan.is_empty(), "build_plan returned an empty ladder");
        self.plan_index = 0;
        self.start_candidate()
    }

    /// اجرای یک پله از نردبان — معادل یک دور `runLadder`.
    fn start_candidate(&mut self) -> Result<()> {
        // قرارداد: برنامه‌ریز (build_plan) همیشه پلن را پر و plan_index را
        // در مرز معتبر نگه می‌دارد؛ این assert فقط رگرسیون منطق را می‌گیرد.
        debug_assert!(!self.plan.is_empty(), "plan must not be empty");
        debug_assert!(self.plan_index < self.plan.len(), "plan_index out of bounds");
        let cand = self.plan[self.plan_index].clone();
        DiagnosticsLog::i(
            TAG,
            &format!("Attempt {}/{} → {}", self.plan_index + 1, self.plan.len(), cand.label),
        );

        // معادل PortProbe.awaitClosed — ریشهٔ باگ «تعویض پروتکل گیر می‌کند».
        if !engine::wait_for_port_release(
            engine::LOCAL_SOCKS_PORT,
            Duration::from_millis(PORT_RELEASE_WAIT_MS),
        ) {
            DiagnosticsLog::w(
                TAG,
                &format!(
                    "Local port {} is still busy after {}s — starting anyway.",
                    engine::LOCAL_SOCKS_PORT,
                    PORT_RELEASE_WAIT_MS / 1000
                ),
            );
        }

        self.effective_protocol = Some(cand.profile.protocol);
        self.engine.start(&cand.profile)?;
        self.deadline = Some(Instant::now() + Duration::from_millis(cand.timeout_ms));
        self.set_state(ConnectionState::Connecting, "Connecting…");
        DiagnosticsLog::i(
            TAG,
            &format!(
                "Waiting for SOCKS5 on 127.0.0.1:{}… (timeout={}s)",
                engine::LOCAL_SOCKS_PORT,
                cand.timeout_ms / 1000
            ),
        );
        Ok(())
    }

    /// معادل بخش establish در connectAttempt — فقط بعد از بازشدن پورت SOCKS5.
    fn bring_up_data_path(&mut self) {
        let profile = self
            .plan
            .get(self.plan_index)
            .map(|c| c.profile.clone())
            .unwrap_or_else(|| self.profile.clone());

        // ۰) گارد نشتی — *قبل* از هر چیز دیگری. ترتیب امنیتی است، نه سلیقه‌ای:
        // تا وقتی مسیر UDP مستقیم باز است نباید مرورگر را به تونل وصل کنیم،
        // وگرنه بین «پروکسی روشن شد» و «گارد نصب شد» یک پنجرهٔ نشتی می‌ماند.
        if self.profile.leak_guard || self.profile.kill_switch || self.profile.ipv6_protection {
            if let Some(mut old_guard) = self.guard.take() {
                old_guard.disarm_without_cleanup();
            }
            self.guard = Some(LeakGuard::engage(&profile));
        } else {
            DiagnosticsLog::w(
                TAG,
                "Leak guard is disabled in the profile — WebRTC may expose your real IP over direct UDP.",
            );
        }

        // ۱) پل محلی HTTP/SOCKS — معادل hev-socks5-tunnel/ShareBridge (مسیر دادهٔ واقعی).
        if let Err(e) = self.share.start(engine::SHARE_SOCKS_PORT, engine::SHARE_HTTP_PORT, profile.lan_share) {
            DiagnosticsLog::e(TAG, &format!("Bridge failed to start: {e}"));
        }

        // ۲) پروکسی سیستمی ویندوز — معادل کارکرد VpnService (کل سیستم از تونل می‌رود).
        self.sysproxy_on = sysproxy::enable(engine::SHARE_HTTP_PORT, engine::SHARE_SOCKS_PORT);

        // ۳) Wintun — اختیاری. شکست آن دیگر اتصال را نمی‌کُشد (رفع ریشه‌ای گیر StartingEngine).
        if self.tunnel.is_none() {
            let wintun = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.join("engine").join("wintun.dll")))
                .unwrap_or_default();
            match Tunnel::establish(&profile, &wintun) {
                Ok(t) => self.tunnel = Some(t),
                Err(e) => DiagnosticsLog::w(
                    "tun",
                    &format!("Wintun unavailable ({e}); continuing with the system-proxy data path."),
                ),
            }
        }
    }

    /// خودآزمای ۴ مرحله‌ای در ترد پس‌زمینه — حلقهٔ tick هرگز مسدود نمی‌شود.
    fn begin_verification(&mut self) {
        let slot: Arc<Mutex<Option<diagnostics::SelfTestOutcome>>> = Arc::new(Mutex::new(None));
        self.verify_slot = Some(slot.clone());
        let remaining = self
            .deadline
            .map(|d| d.saturating_duration_since(Instant::now()).as_millis() as u64)
            .unwrap_or(OUTBOUND_GRACE_MS);
        let grace = remaining.clamp(20_000, OUTBOUND_GRACE_MS);
        std::thread::Builder::new()
            .name("aether-selftest".into())
            .spawn(move || {
                let outcome = diagnostics::self_test(grace);
                *slot.lock() = Some(outcome);
            })
            .ok();
        self.set_state(ConnectionState::Verifying, "Verifying…");
    }

    fn disconnect(&mut self) {
        self.set_state(ConnectionState::Disconnecting, "Disconnecting…");
        self.cleanup_native(false);
        // v16: تیک‌های سبز Diagnostics باید بلافاصله بعد از دیسکانکت
        // ریست شوند تا برای اتصال بعدی آماده باشند (معادل resetChecks اندروید).
        diagnostics::reset_checks();
        self.latency_probe_at = None;
        self.watchdog_probe_at = None;
        self.watchdog_failures = 0;
        *self.watchdog_slot.lock() = None;
        *self.latency_slot.lock() = None;
        self.connected_at = None;
        self.endpoint = None;
        self.latency_ms = None;
        self.effective_protocol = None;
        self.deadline = None;
        self.reconnect_at = None;
        self.verify_slot = None;
        self.plan.clear();
        self.plan_index = 0;
        self.set_state(ConnectionState::Disconnected, "");
    }

    /// ترتیب ۱.۲.۲: اول پروکسی سیستمی (تا مرورگر به پل مُرده نچسبد)، بعد
    /// اشتراک، بعد تونل، بعد موتور — بدون فریز.
    fn cleanup_native(&mut self, preserve_kill_switch: bool) {
        if self.sysproxy_on {
            sysproxy::disable();
            self.sysproxy_on = false;
        }
        // گارد بعد از پروکسی آزاد می‌شود: تا آخرین لحظه‌ای که مرورگر ممکن است
        // به پل وصل باشد، مسیر UDP هم بسته می‌ماند.
        if preserve_kill_switch {
            if let Some(g) = self.guard.as_mut() {
                g.release_for_reconnect();
            }
        } else if let Some(mut g) = self.guard.take() {
            g.release();
        }
        self.webrtc_leak = None;
        self.share.stop();
        if let Some(mut t) = self.tunnel.take() {
            t.close();
        }
        self.engine.stop();
    }

    /// شکست یک پله → پلهٔ بعدی نردبان؛ تمام‌شدن نردبان → Failed با پیام روشن.
    fn advance_or_fail(&mut self, why: &str) {
        DiagnosticsLog::w(TAG, &format!("{why} — tearing down this attempt."));
        // فقط موتور/مسیر داده را جمع می‌کنیم، وضعیت UI همچنان busy می‌ماند.
        self.cleanup_native(true);
        self.verify_slot = None;
        diagnostics::reset_checks();
        self.plan_index += 1;
        if self.plan_index < self.plan.len() {
            if let Err(e) = self.start_candidate() {
                let msg = e.to_string();
                self.fail(&msg);
            }
        } else if self.profile.protocol == Protocol::Smart {
            self.fail("Smart Auto tried every strategy and none passed the self-test on this network.");
        } else {
            self.fail(
                "This protocol could not establish a working tunnel on this network, even with anti-DPI hardening. Try Smart Auto or another protocol.",
            );
        }
    }

    fn apply_pending_security_refresh(&mut self) {
        if !self.security_refresh_pending { return; }
        self.security_refresh_pending = false;
        if self.profile.leak_guard || self.profile.kill_switch || self.profile.ipv6_protection {
            if let Some(mut old_guard) = self.guard.take() { old_guard.disarm_without_cleanup(); }
            self.guard = Some(LeakGuard::engage(&self.profile));
        } else if let Some(mut guard) = self.guard.take() {
            guard.release();
        }
    }

    /// هر ۲۰۰ms از main.rs صدا زده می‌شود — معادل حلقهٔ نظارت اندروید.
    pub fn tick(&mut self) {
        self.apply_pending_security_refresh();

        match self.state {
            ConnectionState::Connecting => {
                if !self.engine.is_alive() {
                    self.advance_or_fail("Engine exited before it opened the SOCKS5 port");
                    return;
                }
                if probe::socks_ready(engine::LOCAL_SOCKS_PORT) {
                    DiagnosticsLog::i(TAG, "SOCKS5 port is up — bringing up the data path.");
                    self.bring_up_data_path();
                    self.begin_verification();
                } else if self.past_deadline() {
                    self.advance_or_fail("Engine still scanning — the SOCKS5 port never opened in time");
                }
            }
            ConnectionState::Verifying => {
                let outcome = self.verify_slot.as_ref().and_then(|s| s.lock().take());
                if let Some(out) = outcome {
                    self.verify_slot = None;
                    if out.ok {
                        if let Some(exit) = &out.exit {
                            self.endpoint = Some(match &exit.country_code {
                                Some(cc) => format!("{} · {cc}", exit.ip),
                                None => exit.ip.clone(),
                            });
                            // IP خروجی از خودآزما مستقیماً به نشان IP می‌رود —
                            // معادل offerTunnelIpInfo در Diagnostics.kt.
                            let mut g = self.ip_slot.lock();
                            g.session += 1;
                            g.info = Some(IpEndpoint {
                                ip: exit.ip.clone(),
                                country_code: exit.country_code.clone(),
                                via_tunnel: true,
                            });
                            g.loading = false;
                        }
                        // v1.2.0: نتیجهٔ سنجش نشتی مستقیم به نشانِ صفحهٔ اصلی می‌رود.
                        self.webrtc_leak = out.leak.as_ref().map(|l| l.leaking);
                        if self.webrtc_leak == Some(true) {
                            DiagnosticsLog::w(
                                TAG,
                                "Tunnel is up but WebRTC still reached a STUN server directly. Restart the browser so the WebRTC policy applies, or run Aether as administrator for the firewall layer.",
                            );
                        }
                        self.latency_ms = out.latency_ms;
                        self.watchdog_probe_at = Some(Instant::now() + Duration::from_secs(WATCHDOG_INTERVAL_SECS));
                        self.watchdog_failures = 0;
                        self.connected_at = Some(Instant::now());
                        self.attempts = 0;
                        self.set_state(ConnectionState::Connected, "");
                        DiagnosticsLog::i(TAG, "All checks passed — tunnel is ready.");
                        if out.exit.is_none() {
                            spawn_ip_lookup(self.ip_slot.clone(), true);
                        }
                    } else if out.leak.as_ref().map(|l| l.leaking).unwrap_or(false) {
                        // Fail closed. A tunnel that exposes the real IP is not
                        // a successful connection, even when TCP/DNS passed.
                        self.fail(
                            "Connection refused: WebRTC can still reach the real IP over direct UDP. Browser and system protection could not be verified.",
                        );
                    } else {
                        self.advance_or_fail("Tunnel started, but the end-to-end self-test failed");
                    }
                } else if !self.engine.is_alive() {
                    self.advance_or_fail("The engine stopped during verification");
                }
            }
            ConnectionState::Connected => {
                // v1.2.0 watchdog: every 30s run three end-to-end probes in a
                // worker thread. Three consecutive failed rounds are required
                // before restarting, so short network jitter is tolerated.
                let watchdog_result = { self.watchdog_slot.lock().take() };
                if let Some(result) = watchdog_result {
                    if result {
                        self.watchdog_failures = 0;
                        DiagnosticsLog::i(TAG, "Watchdog probe passed (at least 2 of 3 targets reachable through SOCKS5).");
                    } else {
                        self.watchdog_failures = self.watchdog_failures.saturating_add(1);
                        DiagnosticsLog::w(TAG, &format!("Watchdog probe failed ({}/{})", self.watchdog_failures, WATCHDOG_FAILURE_THRESHOLD));
                        if self.watchdog_failures >= WATCHDOG_FAILURE_THRESHOLD {
                            DiagnosticsLog::e(TAG, "Watchdog confirmed a persistent upstream failure — restarting the engine.");
                            self.cleanup_native(true);
                            self.watchdog_failures = 0;
                            self.connected_at = None;
                            self.reconnect_at = Some(Instant::now() + Duration::from_secs(2));
                            self.set_state(ConnectionState::Reconnecting, "Watchdog reconnect…");
                            return;
                        }
                    }
                }
                let watchdog_due = self.watchdog_probe_at
                    .map(|t| Instant::now() >= t)
                    .unwrap_or(true);
                let watchdog_busy = { self.watchdog_slot.lock().is_some() };
                if watchdog_due && !watchdog_busy {
                    self.watchdog_probe_at = Some(Instant::now() + Duration::from_secs(WATCHDOG_INTERVAL_SECS));
                    let slot = self.watchdog_slot.clone();
                    std::thread::Builder::new()
                        .name("aether-watchdog".into())
                        .spawn(move || {
                            let ok = probe::watchdog_probe();
                            *slot.lock() = Some(ok);
                        })
                        .ok();
                }

                // v16: پینگ نمایشی قبلاً فقط یک‌بار هنگام خودآزمای اتصال اندازه
                // گرفته می‌شد (شامل زمان دریافت HTTP در شلوغی لحظهٔ اتصال)
                // و دیگر به‌روز نمی‌شد — برای همین عددی مثل ۸۰۰۰ms می‌ماند.
                // حالا هر ۱۵ ثانیه یک اتصال TCP سبک از داخل تونل زمان‌گیری
                // می‌شود تا پینگ واقعی و زنده نمایش داده شود (بدون فریز UI).
                if let Some(ms) = self.latency_slot.lock().take() {
                    self.latency_ms = Some(ms);
                }
                let latency_due = self
                    .latency_probe_at
                    .map(|t| Instant::now() >= t)
                    .unwrap_or(true);
                if latency_due {
                    self.latency_probe_at = Some(Instant::now() + Duration::from_secs(15));
                    let slot = self.latency_slot.clone();
                    std::thread::Builder::new()
                        .name("aether-latency".into())
                        .spawn(move || {
                            let started = Instant::now();
                            if probe::tcp_via_proxy("1.1.1.1", 80) {
                                *slot.lock() = Some(started.elapsed().as_millis() as u64);
                            }
                        })
                        .ok();
                }
                if !self.engine.is_alive() {
                    // معادل superviseEngine: بک‌آف پلکانی ۲/۵/۱۰ ثانیه، حداکثر ۳ تلاش.
                    let max_retries = self.profile.reconnect_attempts.max(DEFAULT_MAX_RETRIES);
                    if self.attempts >= max_retries {
                        self.fail("The engine keeps dying — giving up after repeated restarts.");
                        return;
                    }
                    let backoff = BACKOFF_MS[(self.attempts as usize).min(BACKOFF_MS.len() - 1)];
                    self.attempts += 1;
                    self.connected_at = None;
                    self.reconnect_at = Some(Instant::now() + Duration::from_millis(backoff));
                    DiagnosticsLog::w(
                        TAG,
                        &format!("Engine died while connected — restarting in {}s.", backoff / 1000),
                    );
                    let detail = format!("Attempt {} of {}", self.attempts, max_retries);
                    self.set_state(ConnectionState::Reconnecting, &detail);
                }
            }
            ConnectionState::Reconnecting => {
                if let Some(at) = self.reconnect_at {
                    if Instant::now() >= at {
                        self.reconnect_at = None;
                        // همان پلهٔ برنده دوباره اجرا می‌شود — معادل restart در superviseEngine.
                        if self.plan.is_empty() {
                            self.plan = smart_auto::build_plan(&self.profile, probe::network_looks_filtered());
                            self.plan_index = 0;
                        }
                        self.cleanup_native(true);
                        if let Err(e) = self.start_candidate() {
                            let msg = e.to_string();
                            self.fail(&msg);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn past_deadline(&self) -> bool {
        self.deadline.map(|d| Instant::now() > d).unwrap_or(false)
    }

    fn fail(&mut self, why: &str) {
        DiagnosticsLog::e(TAG, why);
        self.cleanup_native(true);
        self.error = Some(why.to_string());
        self.connected_at = None;
        self.deadline = None;
        self.reconnect_at = None;
        self.verify_slot = None;
        self.set_state(ConnectionState::Failed, "Connection failed");
    }

    fn set_state(&mut self, state: ConnectionState, detail: &str) {
        let prev = self.state;
        self.state = state;
        self.detail = detail.to_string();
        DiagnosticsLog::i(TAG, &format!("{state:?} {detail}"));
        if prev != state {
            self.on_phase_change(state);
        }
    }

    /// معادل LaunchedEffect فازهای IP در MainActivity.kt:
    ///   connected → IP سرور از دل تونل — idle/failed → IP واقعی کاربر — busy → خالی.
    fn on_phase_change(&mut self, state: ConnectionState) {
        match state {
            ConnectionState::Connected => { /* خودآزما قبلاً IP را تحویل داده است */ }
            ConnectionState::Disconnected | ConnectionState::Failed => {
                spawn_ip_lookup(self.ip_slot.clone(), false);
            }
            _ => {
                let mut g = self.ip_slot.lock();
                g.session += 1;
                g.info = None;
                g.loading = false;
            }
        }
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }
}

impl Drop for AetherController {
    fn drop(&mut self) {
        // خروج برنامه هرگز نباید پروکسی سیستمی را فعال رها کند.
        self.cleanup_native(false);
    }
}

/// جست‌وجوی IP در ترد پس‌زمینه — همان تعداد تلاش/تأخیرهای NetProbe اندروید:
/// مستقیم ۶×۲۰۰۰ms، از دل تونل ۱۲×۱۰۰۰ms.
fn spawn_ip_lookup(slot: Arc<Mutex<IpSlot>>, via_tunnel: bool) {
    let session = {
        let mut g = slot.lock();
        g.session += 1;
        g.loading = true;
        if !via_tunnel {
            g.info = None;
        }
        g.session
    };
    std::thread::Builder::new()
        .name("aether-ipinfo".into())
        .spawn(move || {
            let result = if via_tunnel {
                probe::fetch_ip_via_socks_retry(12, 1_000, 6_000)
            } else {
                probe::fetch_ip_direct_retry(6, 2_000, 6_000)
            };
            let mut g = slot.lock();
            if g.session != session {
                return; // نتیجهٔ کهنه — فاز عوض شده است.
            }
            g.info = result.map(|i| IpEndpoint {
                ip: i.ip,
                country_code: i.country_code,
                via_tunnel,
            });
            g.loading = false;
        })
        .ok();
}

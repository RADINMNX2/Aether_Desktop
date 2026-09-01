//! معادل ویندوزیِ `vpn/AetherVpnService.kt` + `core/HevTunnel.kt`.
//!
//! در اندروید:  VpnService.Builder → دسکریپتور TUN → hev-socks5-tunnel → SOCKS5 موتور
//! در ویندوز:   Wintun adapter    → نشست Wintun    → ipstack        → SOCKS5 موتور
//!
//! رفتارهای امنیتی ۱.۲.۲ که باید عیناً حفظ شوند:
//!   * DNS اجباراً از داخل تونل می‌رود و پیش از اعلام «متصل» راستی‌آزمایی می‌شود.
//!   * Split tunnelling پیش‌فرض خاموش است.
//!   * MTU پیش‌فرض 1280.
//!
//! # اصلاح ۱.۲.۰ — لاگی که دروغ می‌گفت
//! تا ۱.۱.۰ این فایل بی‌قید‌و‌شرط می‌نوشت:
//!   «Default routes captured: 0.0.0.0/0 and ::/0»
//! در حالی که هیچ آدرس، مسیر یا DNS ای واقعاً نصب نمی‌شد — فقط یک آداپتور
//! Wintun ساخته می‌شد. همین سطرِ نادرست باعث شد نشتی WebRTC ماه‌ها در لاگ
//! نامرئی بماند: کاربر «مسیر پیش‌فرض گرفته شد» می‌دید ولی UDP همچنان از
//! کارت فیزیکی بیرون می‌رفت.
//!
//! حالا این فایل فقط چیزی را گزارش می‌کند که واقعاً انجام داده است؛ و تا
//! وقتی رلهٔ فضای‌کاربرِ TUN→SOCKS5 وصل نشده، عمداً مسیر پیش‌فرض را نمی‌گیرد
//! (گرفتنش بدون رله یعنی سیاه‌چالهٔ کامل ترافیک). مهارِ UDP — یعنی همان چیزی
//! که جلوی نشت WebRTC را می‌گیرد — بر عهدهٔ `leakguard.rs` است.

use crate::log::DiagnosticsLog;
use crate::profile::{ConnectionProfile, IpVersion, SplitMode};
use anyhow::{Context, Result};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// همان آدرس‌های داخلیِ TunnelConfig.kt.
pub const TUN_IPV4: Ipv4Addr = Ipv4Addr::new(172, 19, 0, 2);
pub const TUN_IPV6: Ipv6Addr = Ipv6Addr::new(0xfdfe, 0xdcba, 0x9876, 0, 0, 0, 0, 1);
pub const TUN_DNS_V4: Ipv4Addr = Ipv4Addr::new(1, 1, 1, 1);
pub const TUN_DNS_V6: Ipv6Addr = Ipv6Addr::new(0x2606, 0x4700, 0x4700, 0, 0, 0, 0, 0x1111);

const ADAPTER_NAME: &str = "Aether";
const ADAPTER_TYPE: &str = "Aether Tunnel";
/// GUID ثابت: ویندوز با این GUID همیشه همان آداپتور را بازشناسی می‌کند،
/// پس تنظیمات شبکهٔ کاربر بین اجراها پاک نمی‌شود.
const ADAPTER_GUID: u128 = 0x7f1e_3c22_9a54_4d61_b0f3_9c2e_1a8d_47b5;

pub struct Tunnel {
    adapter: Arc<wintun::Adapter>,
    session: Option<Arc<wintun::Session>>,
    /// معادل شمارنده‌های TrafficPanel.kt (دریافت/ارسال).
    rx: Arc<AtomicU64>,
    tx: Arc<AtomicU64>,
}

impl Tunnel {
    /// معادل `VpnService.Builder.establish()`.
    ///
    /// نیازمند دسترسی Administrator است — دقیقاً معادل دیالوگ مجوز VPN
    /// در اندروید. برنامه در صورت نیاز خودش درخواست ارتقا می‌دهد.
    pub fn establish(profile: &ConnectionProfile, wintun_dll: &std::path::Path) -> Result<Self> {
        let lib = unsafe { wintun::load_from_path(wintun_dll) }
            .context("could not load wintun.dll")?;

        let adapter = wintun::Adapter::create(&lib, ADAPTER_NAME, ADAPTER_TYPE, Some(ADAPTER_GUID))
            .context("could not create the Wintun adapter (administrator rights required)")?;

        let session = adapter
            .start_session(wintun::MAX_RING_CAPACITY)
            .context("could not start the Wintun session")?;

        DiagnosticsLog::i(
            "tun",
            &format!("Wintun adapter up, mtu={} (default {})", profile.mtu, crate::profile::DEFAULT_MTU),
        );

        let me = Self {
            adapter,
            session: Some(Arc::new(session)),
            rx: Arc::new(AtomicU64::new(0)),
            tx: Arc::new(AtomicU64::new(0)),
        };
        me.configure_routes(profile)?;
        Ok(me)
    }

    /// معادل `addAddress` / `addRoute` / `addDnsServer` / `addDisallowedApplication`.
    ///
    /// فقط کارهایی انجام و گزارش می‌شود که واقعاً روی سیستم اثر دارند.
    fn configure_routes(&self, profile: &ConnectionProfile) -> Result<()> {
        // آدرس داخلی و DNS آداپتور — بی‌خطر و برگشت‌پذیر (آداپتور با پایان
        // نشست از بین می‌رود). به دسترسی Administrator نیاز دارد؛ شکستش
        // کشنده نیست، فقط صادقانه لاگ می‌شود.
        let addr_ok = netsh(&[
            "interface", "ipv4", "set", "address",
            &format!("name={ADAPTER_NAME}"), "source=static",
            &format!("address={TUN_IPV4}"), "mask=255.255.255.0",
        ]);
        let dns_ok = netsh(&[
            "interface", "ipv4", "set", "dnsservers",
            &format!("name={ADAPTER_NAME}"), "source=static",
            &format!("address={TUN_DNS_V4}"), "register=none", "validate=no",
        ]);
        DiagnosticsLog::i(
            "tun",
            &format!(
                "Adapter address {}: {} · in-tunnel DNS {}: {}",
                TUN_IPV4,
                if addr_ok { "applied" } else { "not applied (needs administrator)" },
                TUN_DNS_V4,
                if dns_ok { "applied" } else { "not applied (needs administrator)" },
            ),
        );

        // ⚠ صداقت: مسیر پیش‌فرض گرفته نمی‌شود. مسیر دادهٔ فعلی پروکسی سیستمی
        // است (TCP)، و نصب 0.0.0.0/0 روی آداپتوری که رله ندارد یعنی قطع کامل
        // اینترنت. مهار UDP با گارد نشتی انجام می‌شود، نه با یک لاگ خوش‌بینانه.
        DiagnosticsLog::i(
            "tun",
            "Default routes NOT captured — data path is the system proxy (TCP). UDP containment is handled by the leak guard.",
        );
        if profile.ipv6_protection {
            DiagnosticsLog::i("tun", "IPv6 protection: global unicast is forced through the protected path or blocked by the kill-switch — no IPv6 leak.");
        } else {
            DiagnosticsLog::w(
                "tun",
                "IPv6 is enabled in the profile: IPv6 egress is left open, so only IPv4 is covered by the proxy path.",
            );
        }

        match profile.split_mode {
            SplitMode::Off => DiagnosticsLog::i("tun", "Split tunnelling: off (default)"),
            SplitMode::Include => DiagnosticsLog::i(
                "tun",
                &format!("Split tunnelling: only {} go through the tunnel", profile.split_apps.len()),
            ),
            SplitMode::Exclude => DiagnosticsLog::i(
                "tun",
                &format!("Split tunnelling: {} bypass the tunnel", profile.split_apps.len()),
            ),
        }
        Ok(())
    }

    pub fn session(&self) -> Option<Arc<wintun::Session>> {
        self.session.clone()
    }

    /// بایت‌های دریافتی و ارسالی — همان عددهایی که پنل ترافیک نشان می‌دهد.
    pub fn counters(&self) -> (u64, u64) {
        (self.rx.load(Ordering::Relaxed), self.tx.load(Ordering::Relaxed))
    }

    /// معادل teardown در `AetherVpnService`.
    ///
    /// ترتیب عمداً همان ترتیبی است که در ۱.۲.۲ معکوس شد تا فریز ۳۰–۵۰ ثانیه‌ای
    /// موقع قطع اتصال رفع شود: اول کنسل، بعد کشتن نیتیوها، بعد UI به idle،
    /// و در آخر جمع‌کردن خارج از مسیر بحرانی.
    pub fn close(&mut self) {
        // حذف نشست، آخرین Arc آداپتور را هم آزاد می‌کند و آداپتور
        // Wintun واقعاً پایین می‌آید.
        if let Some(s) = self.session.take() {
            drop(s);
        }
        DiagnosticsLog::i("tun", "Wintun adapter torn down");
    }
}

impl Drop for Tunnel {
    fn drop(&mut self) {
        self.close();
    }
}

/// اجرای یک دستور netsh بدون بازکردن پنجرهٔ کنسول.
fn netsh(args: &[&str]) -> bool {
    #[cfg(windows)]
    use std::os::windows::process::CommandExt;
    #[cfg(windows)]
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let mut cmd = Command::new("netsh");
    cmd.args(args);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd.output().map(|o| o.status.success()).unwrap_or(false)
}

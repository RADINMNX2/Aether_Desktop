//! پورت ۱:۱ از `app/src/main/java/studio/cluvex/aether/model/Profile.kt`
//!
//! هر تغییری در سمت اندروید باید دقیقاً همین‌جا هم اعمال شود؛ منطق ساخت
//! آرگومان‌های خط فرمان و متغیرهای محیطیِ موتور باید بایت‌به‌بایت یکسان بماند.
//!
//! v10 (هسته‌ی 1.5.0): سه قابلیت جدیدِ کاربرمحورِ هسته اضافه شد و هر کدام
//! پشت یک «قابلیت نسخه» (CoreCaps) گِیت شده‌اند تا هسته‌ی قدیمی‌تر هرگز فلگ
//! ناشناخته نگیرد و اتصال نشکند:
//!   * Zero Trust / WARP سازمانی  (--team, --access-*, --gateway)
//!   * قوانین مسیریابی            (--route-block, --route-direct)
//!   * DNS داخل تونل              (--dns)
//!
//! v11 (هسته‌ی 1.7.0): سه قابلیت جدید هسته اضافه شد و مثل قبل هر کدام پشت
//! «قابلیت نسخه» (CoreCaps) گِیت شده‌اند تا هسته‌ی 1.6.0 یا قدیمی‌تر هرگز
//! فلگ یا متغیرِ ناشناخته نبیند:
//!   * پروکسی بالادست            (--upstream / AETHER_UPSTREAM)
//!   * تشخیص نام از بایت‌های اول  (AETHER_ROUTE_SNIFF, AETHER_ROUTE_SNIFF_MS)
//!   * جایگزینی هویتِ ردشده       (AETHER_REPROVISION)

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Protocol {
    /// v8: "Auto" renamed to "Smart" (same as mobile). The serde alias keeps
    /// old profile.json files that stored "AUTO" loading fine.
    #[serde(alias = "AUTO")]
    Smart,
    Masque,
    Wireguard,
    Gool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ScanMode { Turbo, Balanced, Thorough, Stealth, Ironclad }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum IpVersion { V4, V6, Both }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Noize { Off, Light, Firewall, Balanced, Gfw, Aggressive }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EndpointMode { Auto, ManualPeer, ManualRange }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SplitMode { Off, Include, Exclude }

/// v10 (هسته‌ی 1.5.0): روش ورود به سازمان Cloudflare Zero Trust.
///   Off          → ثبت‌نام معمولی (کاربر ناشناس WARP) — رفتار قبلی، پیش‌فرض.
///   Email        → کد یک‌بارمصرف به ایمیل (`--access-email`).
///   ServiceToken → توکن سرویس Access برای ماشین‌های بدون تعامل / CI
///                  (`--access-id` + `--access-secret`).
///   Token        → یک JWT از پیش‌گرفته‌شده (`--access-token`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccessMode { Off, Email, ServiceToken, Token }

/// قابلیت‌های هسته‌ی همراه — تعیین می‌کند کدام فلگ‌ها امن‌اند که فرستاده شوند.
///
/// قاعده‌ی همیشگیِ مخزن: «ارتقای خودکار هسته هرگز نباید یک انتشار را بشکند».
/// فلگ‌های مخصوص 1.5.0 فقط وقتی به موتور می‌روند که نسخه‌ی واقعیِ سینک‌شده
/// آن‌ها را بفهمد؛ اگر کاربر هسته‌ی قدیمی‌تری را پین کرده باشد این گزینه‌ها
/// بی‌صدا نادیده گرفته می‌شوند تا یک فلگ ناشناخته موتور را نکشد.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreCaps {
    pub zero_trust: bool,
    pub routing: bool,
    pub custom_dns: bool,
    /// v11 — پروکسی بالادست (`--upstream`) از هسته‌ی 1.7.0.
    pub upstream: bool,
    /// v11 — تشخیص نام میزبان از بایت‌های اول برای قواعد دامنه‌ای و
    /// جایگزینی خودکار هویتی که Cloudflare قبولش ندارد. هر دو از 1.7.0.
    pub route_sniff: bool,
}

impl CoreCaps {
    /// همه‌ی قابلیت‌ها خاموش — پیش‌فرضِ محافظه‌کار وقتی نسخه‌ی هسته نامعلوم است.
    pub fn none() -> Self {
        Self {
            zero_trust: false,
            routing: false,
            custom_dns: false,
            upstream: false,
            route_sniff: false,
        }
    }

    /// همه‌ی قابلیت‌ها فعال — برای تست‌ها و مسیرهایی که نسخه‌ی هسته اهمیت ندارد.
    pub fn all() -> Self {
        Self {
            zero_trust: true,
            routing: true,
            custom_dns: true,
            upstream: true,
            route_sniff: true,
        }
    }

    /// نگاشت نسخه‌ی هسته به قابلیت‌ها. Zero Trust / routing / --dns از 1.5.0
    /// و پروکسی بالادست / تشخیص نام / بازثبت هویت از 1.7.0.
    pub fn for_version(major: u32, minor: u32) -> Self {
        let v15 = (major, minor) >= (1, 5);
        let v17 = (major, minor) >= (1, 7);
        Self {
            zero_trust: v15,
            routing: v15,
            custom_dns: v15,
            upstream: v17,
            route_sniff: v17,
        }
    }
}

/// v11 (هسته‌ی 1.7.0) — نوع پروکسی بالادست.
///
/// SOCKS5 با UDP associate هر سه پروتکل را حمل می‌کند؛ HTTP CONNECT فقط
/// TCP است، پس تنها مسیرِ کارآمد از آن، MASQUE روی HTTP/2 است (همان جدولِ
/// `Docs/DOCS.en.md` خودِ هسته).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpstreamKind {
    Socks5,
    Http,
}

/// آینه‌ی `upstream::Upstream::parse` هسته‌ی 1.7.0.
///
/// چرا اینجا تکرار شده: اگر رشته‌ی کاربر بی‌معنا باشد هسته فقط یک خط خطا
/// لاگ می‌کند و **بی‌صدا** بدون پروکسی ادامه می‌دهد؛ آن‌وقت کاربر خیال
/// می‌کند ترافیکش از پروکسی می‌رود. با این تابع، مقدار نامعتبر هرگز به
/// آرگومان‌ها راه پیدا نمی‌کند و UI هم می‌تواند همان لحظه هشدار بدهد.
///
/// قواعد دقیقاً مثل هسته: طرح‌واره‌ی خالی = `socks5`، پورت الزامی،
/// IPv6 داخل `[]`، و `user:pass@` اختیاری با آخرین `@` به‌عنوان مرز.
pub fn parse_upstream(raw: &str) -> Option<(UpstreamKind, String)> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }

    let (scheme, rest) = match raw.split_once("://") {
        Some((scheme, rest)) => (scheme.to_ascii_lowercase(), rest),
        None => ("socks5".to_string(), raw),
    };

    let kind = match scheme.as_str() {
        "socks5" | "socks5h" | "socks" => UpstreamKind::Socks5,
        "http" | "https" => UpstreamKind::Http,
        _ => return None,
    };

    let endpoint = match rest.rsplit_once('@') {
        Some((_credentials, endpoint)) => endpoint,
        None => rest,
    };
    let endpoint = endpoint.trim_end_matches('/');

    let (host, port) = if let Some(tail) = endpoint.strip_prefix('[') {
        let (host, tail) = tail.split_once(']')?;
        (host, tail.strip_prefix(':')?)
    } else {
        endpoint.rsplit_once(':')?
    };

    if host.is_empty() {
        return None;
    }
    match port.parse::<u16>() {
        Ok(0) | Err(_) => None,
        Ok(_) => Some((kind, raw.to_string())),
    }
}

pub const DEFAULT_MTU: u32 = 1280;
pub const MTU_PRESETS: [u32; 5] = [1280, 1380, 1420, 1500, 8500];
pub const KEEPALIVE_PRESETS: [u32; 4] = [0, 10, 25, 45];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ConnectionProfile {
    pub protocol: Protocol,
    pub scan_mode: ScanMode,
    pub ip_version: IpVersion,
    pub quick_reconnect: bool,
    pub masque_http2: bool,
    /// اشتراک تونل با دستگاه‌های دیگر روی همان شبکه (پورت‌های 10810/10811).
    pub lan_share: bool,
    /// v1.2.0 — هنگام قطع تونل، ترافیک مستقیم مرورگرها را قطع می‌کند.
    pub kill_switch: bool,
    /// v1.2.0 — IPv6 عمومی فقط از مسیر حفاظت‌شده عبور کند؛ پیش‌فرض روشن.
    pub ipv6_protection: bool,
    /// v1.2.0 — سقف تلاش‌های اتصال مجدد خودکار، بین ۳ تا ۲۰.
    pub reconnect_attempts: u32,
    /// v1.2.0 — گارد نشتی WebRTC/UDP. پیش‌فرض **روشن**.
    ///
    /// مسیر دادهٔ ویندوز پروکسی است و پروکسی فقط TCP را می‌گیرد؛ بدون این
    /// گارد، WebRTC با UDP خام به سرور STUN می‌رود و آی‌پی واقعی کاربر را لو
    /// می‌دهد (همان چیزی که در «WebRTC Leak Test» دیده می‌شد). فقط برای
    /// موارد خیلی خاص — مثل تماس تصویری داخلِ شبکهٔ سازمانی — خاموش می‌شود.
    pub leak_guard: bool,
    pub noize: Noize,
    pub endpoint_mode: EndpointMode,
    pub manual_peer: String,
    pub manual_range: String,
    pub keepalive: u32,
    pub fragment: bool,
    pub ech: bool,
    pub mtu: u32,
    // ---------------------------------------------------------------------
    // عمداً حذف شده نسبت به اندروید: `proxyMode`.
    // در ویندوز هیچ اپلیکیشنی به SOCKS5 محلی «به‌جای» VPN نیاز ندارد چون
    // Wintun کل سیستم را می‌گیرد و پروکسی سیستمی هم بومی است؛ نگه داشتنش
    // فقط یک مسیر کد بلااستفاده و یک حالت خطای اضافه می‌ساخت.
    // ---------------------------------------------------------------------
    pub split_mode: SplitMode,
    /// در ویندوز به‌جای package name، مسیر یا نام فرآیند (`chrome.exe`).
    pub split_apps: Vec<String>,

    // ====================================================================
    //  v10 — قابلیت‌های هسته‌ی 1.5.0 (هم‌ترازی با نسخه‌ی اندروید)
    // ====================================================================
    /// Zero Trust: نام سازمان (تیم) Cloudflare. خالی = ثبت‌نام معمولی WARP.
    pub team: String,
    /// روش احراز هویتِ Zero Trust.
    pub access_mode: AccessMode,
    /// ایمیل برای دریافت کد یک‌بارمصرف (فقط AccessMode::Email).
    pub access_email: String,
    /// شناسه‌ی توکن سرویس Access (فقط AccessMode::ServiceToken). غیرمحرمانه.
    pub access_id: String,
    /// عبور تمام HTTP/HTTPS از پراکسیِ Gateway سازمان (فیلترینگ/لاگِ سازمانی).
    /// پیش‌فرض خاموش — دقیقاً مثل هسته: یک هاپ اضافه و لاگ مرور را می‌افزاید.
    pub gateway: bool,

    /// قوانین مسیریابی: مقصدهایی که کاملاً مسدود می‌شوند (`--route-block`).
    pub route_block: Vec<String>,
    /// قوانین مسیریابی: مقصدهایی که از مسیر مستقیم (نه تونل) می‌روند
    /// (`--route-direct`) — برای بانک، سرویس‌های LAN و سایت‌های داخلی.
    pub route_direct: Vec<String>,

    /// DNS داخل تونل (`--dns`). خالی = پیش‌فرض هسته.
    pub dns: Vec<String>,

    // ====================================================================
    //  v11 — قابلیت‌های هسته‌ی 1.7.0
    // ====================================================================
    /// پروکسی بالادست (`--upstream`): هسته همه‌ی اتصال‌های بیرونی‌اش را از
    /// این پروکسی می‌گیرد تا بتوان اِتِر را پشت یک VPN یا پروکسیِ در حال
    /// اجرا روی همین ویندوز زنجیره کرد. خالی = اتصال مستقیم (پیش‌فرض).
    pub upstream: String,
    /// تشخیص نام میزبان از بایت‌های اول (`AETHER_ROUTE_SNIFF`).
    ///
    /// در ویندوز مسیر داده همیشه Wintun است؛ یعنی وقتی قاعده‌ی دامنه‌ای
    /// داریم پروکسی فقط یک آی‌پی می‌بیند و قواعد دامنه بی‌اثر می‌شدند.
    /// هسته‌ی 1.7.0 نام را از SNI یا هدر Host می‌خواند. پیش‌فرض **روشن** —
    /// دقیقاً مثل خود هسته.
    pub route_sniff: bool,
    /// جایگزینی خودکار هویتی که Cloudflare دیگر نمی‌پذیرد
    /// (`AETHER_REPROVISION`). پیش‌فرض روشن: وگرنه تونل دست می‌دهد ولی
    /// هیچ ترافیکی عبور نمی‌کند.
    pub reprovision: bool,

    // ----- اسرارِ در-حافظه (هرگز روی دیسک نوشته نمی‌شوند) ----------------
    // سخت‌سازی امنیتی: توکن سرویس و JWT حساس‌اند و مثل رفتار خودِ هسته
    // (کش در حافظه برای طول عمر فرآیند) فقط در حافظه نگه‌داری می‌شوند.
    // `skip_serializing` یعنی UI می‌تواند مقدار را بفرستد (deserialize مجاز)
    // ولی هیچ‌وقت در profile.json یا پاسخ get_profile برنمی‌گردد — نه هنگام
    // ذخیرهٔ معمول، نه هنگام Reset، نه در خروجی لاگ.
    /// راز توکن سرویس Access (فقط AccessMode::ServiceToken).
    #[serde(skip_serializing, default)]
    pub access_secret: String,
    /// JWT از پیش‌گرفته‌شده (فقط AccessMode::Token).
    #[serde(skip_serializing, default)]
    pub access_token: String,
}

impl Default for ConnectionProfile {
    fn default() -> Self {
        Self {
            protocol: Protocol::Smart,
            scan_mode: ScanMode::Balanced,
            ip_version: IpVersion::V4,
            quick_reconnect: true,
            masque_http2: false,
            lan_share: false,
            kill_switch: true,
            ipv6_protection: true,
            reconnect_attempts: 3,
            leak_guard: true,
            noize: Noize::Off,
            endpoint_mode: EndpointMode::Auto,
            manual_peer: String::new(),
            manual_range: String::new(),
            keepalive: 0,
            fragment: false,
            ech: false,
            mtu: DEFAULT_MTU,
            split_mode: SplitMode::Off,
            split_apps: Vec::new(),
            team: String::new(),
            access_mode: AccessMode::Off,
            access_email: String::new(),
            access_id: String::new(),
            gateway: false,
            route_block: Vec::new(),
            route_direct: Vec::new(),
            dns: Vec::new(),
            upstream: String::new(),
            route_sniff: true,
            reprovision: true,
            access_secret: String::new(),
            access_token: String::new(),
        }
    }
}

impl ConnectionProfile {
    /// Clamp user-controlled resilience settings at the trust boundary.
    pub fn normalize(&mut self) {
        self.reconnect_attempts = self.reconnect_attempts.clamp(3, 20);
        // Leak protection is mandatory and intentionally not user-editable.
        self.leak_guard = true;
    }

    pub fn has_manual_peer(&self) -> bool {
        self.endpoint_mode == EndpointMode::ManualPeer && !self.manual_peer.trim().is_empty()
    }

    /// آیا این پروفایل قصد ورود به یک سازمان Zero Trust را دارد؟
    pub fn uses_zero_trust(&self) -> bool {
        !self.team.trim().is_empty()
    }

    /// v11 — پروکسی بالادستِ معتبر، یا None اگر خالی/نامعتبر باشد.
    pub fn upstream_proxy(&self) -> Option<(UpstreamKind, String)> {
        parse_upstream(&self.upstream)
    }

    /// v11 — آیا پروکسی بالادست فقط TCP است؟ HTTP CONNECT نمی‌تواند UDP
    /// حمل کند، پس MASQUE باید روی HTTP/2 برود و WireGuard/WARP×2 از این
    /// نوع پروکسی رد نمی‌شوند.
    pub fn upstream_is_tcp_only(&self) -> bool {
        matches!(self.upstream_proxy(), Some((UpstreamKind::Http, _)))
    }

    /// معادل `Profile.kt::toArgs()` — رفتار قدیمی حفظ می‌شود.
    /// همه‌ی قابلیت‌ها فعال فرض می‌شوند؛ چون فیلدهای جدید به‌طور پیش‌فرض
    /// خالی‌اند، خروجی برای پروفایل پیش‌فرض دقیقاً مثل قبل است (قرارداد اندروید).
    pub fn to_args(&self) -> Vec<String> {
        self.to_args_with_caps(CoreCaps::all())
    }

    /// نسخه‌ی گِیت‌شده‌ی `toArgs()` — فلگ‌های 1.5.0 فقط با هسته‌ی سازگار.
    pub fn to_args_with_caps(&self, caps: CoreCaps) -> Vec<String> {
        let mut args: Vec<String> = Vec::new();

        match self.protocol {
            // AUTO هرگز به موتور نمی‌رسد: SmartAuto قبل از اجرا آن را به یک
            // پروتکل مشخص تبدیل می‌کند (دقیقاً مثل اندروید).
            Protocol::Smart => {}
            Protocol::Masque => args.push("--masque".into()),
            Protocol::Wireguard => args.push("--wg".into()),
            Protocol::Gool => args.push("--gool".into()),
        }

        if !self.has_manual_peer() {
            args.push(match self.scan_mode {
                ScanMode::Turbo => "--turbo",
                ScanMode::Balanced => "--balanced",
                ScanMode::Thorough => "--thorough",
                ScanMode::Stealth => "--stealth",
                ScanMode::Ironclad => "--ironclad",
            }.into());
        }

        args.push(match self.ip_version {
            IpVersion::V4 => "-4",
            IpVersion::V6 => "-6",
            IpVersion::Both => "--dual",
        }.into());

        args.push(if self.quick_reconnect { "--quick-reconnect" } else { "--no-quick-reconnect" }.into());

        if self.noize != Noize::Off {
            args.push("--noize".into());
            args.push(format!("{:?}", self.noize).to_lowercase());
        }

        if self.has_manual_peer() {
            args.push("--peer".into());
            args.push(self.manual_peer.trim().to_string());
        }

        if self.fragment { args.push("--fragment".into()); }
        if self.ech { args.push("--ech".into()); args.push("auto".into()); }
        if self.keepalive > 0 { args.push("--keepalive".into()); args.push(self.keepalive.to_string()); }

        // ----- Zero Trust / WARP سازمانی (هسته‌ی 1.5.0) -----------------
        if caps.zero_trust && self.uses_zero_trust() {
            args.push("--team".into());
            args.push(self.team.trim().to_string());
            match self.access_mode {
                AccessMode::Email if !self.access_email.trim().is_empty() => {
                    args.push("--access-email".into());
                    args.push(self.access_email.trim().to_string());
                }
                AccessMode::ServiceToken
                    if !self.access_id.trim().is_empty() && !self.access_secret.trim().is_empty() =>
                {
                    args.push("--access-id".into());
                    args.push(self.access_id.trim().to_string());
                    args.push("--access-secret".into());
                    args.push(self.access_secret.trim().to_string());
                }
                AccessMode::Token if !self.access_token.trim().is_empty() => {
                    args.push("--access-token".into());
                    args.push(self.access_token.trim().to_string());
                }
                _ => {}
            }
            if self.gateway {
                args.push("--gateway".into());
            }
        }

        // ----- قوانین مسیریابی (هسته‌ی 1.5.0) ---------------------------
        if caps.routing {
            let block: Vec<String> = clean_list(&self.route_block);
            if !block.is_empty() {
                args.push("--route-block".into());
                args.push(block.join(","));
            }
            let direct: Vec<String> = clean_list(&self.route_direct);
            if !direct.is_empty() {
                args.push("--route-direct".into());
                args.push(direct.join(","));
            }
        }

        // ----- DNS داخل تونل (هسته‌ی 1.5.0) -----------------------------
        if caps.custom_dns {
            let dns: Vec<String> = clean_list(&self.dns);
            if !dns.is_empty() {
                args.push("--dns".into());
                args.push(dns.join(","));
            }
        }

        // ----- پروکسی بالادست (هسته‌ی 1.7.0) ----------------------------
        // مقدار نامعتبر عمداً فرستاده نمی‌شود؛ هسته آن را بی‌صدا نادیده
        // می‌گیرد و کاربر گمان می‌کند زنجیره برقرار است.
        if caps.upstream {
            if let Some((_, value)) = self.upstream_proxy() {
                args.push("--upstream".into());
                args.push(value);
            }
        }

        args
    }

    /// معادل دقیق `Profile.kt::toEnv()` — رفتار قدیمی حفظ می‌شود.
    pub fn to_env(&self) -> BTreeMap<String, String> {
        self.to_env_with_caps(CoreCaps::all())
    }

    /// v11 — نسخه‌ی گِیت‌شده‌ی `toEnv()`. متغیرهای 1.7.0 فقط به هسته‌ای
    /// فرستاده می‌شوند که آن‌ها را می‌فهمد؛ همان قاعده‌ی همیشگیِ «هیچ‌چیز
    /// ناشناخته‌ای به موتور نفرست».
    pub fn to_env_with_caps(&self, caps: CoreCaps) -> BTreeMap<String, String> {
        let mut env = BTreeMap::new();

        // HTTP CONNECT هیچ UDP‌ای حمل نمی‌کند؛ با پروکسی بالادستِ HTTP تنها
        // مسیر کارآمد MASQUE روی HTTP/2 است. پس همان چیزی که هسته با
        // `--h2` می‌فهمد را خودمان روشن می‌کنیم تا کاربر با یک تونلِ خاموش
        // تنها نماند.
        let force_h2 = caps.upstream && self.upstream_is_tcp_only();
        env.insert(
            "AETHER_MASQUE_HTTP2".into(),
            if self.masque_http2 || force_h2 { "1".into() } else { "0".into() },
        );

        let range = self.manual_range.trim();
        if self.endpoint_mode == EndpointMode::ManualRange && !range.is_empty() {
            env.insert("AETHER_SCAN_CIDRS".into(), range.to_string());
            env.insert("AETHER_MASQUE_CIDRS".into(), range.to_string());
            env.insert("AETHER_WG_CIDRS".into(), range.to_string());
        }

        // ----- هسته‌ی 1.7.0 ---------------------------------------------
        if caps.route_sniff {
            if !self.route_sniff {
                env.insert("AETHER_ROUTE_SNIFF".into(), "0".into());
            }
            if !self.reprovision {
                env.insert("AETHER_REPROVISION".into(), "0".into());
            }
        }

        env
    }

    /// معادل دقیق `Profile.kt::connectTimeoutMs()`
    pub fn connect_timeout_ms(&self) -> u64 {
        if self.has_manual_peer() { return 45_000; }
        match self.scan_mode {
            ScanMode::Turbo => 60_000,
            ScanMode::Balanced => 150_000,
            ScanMode::Stealth => 240_000,
            ScanMode::Thorough => 300_000,
            ScanMode::Ironclad => 360_000,
        }
    }
}

/// حذف فاصله‌های اضافی و ورودی‌های خالی از یک فهرست (route/dns).
fn clean_list(items: &[String]) -> Vec<String> {
    items
        .iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// این تست همان «قرارداد» بین اندروید و ویندوز است. اگر روزی خروجی فرق
    /// کند، CI باید قرمز شود.
    #[test]
    fn default_profile_matches_android_argv() {
        let p = ConnectionProfile::default();
        assert_eq!(p.to_args(), vec!["--balanced", "-4", "--quick-reconnect"]);
        // v11: پروفایل پیش‌فرض هیچ متغیر جدیدی هم اضافه نمی‌کند.
        assert_eq!(
            p.to_env().keys().cloned().collect::<Vec<String>>(),
            vec!["AETHER_MASQUE_HTTP2".to_string()]
        );
    }

    /// v1.2.0: گارد نشتی باید پیش‌فرض روشن باشد و روی آرگومان‌های موتور
    /// اثری نگذارد (یک قابلیت کاملاً سمتِ ویندوز است).
    #[test]
    fn safety_defaults_are_on_and_retry_limit_is_bounded() {
        let mut p = ConnectionProfile::default();
        assert!(p.kill_switch);
        assert!(p.ipv6_protection);
        assert_eq!(p.reconnect_attempts, 3);
        p.reconnect_attempts = 99;
        p.normalize();
        assert_eq!(p.reconnect_attempts, 20);
    }

    #[test]
    fn leak_guard_is_on_by_default_and_never_reaches_the_engine() {
        let p = ConnectionProfile::default();
        assert!(p.leak_guard);
        assert!(!p.to_args().iter().any(|a| a.contains("leak")));
    }

    #[test]
    fn manual_peer_skips_scan_mode() {
        let p = ConnectionProfile {
            endpoint_mode: EndpointMode::ManualPeer,
            manual_peer: "188.114.96.1:2408".into(),
            protocol: Protocol::Masque,
            ..Default::default()
        };
        assert_eq!(p.to_args(), vec!["--masque", "-4", "--quick-reconnect", "--peer", "188.114.96.1:2408"]);
        assert_eq!(p.connect_timeout_ms(), 45_000);
    }

    #[test]
    fn noize_and_hardening_flags() {
        let p = ConnectionProfile {
            protocol: Protocol::Wireguard,
            noize: Noize::Gfw,
            fragment: true,
            ech: true,
            keepalive: 25,
            ..Default::default()
        };
        assert_eq!(
            p.to_args(),
            vec!["--wg", "--balanced", "-4", "--quick-reconnect", "--noize", "gfw",
                 "--fragment", "--ech", "auto", "--keepalive", "25"]
        );
    }

    // --- v10: قابلیت‌های هسته‌ی 1.5.0 -------------------------------------

    #[test]
    fn zero_trust_email_flags() {
        let p = ConnectionProfile {
            team: "acme".into(),
            access_mode: AccessMode::Email,
            access_email: "user@acme.com".into(),
            gateway: true,
            ..Default::default()
        };
        let args = p.to_args_with_caps(CoreCaps::all());
        assert!(args.windows(2).any(|w| w == ["--team", "acme"]));
        assert!(args.windows(2).any(|w| w == ["--access-email", "user@acme.com"]));
        assert!(args.contains(&"--gateway".to_string()));
    }

    #[test]
    fn zero_trust_service_token_flags() {
        let p = ConnectionProfile {
            team: "acme".into(),
            access_mode: AccessMode::ServiceToken,
            access_id: "id-123".into(),
            access_secret: "shh-secret".into(),
            ..Default::default()
        };
        let args = p.to_args_with_caps(CoreCaps::all());
        assert!(args.windows(2).any(|w| w == ["--access-id", "id-123"]));
        assert!(args.windows(2).any(|w| w == ["--access-secret", "shh-secret"]));
    }

    #[test]
    fn routing_and_dns_flags() {
        let p = ConnectionProfile {
            route_block: vec!["ads.example".into(), "  ".into()],
            route_direct: vec!["bank.ir".into(), "192.168.0.0/16".into()],
            dns: vec!["1.1.1.1".into(), "8.8.8.8".into()],
            ..Default::default()
        };
        let args = p.to_args_with_caps(CoreCaps::all());
        assert!(args.windows(2).any(|w| w == ["--route-block", "ads.example"]));
        assert!(args.windows(2).any(|w| w == ["--route-direct", "bank.ir,192.168.0.0/16"]));
        assert!(args.windows(2).any(|w| w == ["--dns", "1.1.1.1,8.8.8.8"]));
    }

    #[test]
    fn old_core_never_gets_15_flags() {
        // هسته‌ی 1.4: هیچ‌کدام از فلگ‌های 1.5.0 نباید فرستاده شوند.
        let p = ConnectionProfile {
            team: "acme".into(),
            access_mode: AccessMode::Email,
            access_email: "user@acme.com".into(),
            route_block: vec!["ads.example".into()],
            dns: vec!["1.1.1.1".into()],
            ..Default::default()
        };
        let caps = CoreCaps::for_version(1, 4);
        let args = p.to_args_with_caps(caps);
        assert!(!args.iter().any(|a| a.starts_with("--team")));
        assert!(!args.iter().any(|a| a.starts_with("--access")));
        assert!(!args.iter().any(|a| a.starts_with("--route")));
        assert!(!args.iter().any(|a| a == "--dns"));
        // ولی فلگ‌های پایه باید باشند.
        assert!(args.contains(&"--balanced".to_string()));
    }

    #[test]
    fn caps_gate_maps_versions() {
        assert!(!CoreCaps::for_version(1, 4).zero_trust);
        assert!(CoreCaps::for_version(1, 5).zero_trust);
        assert!(CoreCaps::for_version(1, 5).routing);
        assert!(CoreCaps::for_version(2, 0).custom_dns);
        // v11: قابلیت‌های هستهٔ 1.7.0 روی هستهٔ 1.6.0 خاموش‌اند.
        assert!(!CoreCaps::for_version(1, 6).upstream);
        assert!(!CoreCaps::for_version(1, 6).route_sniff);
        assert!(CoreCaps::for_version(1, 7).upstream);
        assert!(CoreCaps::for_version(1, 7).route_sniff);
        assert!(CoreCaps::for_version(2, 0).upstream);
    }

    // --- v11: قابلیت‌های هستهٔ 1.7.0 -------------------------------------

    #[test]
    fn upstream_parser_mirrors_the_core() {
        assert_eq!(
            parse_upstream("127.0.0.1:1080").map(|(k, _)| k),
            Some(UpstreamKind::Socks5)
        );
        assert_eq!(
            parse_upstream("socks5://alice:s3cret@127.0.0.1:1080").map(|(k, _)| k),
            Some(UpstreamKind::Socks5)
        );
        assert_eq!(
            parse_upstream("HTTP://proxy.example:8080/").map(|(k, _)| k),
            Some(UpstreamKind::Http)
        );
        assert_eq!(
            parse_upstream("socks5h://[::1]:1080").map(|(k, _)| k),
            Some(UpstreamKind::Socks5)
        );
        // بی‌پورت، طرح‌وارهٔ ناشناس، پورت صفر و پورت غیرعددی رد می‌شوند.
        assert!(parse_upstream("127.0.0.1").is_none());
        assert!(parse_upstream("ftp://127.0.0.1:21").is_none());
        assert!(parse_upstream("127.0.0.1:0").is_none());
        assert!(parse_upstream("127.0.0.1:https").is_none());
        assert!(parse_upstream("   ").is_none());
    }

    #[test]
    fn upstream_flag_only_reaches_a_17_core_and_only_when_valid() {
        let p = ConnectionProfile {
            upstream: " socks5://127.0.0.1:1080 ".into(),
            ..Default::default()
        };
        let args = p.to_args_with_caps(CoreCaps::all());
        assert!(args
            .windows(2)
            .any(|w| w == ["--upstream", "socks5://127.0.0.1:1080"]));
        // هستهٔ 1.6.0 این فلگ را نمی‌شناسد.
        assert!(!p
            .to_args_with_caps(CoreCaps::for_version(1, 6))
            .iter()
            .any(|a| a == "--upstream"));
        // مقدار بی‌معنا هرگز فرستاده نمی‌شود.
        let bad = ConnectionProfile { upstream: "not a proxy".into(), ..Default::default() };
        assert!(!bad.to_args().iter().any(|a| a == "--upstream"));
    }

    #[test]
    fn an_http_upstream_forces_masque_over_http2() {
        let p = ConnectionProfile {
            upstream: "http://proxy.example:8080".into(),
            ..Default::default()
        };
        assert!(p.upstream_is_tcp_only());
        assert_eq!(p.to_env().get("AETHER_MASQUE_HTTP2").map(String::as_str), Some("1"));
        // با پروکسی SOCKS5 انتخاب کاربر دست‌نخورده می‌ماند (UDP عبور می‌کند).
        let s = ConnectionProfile {
            upstream: "socks5://127.0.0.1:1080".into(),
            ..Default::default()
        };
        assert!(!s.upstream_is_tcp_only());
        assert_eq!(s.to_env().get("AETHER_MASQUE_HTTP2").map(String::as_str), Some("0"));
        // روی هستهٔ 1.6.0 اجباری در کار نیست چون --upstream هم فرستاده نمی‌شود.
        assert_eq!(
            p.to_env_with_caps(CoreCaps::for_version(1, 6))
                .get("AETHER_MASQUE_HTTP2")
                .map(String::as_str),
            Some("0")
        );
    }

    #[test]
    fn sniffing_and_reprovision_are_on_by_default_and_opt_out_only() {
        let p = ConnectionProfile::default();
        assert!(p.route_sniff);
        assert!(p.reprovision);
        let env = p.to_env();
        assert!(!env.contains_key("AETHER_ROUTE_SNIFF"));
        assert!(!env.contains_key("AETHER_REPROVISION"));

        let off = ConnectionProfile { route_sniff: false, reprovision: false, ..Default::default() };
        let env = off.to_env();
        assert_eq!(env.get("AETHER_ROUTE_SNIFF").map(String::as_str), Some("0"));
        assert_eq!(env.get("AETHER_REPROVISION").map(String::as_str), Some("0"));
        // روی هستهٔ 1.6.0 هیچ‌کدام فرستاده نمی‌شوند.
        let old = off.to_env_with_caps(CoreCaps::for_version(1, 6));
        assert!(!old.contains_key("AETHER_ROUTE_SNIFF"));
        assert!(!old.contains_key("AETHER_REPROVISION"));
    }

    #[test]
    fn secrets_are_never_serialised_to_disk() {
        // سخت‌سازی امنیتی: access_secret / access_token با serde(skip) هرگز
        // در profile.json ذخیره نمی‌شوند.
        let p = ConnectionProfile {
            access_secret: "top-secret".into(),
            access_token: "jwt-token".into(),
            ..Default::default()
        };
        let json = serde_json::to_string(&p).unwrap();
        assert!(!json.contains("top-secret"));
        assert!(!json.contains("jwt-token"));
        assert!(!json.contains("accessSecret"));
        assert!(!json.contains("accessToken"));
    }
}

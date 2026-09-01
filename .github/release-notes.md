# Aether Desktop 1.2.2

## What's new

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

<div dir="rtl">

## تازه‌های نسخهٔ ۱.۲.۲

**یادآوری ارتقا:** نسخهٔ ۱.۲.۲ با هستهٔ Aether Core 1.7.0 می‌آید: قواعد مسیریابی دامنه‌ای دیگر پشت درایور Wintun هم کار می‌کنند، اِتِر می‌تواند پشت یک VPN یا پروکسیِ دیگر روی همین ویندوز زنجیره شود، و هویتی که Cloudflare دیگر قبولش ندارد جایگزین می‌شود تا دیگر گرفتار تونلی نشوید که دست می‌دهد ولی ترافیک رد نمی‌کند. تمام محافظت‌های ۱.۲.۰ و ۱.۲.۱ سر جایش است.

### مقایسهٔ خلاصه با نسخهٔ ۱.۲.۱

**افزوده شد:** سورس کامل هستهٔ Aether Core 1.7.0 و baseline بیلد آن؛ کنترل **پروکسی بالادست** (`--upstream`) برای زنجیره‌کردن اِتِر پشت یک VPN یا پروکسیِ در حال اجرا؛ تطبیق قواعد دامنه با نام واقعی میزبان (TLS SNI یا هدر `Host`)؛ ثبت خودکار هویت تازه وقتی هویت قدیمی رد می‌شود؛ و گیت قابلیت 1.7.0 برای فلگ و متغیرهای جدید.

**تغییر کرد:** نسخهٔ دسکتاپ به ۱.۲.۲؛ `CORE_VERSION` ریشه و `native/aether/CORE_VERSION` و baseline همگام‌سازی به 1.7.0؛ با پروکسی بالادستِ HTTP خودکار MASQUE روی HTTP/2 می‌رود (چون HTTP CONNECT توان حمل UDP ندارد)؛ مقدار `--upstream` در لاگ ماندگار ماسک می‌شود؛ و WARP×2 (`gool`) دو هاپ خود را روی دو لبهٔ متفاوت Cloudflare می‌سازد.

**حفظ شد:** محافظت اجباری WebRTC و IPv6 (fail-closed)، کیل‌سوییچ، واچداگ سه‌هدفه، بازگردانی دقیق proxy/PAC، خروج محدود به زمان، نگه‌داری اسرار Zero Trust فقط در حافظه، و قاعدهٔ «هیچ فلگ یا متغیری به هسته‌ای که نمی‌فهمد فرستاده نمی‌شود».

### جزئیات تغییرها

**یکپارچه‌سازی هستهٔ ۱.۷.۰:** پوشهٔ `native/aether` اکنون سورس ۱.۷.۰ و وابستگی Quiche را دارد. بیلد دسکتاپ، payload پرتابل، پنل درباره، مسیر rollback و artifact هسته در CI همگی یک `CORE_VERSION` واحد می‌خوانند و baseline پروبرهای پچ‌شده از خود ۱.۷.۰ دوباره کاشته شده است.

**پروکسی بالادست (تازه):** پیشرفته ← *پروکسی بالادست* قالب‌های `socks5://host:port`، `socks5://user:pass@host:port`، `http://host:port` و یا فقط `host:port` (معنای SOCKS5) را می‌پذیرد. هسته همهٔ اتصال‌های خروجی‌اش — از جمله اسکن نقطهٔ اتصال، ثبت‌نام و جستجوی ECH — را از همان پروکسی می‌گیرد. مقدار ورودی هم در رابط کاربری و هم در Rust اعتبارسنجی می‌شود؛ مقدار نامعتبر هرگز به موتور نمی‌رسد، چون خود موتور فقط یک خط خطا می‌نویسد و بی‌صدا بدون پروکسی ادامه می‌دهد. پروکسی SOCKS5 با UDP associate هر سه پروتکل را حمل می‌کند؛ پروکسی HTTP فقط TCP است، پس MASQUE خودکار روی HTTP/2 می‌رود و پنل هم همین را می‌گوید.

**قواعد دامنه‌ای دیگر پشت Wintun کار می‌کنند:** در ویندوز مسیر داده همیشه درایور TUN است؛ پس پروکسی محلی فقط یک آی‌پی می‌دید و قواعد دامنه بی‌صدا بی‌اثر می‌شدند. هستهٔ 1.7.0 نام میزبان را از بایت‌های اول اتصال می‌خواند و تصمیم را بر همان می‌گیرد، در عین حال اتصال همان نشانی‌ درخواستی مشتری را دنبال می‌کند. کلید *تطبیق قواعد دامنه با نام واقعی میزبان* پیش‌فرض روشن است.

**هویتِ ردشده جایگزین می‌شود:** Cloudflare می‌تواند دستگاه ذخیره‌شده را دیگر نپذیرد؛ در این حالت دست‌دادن موفق است ولی هیچ ترافیکی عبور نمی‌کند. موتور این رد را تشخیص می‌دهد و دستگاه تازه ثبت می‌کند؛ کلید *جایگزینی هویتِ ردشده* پیش‌فرض روشن است و می‌توانید خاموشش کنید تا فقط خبرتان کند.

**WARP×2 روی دو لبهٔ متفاوت:** وارپ تودرتو دیگر یک لبه را درون خودش تونل نمی‌کند و برای هاپ بیرونی و درونی دو نقطهٔ جدا انتخاب می‌کند. روی شبکه‌های سخت‌گیر — مخصوصاً با رنج دستی باریک — ممکن است یک دور اسکن اضافه لازم شود.

**گسترش گیت نسخه:** به `CoreCaps` قابلیت‌های 1.7.0 اضافه شد. روی هستهٔ پین‌شدهٔ قدیمی‌تر، فلگ و متغیرهای جدید فرستاده نمی‌شوند و بخش‌های مربوطه در رابط کاربری با توضیح غیرفعال می‌شوند — دقیقاً مثل قابلیت‌های 1.5.0.

**لاگ:** `--upstream` کنار `--access-secret`، `--access-token` و `--access-email` در فهرست ماسک قرار گرفته؛ اعتبارنامهٔ پروکسی هرگز در فایل لاگ نمی‌نشیند.

### خلاصهٔ ممیزی امنیتی

| بخش | نتیجه |
|---|---|
| کلیدها و اسرار | اعتبارنامهٔ هاردکدشده وجود ندارد؛ مقادیر حساس و اعتبارنامهٔ پروکسی ذخیره نمی‌شوند |
| TLS و گواهی‌ها | اعتبارسنجی سیستم‌عامل به‌همراه پین SPKI |
| DNS، IPv6 و WebRTC | مسیر حفاظت‌شده بررسی می‌شود؛ UDP مستقیم و مسیر ناامن IPv6 مسدود است |
| ذخیره‌سازی و لاگ | آی‌پی‌ها ماسک و اسرار حذف می‌شوند؛ حفاطت فایل هویتی هنوز نیازمند سخت‌سازی است |
| مجوزها و بیلد | UAC اجباری است؛ CI سورس، تست، مانیفست، نصاب و پاک‌سازی را بررسی می‌کند |

گزارش کامل: [SECURITY-AUDIT.md](SECURITY-AUDIT.md).

</div>

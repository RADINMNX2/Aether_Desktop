# Aether Desktop 1.3.0

## What's new

**Release 1.3.0** bundles **Aether Core 1.8.0**, ships a new glass-style interface with smooth motion, and rebuilds the CI pipeline around fast, modular, deterministic jobs.

### Short comparison with 1.2.2

**Upgraded the engine to Aether Core 1.8.0.** The stream-head reader that recognises the real host name behind Wintun was reworked and covered with new unit tests, the server-selection logic was tightened, and the SOCKS data plane, Noise handshake helpers, MASQUE/QUIC handling and the upstream/fragment paths received another round of robustness fixes. No new command-line surface was added, so every existing capability gate and UI control keeps its exact behaviour.

**New visual identity.** The entire interface was redrawn around glassmorphism — ambient animated aurora behind the app, frosted-glass cards, subtle gradients, spring-like motion and a smooth cross-fade when switching pages. The window still honours `prefers-reduced-motion` and stays responsive on modest hardware.

**New CI pipeline.** The build was split into four roles: a cheap `validate` job (repository sanity, JavaScript syntax gate, vendored-core consistency) that fails in seconds before any Windows runner boots; core sync is now opt-in and only runs on a manual version pin, so every main push is byte-for-byte reproducible; the two Windows builds (x64/x86) run in parallel with Rust caches that now also cover the heavy engine crate; `npm ci` runs against a committed lockfile; and clippy/cargo-audit run once on x64 instead of twice. Release publishing stays idempotent through the official `gh` CLI.

### Detailed changes

**Core 1.8.0 integration:** `native/aether` now contains the supplied 1.8.0 engine and its Quiche dependency; the sync baseline, the root `CORE_VERSION`, the vendored `native/aether/CORE_VERSION` and the About panel all report 1.8.0. The engine binary is built with the same `--locked` profile and static CRT as before.

**Domain routing hardening (core):** the code that reads the TLS server name / HTTP `Host` from the first bytes of a connection was reworked in 1.8.0 and now has dedicated tests for a head arriving in one piece, split across writes, and for leaving trailing bytes untouched. This is exactly the code that makes domain rules work behind Wintun, so the switch continues to behave as documented in 1.2.2.

**Selection robustness (core):** the endpoint-selection paths gained tighter internal handling, and the SOCKS/upstream/fragment paths received minor fixes. There is no behavioural switch for the user; existing protection, kill-switch, watchdog and cleanup behaviour is preserved.

**Visual refresh:** ambient background glow, frosted glass panels, gradient accents and spring-based motion replace the previous flat theme. Page navigation now cross-fades, and the whole animation layer is skipped automatically for users who prefer reduced motion.

**Faster, modular CI:** no upstream clone or ~10MB core artifact upload on ordinary pushes; deterministic vendored core; `npm ci` on the committed lockfile with npm cache; Rust cache now keyed over both workspaces; `cargo generate-lockfile` staged before caching and audit; clippy and audit on x64 only; the engine build and Tauri build have hard timeouts so a hang can never stall the queue.

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

## تازه‌های نسخهٔ ۱.۳.۰

**نسخهٔ ۱.۳.۰** هستهٔ **Aether Core 1.8.0** را همراه می‌آورد، رابط کاربری را با امضای شیشه‌ای و حرکت نرم بازطراحی می‌کند و خط لولهٔ CI را به‌صورت ماژولار، سریع و تکرارپذیر از نو می‌سازد.

### مقایسهٔ خلاصه با نسخهٔ ۱.۲.۲

**موتور به Aether Core 1.8.0 ارتقا یافت.** خوانندهٔ سرِ جریان که نام واقعی میزبان را پشت Wintun تشخیص می‌دهد بازنویسی و با تست‌های واحد جدید پوشش داده شد، منطق انتخاب نقطهٔ سمت سرور سفت‌تر شد، و به مسیر دادهٔ SOCKS، helpers دست‌دادن Noise، MASQUE/QUIC و مسیرهای upstream/fragment یک دور دیگر اصلاح پایداری خورد. سطح ورودی CLI تغییری نکرد؛ پس همهٔ گیت‌های نسخه و کنترل‌های رابط کاربری دقیقاً همان رفتار قبلی را دارند.

**مظهر تازهٔ بصری.** کل رابط با سبک شیشه‌ای از نو کشیده شد — پس‌زمینهٔ محو متحرک، کارت‌های شیشه‌ای، گرادیان‌های ظریف، حرکت فنری و cross-fade نرم بین صفحات. پنجره هنوز `prefers-reduced-motion` را رعایت می‌کند و روی سخت‌افزار ضعیف هم روان است.

**خط لولهٔ CI تازه.** بیلد به چهار نقش تقسیم شد: job سبک `validate` (سلامت مخزن، گیت syntax جاوااسکریپت، یکدستی هستهٔ vendored) که قبل از روشن‌شدن هر runner ویندوزی در چند ثانیه خطا می‌دهد؛ همگام‌سازی هسته حالا opt-in است و فقط با پین دستی نسخه اجرا می‌شود تا هر push روی main بیت‌به‌بیت تکرارپذیر باشد؛ دو بیلد ویندوز (x64/x86) موازی با کش Rust که این بار کریت سنگین موتور را هم می‌پوشاند؛ `npm ci` با lockfile کامیت‌شده؛ و clippy/cargo-audit فقط یک‌بار روی x64. انتشار همچنان با CLI رسمی `gh` و به‌صورت idempotent انجام می‌شود.

### جزئیات تغییرها

**یکپارچه‌سازی هستهٔ ۱.۸.۰:** پوشهٔ `native/aether` اکنون سورس ۱.۸.۰ و وابستگی Quiche را دارد؛ baseline همگام‌سازی، `CORE_VERSION` ریشه، `native/aether/CORE_VERSION` و پنل درباره همگی ۱.۸.۰ را گزارش می‌دهند. موتور با همان پروفایل `--locked` و CRT استاتیک قبل ساخته می‌شود.

**سفت‌سازی مسیریابی دامنه (هسته):** کدی که TLS SNI یا هدر `Host` را از بایت‌های اول اتصال می‌خواند در ۱.۸.۰ بازنویسی شد و حالا تست‌های اختصاصی دارد برای رسیدنِ سرِ جریانِ یک‌تکه، پاره‌شده بین چند write، و دست‌نخوردن بایت‌های بعدی. این دقیقاً همان کدی است که قواعد دامنه را پشت Wintun کارگر می‌کند؛ پس کلید مقرون‌به‌صرفه همان رفتارِ مستندِ ۱.۲.۲ را دارد.

**پایداری انتخاب (هسته):** مسیرهای انتخاب نقطهٔ سمت سرور سفت‌تر شد و مسیرهای SOCKS/upstream/fragment اصلاحات جزئی گرفتند. هیچ کلید رفتاری برای کاربر اضافه نشده؛ محافظت‌ها، کیل‌سوییچ، واچداگ و رفتار پاک‌سازی قبلی حفظ شده‌اند.

**بازطراحی بصری:** درخشش محیطی پس‌زمینه، پنل‌های شیشه‌ای مات، گرادیان‌ها و حرکت فنری جایگزین تم سادهٔ قبل شدند. جابه‌جایی بین صفحات با cross-fade انجام می‌شود و کل لایهٔ انیمیشن برای کاربرانی که حرکت کم‌حرکت ترجیح می‌دهند خودکار حذف می‌شود.

**CI سریع‌تر و ماژولار:** در pushهای عادی دیگر کلون بالادست و آپلود ~۱۰MB هسته وجود ندارد؛ هستهٔ vendored قطعی است؛ `npm ci` روی lockfile کامیت‌شده با کش npm؛ کش Rust روی هر دو کارسپیس؛ `cargo generate-lockfile` قبل از کش و audit؛ clippy و audit فقط روی x64؛ و بیلد موتور و بیلد Tauri سقف زمانی سخت دارند تا هیچ‌وقت صف را قفل نکنند.

### خلاصهٔ ممیزی امنیتی

| بخش | نتیجه |
|---|---|
| کلیدها و اسرار | اعتبارنامهٔ هاردکدشده وجود ندارد؛ مقادیر حساس و اعتبارنامهٔ پروکسی ذخیره نمی‌شوند |
| TLS و گواهی‌ها | اعتبارسنجی سیستم‌عامل به‌همراه پین SPKI |
| DNS، IPv6 و WebRTC | مسیر حفاظت‌شده بررسی می‌شود؛ UDP مستقیم و مسیر ناامن IPv6 مسدود است |
| ذخیره‌سازی و لاگ | آی‌پی‌ها ماسک و اسرار حذف می‌شوند؛ حفاظت فایل هویتی هنوز نیازمند سخت‌سازی است |
| مجوزها و بیلد | UAC اجباری است؛ CI سورس، تست، مانیفست، نصاب و پاک‌سازی را بررسی می‌کند |

گزارش کامل: [SECURITY-AUDIT.md](SECURITY-AUDIT.md).

</div>
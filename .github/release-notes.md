# Aether Desktop 1.4.0

## What's new

**Release 1.4.0** ports the Aether Mobile 1.2.7 home screen onto the desktop shell: one glass connection card, the travelling four-colour light and the ping-strength meter, all built from the same design tokens as the app. The engine stays on Aether Core 1.8.0 and every 1.3.0 protection is unchanged.

### Short comparison with 1.2.2

**New connection card.** The loose status / meta / traffic blocks were replaced by a single glass surface — 26 dp radius and the same two-stop card gradient (`0xF0142039 → 0xF6090F1B`) as `ConnectionCard.kt`. The Connect button is now a tinted disc: a rotating arc while connecting, a breathing halo while idle, and a tick in the connected state.

**GlowCycle light.** While connected, the four-colour travelling light from `GlowCycle.kt` (cyan-green → mint → blue → amber) orbits the card's rim on a canvas, following the same pointAt/perimeter motion as Android.

**Ping strength meter.** A 26-bar waveform replaces the plain latency line. Its colour and quality label (Excellent / Good / Fair / Poor) match mobile exactly, showing "Measuring…" while probing and "Not connected" while offline.

**Design tokens.** The interface now consumes the exact seven-step navy palette (`#070B14 → #223154`) and the `#E8EEF9 / #93A1BC / #64718C` text dims from `Color.kt`. The animated aurora backdrop was dropped for a flat surface with predictable performance. All new labels ship in the Persian UI as well.

### Detailed changes

**Home (`src/views/home.js`):** full rewrite around a `ConnectionCard` component; HH:MM:SS uptime timer, single IP pill, speed strip (down / up / total) with live bandwidth totals and per-interval averaging, plus protocol / endpoint / latency slides as overlays inside the card. All spins and interactions run through WAAPI and `prefers-reduced-motion` is honoured throughout.

**Glow (`GlowCycle` port):** canvas-driven layered light with multiplicative additive blending, four-colour lap loop, breath pulse and automatic start/stop tied to the connection state and element visibility.

**Ping (`PingStrength` port):** 26-bar waveform with per-frame fall smoothing and a tiered tint (mint / amber / error / dim) matching the mobile quality thresholds (≤80 ms excellent, ≥300 ms worst).

**Design system (`tokens.css`, `app.css`, `desktop.css`):** token file now mirrors `Color.kt` 1:1; flat `#070B14` backdrop; card, button, rail and form tokens aligned to the new palette; RTL slide layout fixed so Persian labels keep their side while LTR values stay intact.

**No core or CLI change:** Aether Core 1.8.0 behaviour, the kill-switch, watchdog, leak protection, mandatory UAC manifest and installer are byte-identical to 1.3.0.

### Security audit summary

This release changed no security surface — no new network-facing code, no new persistence, no new permissions. The 1.3.0 audit results remain the current baseline.

Full report: [SECURITY-AUDIT.md](SECURITY-AUDIT.md).

<div dir="rtl">

## تازه‌های نسخهٔ ۱.۴.۰

**نسخهٔ ۱.۴.۰** صفحهٔ اصلی Aether Mobile 1.2.7 را به پوستهٔ دسکتاپ می‌آورد: یک کارت اتصال شیشه‌ای، نور گردشگر چهاررنگ و متر قدرت پینگ، همه با همان توکن‌های طراحی اپ. موتور همچنان Aether Core 1.8.0 است و هیچ‌یک از محافظت‌های ۱.۳.۰ تغییر نکرده.

### مقایسهٔ خلاصه با نسخهٔ ۱.۲.۲

**کارت اتصال تازه.** بلوک‌های پراکندهٔ وضعیت/ابرداده/ترافیک با یک سطح شیشه‌ای واحد جایگزین شدند — شعاع ۲۶dp و همان گرادیان دو-توقف کارت (`0xF0142039 → 0xF6090F1B`) مثل `ConnectionCard.kt`. دکمهٔ اتصال حالا یک دیسک کم‌رنگ است: در حالت اتصال حلقه می‌چرخد، در حالت بیکار هاله می‌تپد و در حالت متصل یک تیک نشان می‌دهد.

**نور GlowCycle.** تا وقتی متصل هستید، نور گردشگر چهاررنگ از `GlowCycle.kt` (فیروزه‌ای → نعنایی → آبی → کهربایی) دور لبهٔ کارت روی بوم‌رنگ می‌گردد و همان حرکت pointAt/perimeter اندروید را دنبال می‌کند.

**متر قدرت پینگ.** موج ۲۶ میله‌ای جای خط پینگِ ساده را گرفت. رنگ و برچسب کیفیت آن دقیقاً مثل موبایل است (عالی / خوب / متوسط / ضعیف) و در حالت اندازه‌گیری «در حال اندازه‌گیری…» و در حالت قطع «متصل نیست» نشان داده می‌شود.

**توکن‌های طراحی.** رابط از همان پالت هفت‌پله‌ای سرمه‌ای (`#070B14 → #223154`) و میزان‌های محوِ متنِ `#E8EEF9 / #93A1BC / #64718C` از `Color.kt` استفاده می‌کند. پس‌زمینهٔ شفق متحرک برای کارایی قابل‌پیش‌بینی با سطح تخت جایگزین شد و همهٔ برچسب‌های جدید در رابط فارسی هم ترجمه شده‌اند.

### جزئیات تغییرها

**صفحهٔ اصلی (`src/views/home.js`):** بازنویسی کامل حول کامپوننت `ConnectionCard`؛ تایمر اتصال HH:MM:SS، پیل آی‌پی، نوار سرعت (دانلود / آپلود / مجموع) با مجموع پهنای باند زنده و میانگین‌گیری دوره‌ای، و اسلایدهای پروتکل / نقطهٔ سمت سرور / تأخیر به‌عنوان لایه‌های روی هم داخل کارت. همهٔ چرخش‌ها با WAAPI و با رعایت `prefers-reduced-motion` انجام می‌شود.

**درخشش (پورت GlowCycle):** نور لایه‌ای canvas با ترکیب جمعی ضربی، حلقهٔ چهاررنگ، تپش نفس و شروع/توقف خودکار وابسته به وضعیت اتصال و نمایان بودن عنصر.

**پینگ (پورت PingStrength):** موج ۲۶ میله‌ای با نرم‌کردن افت در هر فریم و رنگ‌بندی پله‌ای (نعنایی / کهربایی / خطا / محو) منطبق بر آستانه‌های موبایل (تا ۸۰ms عالی، ۳۰۰ms+ ضعیف).

**نظام طراحی (`tokens.css`، `app.css`، `desktop.css`):** فایل توکن اکنون ۱:۱ با `Color.kt` است؛ پس‌زمینهٔ تخت ِ`#070B14`؛ توکن‌های کارت، دکمه، نوار جانبی و فرم‌ها با پالت جدید یکدست شدند؛ چیدمان اسلایدها در RTL اصلاح شد تا برچسب فارسی سر جایش بماند و مقدارهای LTR دست‌نخورده باشند.

**بدون تغییر در هسته یا CLI:** رفتار Aether Core 1.8.0، کیل‌سوییچ، واچ‌داگ، محافظت در برابر نشت، مانیفست UAC اجباری و نصب‌کننده بیت‌به‌بیت با ۱.۳.۰ یکسان است.

### خلاصهٔ ممیزی امنیتی

این نسخه هیچ سطح امنیتی جدیدی باز نکرده — نه کد شبکهٔ جدید، نه ذخیره‌سازی تازه، نه مجوز جدید. نتایج ممیزی ۱.۳.۰ همچنان مرجع فعلی است.

گزارش کامل: [SECURITY-AUDIT.md](SECURITY-AUDIT.md).

</div>
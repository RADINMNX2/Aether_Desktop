// پورت از ui/HomeScreen.kt + ConnectionCard.kt + ConnectButton.kt (1.2.7)
//
// صفحهٔ اصلیِ جدید یک «کارت اتصال» واحد است — دقیقاً مثل ConnectionCard.kt:
//   وضعیت → تایمر بزرگ مونو → پیل آی‌پی سرور → نوار سرعت → سه اسلاید متا
// (پروتکل/اندپوینت/تأخیر) همگی داخل یک سطح شیشه‌ایِ 26dp. هیچ‌چیز بیرون
// کارت شناور نیست؛ حتی صفحه‌ای که قبلاً چهار سطح جدا داشت.
//
// دو انیمیشن مشخصهٔ 1.2.7 هم اینجا پورت شده‌اند:
//  * GlowCycle — نورِ متحرک دورِ لبهٔ کارت هنگام اتصال: هر دور کامل پیرامون
//    یک رنگ اصلی (قرمز/سبز/آبی/زرد)، ۵.۲ ثانیه هر دور + تنفس ۱.۷ ثانیه‌ای.
//    روی canvas با blend-mode جمعی (additive) نقاشی می‌شود تا عیناً همان
//    "نور" باشد نه خط سطحی. فقط هنگام متصل فعال است (بدون فریم‌ساب‌اسکریپت).
//  * PingStrength — موج سینوسیِ ۲۶ میله‌ایِ در حال حرکت زیر اسلاید تأخیر.
//    رشدِ میله‌ها به کیفیتِ آخرین پینگ است (≤80ms عالی، ≥400ms ضعیف).
import { app, onChange, toggleConnection, accentFor, formatBytes } from '../main.js'
import { flagHtml } from '../flags.js'
import { t } from '../i18n.js'

const BUSY_STATES = ['STARTING_ENGINE', 'CONNECTING', 'VERIFYING', 'RECONNECTING', 'DISCONNECTING']

// همان آیکون‌های متریال ConnectButton.kt — PowerSettingsNew / Autorenew / Check.
const ICON_POWER =
  '<svg viewBox="0 0 24 24" width="56" height="56" aria-hidden="true"><path d="M12 3v9" stroke="currentColor" stroke-width="2.2" stroke-linecap="round"/><path d="M6.5 6.8a8 8 0 1 0 11 0" stroke="currentColor" stroke-width="2.2" fill="none" stroke-linecap="round"/></svg>'
const ICON_RENEW =
  '<svg viewBox="0 0 24 24" width="56" height="56" aria-hidden="true"><path d="M12 5a7 7 0 0 1 6.3 4" stroke="currentColor" stroke-width="2.2" fill="none" stroke-linecap="round"/><path d="M18.6 4.6V9h-4.4" stroke="currentColor" stroke-width="2.2" fill="none" stroke-linecap="round" stroke-linejoin="round"/><path d="M12 19a7 7 0 0 1-6.3-4" stroke="currentColor" stroke-width="2.2" fill="none" stroke-linecap="round"/><path d="M5.4 19.4V15h4.4" stroke="currentColor" stroke-width="2.2" fill="none" stroke-linecap="round" stroke-linejoin="round"/></svg>'
// 1.2.7: به‌جای برق (که پیام دکمهٔ خاموش بود)، تیک بزرگ — «از اتصال گذشته‌ای».
const ICON_CHECK =
  '<svg viewBox="0 0 24 24" width="84" height="84" aria-hidden="true"><path d="M5 13l4.5 4.5L19 7" stroke="currentColor" stroke-width="2.6" fill="none" stroke-linecap="round" stroke-linejoin="round"/></svg>'

// فلش‌های سرعت — ArrowDownward / ArrowUpward (15px مثل موبایل).
const ICON_DOWN =
  '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 4.5v15m0 0 6-6m-6 6-6-6" stroke="currentColor" stroke-width="2.4" fill="none" stroke-linecap="round" stroke-linejoin="round"/></svg>'
const ICON_UP =
  '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 19.5v-15m0 0-6 6m6-6 6 6" stroke="currentColor" stroke-width="2.4" fill="none" stroke-linecap="round" stroke-linejoin="round"/></svg>'

// زیرنویس وضعیت — همان رشته‌های strings.xml در StatusLine.kt.
function subtitleFor(snapshot) {
  switch (snapshot.state) {
    case 'DISCONNECTED': return t('Tap to connect securely')
    case 'CONNECTED': return t('Tap to disconnect')
    case 'FAILED': return snapshot.error || t('Something went wrong')
    case 'VERIFYING': return t('Verifying connection…')
    default: return snapshot.detail || ''
  }
}

// تایمر کارت — همیشه HH:MM:SS جلوی «مدت اتصال» (مثل TimerBlock موبایل).
function clock(secs) {
  const s = Math.max(0, Math.floor(secs))
  const pad = (x) => String(x).padStart(2, '0')
  return `${pad(Math.floor(s / 3600))}:${pad(Math.floor((s % 3600) / 60))}:${pad(s % 60)}`
}

// ---- متر قدرت پینگ — اعداد همان PingStrength.kt ثابت‌اند -----------------
const PING_BEST_MS = 80
const PING_WORST_MS = 400
const PING_BARS = 26
const PING_GAP = 3
const PING_H = 22
const PING_WAVE_MS = 1500
const PING_FAIR = '#FFB43C'
function pingStrength(ms) {
  if (ms == null) return 0
  if (ms <= PING_BEST_MS) return 1
  if (ms >= PING_WORST_MS) return 0.08
  return Math.max(0, Math.min(1, 1 - (ms - PING_BEST_MS) / (PING_WORST_MS - PING_BEST_MS)))
}
// توکن‌های رنگ فقط یک‌بار در زمان ساخت ماژول خوانده می‌شوند — خواندن
// getComputedStyle در هر فریمِ موج، recalc مصرفی داشت.
const _CSS_MINT = getCSS('--aether-mint', '#3EDBB0')
const _CSS_ERROR = getCSS('--aether-error', '#FF5C7A')
const _CSS_DIM = getCSS('--on-dark-dim', '#64718C')
function pingTint(strength) {
  if (strength >= 0.66) return _CSS_MINT
  if (strength >= 0.33) return PING_FAIR
  if (strength > 0.10) return _CSS_ERROR
  return _CSS_DIM
}
function pingQualityLabel(connected, ms) {
  if (!connected) return t('Not connected')
  if (ms == null) return t('Measuring')
  if (ms <= PING_BEST_MS) return t('Excellent')
  if (ms <= 160) return t('Good')
  if (ms <= 300) return t('Fair')
  return t('Poor')
}

// ---- GlowCycle — پورت مستقیم GlowCycle.kt --------------------------------
const GLOW_COLORS = ['#FF1E1E', '#00E23C', '#2A6BFF', '#FFD400']
const GLOW_LAP_MS = 5200
const GLOW_INSET = 12 // اتاقِ بلومِ بیرونِ کارت (clip=false در موبایل)
const GLOW_BANDS = [
  [0.00, 0.15, 2, 0.00, 0.00],
  [0.13, 0.08, 3, 0.34, 0.45],
  [0.28, 0.13, 5, 0.11, 0.20],
  [0.43, 0.06, 7, 0.61, 0.85],
  [0.56, 0.14, 3, 0.79, 0.35],
  [0.70, 0.09, 5, 0.24, 0.65],
  [0.85, 0.12, 2, 0.50, 1.00],
]
const GLOW_LAYERS = [
  [3.2, 0.045],
  [2.4, 0.085],
  [1.7, 0.16],
  [1.25, 0.34],
  [1.0, 0.82],
]
const TWO_PI = Math.PI * 2

// حرکت دورِ یک گوشه‌گرد 26dp — همان measure/path در GlowCycle.kt.
function buildPerimeter(w, h, r) {
  const sw = w - 2 * r
  const sh = h - 2 * r
  const arc = (Math.PI * r) / 2
  const q = Math.PI / 2
  const segs = [
    { len: sw, draw: (d) => ({ x: r + d, y: 0 }) },
    { len: arc, draw: (d) => ({ x: w - r + r * Math.sin((d / arc) * q), y: r - r * Math.cos((d / arc) * q) }) },
    { len: sh, draw: (d) => ({ x: w, y: r + d }) },
    { len: arc, draw: (d) => ({ x: w - r + r * Math.cos((d / arc) * q), y: h - r + r * Math.sin((d / arc) * q) }) },
    { len: sw, draw: (d) => ({ x: w - r - d, y: h }) },
    { len: arc, draw: (d) => ({ x: r - r * Math.sin((d / arc) * q), y: h - r + r * Math.cos((d / arc) * q) }) },
    { len: sh, draw: (d) => ({ x: 0, y: h - r - d }) },
    { len: arc, draw: (d) => ({ x: r - r * Math.cos((d / arc) * q), y: r - r * Math.sin((d / arc) * q) }) },
  ]
  let acc = 0
  for (const s of segs) { s.from = acc; acc += s.len }
  return { len: acc, segs }
}
function pointAt(track, dist) {
  let d = ((dist % track.len) + track.len) % track.len
  for (const s of track.segs) {
    if (d <= s.len) return s.draw(d)
    d -= s.len
  }
  return track.segs[0].draw(0)
}
function lerpColor(hex, target, k) {
  const a = parseInt(hex.slice(1), 16)
  const r = (a >> 16) & 255, g = (a >> 8) & 255, b = a & 255
  const tr = (target >> 16) & 255, tg = (target >> 8) & 255, tb = target & 255
  const mix = (x, y) => Math.round(x + (y - x) * k)
  return `rgb(${mix(r, tr)},${mix(g, tg)},${mix(b, tb)})`
}

// ---- ابزارها ------------------------------------------------------------
function getCSS(name, fallback) {
  try {
    return getComputedStyle(document.documentElement).getPropertyValue(name).trim() || fallback
  } catch {
    return fallback
  }
}

export function renderHome() {
  const root = document.createElement('div')
  root.className = 'view view--home'
  root.innerHTML = `
    <header class="hero">
      <h1 class="hero__title">${t('Aether')}</h1>
      <p class="hero__tagline">${t('Freedom, in one tap')}</p>
    </header>

    <div class="connect-wrap">
      <span class="connect__halo" id="halo"></span>
      <button class="connect" id="connect" type="button" aria-describedby="status">
        <svg class="connect__arc" viewBox="0 0 100 100" aria-hidden="true">
          <circle cx="50" cy="50" r="47" fill="none" stroke="currentColor" stroke-width="2.5"
                  stroke-linecap="round" stroke-dasharray="73.8 221.4"/>
        </svg>
        <span class="connect__icon" id="cicon">${ICON_POWER}</span>
      </button>
    </div>

    <!-- کارت اتصال — فقط همین سطح، هیچ‌چیز بیرونش شناور نیست. -->
    <section class="conn-card" id="conn-card" aria-label="${t('Connection')}">
      <canvas class="conn-card__glow" id="glow" aria-hidden="true"></canvas>

      <div class="conn-status">
        <p class="conn-status__title" id="status" role="status" aria-live="polite">${t('Disconnected')}</p>
        <p class="conn-status__caption" id="detail">${t('Tap to connect securely')}</p>
      </div>

      <div class="conn-timer" id="timer">
        <span class="conn-timer__k">${t('Connected for')}</span>
        <span class="conn-timer__v ltr" dir="ltr" id="timer-v">00:00:00</span>
      </div>

      <div class="ipbadge ltr" dir="ltr" id="ipbadge">
        <span class="ipbadge__label" id="ip-label">${t('Your IP')}</span>
        <span class="ipbadge__flag" id="ip-flag">${flagHtml(null)}</span>
        <span class="ipbadge__value" id="ip-value">${t('Checking IP…')}</span>
      </div>

      <div class="speedstrip" id="speed" dir="ltr">
        <div class="speedstrip__cell">
          <span class="speedstrip__badge speedstrip__badge--down" aria-hidden="true">${ICON_DOWN}</span>
          <span class="speedstrip__text">
            <span class="speedstrip__rate" id="t-rx-rate">0 B/s</span>
            <span class="speedstrip__total" id="t-rx">0 B</span>
          </span>
        </div>
        <span class="speedstrip__divider" aria-hidden="true"></span>
        <div class="speedstrip__cell">
          <span class="speedstrip__badge speedstrip__badge--up" aria-hidden="true">${ICON_UP}</span>
          <span class="speedstrip__text">
            <span class="speedstrip__rate" id="t-tx-rate">0 B/s</span>
            <span class="speedstrip__total" id="t-tx">0 B</span>
          </span>
        </div>
      </div>

      <div class="conn-slides">
        <div class="conn-slide">
          <div class="conn-slide__row">
            <span class="conn-slide__k">${t('Protocol')}</span>
            <span class="conn-slide__v ltr" id="m-proto">—</span>
          </div>
        </div>
        <div class="conn-slide conn-slide--endpoint">
          <div class="conn-slide__row">
            <span class="conn-slide__k">${t('Endpoint')}</span>
            <span class="conn-slide__v ltr" id="m-endpoint">—</span>
          </div>
        </div>
        <div class="conn-slide conn-slide--ping">
          <div class="conn-slide__row">
            <span class="conn-slide__k">${t('Latency')}</span>
            <span class="conn-slide__v ltr" id="m-latency">—</span>
          </div>
          <div class="ping">
            <div class="ping__head">
              <span class="ping__k">${t('Ping strength')}</span>
              <span class="ping__quality" id="ping-quality">${t('Not connected')}</span>
            </div>
            <canvas class="ping__wave" id="ping-wave" aria-hidden="true"></canvas>
          </div>
        </div>
      </div>
    </section>

    <!-- v1.2.0: نشان محافظت WebRTC — نتیجهٔ سنجش واقعی، نه ادعای تزئینی. -->
    <p class="shield" id="shield" hidden><span class="shield__dot"></span><span id="shield-text"></span></p>
  `

  root.querySelector('#connect').addEventListener('click', toggleConnection)

  // محاسبهٔ نرخ لحظه‌ای — همان کاری که SpeedStrip.kt با دلتای بایت‌ها می‌کرد.
  let lastRx = 0, lastTx = 0, lastAt = 0
  const spinAnims = []

  // پرچم کشور از `countryCode` می‌سازد — فقط وقتی ورودی دو حرف A-Z باشد،
  // وگرنه گلاب بی‌خطر. (کد نامعتبر می‌توانست HTML تزریق کند.)
  const flagFor = (cc) => (typeof cc === 'string' && /^[A-Z]{2}$/.test(cc) ? flagHtml(cc) : flagHtml(null))

  // کش کردن رفرنس المان‌ها — paint هر ۲۰۰ms صدا زده می‌شود و querySelector
  // مدام کل DOM را درخت‌پیمایی می‌کرد.
  const els = {
    btn: root.querySelector('#connect'),
    halo: root.querySelector('#halo'),
    icon: root.querySelector('#cicon'),
    arc: root.querySelector('.connect__arc'),
    status: root.querySelector('#status'),
    detail: root.querySelector('#detail'),
    timer: root.querySelector('#timer'),
    timerV: root.querySelector('#timer-v'),
    badge: root.querySelector('#ipbadge'),
    flag: root.querySelector('#ip-flag'),
    ipLabel: root.querySelector('#ip-label'),
    ipValue: root.querySelector('#ip-value'),
    shield: root.querySelector('#shield'),
    shieldText: root.querySelector('#shield-text'),
    proto: root.querySelector('#m-proto'),
    endpoint: root.querySelector('#m-endpoint'),
    latency: root.querySelector('#m-latency'),
    speed: root.querySelector('#speed'),
    rxRate: root.querySelector('#t-rx-rate'),
    txRate: root.querySelector('#t-tx-rate'),
    rx: root.querySelector('#t-rx'),
    tx: root.querySelector('#t-tx'),
    quality: root.querySelector('#ping-quality'),
    glow: root.querySelector('#glow'),
    ping: root.querySelector('#ping-wave'),
  }

  // ---- GlowCycle canvas — فقط هنگام متصل و بدون prefers-reduced-motion ----
  const reducedMotion = () =>
    typeof matchMedia === 'function' && matchMedia('(prefers-reduced-motion: reduce)').matches

  const glowState = { raf: 0, running: false, t0: 0 }
  const card = els.glow.parentElement

  function clearGlow() {
    const ctx = els.glow.getContext('2d')
    ctx.clearRect(0, 0, els.glow.width, els.glow.height)
  }
  function glowFrame(now) {
    glowState.raf = requestAnimationFrame(glowFrame)
    const el = now - glowState.t0
    const lap = Math.floor(el / GLOW_LAP_MS) % GLOW_COLORS.length
    const phase = (el % GLOW_LAP_MS) / GLOW_LAP_MS
    const breath = 0.74 + 0.26 * (0.5 + 0.5 * Math.sin((TWO_PI * el) / 3400 - Math.PI / 2))
    drawGlow(els.glow, phase, breath, GLOW_COLORS[lap])
  }
  function drawGlow(canvas, phase, breath, lapColor) {
    const dpr = window.devicePixelRatio || 1
    const w = canvas.clientWidth
    const h = canvas.clientHeight
    if (w < 4 || h < 4) return
    if (canvas.width !== Math.round(w * dpr) || canvas.height !== Math.round(h * dpr)) {
      canvas.width = Math.round(w * dpr)
      canvas.height = Math.round(h * dpr)
    }
    const ctx = canvas.getContext('2d')
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
    ctx.clearRect(0, 0, w, h)

    // مسیر روی لبهٔ کارت (به‌اندازهٔ 1.5px نور روی خط حاشیه) + گوشه‌گرد 26dp.
    const r = 26
    const x = GLOW_INSET
    const y = GLOW_INSET
    const pw = w - 2 * x
    const ph = h - 2 * y
    const track = buildPerimeter(pw, ph, r)
    if (track.len <= 0) return
    const base = lapColor
    const highlight = lerpColor(base, 0xffffff, 0.3)
    const stroke = 1.5

    ctx.globalCompositeOperation = 'lighter'
    ctx.lineCap = 'round'
    ctx.lineJoin = 'round'

    for (const [offset, span, harmonic, skew, tint] of GLOW_BANDS) {
      const raw = 0.5 + 0.5 * Math.sin(TWO_PI * (harmonic * phase + skew))
      const amp = raw * raw * (3 - 2 * raw) // smoothstep
      const length = track.len * span * (0.30 + 0.95 * amp)
      const start = ((phase + offset) % 1) * track.len
      const colour = lerpColor(base, highlight, Math.min(1, Math.max(0, tint * 0.6 + amp * 0.4)))
      const width = stroke * (1.05 + 1.75 * amp)
      const alpha = (0.18 + 0.82 * amp) * breath

      // نمونه برداری از کمان — یک مسیر نرم به‌جای PathMeasure.appendSegment.
      const steps = Math.max(2, Math.ceil(length / 2))
      let p = pointAt(track, start)
      ctx.strokeStyle = colour
      for (const [widthScale, alphaScale] of GLOW_LAYERS) {
        ctx.globalAlpha = Math.min(1, alpha * alphaScale)
        ctx.lineWidth = width * widthScale
        ctx.beginPath()
        ctx.moveTo(p.x, p.y)
        for (let i = 1; i <= steps; i++) {
          const q = pointAt(track, start + (length * i) / steps)
          ctx.lineTo(q.x, q.y)
        }
        ctx.stroke()
      }
    }
    ctx.globalCompositeOperation = 'source-over'
    ctx.globalAlpha = 1
  }
  function syncGlow() {
    const want = glowState.want === true && !document.hidden && !reducedMotion()
    if (want && !glowState.running) {
      glowState.running = true
      glowState.t0 = performance.now()
      glowState.raf = requestAnimationFrame(glowFrame)
    } else if (!want && glowState.running) {
      cancelAnimationFrame(glowState.raf)
      glowState.running = false
      clearGlow()
    } else if (!glowState.want && !glowState.running) {
      clearGlow()
    }
  }

  // ---- متر پینگ — موجِ متحرک فقط هنگام متصل ------------------------------
  const pingState = { raf: 0, running: false, t0: 0, level: 0, target: 0, phase: 0 }
  function drawPing(canvas, phase, level, tint) {
    const dpr = window.devicePixelRatio || 1
    const w = canvas.clientWidth
    const h = canvas.clientHeight
    if (w < 4 || h < 4) return
    if (canvas.width !== Math.round(w * dpr) || canvas.height !== Math.round(h * dpr)) {
      canvas.width = Math.round(w * dpr)
      canvas.height = Math.round(h * dpr)
    }
    const ctx = canvas.getContext('2d')
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
    ctx.clearRect(0, 0, w, h)

    const gap = PING_GAP
    const barW = Math.max(1, (w + gap) / PING_BARS - gap)
    const radius = barW / 2
    const floorH = h * 0.16
    ctx.fillStyle = tint
    for (let i = 0; i < PING_BARS; i++) {
      const x = i / (PING_BARS - 1)
      const a = 0.5 + 0.5 * Math.sin(TWO_PI * (2 * x - phase))
      const b = 0.5 + 0.5 * Math.sin(TWO_PI * (3.7 * x + 1.6 * phase))
      const wave = 0.58 * a + 0.42 * b
      const bh = floorH + (h - floorH) * level * wave
      const top = (h - bh) / 2
      const left = i * (barW + gap)
      const alpha = Math.min(1, Math.max(0, 0.30 + 0.70 * wave * (0.25 + 0.75 * level)))
      ctx.globalAlpha = alpha
      ctx.beginPath()
      ctx.roundRect(left, top, barW, bh, radius)
      ctx.fill()
    }
    ctx.globalAlpha = 1
  }
  function pingFrame(now) {
    pingState.raf = requestAnimationFrame(pingFrame)
    const el = now - pingState.t0
    pingState.phase = (el / PING_WAVE_MS) % 1
    // نرم‌سازی ارتفاع — مثل animateFloatAsState(tween(700)) موبایل.
    pingState.level += (pingState.target - pingState.level) * 0.1
    drawPing(
      els.ping,
      pingState.phase,
      Math.max(0.01, pingState.level),
      pingTint(pingState.target),
    )
  }
  function syncPing() {
    const want = pingState.want === true && !document.hidden && !reducedMotion()
    if (want && !pingState.running) {
      pingState.running = true
      pingState.t0 = performance.now()
      pingState.raf = requestAnimationFrame(pingFrame)
    } else if (!want && pingState.running) {
      cancelAnimationFrame(pingState.raf)
      pingState.running = false
      drawPing(els.ping, 0, Math.max(0.01, pingState.target), pingTint(pingState.target))
    } else if (!want && !pingState.running) {
      drawPing(els.ping, 0, Math.max(0.01, pingState.target), pingTint(pingState.target))
    }
  }

  // تغییر سایز — canvasها باید با DPR واقعی دوباره ساخته شوند.
  const ro = new ResizeObserver(() => {
    clearGlow()
    syncGlow()
    drawPing(els.ping, 0, Math.max(0.01, pingState.target), pingTint(pingState.target))
  })
  ro.observe(card)
  ro.observe(els.ping)
  const onVis = () => { syncGlow(); syncPing() }
  document.addEventListener('visibilitychange', onVis)
  const onReduced = () => { syncGlow(); syncPing() }
  const reducedMq = matchMedia('(prefers-reduced-motion: reduce)')
  reducedMq.addEventListener?.('change', onReduced)

  // آخرین خوانشِ خوبِ پینگ — پروبِ در حال پرواز ms=null می‌دهد و نباید
  // عددِ نمایشی پلک بزند (مثل lastMs در MetaSlides موبایل).
  let lastLatency = null

  const paint = ({ snapshot }) => {
    const accent = accentFor(snapshot.state)
    const busy = BUSY_STATES.includes(snapshot.state)
    const connected = snapshot.state === 'CONNECTED'

    els.btn.style.setProperty('--accent', accent)
    els.btn.classList.toggle('is-busy', busy)
    els.btn.classList.toggle('is-on', connected)
    els.btn.classList.toggle('is-error', snapshot.state === 'FAILED')
    els.halo.classList.toggle('is-on', connected)

    // تعویض آیکون — فقط وقتی مود واقعاً عوض شده (تا انیمیشن ریست نشود).
    const mode = busy ? 'busy' : connected ? 'on' : 'idle'
    if (els.icon.dataset.mode !== mode) {
      els.icon.dataset.mode = mode
      els.icon.innerHTML = busy ? ICON_RENEW : connected ? ICON_CHECK : ICON_POWER
      // Root fix (recurring problem 2): drive the busy spin with the Web
      // Animations API. Plain CSS animations get globally neutralised when
      // Windows reports prefers-reduced-motion (animation effects off /
      // VM / RDP), which is exactly why the arc + arrows looked frozen.
      // WAAPI animations run on the compositor and win over stylesheets.
      for (const a of spinAnims.splice(0)) a.cancel()
      if (busy) {
        const spin = [
          { transform: 'translateZ(0) rotate(0deg)' },
          { transform: 'translateZ(0) rotate(360deg)' },
        ]
        if (els.arc && els.arc.animate) {
          spinAnims.push(els.arc.animate(spin, { duration: 1200, iterations: Infinity }))
        }
        if (els.icon.animate) {
          spinAnims.push(els.icon.animate(spin, { duration: 1200, iterations: Infinity }))
        }
      }
    }

    const title = {
      DISCONNECTED: t('Disconnected'),
      STARTING_ENGINE: t('Starting engine…'),
      CONNECTING: t('Connecting…'),
      VERIFYING: t('Verifying connection…'),
      CONNECTED: t('Connected'),
      RECONNECTING: t('Reconnecting…'),
      DISCONNECTING: t('Disconnecting…'),
      FAILED: t('Connection failed'),
    }[snapshot.state] ?? snapshot.state
    els.status.textContent = title
    els.status.style.color = accent
    els.detail.textContent = subtitleFor(snapshot)

    // تایمر — مثل TimerBlock: همیشه جلوی «مدت اتصال»، هنگام قطع کم‌رنگ.
    els.timer.classList.toggle('is-off', !connected)
    els.timerV.textContent = connected ? clock(snapshot.uptimeSecs) : '00:00:00'

    // نشان IP + پرچم — همان رفتار MainActivity: در حالت‌های گذار مخفی.
    els.badge.hidden = busy
    if (!busy) {
      const info = snapshot.ipInfo
      els.ipLabel.textContent = info && info.viaTunnel ? t('Server IP') : t('Your IP')
      els.badge.classList.toggle('is-tunnel', !!(info && info.viaTunnel))
      if (info) {
        els.flag.innerHTML = flagFor(info.countryCode)
        els.ipValue.textContent = info.countryCode ? `${info.ip} · ${info.countryCode}` : info.ip
      } else {
        els.flag.innerHTML = flagHtml(null)
        els.ipValue.textContent = snapshot.ipLoading ? t('Checking IP…') : t('IP unavailable')
      }
    }

    // نشان محافظت WebRTC — سه حالت: در حال سنجش / محافظت‌شده / نشتی.
    els.shield.hidden = !connected
    if (connected) {
      const leak = snapshot.webrtcLeak
      const shieldState = leak === true ? 'leak' : leak === false ? 'safe' : 'unknown'
      els.shield.dataset.state = shieldState
      els.shieldText.textContent =
        shieldState === 'leak'
          ? t('WebRTC is leaking your real IP')
          : shieldState === 'safe'
            ? t('WebRTC protected — no IP leak')
            : t('Checking for WebRTC leaks…')
    }

    // اسلایدهای متا — هر فکت یک ردیف تمام‌عرض با کل عرض کارت.
    els.proto.textContent = connected ? (snapshot.protocol ?? '—') : '—'
    els.endpoint.textContent = connected ? (snapshot.endpoint ?? '…') : '—'

    if (connected) {
      if (snapshot.latencyMs != null) lastLatency = snapshot.latencyMs
    } else {
      lastLatency = null
    }
    const ms = connected && lastLatency != null ? lastLatency : null
    els.latency.textContent = ms != null ? `${ms} ms` : connected ? '…' : '—'

    // پینگ — ارتفاع/رنگ موج از کیفیت آخرین پروب.
    const strength = pingStrength(ms)
    pingState.target = connected ? strength : 0
    pingState.level = connected ? Math.max(pingState.level, strength * 0.2) : pingState.level
    els.quality.style.color = connected ? pingTint(strength) : _CSS_DIM
    els.quality.textContent = pingQualityLabel(connected, ms)

    // نوار سرعت — نرخ لحظه‌ای از دلتای بایت‌ها (مثل TrafficPanel/TrafficStrip).
    els.speed.classList.toggle('is-off', !connected)
    if (connected) {
      const now = Date.now()
      if (lastAt && now > lastAt) {
        const dt = (now - lastAt) / 1000
        const rxRate = Math.max(0, (snapshot.rxBytes - lastRx) / dt)
        const txRate = Math.max(0, (snapshot.txBytes - lastTx) / dt)
        els.rxRate.textContent = `${formatBytes(rxRate)}/s`
        els.txRate.textContent = `${formatBytes(txRate)}/s`
      }
      els.rx.textContent = formatBytes(snapshot.rxBytes)
      els.tx.textContent = formatBytes(snapshot.txBytes)
      lastRx = snapshot.rxBytes
      lastTx = snapshot.txBytes
      lastAt = now
    } else {
      els.rxRate.textContent = '0 B/s'
      els.txRate.textContent = '0 B/s'
      els.rx.textContent = formatBytes(snapshot.rxBytes)
      els.tx.textContent = formatBytes(snapshot.txBytes)
      lastRx = 0
      lastTx = 0
      lastAt = 0
    }

    // انیمیشن‌های کارت فقط هنگام اتصال.
    glowState.want = connected
    pingState.want = connected
    syncGlow()
    syncPing()
  }

  paint(app)
  const off = onChange(paint)
  root.cleanup = () => {
    off()
    for (const a of spinAnims.splice(0)) a.cancel()
    cancelAnimationFrame(glowState.raf)
    cancelAnimationFrame(pingState.raf)
    ro.disconnect()
    document.removeEventListener('visibilitychange', onVis)
    reducedMq.removeEventListener?.('change', onReduced)
  }
  return root
}
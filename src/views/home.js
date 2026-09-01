// پورت از ui/HomeScreen.kt + ConnectButton.kt + StatusLine.kt + ConnectionMeta.kt + TrafficPanel.kt
//
// رفع ریشه‌ای مشکل ۷: دکمهٔ قبلی فقط یک رنگ عوض می‌کرد؛ حالا مثل
// ConnectButton.kt چهار مود کامل دارد (IDLE / BUSY / CONNECTED / ERROR):
//   * آیکون عوض می‌شود: Power → Autorenew (چرخان) → Bolt
//   * در BUSY یک کمان ۹۰درجهٔ دور دکمه می‌چرخد (۱۱۰۰ms)
//   * در CONNECTED هاله نبض می‌زند (.92→1.06، ۱۶۰۰ms)
//   * گذار رنگ ۶۰۰ms — همان tween اندروید
// به‌علاوه نشان IP + پرچم کشور (Your IP / Server IP) و تایمر «Connected for».
import { app, onChange, toggleConnection, accentFor, formatBytes, formatUptime } from '../main.js'
import { flagHtml } from '../flags.js'
import { t } from '../i18n.js'

const BUSY_STATES = ['STARTING_ENGINE', 'CONNECTING', 'VERIFYING', 'RECONNECTING', 'DISCONNECTING']

// همان آیکون‌های متریال ConnectButton.kt — PowerSettingsNew / Autorenew / Bolt.
const ICON_POWER =
  '<svg viewBox="0 0 24 24" width="56" height="56" aria-hidden="true"><path d="M12 3v9" stroke="currentColor" stroke-width="2.2" stroke-linecap="round"/><path d="M6.5 6.8a8 8 0 1 0 11 0" stroke="currentColor" stroke-width="2.2" fill="none" stroke-linecap="round"/></svg>'
const ICON_RENEW =
  '<svg viewBox="0 0 24 24" width="56" height="56" aria-hidden="true"><path d="M12 5a7 7 0 0 1 6.3 4" stroke="currentColor" stroke-width="2.2" fill="none" stroke-linecap="round"/><path d="M18.6 4.6V9h-4.4" stroke="currentColor" stroke-width="2.2" fill="none" stroke-linecap="round" stroke-linejoin="round"/><path d="M12 19a7 7 0 0 1-6.3-4" stroke="currentColor" stroke-width="2.2" fill="none" stroke-linecap="round"/><path d="M5.4 19.4V15h4.4" stroke="currentColor" stroke-width="2.2" fill="none" stroke-linecap="round" stroke-linejoin="round"/></svg>'
const ICON_BOLT =
  '<svg viewBox="0 0 24 24" width="56" height="56" aria-hidden="true"><path d="M13 2 4.5 13.5H11L9.8 22 18.5 10.5H12L13 2Z" fill="currentColor"/></svg>'

// پرچم کشور: SVG درون‌ساخت از flags.js — ویندوز فونت ایموجی پرچم ندارد و
// کد کشور به‌صورت دو حرف (مثل «DE») رندر می‌شد؛ رفع ریشه‌ای بخش پرچم مشکل ۴.

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

export function renderHome() {
  const root = document.createElement('div')
  root.className = 'view view--home'
  root.innerHTML = `
    <div class="hero">
      <h1 class="hero__title">${t('Aether')}</h1>
      <p class="hero__tagline">${t('Freedom, in one tap')}</p>
    </div>

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

    <p class="status" id="status" role="status" aria-live="polite">${t('Disconnected')}</p>
    <p class="status__detail" id="detail">${t('Tap to connect securely')}</p>

    <div class="ipbadge ltr" dir="ltr" id="ipbadge">
      <span class="ipbadge__flag" id="ip-flag">${flagHtml(null)}</span>
      <span class="ipbadge__label" id="ip-label">${t('Your IP')}</span>
      <span class="ipbadge__value" id="ip-value">${t('Checking IP…')}</span>
    </div>

    <!-- v1.2.0: نشان محافظت WebRTC — نتیجهٔ سنجش واقعی، نه ادعای تزئینی. -->
    <p class="shield" id="shield" hidden><span class="shield__dot"></span><span id="shield-text"></span></p>

    <p class="uptime" id="uptime" hidden><span class="uptime__k">${t('Connected for')}</span> <span class="uptime__v ltr" dir="ltr" id="uptime-v">00:00</span></p>

    <div class="meta" id="meta">
      <div class="meta__cell"><span class="meta__k">${t('Protocol')}</span><span class="meta__v" id="m-proto">—</span></div>
      <div class="meta__cell"><span class="meta__k">${t('Endpoint')}</span><span class="meta__v ltr" dir="ltr" id="m-endpoint">—</span></div>
      <div class="meta__cell"><span class="meta__k">${t('Latency')}</span><span class="meta__v" id="m-latency">—</span></div>
    </div>

    <div class="traffic" id="traffic" hidden>
      <div class="traffic__cell">
        <span class="traffic__badge traffic__badge--down" aria-hidden="true">↓</span>
        <span class="traffic__text">
          <span class="traffic__k">${t('Download')}</span>
          <span class="traffic__rate ltr" dir="ltr" id="t-rx-rate">0 B/s</span>
          <span class="traffic__total ltr" dir="ltr" id="t-rx">0 B</span>
        </span>
      </div>
      <span class="traffic__divider" aria-hidden="true"></span>
      <div class="traffic__cell">
        <span class="traffic__badge traffic__badge--up" aria-hidden="true">↑</span>
        <span class="traffic__text">
          <span class="traffic__k">${t('Upload')}</span>
          <span class="traffic__rate ltr" dir="ltr" id="t-tx-rate">0 B/s</span>
          <span class="traffic__total ltr" dir="ltr" id="t-tx">0 B</span>
        </span>
      </div>
    </div>
  `

  root.querySelector('#connect').addEventListener('click', toggleConnection)

  // محاسبهٔ نرخ لحظه‌ای — همان کاری که TrafficPanel.kt با دلتای بایت‌ها می‌کرد.
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
    badge: root.querySelector('#ipbadge'),
    flag: root.querySelector('#ip-flag'),
    ipLabel: root.querySelector('#ip-label'),
    ipValue: root.querySelector('#ip-value'),
    shield: root.querySelector('#shield'),
    shieldText: root.querySelector('#shield-text'),
    uptime: root.querySelector('#uptime'),
    uptimeV: root.querySelector('#uptime-v'),
    proto: root.querySelector('#m-proto'),
    endpoint: root.querySelector('#m-endpoint'),
    latency: root.querySelector('#m-latency'),
    traffic: root.querySelector('#traffic'),
    rxRate: root.querySelector('#t-rx-rate'),
    txRate: root.querySelector('#t-tx-rate'),
    rx: root.querySelector('#t-rx'),
    tx: root.querySelector('#t-tx'),
  }

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
      els.icon.innerHTML = busy ? ICON_RENEW : connected ? ICON_BOLT : ICON_POWER
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
          spinAnims.push(els.arc.animate(spin, { duration: 1100, iterations: Infinity }))
        }
        if (els.icon.animate) {
          spinAnims.push(els.icon.animate(spin, { duration: 1400, iterations: Infinity }))
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

    // تایمر «Connected for»
    els.uptime.hidden = !connected
    if (connected) els.uptimeV.textContent = formatUptime(snapshot.uptimeSecs)

    els.proto.textContent = snapshot.protocol ?? '—'
    els.endpoint.textContent = snapshot.endpoint ?? '—'
    els.latency.textContent = snapshot.latencyMs != null ? `${snapshot.latencyMs} ms` : '—'

    // پنل ترافیک — فقط در حالت متصل (مثل TrafficPanel موبایل).
    els.traffic.hidden = !connected
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
      lastRx = 0
      lastTx = 0
      lastAt = 0
    }
  }

  paint(app)
  const off = onChange(paint)
  root.cleanup = () => off()
  return root
}
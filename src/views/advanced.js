// پورت از ui/AdvancedPanel.kt (+ SegmentedSelector.kt ، DropdownSelector.kt ، LtrInput.kt ، AppPickerDialog.kt)
// توجه: گزینهٔ Proxy Mode عمداً حذف شده — در ویندوز کاربردی ندارد.
// v8:
//   * پروتکل «Auto» به «Smart» تغییر نام داد — دقیقاً هم‌نام نسخهٔ موبایل.
//   * انتخاب زبان برنامه (English/فارسی) + دکمهٔ «بازنشانی به تنظیمات پیش‌فرض».
import { invoke } from '@tauri-apps/api/core'
import { app, saveProfile, rerender } from '../main.js'
import { t, getLang, setLang, LANGS } from '../i18n.js'
import { escAttr, wireSwitch } from '../utils.js'

// v10: فیلدهای فهرستی — هر خط یک مقدار (مثل splitApps).
const LIST_KEYS = ['splitApps', 'routeBlock', 'routeDirect', 'dns']

// v11 (هستهٔ 1.7.0):
//   * پروکسی بالادست (--upstream) — زنجیره‌کردن اِتِر پشت یک VPN/پروکسی دیگر.
//   * تطبیق قواعد دامنه‌ای از روی نام واقعی میزبان (SNI/Host) پشت Wintun.
//   * ثبت دوبارهٔ خودکار هویتی که Cloudflare دیگر قبولش ندارد.
const PROTOCOLS = [
  ['SMART', 'Smart'],
  ['MASQUE', 'MASQUE'],
  ['WIREGUARD', 'WireGuard'],
  ['GOOL', 'WARP×2'],
]
const SCAN_MODES = [
  ['TURBO', 'Turbo'],
  ['BALANCED', 'Balanced'],
  ['THOROUGH', 'Thorough'],
  ['STEALTH', 'Stealth'],
  ['IRONCLAD', 'Ironclad'],
]
const IP_VERSIONS = [['V4', 'IPv4'], ['V6', 'IPv6'], ['BOTH', 'Both']]
const NOIZE = [
  ['OFF', 'Off'], ['LIGHT', 'Light'], ['FIREWALL', 'Firewall'],
  ['BALANCED', 'Balanced'], ['GFW', 'GFW'], ['AGGRESSIVE', 'Aggressive'],
]
const ENDPOINT_MODES = [
  ['AUTO', 'Automatic'],
  ['MANUAL_PEER', 'Manual peer'],
  ['MANUAL_RANGE', 'Manual range'],
]
const SPLIT_MODES = [['OFF', 'Off'], ['INCLUDE', 'Only these apps'], ['EXCLUDE', 'All except these']]
const MTU_PRESETS = [1280, 1380, 1420, 1500]
const KEEPALIVE_PRESETS = [0, 10, 25, 45]
const RECONNECT_ATTEMPTS = Array.from({ length: 18 }, (_, i) => i + 3)

// v10 — هستهٔ 1.5.0: روش‌های ورود Zero Trust (همان گزینه‌های موبایل/هسته).
const ACCESS_MODES = [
  ['OFF', 'Off'],
  ['EMAIL', 'Email code'],
  ['SERVICE_TOKEN', 'Service token'],
  ['TOKEN', 'Access token'],
]

// v11 — آینهٔ دقیق parse_upstream در profile.rs (و upstream.rs هسته).
// فقط برای بازخورد زنده به کاربر است؛ تصمیم نهایی همیشه سمت Rust گرفته می‌شود.
export function parseUpstream(raw) {
  const value = (raw || '').trim()
  if (!value) return null
  const at = value.indexOf('://')
  const scheme = at === -1 ? 'socks5' : value.slice(0, at).toLowerCase()
  const rest = at === -1 ? value : value.slice(at + 3)
  let kind
  if (['socks5', 'socks5h', 'socks'].includes(scheme)) kind = 'socks5'
  else if (['http', 'https'].includes(scheme)) kind = 'http'
  else return null

  const cut = rest.lastIndexOf('@')
  let endpoint = cut === -1 ? rest : rest.slice(cut + 1)
  endpoint = endpoint.replace(/\/+$/, '')

  let host
  let port
  if (endpoint.startsWith('[')) {
    const end = endpoint.indexOf(']')
    if (end === -1 || endpoint[end + 1] !== ':') return null
    host = endpoint.slice(1, end)
    port = endpoint.slice(end + 2)
  } else {
    const colon = endpoint.lastIndexOf(':')
    if (colon === -1) return null
    host = endpoint.slice(0, colon)
    port = endpoint.slice(colon + 1)
  }
  if (!host || !/^[0-9]{1,5}$/.test(port)) return null
  const number = Number(port)
  if (number < 1 || number > 65535) return null
  return { kind, host, port: number }
}

function segmented(label, key, options, current) {
  return `
    <section class="field">
      <span class="field__label">${label}</span>
      <div class="seg" role="radiogroup" data-key="${key}">
        ${options.map(([v, tx]) => `
          <button type="button" role="radio" class="seg__item ${current === v ? 'is-active' : ''}"
                  data-value="${v}" aria-checked="${current === v}">${t(tx)}</button>`).join('')}
      </div>
    </section>`
}

function dropdown(label, key, options, current) {
  return `
    <section class="field field--row">
      <span class="field__label">${label}</span>
      <select class="select" data-key="${key}">
        ${options.map(([v, tx]) => `<option value="${v}" ${current === v ? 'selected' : ''}>${t(tx)}</option>`).join('')}
      </select>
    </section>`
}

function toggle(label, key, hint, on) {
  return `
    <section class="field field--row">
      <div>
        <span class="field__label">${label}</span>
        ${hint ? `<span class="field__hint">${hint}</span>` : ''}
      </div>
      <button type="button" class="switch ${on ? 'is-on' : ''}" data-key="${key}" role="switch" aria-checked="${on}">
        <span class="switch__knob"></span>
      </button>
    </section>`
}

function textField(label, key, value, placeholder, opts = {}) {
  const type = opts.secret ? 'password' : 'text'
  const hint = opts.hint ? `<span class=\"field__hint\">${opts.hint}</span>` : ''
  return `
    <section class="field">
      <span class="field__label">${label}</span>
      <input class="input ltr" dir="ltr" type="${type}" data-key="${key}" value="${escAttr(value ?? '')}" placeholder="${escAttr(placeholder)}" ${opts.secret ? 'autocomplete="off"' : ''}>
      ${hint}
    </section>`
}

// v10: تری‌ایریای چندخطی برای فهرست‌ها (routing/dns) — هر خط یک قاعده.
function listArea(label, key, values, placeholder, hint) {
  return `
    <section class="field">
      <span class="field__label">${label}</span>
      <textarea class="input input--area ltr" dir="ltr" data-key="${key}"
        placeholder="${escAttr(placeholder)}">${escAttr((values || []).join('\n'))}</textarea>
      ${hint ? `<span class=\"field__hint\">${hint}</span>` : ''}
    </section>`
}

// v10: تغییر روش ورود Zero Trust فیلدهای متفاوتی می‌خواهد — بازرندر.
// جایگزینی فقط همان بخش وابسته، نه کل ویو (تمرکز و اسکرول حفظ می‌شود).
function swapHost(root, render) {
  const host = root.parentElement
  if (!host) return
  host.innerHTML = ''
  host.appendChild(render())
}

export function renderAdvanced() {
  const p = app.profile
  const root = document.createElement('div')
  root.className = 'view view--advanced'
  // اگر boot هنوز پروفایل را نگرفته (مسابقهٔ setLang→rerender)، یک قاب
  // خالی برگردانیم تا دربارهٔ null منفجر نشویم؛ با رسیدن state بازرندر می‌شود.
  if (!p) {
    root.innerHTML = '<p class="view__lead">' + t('Loading…') + '</p>'
    return root
  }
  root.innerHTML = `
    <h2 class="view__title">${t('Advanced')}</h2>

    ${segmented(t('Language'), '__lang', LANGS, getLang())}

    ${segmented(t('Protocol'), 'protocol', PROTOCOLS, p.protocol)}
    ${segmented(t('Scan mode'), 'scanMode', SCAN_MODES, p.scanMode)}
    ${segmented(t('IP version'), 'ipVersion', IP_VERSIONS, p.ipVersion)}
    ${dropdown(t('Noize'), 'noize', NOIZE, p.noize)}
    ${dropdown(t('Endpoint'), 'endpointMode', ENDPOINT_MODES, p.endpointMode)}

    <div id="endpoint-extra">
      ${p.endpointMode === 'MANUAL_PEER' ? textField(t('Peer address'), 'manualPeer', p.manualPeer, '1.2.3.4:443') : ''}
      ${p.endpointMode === 'MANUAL_RANGE' ? textField(t('Address range'), 'manualRange', p.manualRange, '162.159.192.0/24') : ''}
    </div>

    ${dropdown('MTU', 'mtu', MTU_PRESETS.map((v) => [String(v), String(v)]), String(p.mtu))}
    ${dropdown('Keepalive', 'keepalive', KEEPALIVE_PRESETS.map((v) => [String(v), v === 0 ? t('Off') : `${v}s`]), String(p.keepalive))}

    ${toggle(t('Quick reconnect'), 'quickReconnect', t('Reconnect instantly after a drop'), p.quickReconnect)}
    ${toggle(t('MASQUE over HTTP/2'), 'masqueHttp2', t('Helps on networks that block HTTP/3'), p.masqueHttp2)}
    ${toggle(t('Packet fragmentation'), 'fragment', t('Splits the handshake to evade filtering'), p.fragment)}
    ${toggle('ECH', 'ech', t('Encrypted Client Hello (auto)'), p.ech)}
    ${toggle(t('Share over LAN'), 'lanShare', t('Let other devices on your network use this tunnel'), p.lanShare)}

    <h3 class="view__subtitle">${t('Connection safety')}</h3>
    ${toggle(t('Kill switch'), 'killSwitch', t('Block browser traffic if the tunnel drops'), p.killSwitch)}
    ${toggle(t('IPv6 leak protection'), 'ipv6Protection', t('Keep the IPv6 default route protected or block it safely'), p.ipv6Protection)}
    ${dropdown(t('Automatic reconnect attempts'), 'reconnectAttempts', RECONNECT_ATTEMPTS.map((v) => [String(v), `${v}`]), String(p.reconnectAttempts ?? 3))}


    ${dropdown(t('Split tunneling'), 'splitMode', SPLIT_MODES, p.splitMode)}
    <section class="field" id="split-apps" ${p.splitMode === 'OFF' ? 'hidden' : ''}>
      <span class="field__label">${t('Applications')}</span>
      <textarea class="input input--area ltr" dir="ltr" data-key="splitApps"
        placeholder="chrome.exe&#10;telegram.exe">${escAttr((p.splitApps || []).join('\n'))}</textarea>
      <span class="field__hint">${t('One executable name per line.')}</span>
    </section>

    <h3 class="view__subtitle">${t('Zero Trust')}</h3>
    <section class="field" id="caps-note" hidden>
      <span class="field__hint" id="caps-note-text"></span>
    </section>
    <div id="v15-zt">
    ${textField(t('Team name'), 'team', p.team, 'your-team', { hint: t('Connect as a managed device of a Cloudflare Zero Trust organization. Leave empty for normal WARP.') })}
    <div id="zt-extra" ${(p.team || '').trim() ? '' : 'hidden'}>
      ${segmented(t('Sign-in method'), 'accessMode', ACCESS_MODES, p.accessMode)}
      <div id="zt-fields">
        ${p.accessMode === 'EMAIL' ? textField(t('Access email'), 'accessEmail', p.accessEmail, 'user@example.com', { hint: t('A one-time code is sent to this mailbox on connect.') }) : ''}
        ${p.accessMode === 'SERVICE_TOKEN' ? textField('Access ID', 'accessId', p.accessId, 'xxxxxxxx.access', {}) : ''}
        ${p.accessMode === 'SERVICE_TOKEN' ? textField('Access Secret', 'accessSecret', '', '••••••', { secret: true, hint: t('Stored in memory only — never written to disk.') }) : ''}
        ${p.accessMode === 'TOKEN' ? textField('Access Token (JWT)', 'accessToken', '', '••••••', { secret: true, hint: t('Stored in memory only — never written to disk.') }) : ''}
      </div>
      ${toggle(t('Gateway proxy'), 'gateway', t('Route HTTP/HTTPS through your organization\'s Gateway (adds a hop and logs browsing)'), p.gateway)}
    </div>
    </div>

    <div id="v15-routing">
    <h3 class="view__subtitle">${t('Routing rules')}</h3>
    ${listArea(t('Blocked destinations'), 'routeBlock', p.routeBlock, 'ads.example.com&#10;203.0.113.0/24', t('One rule per line — domain, IP or CIDR. These connections are refused.'))}
    ${listArea(t('Direct destinations'), 'routeDirect', p.routeDirect, 'bank-domain.ir&#10;192.168.0.0/16', t('One rule per line. These bypass the tunnel — for banking apps, LAN services and domestic sites.'))}
    <div id="v17-sniff">
    ${toggle(t('Match domain rules by real host name'), 'routeSniff', t('Reads the name from the first bytes (TLS SNI or HTTP Host), so domain rules keep working even though Windows hands the tunnel an IP address'), p.routeSniff !== false)}
    </div>
    </div>

    <div id="v15-dns">
    <h3 class="view__subtitle">DNS</h3>
    ${listArea(t('In-tunnel DNS servers'), 'dns', p.dns, '1.1.1.1&#10;9.9.9.9', t('Resolvers used inside the tunnel. Empty = engine defaults.'))}
    </div>

    <div id="v17-upstream">
    <h3 class="view__subtitle">${t('Upstream proxy')}</h3>
    ${textField(t('Proxy address'), 'upstream', p.upstream, 'socks5://127.0.0.1:1080', { hint: t('Aether dials out through this proxy — use it to chain behind another VPN or proxy already running on this PC. Empty = direct.') })}
    <section class="field">
      <span class="field__hint" id="upstream-note"></span>
    </section>
    </div>

    <div id="v17-identity">
    <h3 class="view__subtitle">${t('Account identity')}</h3>
    ${toggle(t('Replace a refused identity'), 'reprovision', t('If Cloudflare stops accepting the saved device, register a fresh one instead of handshaking a tunnel that carries no traffic'), p.reprovision !== false)}
    </div>

    <section class="field field--row">
      <div>
        <span class="field__label">${t('Reset to defaults')}</span>
        <span class="field__hint">${t('Restores every setting above to its factory value')}</span>
      </div>
      <button type="button" class="btn btn--danger" id="reset-defaults">${t('Reset')}</button>
    </section>
  `

  // --- سیم‌کشی — هر تغییر فوراً ذخیره می‌شود (مثل DataStore در اندروید)
  root.querySelectorAll('.seg__item').forEach((b) => {
    b.addEventListener('click', async () => {
      const key = b.closest('.seg').dataset.key
      // زبان برنامه عضو profile نیست؛ جدا ذخیره و کل پوسته دوباره رندر می‌شود.
      if (key === '__lang') {
        setLang(b.dataset.value)
        rerender()
        return
      }
      await saveProfile({ [key]: b.dataset.value })
      b.closest('.seg').querySelectorAll('.seg__item').forEach((x) => {
        x.classList.toggle('is-active', x === b)
        x.setAttribute('aria-checked', String(x === b))
      })
      // v10: تغییر روش ورود Zero Trust فیلدهای متفاوتی می‌خواهد — بازرندر.
      if (key === 'accessMode') {
        swapHost(root, renderAdvanced)
      }
    })
  })

  // WCAG 4.1.2: پیمایش با کلیدهای جهت‌دار بین رادیوهای یک گروه (مثل رادیوی واقعی).
  root.querySelectorAll('.seg').forEach((group) => {
    group.addEventListener('keydown', (e) => {
      if (e.key !== 'ArrowLeft' && e.key !== 'ArrowRight') return
      e.preventDefault()
      const items = [...group.querySelectorAll('.seg__item')]
      const idx = items.indexOf(document.activeElement)
      if (idx === -1) return
      const next = items[(idx + (e.key === 'ArrowRight' ? 1 : -1) + items.length) % items.length]
      next.focus()
      next.click()
    })
  })

  root.querySelectorAll('.select').forEach((s) => {
    s.addEventListener('change', async () => {
      const key = s.dataset.key
      const raw = s.value
      const value = ['mtu', 'keepalive', 'reconnectAttempts'].includes(key) ? Number(raw) : raw
      await saveProfile({ [key]: value })
      if (key === 'endpointMode' || key === 'splitMode') {
        swapHost(root, renderAdvanced)
      }
    })
  })

  root.querySelectorAll('.switch').forEach((tg) => {
    wireSwitch(tg, (on) => saveProfile({ [tg.dataset.key]: on }))
  })

  // v11 — پیام زندهٔ پروکسی بالادست: مقدار نامعتبر را هسته بی‌صدا دور
  // می‌ریزد، پس همین‌جا به کاربر گفته می‌شود. پروکسی HTTP هم UDP حمل
  // نمی‌کند، پس تنها MASQUE روی HTTP/2 از آن رد می‌شود.
  const upstreamNote = () => {
    const el = root.querySelector('#upstream-note')
    const input = root.querySelector('[data-key="upstream"]')
    if (!el || !input) return
    const raw = input.value.trim()
    if (!raw) {
      el.textContent = ''
      return
    }
    const parsed = parseUpstream(raw)
    if (!parsed) {
      el.textContent = t('That is not a proxy address Aether can use. Expected socks5://host:port or http://host:port — the port is required.')
      return
    }
    el.textContent =
      parsed.kind === 'http'
        ? t('An HTTP proxy cannot carry UDP, so MASQUE is switched to HTTP/2 automatically and WireGuard / WARP×2 will not pass through it. Use a SOCKS5 proxy for those.')
        : t('SOCKS5 with UDP support carries every protocol: MASQUE, WireGuard and WARP×2.')
  }
  upstreamNote()

  root.querySelectorAll('.input').forEach((i) => {
    i.addEventListener('input', () => {
      if (i.dataset.key === 'upstream') upstreamNote()
    })
    i.addEventListener('change', async () => {
      const key = i.dataset.key
      const value = LIST_KEYS.includes(key)
        ? i.value.split('\n').map((x) => x.trim()).filter(Boolean)
        : i.value.trim()
      await saveProfile({ [key]: value })
      // v10: پاک/پر شدن نام تیم، بخش Zero Trust را نشان/پنهان می‌کند.
      if (key === 'team') {
        swapHost(root, renderAdvanced)
      }
      // سخت‌سازی امنیتی: مقدار محرمانه بعد از ذخیره از DOM پاک می‌شود تا
      // در اسکرین‌شات/بازرسی DOM نماند (ذخیره فقط در حافظهٔ بک‌اند است).
      if (key === 'accessSecret' || key === 'accessToken') {
        i.value = ''
        i.placeholder = '•••••• (saved)'
      }
    })
  })

  // بازنشانی به تنظیمات کارخانه — دستور reset_profile سمت Rust پروفایل
  // پیش‌فرض را ذخیره و همان را برمی‌گرداند؛ زبان کاربر دست نمی‌خورد.
  root.querySelector('#reset-defaults').addEventListener('click', async () => {
    const fresh = await invoke('reset_profile')
    app.profile = fresh
    rerender()
  })

  // v10: قابلیت‌سنجی هسته — اگر هستهٔ همراه قدیمی‌تر از 1.5.0 باشد (مثلاً
  // وقتی کاربر نسخهٔ هسته را در پایپ‌لاین پین کرده)، این بخش‌ها غیرفعال و
  // با توضیح نشان داده می‌شوند تا کاربر تنظیمی را پر نکند که بی‌اثر است.
  // خطای این فراخوانی هرگز صفحه را نمی‌شکند.
  invoke('core_caps')
    .then((caps) => {
      const gate = (id, enabled) => {
        const el = root.querySelector(id)
        if (!el || enabled) return
        el.querySelectorAll('input, textarea, button, select').forEach((c) => {
          c.disabled = true
        })
        el.style.opacity = '0.45'
      }
      gate('#v15-zt', caps.zeroTrust)
      gate('#v15-routing', caps.routing)
      gate('#v15-dns', caps.customDns)
      // v11: قابلیت‌های هستهٔ 1.7.0 جداگانه گیت می‌شوند.
      gate('#v17-upstream', caps.upstream)
      gate('#v17-sniff', caps.routeSniff)
      gate('#v17-identity', caps.routeSniff)
      const missing15 = !caps.zeroTrust || !caps.routing || !caps.customDns
      const missing17 = !caps.upstream || !caps.routeSniff
      if (missing15 || missing17) {
        const note = root.querySelector('#caps-note')
        const text = root.querySelector('#caps-note-text')
        if (note && text) {
          text.textContent = missing15
            ? t('These features need engine core 1.5.0 or newer. The bundled core is older, so they are disabled.')
            : t('The upstream proxy, host-name routing and identity replacement need engine core 1.7.0 or newer. The bundled core is older, so they are disabled.')
          note.hidden = false
        }
      }
    })
    .catch(() => {})

  return root
}

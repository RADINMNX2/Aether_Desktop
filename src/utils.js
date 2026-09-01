// ابزارهای مشترک بین ویوها — امنیت HTML، کلید خروج escape، سیم‌کشی سوییچ.

// Escape متن داخل یک المان — از & < > محافظت می‌کند.
export function esc(s) {
  return String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
}

// Escape متن داخل یک attribute یا textarea — به‌علاوه از " و ' محافظت می‌کند.
export function escAttr(s) {
  return String(s)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
}

// سیم‌کشی یک سوییچ پیمانه‌ای: کلیک → جابه‌جایی کلاس + aria-checked → ذخیره.
export function wireSwitch(el, save) {
  el.addEventListener('click', async () => {
    const on = !el.classList.contains('is-on')
    el.classList.toggle('is-on', on)
    el.setAttribute('aria-checked', String(on))
    try {
      await save(on)
    } catch {
      // خطا در ذخیره، state ظاهری را برنمی‌گرداند — کلاس همان است.
    }
  })
}

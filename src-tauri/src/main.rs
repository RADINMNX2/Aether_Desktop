#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
//! Aether Desktop — entry point.
//!
//! Module mapping from Android to Windows:
//!   MainActivity / AetherApp -> main.rs
//!   AetherController         -> state.rs
//!   AetherVpnService         -> tun.rs + sysproxy.rs + share.rs + leakguard.rs
//!   AetherProcess            -> engine.rs
//!   Profile                  -> profile.rs
//!   ProfileStore             -> store.rs
//!   NetProbe / PortProbe     -> probe.rs
//!   SmartAuto                -> smart_auto.rs
//!   Diagnostics              -> diagnostics.rs
//!   DiagnosticsLog           -> log.rs

mod diagnostics;
mod engine;
mod leakguard;
mod log;
mod probe;
mod profile;
mod share;
mod smart_auto;
mod state;
mod store;
mod sysproxy;
mod tun;

use profile::ConnectionProfile;
use state::{AetherController, Snapshot};
use parking_lot::Mutex;
use tauri::{Emitter, Manager, State};

/// The in-app updater was removed in 1.2.2; only a read-only link remains.
pub const RELEASES_URL: &str = "https://github.com/QW-AI-Code/Aether_Desktop/releases";

pub struct AppState {
    controller: Mutex<AetherController>,
}

#[tauri::command]
fn get_snapshot(app: State<'_, AppState>) -> Snapshot {
    app.controller.lock().snapshot()
}

#[tauri::command]
fn get_profile(app: State<'_, AppState>) -> ConnectionProfile {
    app.controller.lock().profile()
}

#[tauri::command]
fn set_profile(app: State<'_, AppState>, profile: ConnectionProfile) -> Result<(), String> {
    app.controller
        .lock()
        .set_profile(profile)
        .map_err(|e| e.to_string())
}

/// v8: "Reset to defaults" - persists the factory profile and returns it
/// so the UI can re-render immediately. UI language is left untouched.
///
/// v10: goes through the controller's dedicated reset path so the in-memory
/// Zero Trust secrets are cleared too (a plain set_profile treats an empty
/// secret as "keep the current one" and would silently retain the token).
#[tauri::command]
fn reset_profile(app: State<'_, AppState>) -> Result<ConnectionProfile, String> {
    app.controller
        .lock()
        .reset_profile()
        .map_err(|e| e.to_string())
}

/// Equivalent of `onToggleConnection` in HomeScreen.kt.
#[tauri::command]
fn toggle_connection(app: State<'_, AppState>) -> Result<(), String> {
    app.controller.lock().toggle().map_err(|e| e.to_string())
}

#[tauri::command]
fn read_logs(limit: Option<usize>) -> Vec<String> {
    log::DiagnosticsLog::tail(limit.unwrap_or(800))
}

/// Equivalent of the Android "Copy logs" button — full text for the clipboard.
#[tauri::command]
fn export_logs() -> String {
    log::DiagnosticsLog::export_text()
}

#[tauri::command]
fn clear_logs() {
    log::DiagnosticsLog::clear();
}

/// Live state of the 4 checks — equivalent of the check StateFlows in Diagnostics.kt.
#[tauri::command]
fn get_checks() -> Vec<log::ComponentCheck> {
    log::DiagnosticsLog::checks()
}

/// Equivalent of the Android "Run test" button — runs in a background thread
/// so the UI never freezes; live results arrive via get_checks/read_logs.
#[tauri::command]
fn run_self_test() {
    match std::thread::Builder::new()
        .name("aether-manual-test".into())
        .spawn(|| {
            diagnostics::self_test(20_000);
        }) {
        Ok(_) => {}
        Err(e) => log::DiagnosticsLog::w("self-test", &format!("thread spawn failed: {e}")),
    }
}

/// Environment report (binary/driver/permissions) — complements the mobile-style self-test.
#[tauri::command]
fn run_diagnostics(app: State<'_, AppState>) -> diagnostics::Report {
    let profile = app.controller.lock().profile();
    diagnostics::run(&profile)
}

/// v1.2.0 — دکمهٔ «آزمایش نشتی WebRTC» در پنل عیب‌یابی.
///
/// همان کاری که یک صفحهٔ وب با WebRTC می‌کند: یک درخواست STUN روی UDP خام.
/// اگر آی‌پی برگشته با آی‌پی خروجی تونل یکی نباشد، یعنی نشتی واقعی است.
#[tauri::command]
fn webrtc_leak_test(app: State<'_, AppState>) -> diagnostics::LeakReport {
    let exit = app
        .controller
        .lock()
        .snapshot()
        .ip_info
        .and_then(|i| if i.via_tunnel { Some(i.ip) } else { None });
    diagnostics::webrtc_leak_check(exit.as_deref())
}

#[tauri::command]
fn about_info() -> serde_json::Value {
    serde_json::json!({
        "appVersion": env!("CARGO_PKG_VERSION"),
        "coreVersion": core_version(),
        "arch": std::env::consts::ARCH,
        "releasesUrl": RELEASES_URL,
    })
}

/// v10: قابلیت‌های هستهٔ همراه — UI با این تصمیم می‌گیرد که بخش‌های
/// Zero Trust / مسیریابی / DNS را فعال نشان بدهد یا با توضیح غیرفعال.
/// بدون این، کاربرِ هستهٔ پین‌شدهٔ قدیمی تنطیمی را پر می‌کرد که بی‌اثر بود.
/// v11: پروکسی بالادست و تشخیص نام میزبان (هستهٔ 1.7.0) هم همین‌جا گزارش
/// می‌شوند تا پنل پیشرفته آن‌ها را روی هستهٔ قدیمی‌تر خاکستری کند.
#[tauri::command]
fn core_caps() -> serde_json::Value {
    let exe = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("engine").join("aether.exe")))
        .unwrap_or_default();
    let caps = engine::engine_caps(&exe);
    serde_json::json!({
        "zeroTrust": caps.zero_trust,
        "routing": caps.routing,
        "customDns": caps.custom_dns,
        "upstream": caps.upstream,
        "routeSniff": caps.route_sniff,
    })
}

fn core_version() -> String {
    // Placed next to aether.exe (equivalent of assets/CORE_VERSION on Android).
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("engine").join("CORE_VERSION")))
        .and_then(|p| std::fs::read_to_string(p).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            // Equivalent of launchMode=singleTask in the Android manifest.
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
        }))
        .setup(|app| {
            let data_dir = app.path().app_local_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            log::DiagnosticsLog::init(&data_dir);

            let controller = AetherController::new(&data_dir);
            app.manage(AppState { controller: Mutex::new(controller) });

            // Equivalent of collecting StateFlow in Compose: a snapshot goes to
            // the UI every 200ms (the same ~5x/second cap as Android).
            let handle = app.handle().clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_millis(200));
                let snap = {
                    let st: State<'_, AppState> = handle.state();
                    let mut c = st.controller.lock();
                    c.tick();
                    c.snapshot()
                };
                let _ = handle.emit("aether://state", snap);
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_snapshot,
            get_profile,
            set_profile,
            reset_profile,
            toggle_connection,
            read_logs,
            export_logs,
            clear_logs,
            get_checks,
            run_self_test,
            run_diagnostics,
            webrtc_leak_test,
            about_info,
            core_caps,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Aether");
}
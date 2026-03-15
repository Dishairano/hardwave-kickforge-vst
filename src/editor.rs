//! WebView-based editor for the KickForge VST plugin.
//!
//! Embeds a wry WebView that loads `kickforge.hardwavestudios.com/vst/kickforge`.
//!
//! Communication:
//! - **Plugin -> WebView**: param state pushed via `evaluate_script()` (Linux/macOS)
//!   or via a local TCP HTTP server polled by JS (Windows).
//! - **WebView -> Plugin**: `window.__hardwave.setParam(key, value)` calls
//!   `window.ipc.postMessage()` -> GuiContext sets the nih-plug param.

use crossbeam_channel::Receiver;
use nih_plug::editor::Editor;
use nih_plug::prelude::{GuiContext, ParentWindowHandle, Param};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::auth;
use crate::params::{KickForgeParams, NUM_FX_SLOTS};
use crate::protocol::{FxSlotState, KickForgePacket};

const KICKFORGE_URL: &str = "http://46.225.219.184:8080/vst/kickforge";
const EDITOR_WIDTH: u32 = 1100;
const EDITOR_HEIGHT: u32 = 700;

/// Wraps a raw window handle value (usize) so wry can use it via rwh 0.6.
struct RwhWrapper(usize);

unsafe impl Send for RwhWrapper {}
unsafe impl Sync for RwhWrapper {}

impl raw_window_handle::HasWindowHandle for RwhWrapper {
    fn window_handle(&self) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        use raw_window_handle::RawWindowHandle;

        #[cfg(target_os = "linux")]
        let raw = {
            let h = raw_window_handle::XlibWindowHandle::new(self.0 as _);
            RawWindowHandle::Xlib(h)
        };

        #[cfg(target_os = "macos")]
        let raw = {
            let ns_view = std::ptr::NonNull::new(self.0 as *mut _).expect("null NSView");
            let h = raw_window_handle::AppKitWindowHandle::new(ns_view);
            RawWindowHandle::AppKit(h)
        };

        #[cfg(target_os = "windows")]
        let raw = {
            let hwnd = std::num::NonZeroIsize::new(self.0 as isize).expect("null HWND");
            let h = raw_window_handle::Win32WindowHandle::new(hwnd);
            RawWindowHandle::Win32(h)
        };

        Ok(unsafe { raw_window_handle::WindowHandle::borrow_raw(raw) })
    }
}

impl raw_window_handle::HasDisplayHandle for RwhWrapper {
    fn display_handle(&self) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        use raw_window_handle::RawDisplayHandle;

        #[cfg(target_os = "linux")]
        let raw = RawDisplayHandle::Xlib(raw_window_handle::XlibDisplayHandle::new(None, 0));

        #[cfg(target_os = "macos")]
        let raw = RawDisplayHandle::AppKit(raw_window_handle::AppKitDisplayHandle::new());

        #[cfg(target_os = "windows")]
        let raw = RawDisplayHandle::Windows(raw_window_handle::WindowsDisplayHandle::new());

        Ok(unsafe { raw_window_handle::DisplayHandle::borrow_raw(raw) })
    }
}

/// Build a map of camelCase param keys to ParamPtr for GuiContext param setting.
fn build_param_map(params: &KickForgeParams) -> HashMap<String, nih_plug::prelude::ParamPtr> {
    let mut map = HashMap::new();

    // Click layer
    map.insert("clickEnabled".into(), params.click_enabled.as_ptr());
    map.insert("clickType".into(), params.click_type.as_ptr());
    map.insert("clickVolume".into(), params.click_volume.as_ptr());
    map.insert("clickPitch".into(), params.click_pitch.as_ptr());
    map.insert("clickDecay".into(), params.click_decay.as_ptr());
    map.insert("clickFilterFreq".into(), params.click_filter_freq.as_ptr());

    // Body layer
    map.insert("bodyPitchStart".into(), params.body_pitch_start.as_ptr());
    map.insert("bodyPitchEnd".into(), params.body_pitch_end.as_ptr());
    map.insert("bodyPitchDecay".into(), params.body_pitch_decay.as_ptr());
    map.insert("bodyPitchCurve".into(), params.body_pitch_curve.as_ptr());
    map.insert("bodyWaveform".into(), params.body_waveform.as_ptr());
    map.insert("bodyDrive".into(), params.body_drive.as_ptr());
    map.insert("bodyDistortionType".into(), params.body_distortion_type.as_ptr());
    map.insert("bodyDecay".into(), params.body_decay.as_ptr());
    map.insert("bodyVolume".into(), params.body_volume.as_ptr());
    map.insert("bodyTone".into(), params.body_tone.as_ptr());
    map.insert("bodyResonance".into(), params.body_resonance.as_ptr());

    // Sub layer
    map.insert("subEnabled".into(), params.sub_enabled.as_ptr());
    map.insert("subFrequency".into(), params.sub_frequency.as_ptr());
    map.insert("subVolume".into(), params.sub_volume.as_ptr());
    map.insert("subDecay".into(), params.sub_decay.as_ptr());

    // Noise layer
    map.insert("noiseEnabled".into(), params.noise_enabled.as_ptr());
    map.insert("noiseType".into(), params.noise_type.as_ptr());
    map.insert("noiseVolume".into(), params.noise_volume.as_ptr());
    map.insert("noiseDecay".into(), params.noise_decay.as_ptr());
    map.insert("noiseFilterFreq".into(), params.noise_filter_freq.as_ptr());

    // Layer solo
    map.insert("clickSolo".into(), params.click_solo.as_ptr());
    map.insert("bodySolo".into(), params.body_solo.as_ptr());
    map.insert("subSolo".into(), params.sub_solo.as_ptr());
    map.insert("noiseSolo".into(), params.noise_solo.as_ptr());

    // Velocity mapping
    map.insert("velToDecay".into(), params.vel_to_decay.as_ptr());
    map.insert("velToPitch".into(), params.vel_to_pitch.as_ptr());
    map.insert("velToDrive".into(), params.vel_to_drive.as_ptr());
    map.insert("velToClick".into(), params.vel_to_click.as_ptr());

    // Modular FX Rack (8 slots)
    for (i, slot) in params.fx_slots.iter().enumerate() {
        map.insert(format!("fxSlot{}Type", i), slot.slot_type.as_ptr());
        map.insert(format!("fxSlot{}On", i), slot.enabled.as_ptr());
        map.insert(format!("fxSlot{}P1", i), slot.p1.as_ptr());
        map.insert(format!("fxSlot{}P2", i), slot.p2.as_ptr());
        map.insert(format!("fxSlot{}P3", i), slot.p3.as_ptr());
        map.insert(format!("fxSlot{}P4", i), slot.p4.as_ptr());
        map.insert(format!("fxSlot{}P5", i), slot.p5.as_ptr());
        map.insert(format!("fxSlot{}P6", i), slot.p6.as_ptr());
    }

    // Master
    map.insert("masterVolume".into(), params.master_volume.as_ptr());
    map.insert("masterTuning".into(), params.master_tuning.as_ptr());
    map.insert("masterOctave".into(), params.master_octave.as_ptr());
    map.insert("masterLimiter".into(), params.master_limiter.as_ptr());
    map.insert("masterLow".into(), params.master_low.as_ptr());
    map.insert("masterMid".into(), params.master_mid.as_ptr());
    map.insert("masterHigh".into(), params.master_high.as_ptr());

    map
}

pub struct KickForgeEditor {
    params: Arc<KickForgeParams>,
    packet_rx: Arc<Mutex<Receiver<KickForgePacket>>>,
    auth_token: Option<String>,
    scale_factor: Mutex<f32>,
}

impl KickForgeEditor {
    pub fn new(
        params: Arc<KickForgeParams>,
        packet_rx: Arc<Mutex<Receiver<KickForgePacket>>>,
        auth_token: Option<String>,
    ) -> Self {
        Self {
            params,
            packet_rx,
            auth_token,
            scale_factor: Mutex::new(1.0),
        }
    }

    fn scaled_size(&self) -> (u32, u32) {
        let f = *self.scale_factor.lock();
        ((EDITOR_WIDTH as f32 * f) as u32, (EDITOR_HEIGHT as f32 * f) as u32)
    }
}

impl Editor for KickForgeEditor {
    fn spawn(
        &self,
        parent: ParentWindowHandle,
        context: Arc<dyn GuiContext>,
    ) -> Box<dyn std::any::Any + Send> {
        let packet_rx = Arc::clone(&self.packet_rx);
        let (width, height) = self.scaled_size();

        // Build the URL with token and version
        let version = env!("CARGO_PKG_VERSION");
        let url = match &self.auth_token {
            Some(t) => format!("{}?token={}&v={}", KICKFORGE_URL, t, version),
            None => format!("{}?v={}", KICKFORGE_URL, version),
        };

        // Build param map for IPC handler
        let param_map = Arc::new(build_param_map(&self.params));

        // Build init script with current param state snapshot
        let init_js = ipc_init_script(&self.params);

        // Extract raw handle value BEFORE spawning threads (ParentWindowHandle isn't Send)
        let raw_handle = extract_raw_handle(&parent);

        #[cfg(target_os = "windows")]
        {
            spawn_windows(raw_handle, url, width, height, packet_rx, context, param_map, init_js)
        }

        #[cfg(not(target_os = "windows"))]
        {
            spawn_unix(raw_handle, url, width, height, packet_rx, context, param_map, init_js)
        }
    }

    fn size(&self) -> (u32, u32) {
        self.scaled_size()
    }

    fn set_scale_factor(&self, factor: f32) -> bool {
        *self.scale_factor.lock() = factor;
        true
    }

    fn param_value_changed(&self, _id: &str, _normalized_value: f32) {}
    fn param_modulation_changed(&self, _id: &str, _modulation_offset: f32) {}
    fn param_values_changed(&self) {}
}

/// Extract the raw handle value from ParentWindowHandle so we can send it across threads.
fn extract_raw_handle(parent: &ParentWindowHandle) -> usize {
    match *parent {
        #[cfg(target_os = "linux")]
        ParentWindowHandle::X11Window(id) => id as usize,
        #[cfg(target_os = "macos")]
        ParentWindowHandle::AppKitNsView(ptr) => ptr as usize,
        #[cfg(target_os = "windows")]
        ParentWindowHandle::Win32Hwnd(h) => h as usize,
        _ => 0, // Fallback — editor will fail gracefully
    }
}

/// Build a KickForgePacket from the current DAW-persisted param values.
/// Called at editor open time so the webview starts with the correct state.
fn snapshot_params(params: &KickForgeParams) -> KickForgePacket {
    KickForgePacket {
        click_enabled: params.click_enabled.value(),
        click_type: params.click_type.value() as i32,
        click_volume: params.click_volume.value(),
        click_pitch: params.click_pitch.value(),
        click_decay: params.click_decay.value(),
        click_filter_freq: params.click_filter_freq.value(),
        body_pitch_start: params.body_pitch_start.value(),
        body_pitch_end: params.body_pitch_end.value(),
        body_pitch_decay: params.body_pitch_decay.value(),
        body_pitch_curve: params.body_pitch_curve.value() as i32,
        body_waveform: params.body_waveform.value() as i32,
        body_drive: params.body_drive.value(),
        body_distortion_type: params.body_distortion_type.value() as i32,
        body_decay: params.body_decay.value(),
        body_volume: params.body_volume.value(),
        body_tone: params.body_tone.value(),
        body_resonance: params.body_resonance.value(),
        sub_enabled: params.sub_enabled.value(),
        sub_frequency: params.sub_frequency.value(),
        sub_volume: params.sub_volume.value(),
        sub_decay: params.sub_decay.value(),
        noise_enabled: params.noise_enabled.value(),
        noise_type: params.noise_type.value() as i32,
        noise_volume: params.noise_volume.value(),
        noise_decay: params.noise_decay.value(),
        noise_filter_freq: params.noise_filter_freq.value(),
        click_solo: params.click_solo.value(),
        body_solo: params.body_solo.value(),
        sub_solo: params.sub_solo.value(),
        noise_solo: params.noise_solo.value(),
        vel_to_decay: params.vel_to_decay.value(),
        vel_to_pitch: params.vel_to_pitch.value(),
        vel_to_drive: params.vel_to_drive.value(),
        vel_to_click: params.vel_to_click.value(),
        fx_slots: (0..NUM_FX_SLOTS)
            .map(|i| FxSlotState {
                slot_type: params.fx_slots[i].slot_type.value(),
                enabled: params.fx_slots[i].enabled.value(),
                p1: params.fx_slots[i].p1.value(),
                p2: params.fx_slots[i].p2.value(),
                p3: params.fx_slots[i].p3.value(),
                p4: params.fx_slots[i].p4.value(),
                p5: params.fx_slots[i].p5.value(),
                p6: params.fx_slots[i].p6.value(),
            })
            .collect(),
        comp_gain_reduction: 0.0,
        waveform_buffer: Vec::new(),
        master_volume: params.master_volume.value(),
        master_tuning: params.master_tuning.value(),
        master_octave: params.master_octave.value() as i32,
        master_limiter: params.master_limiter.value(),
        master_low: params.master_low.value(),
        master_mid: params.master_mid.value(),
        master_high: params.master_high.value(),
    }
}

/// IPC init script injected into the WebView.
fn ipc_init_script(params: &KickForgeParams) -> String {
    let snapshot = snapshot_params(params);
    let initial_json = serde_json::to_string(&snapshot).unwrap_or_else(|_| "null".into());

    format!(
        r#"
    window.__HARDWAVE_VST = true;
    window.__HARDWAVE_VST_VERSION = '{version}';
    window.__hardwave = {{
        setParam: function(key, value) {{
            var v = value;
            if (typeof v === 'boolean') v = v ? 1 : 0;
            window.ipc.postMessage('setParam:' + key + ':' + v);
        }},
        saveToken: function(token) {{
            window.ipc.postMessage('saveToken:' + token);
        }}
    }};

    /* Push DAW-persisted state as soon as the page defines __onKickForgePacket */
    (function() {{
        var _init = {initial_json};
        function pushInit() {{
            if (window.__onKickForgePacket) {{
                window.__onKickForgePacket(_init);
            }} else {{
                setTimeout(pushInit, 50);
            }}
        }}
        if (document.readyState === 'complete') {{
            pushInit();
        }} else {{
            window.addEventListener('load', pushInit);
        }}
    }})();
    "#,
        version = env!("CARGO_PKG_VERSION"),
        initial_json = initial_json,
    )
}

/// Handle IPC messages from the WebView. Uses GuiContext to properly set nih-plug params.
fn handle_ipc(
    context: &Arc<dyn GuiContext>,
    param_map: &Arc<HashMap<String, nih_plug::prelude::ParamPtr>>,
    message: &str,
) {
    if let Some(rest) = message.strip_prefix("setParam:") {
        if let Some((key, val_str)) = rest.split_once(':') {
            if let Ok(value) = val_str.parse::<f64>() {
                if let Some(&param_ptr) = param_map.get(key) {
                    // SAFETY: param_ptr is valid for the lifetime of the plugin.
                    // We hold Arc<KickForgeParams> which keeps the params alive.
                    unsafe {
                        let normalized = param_ptr.preview_normalized(value as f32);
                        context.raw_begin_set_parameter(param_ptr);
                        context.raw_set_parameter_normalized(param_ptr, normalized);
                        context.raw_end_set_parameter(param_ptr);
                    }
                }
            }
        }
    } else if let Some(token) = message.strip_prefix("saveToken:") {
        auth::save_token(token.trim());
    }
}

// ─── Windows: TCP packet server approach ────────────────────────────────────

#[cfg(target_os = "windows")]
fn webview2_data_dir() -> std::path::PathBuf {
    dirs::data_local_dir()
        .map(|d| d.join("Hardwave").join("KickForge").join("WebView2"))
        .unwrap_or_else(|| std::path::PathBuf::from("C:\\HardwaveWebView2Data"))
}

#[cfg(target_os = "windows")]
fn spawn_windows(
    raw_handle: usize,
    url: String,
    width: u32,
    height: u32,
    packet_rx: Arc<Mutex<Receiver<KickForgePacket>>>,
    context: Arc<dyn GuiContext>,
    param_map: Arc<HashMap<String, nih_plug::prelude::ParamPtr>>,
    base_init_js: String,
) -> Box<dyn std::any::Any + Send> {
    use std::io::{Read, Write};
    use std::net::TcpListener;

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = Arc::clone(&running);

    let (port_tx, port_rx) = crossbeam_channel::bounded::<u16>(1);
    let packet_rx_server = Arc::clone(&packet_rx);
    let running_server = Arc::clone(&running);

    let server_thread = std::thread::spawn(move || {
        let listener = match TcpListener::bind("127.0.0.1:0") {
            Ok(l) => l,
            Err(_) => return,
        };
        let port = match listener.local_addr() {
            Ok(addr) => addr.port(),
            Err(_) => return,
        };
        let _ = port_tx.send(port);
        listener.set_nonblocking(true).ok();

        let mut latest_json = String::from("null");

        while running_server.load(Ordering::Relaxed) {
            if let Some(rx) = packet_rx_server.try_lock() {
                while let Ok(pkt) = rx.try_recv() {
                    if let Ok(json) = serde_json::to_string(&pkt) {
                        latest_json = json;
                    }
                }
            }

            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 1024];
                let _ = stream.read(&mut buf);
                let response = format!(
                    "HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\nContent-Type: application/json\r\nCache-Control: no-store\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    latest_json.len(),
                    latest_json
                );
                let _ = stream.write_all(response.as_bytes());
            }

            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    });

    let port = match port_rx.recv() {
        Ok(p) => p,
        Err(_) => {
            return Box::new(EditorHandle {
                running: running_clone,
                _webview: None,
                _web_context: None,
                _server_thread: Some(server_thread),
                _editor_thread: None,
            });
        }
    };

    let wrapper = RwhWrapper(raw_handle);

    let poll_script = format!(
        r#"
        (function() {{
            const POLL_URL = 'http://127.0.0.1:{}/';
            function poll() {{
                fetch(POLL_URL).then(r => r.json()).then(data => {{
                    if (window.__onKickForgePacket) window.__onKickForgePacket(data);
                }}).catch(() => {{}});
                requestAnimationFrame(poll);
            }}
            poll();
        }})();
        "#,
        port
    );

    let init_js = format!("{}\n{}", base_init_js, poll_script);
    let ctx = Arc::clone(&context);
    let pmap = Arc::clone(&param_map);

    // Create a writable WebView2 data directory to avoid E_ACCESSDENIED
    // when the DAW is installed in Program Files (read-only).
    let data_dir = webview2_data_dir();
    let _ = std::fs::create_dir_all(&data_dir);
    let mut web_context = wry::WebContext::new(Some(data_dir));

    // with_web_context is a constructor (replaces ::new())
    use wry::WebViewBuilderExtWindows;
    let webview = match wry::WebViewBuilder::with_web_context(&mut web_context)
        .with_url(&url)
        .with_initialization_script(&init_js)
        .with_ipc_handler(move |msg| {
            handle_ipc(&ctx, &pmap, &msg.body());
        })
        .with_bounds(wry::Rect {
            position: wry::dpi::Position::Logical(wry::dpi::LogicalPosition::new(0.0, 0.0)),
            size: wry::dpi::Size::Logical(wry::dpi::LogicalSize::new(width as f64, height as f64)),
        })
        .with_transparent(false)
        .with_devtools(false)
        .with_background_color((10, 10, 11, 255))
        .with_additional_browser_args("--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection --allow-insecure-localhost")
        .build(&wrapper)
    {
        Ok(wv) => wv,
        Err(e) => {
            eprintln!("[KickForge] failed to create WebView: {}", e);
            return Box::new(EditorHandle {
                running: running_clone,
                _webview: None,
                _web_context: Some(web_context),
                _server_thread: Some(server_thread),
                _editor_thread: None,
            });
        }
    };

    Box::new(EditorHandle {
        running: running_clone,
        _webview: Some(webview),
        _web_context: Some(web_context),
        _server_thread: Some(server_thread),
        _editor_thread: None,
    })
}

// ─── Linux / macOS: evaluate_script approach ────────────────────────────────

#[cfg(not(target_os = "windows"))]
fn spawn_unix(
    raw_handle: usize,
    url: String,
    width: u32,
    height: u32,
    packet_rx: Arc<Mutex<Receiver<KickForgePacket>>>,
    context: Arc<dyn GuiContext>,
    param_map: Arc<HashMap<String, nih_plug::prelude::ParamPtr>>,
    init_js: String,
) -> Box<dyn std::any::Any + Send> {
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = Arc::clone(&running);

    let editor_thread = std::thread::spawn(move || {
        #[cfg(target_os = "linux")]
        {
            let _ = gtk::init();
        }

        let wrapper = RwhWrapper(raw_handle);
        let ctx = Arc::clone(&context);
        let pmap = Arc::clone(&param_map);

        let webview = match wry::WebViewBuilder::new()
            .with_url(&url)
            .with_initialization_script(&init_js)
            .with_ipc_handler(move |msg| {
                handle_ipc(&ctx, &pmap, &msg.body());
            })
            .with_bounds(wry::Rect {
                position: wry::dpi::Position::Logical(wry::dpi::LogicalPosition::new(0.0, 0.0)),
                size: wry::dpi::Size::Logical(wry::dpi::LogicalSize::new(width as f64, height as f64)),
            })
            .build_as_child(&wrapper)
        {
            Ok(wv) => wv,
            Err(e) => {
                eprintln!("[KickForge] failed to create WebView: {}", e);
                return;
            }
        };

        while running.load(Ordering::Relaxed) {
            if let Some(rx) = packet_rx.try_lock() {
                while let Ok(pkt) = rx.try_recv() {
                    if let Ok(json) = serde_json::to_string(&pkt) {
                        let js = format!(
                            "window.__onKickForgePacket && window.__onKickForgePacket({})",
                            json
                        );
                        let _ = webview.evaluate_script(&js);
                    }
                }
            }

            #[cfg(target_os = "linux")]
            {
                while gtk::events_pending() {
                    gtk::main_iteration_do(false);
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(16));
        }
    });

    Box::new(EditorHandle {
        running: running_clone,
        _webview: None,
        _web_context: None,
        _server_thread: None,
        _editor_thread: Some(editor_thread),
    })
}

struct EditorHandle {
    running: Arc<AtomicBool>,
    _webview: Option<wry::WebView>,
    _web_context: Option<wry::WebContext>,
    _server_thread: Option<std::thread::JoinHandle<()>>,
    _editor_thread: Option<std::thread::JoinHandle<()>>,
}

unsafe impl Send for EditorHandle {}

impl Drop for EditorHandle {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self._server_thread.take() {
            let _ = handle.join();
        }
        if let Some(handle) = self._editor_thread.take() {
            let _ = handle.join();
        }
    }
}

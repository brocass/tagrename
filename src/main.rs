//! Tag:Rename — a modern dark-mode music tag editor.
//!
//! A rebuilt tagger and renamer for Linux and other desktops; see
//! MECHANICS.md for where every behaviour comes from.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;

mod data;
mod mask;
mod native_picker;
mod places;
mod theme;
mod tools;
mod ui;
mod ui_dialogs;
mod ui_layout;
mod ui_tabs;
mod ui_tageditor;

/// State for the `--screenshot <file.png>` option.
pub struct ShotState {
    pub path: std::path::PathBuf,
    pub frame: u32,
    pub requested: bool,
    /// The slot the screenshot callback drops the image into.
    pub slot: Option<std::sync::Arc<egui::mutex::Mutex<Option<std::sync::Arc<egui::ColorImage>>>>>,
}

/// Set once the screenshot PNG has been written; the watchdog thread
/// (screenshot mode only) stops waiting when this is true.
pub static SHOT_SAVED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Where the screenshot-mode diagnostic log goes: always inside the
/// project's app/shots/ folder, resolved from the executable's own
/// location so it does not depend on the caller's working directory.
static SHOT_LOG: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();

/// Append one line to the screenshot-mode log (screenshot mode only).
pub fn shot_log(msg: &str) {
    let Some(path) = SHOT_LOG.get() else { return };
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut f| {
            use std::io::Write;
            writeln!(f, "{stamp} pid={} {msg}", std::process::id())
        });
}

fn init_shot_log() {
    let dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().and_then(|p| p.parent()).and_then(|p| p.parent()).map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let log = dir.join("shots").join("shotlog.txt");
    let _ = std::fs::create_dir_all(log.parent().unwrap_or(&dir));
    let _ = SHOT_LOG.set(log);
    let show = |k: &str| {
        std::env::var(k).map(|v| if v.is_empty() { format!("{k}=<empty>") } else { v }).unwrap_or_else(|_| format!("{k}=<unset>"))
    };
    shot_log(&format!(
        "start argv={:?} cwd={} {} {} {}",
        std::env::args().collect::<Vec<_>>(),
        std::env::current_dir().map(|p| p.display().to_string()).unwrap_or_else(|e| e.to_string()),
        show("WAYLAND_DISPLAY"),
        show("DISPLAY"),
        show("XDG_RUNTIME_DIR"),
    ));
    // Watchdog: bound the whole run by wall clock, not by frames. If
    // the compositor never schedules a frame the frame counter (and
    // the frame-based bail-out inside update) never advances, so a
    // time-based backstop is the only way the process can end by
    // itself. 60 s is well under Craft Anything's own 90 s stop.
    std::thread::Builder::new()
        .name("shot-watchdog".into())
        .spawn(|| {
            let start = std::time::Instant::now();
            loop {
                std::thread::sleep(std::time::Duration::from_secs(5));
                let saved = SHOT_SAVED.load(std::sync::atomic::Ordering::SeqCst);
                let elapsed = start.elapsed();
                if saved && elapsed > std::time::Duration::from_secs(25) {
                    // The picture was written but the window didn't
                    // close on its own; end cleanly anyway.
                    shot_log("watchdog: still alive 25 s after saving, force exit");
                    std::process::exit(0);
                }
                if !saved && elapsed > std::time::Duration::from_secs(60) {
                    shot_log("watchdog: 60 s without a picture, force exit");
                    std::process::exit(1);
                }
            }
        })
        .ok();
}

fn main() -> eframe::Result<()> {
    // Parse --screenshot <file.png> (also `--screenshot=<file.png>`, and a
    // bare `--screenshot` that saves to screenshot.png, so any caller's
    // argument style works).
    let args: Vec<String> = std::env::args().collect();
    let mut shot: Option<ShotState> = None;
    let mut launch_paths: Vec<std::path::PathBuf> = Vec::new();
    let mut i = 1;
    while i < args.len() {
        let path = if args[i] == "--screenshot" {
            let p = args.get(i + 1).cloned().unwrap_or_default();
            i += 2;
            if p.is_empty() || p.starts_with("--") {
                "screenshot.png".to_string()
            } else {
                p
            }
        } else if let Some(rest) = args[i].strip_prefix("--screenshot=") {
            let p = rest.to_string();
            i += 1;
            if p.is_empty() {
                "screenshot.png".to_string()
            } else {
                p
            }
        } else if !args[i].starts_with('-') {
            // A path from Explorer / a file manager's "Open with" menu.
            launch_paths.push(std::path::PathBuf::from(&args[i]));
            i += 1;
            continue;
        } else {
            i += 1;
            continue;
        };
        shot = Some(ShotState {
            path: std::path::PathBuf::from(path),
            frame: 0,
            requested: false,
            slot: None,
        });
    }
    let screenshot_mode = shot.is_some();
    if screenshot_mode {
        init_shot_log();
    }

    // Screenshot-mode-only staging knobs (normal runs ignore them):
    // TAGRENAME_WIN=900x620 draws a smaller window (checks that the
    // cover panels still fit), TAGRENAME_TAB=1|2 opens another tools
    // bar tab. The official check runs without them.
    let mut win_size = [1180.0, 800.0];
    if screenshot_mode {
        if let Ok(spec) = std::env::var("TAGRENAME_WIN") {
            let parts: Vec<&str> = spec.split('x').collect();
            if parts.len() == 2 {
                if let (Ok(w), Ok(h)) = (parts[0].parse::<f32>(), parts[1].parse::<f32>()) {
                    win_size = [w.max(700.0), h.max(480.0)];
                }
            }
        }
    }

    // A borderless, see-through window: the app draws its own rounded
    // window frame and title bar (ui_layout.rs window_chrome).
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size(win_size)
        .with_min_inner_size([900.0, 600.0])
        .with_title("Tag:Rename")
        .with_app_id("tagrename")
        .with_decorations(false);
    // A transparent window is expensive for the software renderer in a VM.
    // Windows keeps an opaque surface instead; the rounded chrome is still
    // painted by the app, and no panel gap can show the desktop through it.
    #[cfg(not(target_os = "windows"))]
    {
        viewport = viewport.with_transparent(true);
    }
    if let Some(icon) = app_icon() {
        viewport = viewport.with_icon(icon);
    }
    if screenshot_mode {
        // Open without taking focus so a check never disturbs the person.
        viewport = viewport.with_active(false);
        // On Windows also stay behind other windows.
        #[cfg(target_os = "windows")]
        {
            viewport = viewport.with_window_level(egui::viewport::WindowLevel::AlwaysOnBottom);
        }
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Tag:Rename",
        options,
        Box::new(move |cc| {
            // The theme and colour scheme picked last time are restored.
            // Without a saved choice the app opens on the Default theme
            // in its light scheme.
            theme::install_fonts(&cc.egui_ctx);
            // Custom themes are appended after the built-ins, so load them
            // before the saved numeric theme index is restored.
            theme::load_customs();
            if !screenshot_mode {
                if let Some((t, s)) = theme::saved_choice() {
                    theme::select(&cc.egui_ctx, t, s);
                } else {
                    theme::select(&cc.egui_ctx, 0, theme::SCHEME_LIGHT);
                }
            }
            theme::install(&cc.egui_ctx);
            if screenshot_mode {
                let t = std::env::var("TAGRENAME_THEME").ok().and_then(|t| t.trim().parse::<usize>().ok());
                let s = std::env::var("TAGRENAME_SCHEME").ok().and_then(|t| t.trim().parse::<usize>().ok());
                if t.is_some() || s.is_some() {
                    theme::select(&cc.egui_ctx, t.unwrap_or(0), s.unwrap_or(theme::SCHEME_LIGHT));
                } else {
                    theme::select(&cc.egui_ctx, 0, theme::SCHEME_LIGHT);
                }
            }
            // The original's dialogs open at their designed position every
            // time (TFORMTAGEDITOR: poOwnerFormCenter), so don't restore
            // any remembered window geometry — a stale remembered rect
            // once blew the Tag Editor up past the screen.
            cc.egui_ctx.memory_mut(|m| m.reset_areas());
            // Same for panels: egui restores a remembered panel height
            // even for non-resizable panels, which kept the tools bar
            // at a broken remembered size after the layout was fixed.
            cc.egui_ctx
                .data_mut(|d| d.remove_by_type::<egui::PanelState>());
            let mut app = ui::App::new();
            app.shot = shot;
            if screenshot_mode {
                // Keep a stable, representative view for the picture.
                if let Ok(t) = std::env::var("TAGRENAME_TAB") {
                    if let Ok(t) = t.trim().parse::<usize>() {
                        app.active_tab = t.min(2);
                    }
                }
                // Verification knob: tick "Case:" on the Rename tab so the
                // four case options show in a screenshot.
                if std::env::var("TAGRENAME_CASE").is_ok() {
                    app.rename_case_enabled = true;
                }
                // Verification knob: stage the folder browser over a real
                // folder so a screenshot can show the folder rows. Normal
                // runs (and the official check) do not set this.
                if let Ok(folder) = std::env::var("TAGRENAME_FOLDER") {
                    let p = std::path::PathBuf::from(folder);
                    if p.is_dir() {
                        app.load_folder(&p);
                    }
                }
                // Verification knob: open the Appearance window for a screenshot.
                if std::env::var("TAGRENAME_APPEARANCE").is_ok() {
                    app.dlg.show_appearance = true;
                }
                // Verification knob: open the Create theme panel directly.
                if std::env::var("TAGRENAME_CREATE_THEME").is_ok() {
                    app.dlg.open_new_theme_editor(theme::theme_index());
                }
                // Verification knob: open Create theme already previewing the
                // chosen mode. Accepts "dark" or "light" (default dark).
                if let Ok(mode) = std::env::var("TAGRENAME_EDITOR_MODE") {
                    app.dlg.open_new_theme_editor(theme::theme_index());
                    let dark = mode.trim() != "light";
                    app.dlg.set_theme_editor_mode(dark);
                    app.dlg.preview_theme_editor(&cc.egui_ctx);
                }
                // Verification knob: open the Create theme panel with the colour
                // picker over one box. Accepts dark-bg/text/accent or light-*.
                if let Ok(which) = std::env::var("TAGRENAME_COLOR_PICKER") {
                    app.dlg.open_new_theme_editor(theme::theme_index());
                    let (dark, field) = match which.trim() {
                        "dark-bg" => (true, 0),
                        "dark-text" => (true, 1),
                        "light-bg" => (false, 0),
                        "light-text" => (false, 1),
                        "light-accent" => (false, 2),
                        _ => (true, 2),
                    };
                    app.dlg.open_theme_colour_picker(dark, field);
                }
                // Verification knob: open the Mask Editor (0 = rename mask,
                // 1 = file-name-to-tag mask) for a screenshot.
                if let Ok(m) = std::env::var("TAGRENAME_MASK") {
                    if let Ok(w) = m.trim().parse::<usize>() {
                        let which = w.min(1);
                        if which == 0 {
                            app.dlg.open_mask_editor(0, &app.rename_mask);
                        } else {
                            app.dlg.open_mask_editor(1, &app.f2t_mask);
                        }
                    }
                }
            }
            // A shell-launched file or folder overrides the screenshot-only
            // staging knobs, so tests can check the real "Open with" path.
            app.open_launch_paths(&launch_paths);
            Ok(Box::new(app))
        }),
    )
}

/// The Tag:Rename icon, shown in the window and the taskbar.
fn app_icon() -> Option<egui::IconData> {
    let img = image::load_from_memory(include_bytes!("../assets/tagrename-256.png"))
        .ok()?
        .to_rgba8();
    let (width, height) = img.dimensions();
    Some(egui::IconData {
        rgba: img.into_raw(),
        width,
        height,
    })
}

impl eframe::App for ui::App {
    fn ui(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui_layout::update(self, root);
    }

    // On Windows the surface is opaque: it is faster without a GPU and it
    // stops the gaps between panels showing the desktop. Other desktops keep
    // the see-through rounded-corner look.
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        #[cfg(target_os = "windows")]
        {
            let c = theme::c_panel();
            [c.r() as f32 / 255.0, c.g() as f32 / 255.0, c.b() as f32 / 255.0, 1.0]
        }
        #[cfg(not(target_os = "windows"))]
        {
            [0.0, 0.0, 0.0, 0.0]
        }
    }
}

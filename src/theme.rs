//! Colour themes for the whole app, and the interface scale.
//!
//! The Appearance window picks one theme (each has a light and a dark
//! version) and one colour scheme (System, Light or Dark). Every colour in
//! the app is read from the active palette through the `c_*` functions.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::OnceLock;

use egui::{Color32, Context, Stroke, Style, Visuals};

/// The whole interface (fonts, spacing, panels) is drawn at 80% of the
/// previous size, as the person asked.
pub const UI_ZOOM: f32 = 0.8;

pub const SCHEME_SYSTEM: usize = 0;
pub const SCHEME_LIGHT: usize = 1;
pub const SCHEME_DARK: usize = 2;
pub const SCHEME_NAMES: [&str; 3] = ["System", "Light", "Dark"];

/// Corner radius of every window, the main window included.
pub const WINDOW_RADIUS: u8 = 12;

#[derive(Clone, Copy)]
pub struct Palette {
    pub dark: bool,
    pub bg: Color32,
    pub panel: Color32,
    pub panel_bright: Color32,
    pub widget_bg: Color32,
    pub widget_hover: Color32,
    pub accent: Color32,
    pub accent_dim: Color32,
    pub text: Color32,
    pub text_dim: Color32,
    pub separator: Color32,
    pub selected: Color32,
    pub danger: Color32,
    pub ok: Color32,
    pub dialog_bg: Color32,
    pub dialog_edge: Color32,
    pub dialog_title: Color32,
    pub art_card: Color32,
    pub header: Color32,
    pub extreme_bg: Color32,
}

#[derive(Clone, Copy)]
pub struct Theme {
    pub name: &'static str,
    pub light: Palette,
    pub dark: Palette,
}

fn hex(v: u32) -> Color32 {
    Color32::from_rgb((v >> 16) as u8, (v >> 8) as u8, v as u8)
}

fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let m = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgb(m(a.r(), b.r()), m(a.g(), b.g()), m(a.b(), b.b()))
}

/// One palette from its main colours; the rest of the fields are blended
/// from these so every version of a theme follows the same rules.
fn scheme(dark: bool, bg: u32, panel: u32, widget: u32, text: u32, dim: u32, accent: u32, sep: u32) -> Palette {
    let (bg, panel, widget, text, dim, accent, sep) =
        (hex(bg), hex(panel), hex(widget), hex(text), hex(dim), hex(accent), hex(sep));
    Palette {
        dark,
        bg,
        panel,
        panel_bright: mix(panel, text, 0.05),
        widget_bg: widget,
        widget_hover: mix(widget, text, 0.12),
        accent,
        accent_dim: mix(panel, accent, 0.35),
        text,
        text_dim: dim,
        separator: sep,
        selected: mix(panel, accent, 0.22),
        danger: if dark { hex(0xf07070) } else { hex(0xc83232) },
        ok: if dark { hex(0x6fd69a) } else { hex(0x1e8c46) },
        dialog_bg: panel,
        dialog_edge: mix(sep, text, 0.25),
        dialog_title: mix(panel, accent, 0.22),
        art_card: mix(panel, text, 0.16),
        header: if dark { mix(bg, Color32::BLACK, 0.25) } else { mix(bg, text, 0.04) },
        extreme_bg: if dark { mix(panel, Color32::BLACK, 0.35) } else { mix(panel, Color32::WHITE, 0.5) },
    }
}

/// The eleven themes, in the order the Appearance window lists them. Colours
/// for the named themes are taken from the person's screenshots; Default
/// is the Aurral theme, High Contrast and Spotty are the app's own. Slab's
/// background, text and accent come from the supplied theme values, with the
/// remaining surfaces sampled from the same screenshots.
fn build_themes() -> Vec<Theme> {
    vec![
        Theme {
            name: "Default",
            dark: scheme(true, 0x0f1115, 0x171a21, 0x232832, 0xe6e9ef, 0x8b93a7, 0x5b8cff, 0x2a2f3a),
            light: scheme(false, 0xefefef, 0xffffff, 0xf1f1f1, 0x161616, 0x6b6b6b, 0x2150b5, 0xdedede),
        },
        Theme {
            name: "Everforest",
            dark: scheme(true, 0x2d353b, 0x2d353b, 0x3d484d, 0xd3c6aa, 0x859289, 0xa7c080, 0x475258),
            light: scheme(false, 0xe6dfcc, 0xfdf6e3, 0xf4f0d9, 0x5c6a72, 0x829181, 0x8da101, 0xe0dcc7),
        },
        Theme {
            name: "Gruvbox",
            dark: scheme(true, 0x282828, 0x282828, 0x3c3836, 0xebdbb2, 0xa89984, 0xf4923c, 0x504945),
            light: scheme(false, 0xe4dab1, 0xfbf1c7, 0xebdbb2, 0x3c3836, 0x7c6f64, 0xaf3a03, 0xd5c4a1),
        },
        Theme {
            name: "Nord",
            dark: scheme(true, 0x2e3440, 0x2e3440, 0x3b4252, 0xeceff4, 0x9099aa, 0x88c0d0, 0x434c5e),
            light: scheme(false, 0xced2d9, 0xe5e9f0, 0xd8dee9, 0x2e3440, 0x4c566a, 0x3c8ba0, 0xc0c8d4),
        },
        Theme {
            name: "Tokyo Night",
            dark: scheme(true, 0x1a1b26, 0x1a1b26, 0x292e42, 0xc0caf5, 0x787c99, 0x4bbfdc, 0x33364d),
            light: scheme(false, 0xbebfc4, 0xd5d6db, 0xcbccd1, 0x343b58, 0x6b7089, 0x34548a, 0xb4b5bc),
        },
        Theme {
            name: "Dracula",
            dark: scheme(true, 0x282a36, 0x282a36, 0x44475a, 0xf8f8f2, 0xa0a2b5, 0xbd93f9, 0x3d3f50),
            light: scheme(false, 0xe0e1e8, 0xf7f8ff, 0xe9eaf2, 0x282a36, 0x6272a4, 0xa07ed8, 0xd6d7e0),
        },
        Theme {
            name: "Solarized",
            dark: scheme(true, 0x002b36, 0x002b36, 0x073642, 0x93a1a1, 0x657b83, 0x4b9ace, 0x0b4350),
            light: scheme(false, 0xe6dfcc, 0xfdf6e3, 0xeee8d5, 0x586e75, 0x93a1a1, 0x268bd2, 0xe0d8c0),
        },
        Theme {
            name: "GitHub",
            dark: scheme(true, 0x0d1117, 0x0d1117, 0x161b22, 0xc9d1d9, 0x8b949e, 0x58a6ff, 0x30363d),
            light: scheme(false, 0xe8e8e8, 0xffffff, 0xf6f8fa, 0x24292f, 0x57606a, 0x0969da, 0xd0d7de),
        },
        Theme {
            name: "Slab",
            dark: scheme(true, 0x403d3d, 0x4f4c4c, 0x4f4c4c, 0xd5c4a1, 0xa89984, 0xf37135, 0x5e5b5b),
            light: scheme(false, 0x8b8b8b, 0x818181, 0x818181, 0x504945, 0x656565, 0xf37135, 0x767676),
        },
        Theme {
            name: "High Contrast",
            dark: scheme(true, 0x000000, 0x080808, 0x1a1a1a, 0xffffff, 0xc8c8c8, 0xffd600, 0x808080),
            light: scheme(false, 0xffffff, 0xffffff, 0xe6e6e6, 0x000000, 0x303030, 0x0040c0, 0x404040),
        },
        Theme {
            name: "Spotty",
            dark: scheme(true, 0x000000, 0x121212, 0x272727, 0xffffff, 0xb3b3b3, 0x1ed760, 0x2a2a2a),
            light: scheme(false, 0xf0f0f0, 0xffffff, 0xe6e6e6, 0x121212, 0x5e5e5e, 0x1a9e4a, 0xd4d4d4),
        },
    ]
}

pub fn themes() -> &'static [Theme] {
    static LIST: OnceLock<Vec<Theme>> = OnceLock::new();
    LIST.get_or_init(build_themes)
}

static THEME: AtomicUsize = AtomicUsize::new(0);
static SCHEME: AtomicUsize = AtomicUsize::new(SCHEME_DARK);
/// What the operating system asked for last time we looked. Used when the
/// scheme is System; dark when the system does not say.
static SYSTEM_DARK: AtomicBool = AtomicBool::new(true);

pub fn theme_index() -> usize {
    THEME.load(Ordering::Relaxed).min(themes().len() - 1)
}

pub fn scheme_index() -> usize {
    SCHEME.load(Ordering::Relaxed).min(SCHEME_DARK)
}

pub fn is_dark() -> bool {
    match scheme_index() {
        SCHEME_LIGHT => false,
        SCHEME_DARK => true,
        _ => SYSTEM_DARK.load(Ordering::Relaxed),
    }
}

pub fn palette_of(theme: usize, dark: bool) -> Palette {
    let t = themes()[theme.min(themes().len() - 1)];
    if dark { t.dark } else { t.light }
}

pub fn pal() -> Palette {
    palette_of(theme_index(), is_dark())
}

pub fn c_bg() -> Color32 { pal().bg }
pub fn c_panel() -> Color32 { pal().panel }
pub fn c_panel_bright() -> Color32 { pal().panel_bright }
pub fn c_accent() -> Color32 { pal().accent }
pub fn c_accent_dim() -> Color32 { pal().accent_dim }
pub fn c_text() -> Color32 { pal().text }
pub fn c_text_dim() -> Color32 { pal().text_dim }
pub fn c_separator() -> Color32 { pal().separator }
pub fn c_selected() -> Color32 { pal().selected }
pub fn c_danger() -> Color32 { pal().danger }
pub fn c_ok() -> Color32 { pal().ok }
pub fn c_dialog_bg() -> Color32 { pal().dialog_bg }
pub fn c_dialog_edge() -> Color32 { pal().dialog_edge }
pub fn c_dialog_title() -> Color32 { pal().dialog_title }
pub fn c_art_card() -> Color32 { pal().art_card }
pub fn c_header() -> Color32 { pal().header }

/// Switch theme and colour scheme and restyle the window straight away.
pub fn select(ctx: &Context, theme: usize, scheme: usize) {
    THEME.store(theme.min(themes().len() - 1), Ordering::Relaxed);
    SCHEME.store(scheme.min(SCHEME_DARK), Ordering::Relaxed);
    install(ctx);
}

/// Follow the operating system's light or dark setting while the scheme is
/// System. Called every frame; it only restyles when the setting changes.
pub fn refresh(ctx: &Context) {
    let Some(system) = ctx.system_theme() else { return };
    let dark = system == egui::Theme::Dark;
    let was = SYSTEM_DARK.swap(dark, Ordering::Relaxed);
    if was != dark && scheme_index() == SCHEME_SYSTEM {
        install(ctx);
    }
}

/// The theme and scheme the person picked last time, kept in their config
/// folder as "<theme> <scheme>". None when nothing was saved yet.
pub fn saved_choice() -> Option<(usize, usize)> {
    let file = crate::ui::config_file("appearance.txt")?;
    let text = std::fs::read_to_string(file).ok()?;
    let mut parts = text.split_whitespace();
    let theme = parts.next()?.parse::<usize>().ok()?.min(themes().len() - 1);
    let scheme = parts.next()?.parse::<usize>().ok()?.min(SCHEME_DARK);
    Some((theme, scheme))
}

/// Remember the theme and scheme for the next start.
pub fn save_choice(theme: usize, scheme: usize) {
    let Some(file) = crate::ui::config_file("appearance.txt") else { return };
    if let Some(dir) = file.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(file, format!("{theme} {scheme}"));
}

pub fn install(ctx: &Context) {
    let p = pal();
    let base = if p.dark { egui::Theme::Dark } else { egui::Theme::Light };
    ctx.set_zoom_factor(UI_ZOOM);
    ctx.set_theme(base);
    let mut style: Style = (*ctx.style_of(base)).clone();

    let mut v = if p.dark { Visuals::dark() } else { Visuals::light() };
    v.panel_fill = p.bg;
    v.window_fill = p.panel;
    v.window_stroke = Stroke::new(1.0, p.separator);
    v.extreme_bg_color = p.extreme_bg;
    v.faint_bg_color = p.panel_bright;
    v.code_bg_color = p.extreme_bg;
    v.hyperlink_color = p.accent;
    v.weak_text_color = Some(p.text_dim);

    v.widgets.noninteractive.bg_fill = p.panel;
    v.widgets.noninteractive.weak_bg_fill = p.panel;
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, p.text_dim);
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, p.separator);
    // egui 0.37 styles buttons and window title bars through
    // weak_bg_fill (bg_fill alone does not reach them), which left every
    // button and the window title bar in egui's default mid gray
    // (45,45,45) until this was set.
    v.widgets.inactive.bg_fill = p.widget_bg;
    v.widgets.inactive.weak_bg_fill = p.widget_bg;
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, p.text);
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, p.separator);
    v.widgets.hovered.bg_fill = p.widget_hover;
    v.widgets.hovered.weak_bg_fill = p.widget_hover;
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, p.text);
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, p.accent_dim);
    v.widgets.active.bg_fill = p.accent_dim;
    v.widgets.active.weak_bg_fill = p.accent_dim;
    v.widgets.active.fg_stroke = Stroke::new(1.0, p.text);
    v.widgets.active.bg_stroke = Stroke::new(1.0, p.accent);
    v.widgets.open.bg_fill = p.widget_hover;
    v.widgets.open.weak_bg_fill = p.widget_hover;

    v.selection.bg_fill = p.selected;
    v.selection.stroke = Stroke::new(1.0, p.accent);

    for w in [&mut v.widgets.noninteractive, &mut v.widgets.inactive, &mut v.widgets.hovered, &mut v.widgets.active] {
        w.corner_radius = egui::CornerRadius::same(8);
    }
    v.window_corner_radius = egui::CornerRadius::same(WINDOW_RADIUS);
    v.menu_corner_radius = egui::CornerRadius::same(10);
    v.slider_trailing_fill = true;
    v.striped = true;

    style.visuals = v;
    style.spacing.item_spacing = egui::vec2(6.0, 4.0);
    style.spacing.button_padding = egui::vec2(8.0, 4.0);
    style.spacing.menu_margin = egui::Margin::same(6);

    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::proportional(15.0),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::proportional(14.5),
    );
    style.text_styles.insert(
        egui::TextStyle::Small,
        egui::FontId::proportional(13.5),
    );

    ctx.set_style_of(base, style);
}

/// The app's font, Inter, bundled into the program. Regular text uses it
/// everywhere; "semibold" is the same family for headings.
pub fn install_fonts(ctx: &Context) {
    use egui::{FontData, FontDefinitions, FontFamily};
    use std::sync::Arc;

    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "inter".into(),
        Arc::new(FontData::from_static(include_bytes!("../fonts/Inter-Regular.ttf"))),
    );
    fonts.font_data.insert(
        "inter_semibold".into(),
        Arc::new(FontData::from_static(include_bytes!("../fonts/Inter-SemiBold.ttf"))),
    );
    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        if let Some(list) = fonts.families.get_mut(&family) {
            list.insert(0, "inter".into());
        }
    }
    fonts.families.insert(
        FontFamily::Name("semibold".into()),
        vec!["inter_semibold".into(), "inter".into()],
    );
    ctx.set_fonts(fonts);
}

/// A subtle card look for panels ("modern dark").
pub fn panel_frame(fill: Color32) -> egui::Frame {
    egui::Frame::new()
        .fill(fill)
        .stroke(Stroke::new(1.0, c_separator()))
        .inner_margin(egui::Margin::same(8))
        .corner_radius(egui::CornerRadius::same(8))
}

pub fn tool_button(ui: &mut egui::Ui, label: &str, tooltip: &str) -> egui::Response {
    ui.add(egui::Button::new(
        egui::RichText::new(label).color(c_text()).size(14.0),
    ))
    .on_hover_text(tooltip)
}

pub fn primary_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    let btn = egui::Button::new(
        egui::RichText::new(label).color(c_text()).strong(),
    )
    .fill(c_accent_dim())
    .stroke(Stroke::new(1.0, c_accent()));
    ui.add(btn)
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_chosen_theme_is_remembered_for_the_next_start() {
        super::save_choice(3, 1);
        assert_eq!(super::saved_choice(), Some((3, 1)));
        super::save_choice(99, 9);
        assert_eq!(super::saved_choice(), Some((super::themes().len() - 1, super::SCHEME_DARK)));
    }

    #[test]
    fn every_theme_has_a_light_and_a_dark_version() {
        assert_eq!(super::themes().len(), 11);
        assert_eq!(super::themes()[0].name, "Default");
        assert_eq!(super::themes()[8].name, "Slab");
        for t in super::themes() {
            assert!(!t.light.dark && t.dark.dark, "{}", t.name);
        }
    }

    #[test]
    fn slab_uses_the_supplied_colours() {
        let slab = &super::themes()[8];
        assert_eq!(slab.light.bg, super::hex(0x8b8b8b));
        assert_eq!(slab.light.panel, super::hex(0x818181));
        assert_eq!(slab.light.text, super::hex(0x504945));
        assert_eq!(slab.light.accent, super::hex(0xf37135));
        assert_eq!(slab.dark.bg, super::hex(0x403d3d));
        assert_eq!(slab.dark.panel, super::hex(0x4f4c4c));
        assert_eq!(slab.dark.text, super::hex(0xd5c4a1));
        assert_eq!(slab.dark.accent, super::hex(0xf37135));
    }
}

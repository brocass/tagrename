//! All secondary windows: previews, mask editor, export, auto fill, find,
//! folder/file chooser, options, genre editor, about, confirmations.

use std::path::PathBuf;

use egui::{RichText, Ui};

use crate::data::{Field, EXPORT_FIELDS, ALL_FIELDS};
use crate::mask;
use crate::theme;
use crate::theme::*;
use crate::ui::App;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Csv,
    Txt,
    Html,
    Xml,
}

impl ExportFormat {
    fn label(self) -> &'static str {
        match self {
            ExportFormat::Csv => "CSV",
            ExportFormat::Txt => "TEXT",
            ExportFormat::Html => "HTML",
            ExportFormat::Xml => "XML",
        }
    }

    fn ext(self) -> &'static str {
        match self {
            ExportFormat::Csv => "csv",
            ExportFormat::Txt => "txt",
            ExportFormat::Html => "html",
            ExportFormat::Xml => "xml",
        }
    }
}

/// What the in-app file chooser is being used for right now.
#[derive(Clone)]
pub enum ChoosePurpose {
    OpenFolder,
    /// Add a folder to the tree only — no files are scanned.
    AddRoot,
    SavePlaylist(Vec<usize>),
    SaveExport,
    SaveCover,
    PickCover,
    LoadLyrics,
    SaveLyrics,
}

impl ChoosePurpose {
    fn title(&self) -> &'static str {
        match self {
            ChoosePurpose::OpenFolder => "Open Folder",
            ChoosePurpose::AddRoot => "Add Folder Root",
            ChoosePurpose::SavePlaylist(_) => "Save Playlist (.m3u)",
            ChoosePurpose::SaveExport => "Export file name",
            ChoosePurpose::SaveCover => "Save cover art to file",
            ChoosePurpose::PickCover => "Choose a cover art image",
            ChoosePurpose::LoadLyrics => "Load lyrics from text file",
            ChoosePurpose::SaveLyrics => "Save lyrics to text file",
        }
    }
}

pub struct ChooseState {
    pub purpose: ChoosePurpose,
    pub dir: PathBuf,
    pub selected: Option<PathBuf>,
    pub name: String,
    pub cancelled: bool,
    /// The paste-a-location row's text (like the address bar above the
    /// file tree): paste a path and press Enter to jump straight there.
    pub path_edit: String,
}

pub enum ChooseDone {
    Folder(PathBuf),
    SavePath(PathBuf),
    Bytes(Vec<u8>),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ThemeFilePurpose {
    Export,
    Import,
}

impl ThemeFilePurpose {
    fn title(self) -> &'static str {
        match self {
            ThemeFilePurpose::Export => "Export theme",
            ThemeFilePurpose::Import => "Import theme",
        }
    }

    fn action(self) -> &'static str {
        match self {
            ThemeFilePurpose::Export => "Save",
            ThemeFilePurpose::Import => "Load",
        }
    }
}

struct ThemeFileState {
    purpose: ThemeFilePurpose,
    path: String,
}

/// The three colour strings are kept as the person types them; they are only
/// parsed into colours when saving, exporting or generating.
#[derive(Clone)]
struct ThemeColourFields {
    bg: String,
    text: String,
    accent: String,
}

impl ThemeColourFields {
    fn from_colours(c: ThemeColours) -> Self {
        Self { bg: hex_string(c.bg), text: hex_string(c.text), accent: hex_string(c.accent) }
    }

    fn parse(&self, mode: &str) -> Result<ThemeColours, String> {
        let bad = |field: &str| format!("{mode} {field} must be a #RRGGBB colour");
        Ok(ThemeColours {
            bg: parse_hex(&self.bg).ok_or_else(|| bad("background"))?,
            text: parse_hex(&self.text).ok_or_else(|| bad("text"))?,
            accent: parse_hex(&self.accent).ok_or_else(|| bad("accent"))?,
        })
    }
}

struct ThemeEditorState {
    /// Some(index) when an existing custom theme is being edited.
    origin: Option<usize>,
    name: String,
    /// Which mode's colours are visible, not the app's global colour scheme.
    dark: bool,
    dark_fields: ThemeColourFields,
    light_fields: ThemeColourFields,
}

impl ThemeEditorState {
    fn spec(&self) -> Result<CustomThemeSpec, String> {
        let name = self.name.trim().to_string();
        if name.is_empty() {
            return Err("Give the theme a name first".to_string());
        }
        Ok(CustomThemeSpec {
            name,
            dark: self.dark_fields.parse("Dark")?,
            light: self.light_fields.parse("Light")?,
        })
    }
}

pub struct Dialogs {
    pub tag_editor: Option<crate::ui_tageditor::TagEditor>,
    /// The Tag Editor's size when it is not full screen; kept while the
    /// app runs so the next opening starts at the same size.
    pub editor_size: Option<egui::Vec2>,
    pub show_mask_editor: Option<usize>,
    /// The mask as it was when the Mask Editor opened; Cancel puts it back.
    pub mask_backup: String,
    /// Masks saved from the Mask Editor for this session.
    pub saved_masks: Vec<String>,
    pub saved_mask_sel: Option<usize>,
    /// The masks as last written to masks.txt, to write only on change.
    pub masks_saved: String,
    pub show_rename_preview: bool,
    pub show_f2t_preview: bool,
    pub show_export: bool,
    pub export_fields: Vec<(Field, bool)>,
    pub export_format: ExportFormat,
    pub export_name: String,
    pub export_pending_fields: Option<Vec<Field>>,
    pub show_autofill: bool,
    pub autofill_start: u32,
    pub autofill_use_total: bool,
    pub show_find: bool,
    pub show_appearance: bool,
    pub theme_editor: Option<ThemeEditorState>,
    pub theme_file: Option<ThemeFileState>,
    pub find_text: String,
    pub find_match_case: bool,
    pub show_about: bool,
    pub show_options: bool,
    pub show_genre_editor: bool,
    pub new_genre: String,
    pub choose: Option<ChooseState>,
    pub confirm_remove: Option<Vec<usize>>,
    /// The Advanced… swap dialog and the Copy Text from One Tag Field to
    /// Another dialog (both pick any two fields).
    pub swap_advanced: bool,
    pub copy_field_open: bool,
    pub swap_from: usize,
    pub swap_to: usize,
    pub save_cover_bytes: Option<Vec<u8>>,
    /// Lyrics waiting to be written by the Save-to-text-file chooser.
    pub save_lyrics_text: Option<String>,
    pub pick_cover: bool,
    results: Vec<crate::ui_tageditor::EditorResult>,
}

impl Default for Dialogs {
    fn default() -> Self {
        Dialogs {
            tag_editor: None,
            editor_size: None,
            show_mask_editor: None,
            mask_backup: String::new(),
            saved_masks: Vec::new(),
            saved_mask_sel: None,
            masks_saved: String::new(),
            show_rename_preview: false,
            show_f2t_preview: false,
            show_export: false,
            export_fields: EXPORT_FIELDS.iter().map(|f| (*f, true)).collect(),
            export_format: ExportFormat::Csv,
            export_name: "export".into(),
            export_pending_fields: None,
            show_autofill: false,
            autofill_start: 1,
            autofill_use_total: false,
            show_find: false,
            show_appearance: false,
            theme_editor: None,
            theme_file: None,
            find_text: String::new(),
            find_match_case: false,
            show_about: false,
            show_options: false,
            show_genre_editor: false,
            new_genre: String::new(),
            choose: None,
            confirm_remove: None,
            swap_advanced: false,
            copy_field_open: false,
            swap_from: 0,
            swap_to: 0,
            save_cover_bytes: None,
            save_lyrics_text: None,
            pick_cover: false,
            results: Vec::new(),
        }
    }
}

impl Dialogs {
    pub fn apply_result(&mut self, result: crate::ui_tageditor::EditorResult) {
        self.results.push(result);
    }

    pub fn open_mask_editor(&mut self, which: usize, current: &str) {
        self.show_mask_editor = Some(which);
        self.mask_backup = current.to_string();
        self.saved_mask_sel = None;
    }

    /// Open the Create theme panel. A custom theme is opened for editing;
    /// a built-in one is only used as a starting point for a new theme.
    pub fn open_theme_editor(&mut self, index: usize) {
        let spec = theme::spec_of(index).unwrap_or_else(theme::default_custom_spec);
        let origin = if index >= theme::BUILTIN_COUNT { Some(index) } else { None };
        let name = if origin.is_some() { spec.name } else { "My theme".to_string() };
        self.theme_editor = Some(ThemeEditorState {
            origin,
            name,
            dark: true,
            dark_fields: ThemeColourFields::from_colours(spec.dark),
            light_fields: ThemeColourFields::from_colours(spec.light),
        });
    }

    fn take_results(&mut self) -> Vec<crate::ui_tageditor::EditorResult> {
        std::mem::take(&mut self.results)
    }
}

impl App {
    pub fn rename_preview_rows(&self) -> Vec<(String, String)> {
        let indices = self.checked_indices();
        let opts = mask::RenameOptions {
            case: mask::Case::from_index(self.rename_case),
            case_enabled: self.rename_case_enabled,
            replace_spaces_with_underscores: self.replace_spaces,
            remove_leading_zero_in_disc: self.remove_leading_zero_disc,
        };
        let total = indices.len().max(1) as u32;
        let mut rows = Vec::new();
        for (n, i) in indices.iter().enumerate() {
            let f = &self.files[*i];
            let new = mask::build_new_file_name(f, &self.rename_mask, n as u32 + 1, total, &opts)
                .unwrap_or_else(|| "(can't build: no tag data)".into());
            rows.push((f.name.clone(), new));
        }
        rows
    }

    pub fn f2t_preview_rows(&self) -> Vec<(String, String)> {
        let indices = self.checked_indices();
        let opts = mask::ParseOptions {
            case: mask::Case::from_index(self.f2t_case),
            case_enabled: self.f2t_case_enabled,
            replace_underscores_with_spaces: self.replace_underscores,
        };
        let mut rows = Vec::new();
        for i in &indices {
            let f = &self.files[*i];
            let name = mask::parse_file_name(&f.file_stem(), &self.f2t_mask, &opts)
                .map(|pairs| mask::with_extension(mask::render_parsed(&self.f2t_mask, &pairs), &f.extension()))
                .unwrap_or_else(|e| e);
            rows.push((f.name.clone(), name));
        }
        rows
    }
}

/// The Appearance window: a colour scheme (System, Light or Dark) and one of
/// the built-in themes, laid out like the person's screenshots.
fn appearance_window(ctx: &egui::Context, app: &mut App) {
    let mut open = true;
    let cur_theme = theme_index();
    let cur_scheme = scheme_index();
    let (mut pick_theme, mut pick_scheme) = (cur_theme, cur_scheme);
    egui::Window::new("Appearance")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_width(540.0)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            let gap = 10.0;
            ui.label(appearance_heading("Colour scheme"));
            ui.add_space(6.0);
            let w = (ui.available_width() - 2.0 * gap) / 3.0;
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = gap;
                for s in 0..3 {
                    if scheme_card(ui, w, s, cur_theme, cur_scheme == s).clicked() {
                        pick_scheme = s;
                    }
                }
            });
            ui.add_space(16.0);
            ui.horizontal(|ui| {
                ui.label(appearance_heading("Theme"));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if primary_button(ui, "Create theme").clicked() {
                        app.dlg.open_theme_editor(cur_theme);
                    }
                    if ui.button("Import").clicked() {
                        app.dlg.theme_file = Some(ThemeFileState {
                            purpose: ThemeFilePurpose::Import,
                            path: theme_json_path("theme"),
                        });
                    }
                });
            });
            ui.add_space(6.0);
            let w = (ui.available_width() - 2.0 * gap) / 3.0;
            for (row, chunk) in themes().chunks(3).enumerate() {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    for (col, t) in chunk.iter().enumerate() {
                        let k = row * 3 + col;
                        if theme_card(ui, w, &t.name, k, cur_theme == k, k >= theme::BUILTIN_COUNT).clicked() {
                            pick_theme = k;
                        }
                    }
                });
                ui.add_space(gap);
            }
        });
    if pick_theme != cur_theme || pick_scheme != cur_scheme {
        select(ctx, pick_theme, pick_scheme);
        save_choice(pick_theme, pick_scheme);
    }
    app.dlg.show_appearance &= open;
}

fn appearance_heading(text: &str) -> RichText {
    RichText::new(text)
        .size(15.0)
        .color(c_text())
        .family(egui::FontFamily::Name("semibold".into()))
}

fn appearance_card(ui: &mut Ui, size: egui::Vec2, selected: bool) -> (egui::Response, egui::Rect) {
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    let p = pal();
    let fill = if resp.hovered() { p.widget_hover } else { p.widget_bg };
    ui.painter().rect_filled(rect, egui::CornerRadius::same(12), fill);
    if selected {
        ui.painter().rect_stroke(
            rect,
            egui::CornerRadius::same(12),
            egui::Stroke::new(1.5, p.accent),
            egui::StrokeKind::Inside,
        );
    }
    (resp, rect)
}

/// A small window drawn in one palette: background, panel, two text lines
/// and an accent pill.
fn mini_window(painter: &egui::Painter, rect: egui::Rect, p: Palette) {
    painter.rect_filled(rect, egui::CornerRadius::same(8), p.bg);
    let panel = egui::Rect::from_min_max(
        egui::pos2(rect.min.x + rect.width() * 0.36, rect.min.y + 8.0),
        egui::pos2(rect.max.x - 8.0, rect.max.y - 8.0),
    );
    painter.rect_filled(panel, egui::CornerRadius::same(6), p.panel);
    for (k, w) in [0.5_f32, 0.32, 0.22].iter().enumerate() {
        let y = rect.min.y + 14.0 + k as f32 * 9.0;
        let x0 = rect.min.x + 8.0;
        let bar = egui::Rect::from_min_size(egui::pos2(x0, y), egui::vec2(rect.width() * 0.22 * (1.0 + w), 3.0));
        painter.rect_filled(bar, egui::CornerRadius::same(2), p.text_dim);
    }
    let title = egui::Rect::from_min_size(
        egui::pos2(panel.min.x + 8.0, panel.min.y + 10.0),
        egui::vec2(panel.width() * 0.45, 4.0),
    );
    painter.rect_filled(title, egui::CornerRadius::same(2), p.text);
    for k in 0..2 {
        let line = egui::Rect::from_min_size(
            egui::pos2(panel.min.x + 8.0, panel.min.y + 22.0 + k as f32 * 8.0),
            egui::vec2(panel.width() * (0.7 - 0.2 * k as f32), 3.0),
        );
        painter.rect_filled(line, egui::CornerRadius::same(2), p.text_dim);
    }
    let pill = egui::Rect::from_min_size(
        egui::pos2(panel.min.x + 8.0, panel.max.y - 14.0),
        egui::vec2(16.0, 6.0),
    );
    painter.rect_filled(pill, egui::CornerRadius::same(3), p.accent);
}

fn scheme_card(ui: &mut Ui, w: f32, scheme: usize, theme: usize, selected: bool) -> egui::Response {
    let (resp, rect) = appearance_card(ui, egui::vec2(w, 116.0), selected);
    let p = pal();
    let preview = egui::Rect::from_min_size(rect.min + egui::vec2(10.0, 10.0), egui::vec2(w - 20.0, 72.0));
    let painter = ui.painter();
    match scheme {
        SCHEME_SYSTEM => {
            mini_window(painter, preview, palette_of(theme, true));
            let right = egui::Rect::from_min_max(
                egui::pos2(preview.center().x, preview.min.y),
                preview.max,
            );
            mini_window(&painter.with_clip_rect(right), preview, palette_of(theme, false));
        }
        SCHEME_LIGHT => mini_window(painter, preview, palette_of(theme, false)),
        _ => mini_window(painter, preview, palette_of(theme, true)),
    }
    painter.text(
        egui::pos2(rect.center().x, rect.max.y - 9.0),
        egui::Align2::CENTER_BOTTOM,
        SCHEME_NAMES[scheme],
        egui::FontId::new(13.5, egui::FontFamily::Name("semibold".into())),
        p.text,
    );
    resp
}

fn theme_card(ui: &mut Ui, w: f32, name: &str, k: usize, selected: bool, custom: bool) -> egui::Response {
    let (resp, rect) = appearance_card(ui, egui::vec2(w, 60.0), selected);
    let p = pal();
    let sw = palette_of(k, is_dark());
    let swatch = egui::Rect::from_min_size(rect.min + egui::vec2(10.0, 10.0), egui::vec2(38.0, 38.0));
    ui.painter().rect_filled(swatch, egui::CornerRadius::same(8), sw.bg);
    for j in 0..3 {
        let bar = egui::Rect::from_min_size(
            swatch.min + egui::vec2(8.0, 9.0 + j as f32 * 6.0),
            egui::vec2(if j == 0 { 18.0 } else { 13.0 }, 3.0),
        );
        ui.painter().rect_filled(bar, egui::CornerRadius::same(2), if j == 0 { sw.text } else { sw.text_dim });
    }
    ui.painter().circle_filled(swatch.max - egui::vec2(9.0, 9.0), 4.0, sw.accent);
    ui.painter().text(
        rect.min + egui::vec2(58.0, 22.0),
        egui::Align2::LEFT_CENTER,
        name,
        egui::FontId::new(13.5, egui::FontFamily::Name("semibold".into())),
        p.text,
    );
    ui.painter().text(
        rect.min + egui::vec2(58.0, 40.0),
        egui::Align2::LEFT_CENTER,
        if custom { "Custom" } else { "Built in" },
        egui::FontId::proportional(12.0),
        p.text_dim,
    );
    if selected {
        ui.painter().text(
            rect.max - egui::vec2(12.0, 30.0),
            egui::Align2::CENTER_CENTER,
            "✓",
            egui::FontId::proportional(14.0),
            p.accent,
        );
    }
    resp
}

/// Default location for an exported/imported theme file, in the app's
/// config folder so it is easy to find on both Linux and Windows.
fn theme_json_path(name: &str) -> String {
    let stem: String = name
        .trim()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect();
    let stem = if stem.is_empty() || stem.eq_ignore_ascii_case("themes") { "theme".to_string() } else { stem };
    crate::ui::config_file(&format!("{stem}.json"))
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| format!("{stem}.json"))
}

fn with_json_extension(path: PathBuf) -> PathBuf {
    if path.extension().and_then(|e| e.to_str()) == Some("json") {
        path
    } else {
        path.with_extension("json")
    }
}

fn colour_row(ui: &mut Ui, field: &mut String) {
    ui.horizontal(|ui| {
        let colour = parse_hex(field).unwrap_or(egui::Color32::GRAY);
        let (rect, _) = ui.allocate_exact_size(egui::vec2(28.0, 20.0), egui::Sense::hover());
        ui.painter().rect_filled(rect, egui::CornerRadius::same(5), colour);
        ui.painter().rect_stroke(
            rect,
            egui::CornerRadius::same(5),
            egui::Stroke::new(1.0, c_separator()),
            egui::StrokeKind::Middle,
        );
        ui.add(egui::TextEdit::singleline(field).desired_width(130.0).hint_text("#RRGGBB"));
    });
}

fn import_theme_json(ctx: &egui::Context, value: &serde_json::Value) -> Result<(usize, usize), String> {
    let specs = if value.is_array() || value.get("themes").is_some() {
        theme::custom_themes_from_json(value)?
    } else {
        vec![theme::theme_from_json(value)?]
    };
    let count = specs.len();
    if count == 0 {
        return Err("That file contains no themes".to_string());
    }
    let mut last = 0;
    for spec in specs {
        last = theme::add_or_update_custom(spec, None)?;
    }
    theme::select(ctx, last, scheme_index());
    theme::save_choice(last, scheme_index());
    Ok((last, count))
}

/// The Create theme panel: name, Dark/Light choice, three editable colours,
/// a generate-opposing-colours action, and the bottom action row.
fn create_theme_window(ctx: &egui::Context, app: &mut App) {
    let Some(mut editor) = app.dlg.theme_editor.take() else { return };
    enum Action { None, Save, Remove, Cancel, Export }
    let mut action = Action::None;
    let mut open = true;
    egui::Window::new("Create theme")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_width(440.0)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add_sized(egui::vec2(70.0, 20.0), egui::Label::new("Name"));
                ui.add(egui::TextEdit::singleline(&mut editor.name).desired_width(320.0).hint_text("Theme name"));
            });
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if editor.dark {
                    primary_button(ui, "Dark");
                } else if ui.button("Dark").clicked() {
                    editor.dark = true;
                }
                if !editor.dark {
                    primary_button(ui, "Light");
                } else if ui.button("Light").clicked() {
                    editor.dark = false;
                }
            });
            ui.add_space(8.0);
            egui::Grid::new("create_theme_colours").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                let (label, field) = if editor.dark {
                    ("Background", &mut editor.dark_fields.bg)
                } else {
                    ("Background", &mut editor.light_fields.bg)
                };
                ui.label(label);
                colour_row(ui, field);
                ui.end_row();

                let (label, field) = if editor.dark {
                    ("Text", &mut editor.dark_fields.text)
                } else {
                    ("Text", &mut editor.light_fields.text)
                };
                ui.label(label);
                colour_row(ui, field);
                ui.end_row();

                let (label, field) = if editor.dark {
                    ("Accent", &mut editor.dark_fields.accent)
                } else {
                    ("Accent", &mut editor.light_fields.accent)
                };
                ui.label(label);
                colour_row(ui, field);
                ui.end_row();
            });
            ui.add_space(10.0);
            let generate_label = if editor.dark {
                "Generate dark colours from light"
            } else {
                "Generate Light colours from Dark"
            };
            if primary_button(ui, generate_label).clicked() {
                let source = if editor.dark { &editor.light_fields } else { &editor.dark_fields };
                match source.parse(if editor.dark { "Light" } else { "Dark" }) {
                    Ok(colours) => {
                        let generated = theme::opposing_colours(editor.dark, colours);
                        if editor.dark {
                            editor.dark_fields = ThemeColourFields::from_colours(generated);
                        } else {
                            editor.light_fields = ThemeColourFields::from_colours(generated);
                        }
                    }
                    Err(err) => app.warn(err),
                }
            }
            ui.add_space(14.0);
            ui.separator();
            ui.horizontal(|ui| {
                if ui.add_enabled(editor.origin.is_some(), egui::Button::new("Remove theme")).clicked() {
                    action = Action::Remove;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if primary_button(ui, "Save theme").clicked() {
                        action = Action::Save;
                    }
                    if ui.button("Cancel").clicked() {
                        action = Action::Cancel;
                    }
                    if ui.button("Export").clicked() {
                        action = Action::Export;
                    }
                });
            });
        });

    let mut close = !open;
    match action {
        Action::None => {}
        Action::Cancel => close = true,
        Action::Export => {
            app.dlg.theme_file = Some(ThemeFileState {
                purpose: ThemeFilePurpose::Export,
                path: theme_json_path(&editor.name),
            });
            close = false;
        }
        Action::Save => match editor.spec() {
            Ok(spec) => match theme::add_or_update_custom(spec.clone(), editor.origin) {
                Ok(index) => {
                    theme::select(ctx, index, scheme_index());
                    theme::save_choice(index, scheme_index());
                    app.say(format!("Theme \"{}\" saved", spec.name));
                    close = true;
                }
                Err(err) => app.warn(err),
            },
            Err(err) => app.warn(err),
        },
        Action::Remove => {
            if let Some(origin) = editor.origin {
                let current = theme::theme_index();
                let index = theme::remove_custom(origin, current);
                theme::select(ctx, index, scheme_index());
                theme::save_choice(index, scheme_index());
                app.say("Theme removed");
                close = true;
            } else {
                app.warn("Only custom themes can be removed");
            }
        }
    }
    if !close {
        app.dlg.theme_editor = Some(editor);
    }
}

/// Small in-app path panel used by Export and Import.
fn theme_file_window(ctx: &egui::Context, app: &mut App) {
    let Some(mut state) = app.dlg.theme_file.take() else { return };
    let purpose = state.purpose;
    enum Action { None, Done, Cancel }
    let mut action = Action::None;
    let mut open = true;
    egui::Window::new(purpose.title())
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_width(470.0)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.label("File path");
            ui.add(egui::TextEdit::singleline(&mut state.path).desired_width(420.0).hint_text("~/.config/tagrename/theme.json"));
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if primary_button(ui, purpose.action()).clicked() {
                        action = Action::Done;
                    }
                    if ui.button("Cancel").clicked() {
                        action = Action::Cancel;
                    }
                });
            });
        });

    let mut close = !open;
    match action {
        Action::None => {}
        Action::Cancel => close = true,
        Action::Done => {
            let path = with_json_extension(expand_tilde(&state.path));
            match purpose {
                ThemeFilePurpose::Export => {
                    let result = app
                        .dlg
                        .theme_editor
                        .as_ref()
                        .map(|editor| editor.spec())
                        .unwrap_or_else(|| {
                            theme::spec_of(theme::theme_index())
                                .ok_or_else(|| "No theme to export".to_string())
                        });
                    match result {
                        Ok(spec) => match serde_json::to_string_pretty(&theme::theme_to_json(&spec)) {
                            Ok(json) => {
                                if let Some(dir) = path.parent() {
                                    let _ = std::fs::create_dir_all(dir);
                                }
                                match std::fs::write(&path, json) {
                                    Ok(()) => {
                                        app.say(format!("Theme exported to {}", path.display()));
                                        close = true;
                                    }
                                    Err(err) => app.warn(format!("Could not write theme: {err}")),
                                }
                            }
                            Err(err) => app.warn(format!("Could not build JSON: {err}")),
                        },
                        Err(err) => app.warn(err),
                    }
                }
                ThemeFilePurpose::Import => match std::fs::read_to_string(&path) {
                    Ok(text) => match serde_json::from_str::<serde_json::Value>(&text) {
                        Ok(value) => match import_theme_json(ctx, &value) {
                            Ok((_, count)) => {
                                app.say(format!("Imported {count} theme{}", if count == 1 { "" } else { "s" }));
                                close = true;
                            }
                            Err(err) => app.warn(err),
                        },
                        Err(err) => app.warn(format!("Not a valid theme JSON file: {err}")),
                    },
                    Err(err) => app.warn(format!("Could not read theme file: {err}")),
                },
            }
        }
    }
    if !close {
        app.dlg.theme_file = Some(state);
    }
}

pub fn draw(ctx: &egui::Context, app: &mut App) {
    let results = app.dlg.take_results();

    // --- Rename preview ---
    if app.dlg.show_rename_preview {
        let mut open = true;
        let rows = app.rename_preview_rows();
        egui::Window::new("Rename Files Preview")
            .open(&mut open)
            .default_width(560.0)
            .default_height(320.0)
            .show(ctx, |ui| {
                preview_table(ui, &rows, "Before rename", "After rename");
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Rename now").clicked() {
                        app.do_rename(false);
                        app.dlg.show_rename_preview = false;
                    }
                    if ui.button("Close").clicked() {
                        app.dlg.show_rename_preview = false;
                    }
                });
            });
        app.dlg.show_rename_preview &= open;
    }

    // --- File-to-tag preview ---
    if app.dlg.show_f2t_preview {
        let mut open = true;
        let rows = app.f2t_preview_rows();
        egui::Window::new("File Name to Tag Preview")
            .open(&mut open)
            .default_width(560.0)
            .default_height(320.0)
            .show(ctx, |ui| {
                preview_table(ui, &rows, "File name", "File name as it will look");
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Write tags now").clicked() {
                        app.file_to_tag(false);
                        app.dlg.show_f2t_preview = false;
                    }
                    if ui.button("Close").clicked() {
                        app.dlg.show_f2t_preview = false;
                    }
                });
            });
        app.dlg.show_f2t_preview &= open;
    }

    // --- Mask editor ---
    if let Some(which) = app.dlg.show_mask_editor {
        let mut open = true;
        let mut choice = None;
        egui::Window::new("Mask Editor")
            .open(&mut open)
            .collapsible(false)
            .default_width(780.0)
            .show(ctx, |ui| {
                choice = mask_editor(ui, app, which);
            });
        if choice == Some(false) || (choice.is_none() && !open) {
            let backup = app.dlg.mask_backup.clone();
            if which == 0 {
                app.rename_mask = backup;
            } else {
                app.f2t_mask = backup;
            }
        }
        if choice.is_some() || !open {
            app.dlg.show_mask_editor = None;
        }
    }

    // --- Tag editor ---
    // show() reports whether the editor should stay open; re-inserting
    // it unconditionally used to silently undo every close (the cross,
    // Cancel and Save all "did nothing").
    if let Some(mut ed) = app.dlg.tag_editor.take() {
        if crate::ui_tageditor::show(ctx, app, &mut ed) {
            app.dlg.tag_editor = Some(ed);
        }
    }

    // --- Auto fill ---
    if app.dlg.show_autofill {
        let mut open = true;
        egui::Window::new("Auto Fill Track #")
            .open(&mut open)
            .default_width(320.0)
            .show(ctx, |ui| {
                // The original's dialog (built inline in the handler at
                // 0xcaa73c): the question from resourcestring rec
                // 0x837434, "Start from:" (rec 0x837444) with a spin
                // edit, the "Use xx/xx format" checkbox (rec 0x83744c,
                // persisted as Config `Use_Xx_Xx`) and Yes/No buttons.
                ui.label("Auto fill track # for all selected files?");
                ui.horizontal(|ui| {
                    ui.label("Start from:");
                    ui.add(
                        egui::DragValue::new(&mut app.dlg.autofill_start).range(1..=10000),
                    );
                });
                ui.checkbox(&mut app.dlg.autofill_use_total, "Use xx/xx format");
                // The original's "Don't add leading zero when
                // autofilling track #" lives in Program Options (Config
                // `DontAddLeadingZeroToTrackNr`, default off = zeros
                // added), not in this dialog; width 2/3/4 by FOLDER scope:
                // sub_caa618/sub_caa6a8, verified at 0xcab300.
                ui.separator();
                ui.horizontal(|ui| {
                    if primary_button(ui, "Yes").clicked() {
                        let opts = crate::tools::AutoFill {
                            start: app.dlg.autofill_start,
                            use_total: app.dlg.autofill_use_total,
                            leading_zero: !app.dont_add_leading_zero_track,
                        };
                        app.auto_fill(opts);
                        app.dlg.show_autofill = false;
                    }
                    if ui.button("No").clicked() {
                        app.dlg.show_autofill = false;
                    }
                });
            });
        app.dlg.show_autofill &= open;
    }

    // --- Export ---
    if app.dlg.show_export {
        let mut open = true;
        egui::Window::new("Export/Import Tags")
            .open(&mut open)
            .default_width(520.0)
            .default_height(430.0)
            .show(ctx, |ui| {
                ui.label("Select what fields to export:");
                egui::ScrollArea::vertical()
                    .id_salt("export_fields")
                    .max_height(200.0)
                    .show(ui, |ui| {
                        egui::Grid::new("export_grid").num_columns(3).show(ui, |ui| {
                            for slot in app.dlg.export_fields.iter_mut() {
                                ui.checkbox(&mut slot.1, slot.0.label());
                                ui.end_row();
                            }
                        });
                    });
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Export file name:");
                    ui.add(
                        egui::TextEdit::singleline(&mut app.dlg.export_name)
                            .desired_width(160.0),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("Format:");
                    for fmt in [
                        ExportFormat::Csv,
                        ExportFormat::Txt,
                        ExportFormat::Html,
                        ExportFormat::Xml,
                    ] {
                        if ui
                            .radio(app.dlg.export_format == fmt, fmt.label())
                            .clicked()
                        {
                            app.dlg.export_format = fmt;
                        }
                    }
                });
                ui.separator();
                if ui.button("Export tags information").clicked() {
                    let fields: Vec<Field> = app
                        .dlg
                        .export_fields
                        .iter()
                        .filter(|(_, on)| *on)
                        .map(|(f, _)| *f)
                        .collect();
                    if fields.is_empty() {
                        app.warn("Nothing to export! Select what fields to export at first!");
                    } else {
                        app.dlg.export_pending_fields = Some(fields);
                        app.dlg.choose = Some(ChooseState {
                            purpose: ChoosePurpose::SaveExport,
                            dir: default_start_dir(app),
                            selected: None,
                            name: app.dlg.export_name.clone(),
                            cancelled: false,
                            path_edit: String::new(),
                        });
                    }
                }
            });
        app.dlg.show_export &= open;
    }

    // --- Find ---
    if app.dlg.show_find {
        let mut open = true;
        egui::Window::new("Find")
            .open(&mut open)
            .default_width(360.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Find what:");
                    ui.text_edit_singleline(&mut app.dlg.find_text);
                });
                ui.checkbox(&mut app.dlg.find_match_case, "Match case");
                ui.separator();
                if ui.button("Find Next").clicked() {
                    let needle = if app.dlg.find_match_case {
                        app.dlg.find_text.clone()
                    } else {
                        app.dlg.find_text.to_lowercase()
                    };
                    if needle.is_empty() {
                        return;
                    }
                    let rows = app.visible_rows();
                    if rows.is_empty() {
                        return;
                    }
                    let cur = app.focused.unwrap_or(usize::MAX);
                    let start_pos = rows.iter().position(|r| *r == cur).map(|p| p + 1).unwrap_or(0);
                    let mut hit = None;
                    for k in 0..rows.len() {
                        let i = rows[(start_pos + k) % rows.len()];
                        if i == cur && k > 0 {
                            continue;
                        }
                        let f = &app.files[i];
                        let hay = format!(
                            "{}{}{}{}{}",
                            f.name, f.fields.artist, f.fields.title, f.fields.album, f.fields.comment
                        );
                        let hay = if app.dlg.find_match_case {
                            hay
                        } else {
                            hay.to_lowercase()
                        };
                        if hay.contains(&needle) {
                            hit = Some(i);
                            break;
                        }
                    }
                    match hit {
                        Some(i) => {
                            app.focused = Some(i);
                            app.say(format!("Found in: {}", app.files[i].name));
                        }
                        None => app.say("No more files!"),
                    }
                }
            });
        app.dlg.show_find &= open;
    }

    // --- Appearance ---
    if app.dlg.show_appearance {
        appearance_window(ctx, app);
    }
    if app.dlg.theme_editor.is_some() {
        create_theme_window(ctx, app);
    }
    if app.dlg.theme_file.is_some() {
        theme_file_window(ctx, app);
    }

    // --- Options ---
    if app.dlg.show_options {
        let mut open = true;
        egui::Window::new("Program Options")
            .open(&mut open)
            .default_width(430.0)
            .show(ctx, |ui| {
                egui::Grid::new("opts").num_columns(1).show(ui, |ui| {
                    ui.checkbox(
                        &mut app.remove_leading_zero_disc,
                        "Always remove leading zero from disc # value when renaming files and folders",
                    );
                    ui.end_row();
                    ui.checkbox(
                        &mut app.dont_add_leading_zero_track,
                        "Don't add leading zero when autofilling track #",
                    );
                    ui.end_row();
                    ui.checkbox(
                        &mut app.update_id3v1_if_exists,
                        "Update ID3v1 tag only if it already exists",
                    );
                    ui.end_row();
                    ui.checkbox(&mut app.hide_extensions, "Hide files extension");
                    ui.end_row();
                    ui.checkbox(&mut app.search_subfolders, "Search subfolders");
                    ui.end_row();
                    ui.checkbox(&mut app.show_tools_bar, "Show Tools Bar (F8)");
                    ui.end_row();
                    ui.checkbox(&mut app.show_folders, "Show Folders panel");
                    ui.end_row();
                });
            });
        app.dlg.show_options &= open;
    }

    // --- Genre list editor ---
    if app.dlg.show_genre_editor {
        let mut open = true;
        egui::Window::new("Genre List Editor")
            .open(&mut open)
            .default_width(340.0)
            .default_height(380.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Add genre:");
                    ui.text_edit_singleline(&mut app.dlg.new_genre);
                    if ui.button("Add").clicked() && !app.dlg.new_genre.is_empty() {
                        let g = app.dlg.new_genre.clone();
                        if !app.genres.iter().any(|x| x == &g) {
                            app.genres.push(g);
                            app.genres.sort();
                        }
                        app.dlg.new_genre.clear();
                    }
                });
                ui.separator();
                let mut remove: Option<String> = None;
                egui::ScrollArea::vertical()
                    .id_salt("genres")
                    .show(ui, |ui| {
                        for g in app.genres.clone() {
                            ui.horizontal(|ui| {
                                ui.label(&g);
                                if ui.small_button("Remove").clicked() {
                                    remove = Some(g);
                                }
                            });
                        }
                    });
                if let Some(g) = remove {
                    app.genres.retain(|x| x != &g);
                }
            });
        app.dlg.show_genre_editor &= open;
    }

    // --- About ---
    if app.dlg.show_about {
        let mut open = true;
        egui::Window::new("About")
            .open(&mut open)
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(6.0);
                    ui.label(RichText::new("Tag:Rename 1.1.0").strong().size(22.0));
                    ui.label(
                        RichText::new("Music tag editor — modern dark edition").color(c_text_dim()),
                    );
                    ui.add_space(10.0);
                    ui.label(
                        "An offline music organizer: edit tags, rename files from tags,\n\
                         read tags from file names, manage cover art and export your library.\n\n\
                         Supports these audio formats:\n\
                         mp3, ogg, flac, m4a/mp4, wma, ape, mpc, wavpack, wav and aiff.",
                    );
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(
                            "Built with Rust and egui.",
                        )
                        .color(c_text_dim())
                        .size(13.0),
                    );
                });
            });
        app.dlg.show_about &= open;
    }

    // --- Remove-tags confirmation ---
    if let Some(indices) = app.dlg.confirm_remove.clone() {
        let mut open = true;
        egui::Window::new("Remove Tags")
            .open(&mut open)
            .default_width(380.0)
            .show(ctx, |ui| {
                ui.label(format!(
                    "Do you really want to remove tags from all {} selected files (cannot be undone)?",
                    indices.len()
                ));
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Yes").clicked() {
                        let mut done = 0;
                        let mut errors = Vec::new();
                        for i in &indices {
                            let f = &app.files[*i];
                            match crate::data::remove_tags(&f.path) {
                                Ok(()) => {
                                    if let Some(re) = crate::data::read_file(&f.path) {
                                        app.files[*i] = re;
                                    }
                                    done += 1;
                                }
                                Err(e) => errors.push(e),
                            }
                        }
                        if done > 0 {
                            app.say(format!("Tags removed from {done} file(s)."));
                        }
                        if !errors.is_empty() {
                            app.warn(errors.join("\n"));
                        }
                        app.dlg.confirm_remove = None;
                    }
                    if ui.button("No").clicked() {
                        app.dlg.confirm_remove = None;
                    }
                });
            });
        if !open {
            app.dlg.confirm_remove = None;
        }
    }

    // --- Advanced... (swap any two fields) ---
    if app.dlg.swap_advanced {
        let mut open = true;
        egui::Window::new("Swap Tag Fields")
            .open(&mut open)
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.label("Swap the text of two tag fields in all selected files:");
                ui.separator();
                field_pick(ui, app, "First field:", 'a');
                field_pick(ui, app, "Second field:", 'b');
                ui.separator();
                if ui.button("Swap").clicked() {
                    let a = ALL_FIELDS[app.dlg.swap_from.min(ALL_FIELDS.len() - 1)];
                    let b = ALL_FIELDS[app.dlg.swap_to.min(ALL_FIELDS.len() - 1)];
                    app.swap_selected(a, b);
                    app.dlg.swap_advanced = false;
                }
            });
        if !open {
            app.dlg.swap_advanced = false;
        }
    }

    // --- Copy Text from One Tag Field to Another ---
    if app.dlg.copy_field_open {
        let mut open = true;
        egui::Window::new("Copy Text from One Tag Field to Another")
            .open(&mut open)
            .default_width(420.0)
            .show(ctx, |ui| {
                ui.label("Copy the text of one tag field into another in all selected files:");
                ui.separator();
                field_pick(ui, app, "Copy from:", 'a');
                field_pick(ui, app, "Copy to:", 'b');
                ui.separator();
                if ui.button("Copy").clicked() {
                    let a = ALL_FIELDS[app.dlg.swap_from.min(ALL_FIELDS.len() - 1)];
                    let b = ALL_FIELDS[app.dlg.swap_to.min(ALL_FIELDS.len() - 1)];
                    app.copy_field_selected(a, b);
                    app.dlg.copy_field_open = false;
                }
            });
        if !open {
            app.dlg.copy_field_open = false;
        }
    }

    // --- File / folder chooser ---
    if let Some(mut state) = app.dlg.choose.take() {
        let purpose = state.purpose.clone();
        let mut done: Option<ChooseDone> = None;
        egui::Window::new(purpose.title())
            .default_width(460.0)
            .default_height(400.0)
            .show(ctx, |ui| {
                chooser_body(ui, app, &mut state, &mut done);
            });
        match done {
            Some(d) => {
                handle_choose(app, purpose, d);
                app.dlg.choose = None;
                app.dlg.pick_cover = false;
            }
            None => {
                if state.cancelled {
                    app.dlg.choose = None;
                    app.dlg.pick_cover = false;
                } else {
                    app.dlg.choose = Some(state);
                }
            }
        }
    }

    // Apply tag-editor results after all dialogs are drawn.
    for r in results {
        crate::ui_tageditor::apply_result(app, r);
    }
}

fn handle_choose(app: &mut App, purpose: ChoosePurpose, done: ChooseDone) {
    match (purpose, done) {
        (ChoosePurpose::OpenFolder, ChooseDone::Folder(p)) => {
            app.load_folder(&p);
        }
        (ChoosePurpose::AddRoot, ChooseDone::Folder(p)) => {
            app.add_root(&p);
        }
        (ChoosePurpose::SavePlaylist(indices), ChooseDone::SavePath(p)) => {
            app.save_playlist(&indices, &p);
        }
        (ChoosePurpose::SaveExport, ChooseDone::SavePath(p)) => {
            let fields = app.dlg.export_pending_fields.clone().unwrap_or_default();
            let fmt = app.dlg.export_format;
            let indices: Vec<usize> = if app.checked_indices().is_empty() {
                (0..app.files.len()).collect()
            } else {
                app.checked_indices()
            };
            if indices.is_empty() {
                app.warn("There is no files in program file list!");
            } else {
                app.export(&indices, &fields, &p, fmt);
            }
        }
        (ChoosePurpose::SaveCover, ChooseDone::SavePath(p)) => {
            if let Some(bytes) = app.dlg.save_cover_bytes.take() {
                match std::fs::write(&p, bytes) {
                    Ok(()) => app.say("Cover art saved."),
                    Err(e) => app.warn(format!("Can't save cover art: {e}")),
                }
            }
        }
        (ChoosePurpose::PickCover, ChooseDone::Bytes(b)) => {
            crate::ui_tageditor::stage_cover(app, b);
        }
        (ChoosePurpose::LoadLyrics, ChooseDone::Bytes(b)) => {
            let text = String::from_utf8_lossy(&b).into_owned();
            match app.dlg.tag_editor.as_mut() {
                Some(ed) => {
                    ed.values.lyrics = text;
                    app.say("Lyrics loaded into the editor.");
                }
                None => app.warn("The Tag Editor was closed, so the lyrics were not loaded."),
            }
        }
        (ChoosePurpose::SaveLyrics, ChooseDone::SavePath(p)) => {
            if let Some(text) = app.dlg.save_lyrics_text.take() {
                match std::fs::write(&p, text) {
                    Ok(()) => app.say(format!("Lyrics saved to \"{}\"", p.display())),
                    Err(e) => app.warn(format!("Can't save lyrics: {e}")),
                }
            }
        }
        _ => {}
    }
}

fn chooser_body(ui: &mut Ui, app: &mut App, state: &mut ChooseState, done: &mut Option<ChooseDone>) {
    ui.horizontal(|ui| {
        if ui.button("Up").clicked() {
            if let Some(parent) = state.dir.parent() {
                state.dir = parent.to_path_buf();
            }
            state.selected = None;
        }
        if ui.button("Home").clicked() {
            state.dir = default_start_dir(app);
            state.selected = None;
        }
        if ui.button("Cancel").clicked() {
            state.cancelled = true;
        }
    });
    // The paste-a-location row: the same idea as the address bar above
    // the file tree, so a deep folder (e.g. /mnt/pool/Music) is one
    // paste away instead of a chain of Up clicks.
    let mut go = false;
    ui.horizontal(|ui| {
        ui.label(RichText::new("Location:").color(c_text_dim()).size(13.5));
        let edit = egui::TextEdit::singleline(&mut state.path_edit)
            .desired_width((ui.available_width() - 42.0).max(90.0))
            .hint_text("Paste a folder location…");
        let resp = ui.add(edit);
        if (resp.lost_focus() && ui.ctx().input(|i| i.key_pressed(egui::Key::Enter)))
            || ui.small_button("Go").clicked()
        {
            go = true;
        }
    });
    if go {
        let raw = state.path_edit.trim().to_string();
        let path = expand_tilde(&raw);
        if path.is_dir() {
            if matches!(
                state.purpose,
                ChoosePurpose::OpenFolder | ChoosePurpose::AddRoot
            ) {
                *done = Some(ChooseDone::Folder(path));
            } else {
                state.dir = path;
                state.selected = None;
            }
        } else {
            app.warn(format!(
                "Cannot open \"{raw}\".\nThe folder does not exist, or it is outside the folders this app is allowed to see.\n\
                 (A mounted drive beyond your home, like /mnt/pool, appears after Craft Anything itself is restarted:\n\
                 save your work, quit it completely — including any tray icon — and start it again.)"
            ));
        }
    }
    ui.label(
        RichText::new(state.dir.display().to_string())
            .color(c_text_dim())
            .size(13.5),
    );
    ui.separator();

    let mut entries: Vec<(bool, PathBuf, String)> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&state.dir) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            let path = e.path();
            let is_dir = path.is_dir();
            let is_image = path
                .extension()
                .and_then(|x| x.to_str())
                .map(|x| {
                    matches!(
                        x.to_ascii_lowercase().as_str(),
                        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp"
                    )
                })
                .unwrap_or(false);
            let is_text = path
                .extension()
                .and_then(|x| x.to_str())
                .map(|x| matches!(x.to_ascii_lowercase().as_str(), "txt" | "lrc"))
                .unwrap_or(false);
            match state.purpose {
                ChoosePurpose::OpenFolder | ChoosePurpose::AddRoot => {
                    if is_dir && !name.starts_with('.') {
                        entries.push((true, path, name));
                    }
                }
                ChoosePurpose::SavePlaylist(_)
                | ChoosePurpose::SaveExport
                | ChoosePurpose::SaveCover
                | ChoosePurpose::SaveLyrics => {
                    if is_dir && !name.starts_with('.') {
                        entries.push((true, path, name));
                    }
                }
                ChoosePurpose::PickCover => {
                    if (is_dir || is_image) && (!name.starts_with('.') || is_dir) {
                        entries.push((is_dir, path, name));
                    }
                }
                ChoosePurpose::LoadLyrics => {
                    if (is_dir || is_text) && (!name.starts_with('.') || is_dir) {
                        entries.push((is_dir, path, name));
                    }
                }
            }
        }
    }
    entries.sort_by(|a, b| {
        (!a.0)
            .cmp(&(!b.0))
            .then(a.2.to_lowercase().cmp(&b.2.to_lowercase()))
    });

    let mut clicked: Option<(bool, PathBuf)> = None;
    egui::ScrollArea::vertical()
        .id_salt("chooser_list")
        .show(ui, |ui| {
            for (is_dir, path, name) in &entries {
                let icon = if *is_dir {
                    "[dir] "
                } else if matches!(state.purpose, ChoosePurpose::LoadLyrics) {
                    "[txt] "
                } else {
                    "[img] "
                };
                if ui
                    .selectable_label(
                        state.selected.as_deref() == Some(path.as_path()),
                        format!("{icon}{name}"),
                    )
                    .clicked()
                {
                    clicked = Some((*is_dir, path.clone()));
                }
            }
        });
    ui.separator();

    match state.purpose {
        ChoosePurpose::OpenFolder | ChoosePurpose::AddRoot => {
            if let Some((true, path)) = clicked.clone() {
                state.selected = Some(path);
            }
            let add_root = matches!(state.purpose, ChoosePurpose::AddRoot);
            ui.horizontal(|ui| {
                let label = state
                    .selected
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "select a folder above".into());
                ui.label(RichText::new(label).color(c_text_dim()).size(13.5));
                if let Some(st) = state.selected.clone() {
                    if ui
                        .button(if add_root { "Add this root" } else { "Open this folder" })
                        .clicked()
                    {
                        *done = Some(ChooseDone::Folder(st));
                    }
                }
            });
            if add_root {
                ui.label(
                    RichText::new("Adding a root only adds it to the tree; choose a folder inside it to load music.")
                        .color(c_text_dim())
                        .size(13.0),
                );
            }
        }
        ChoosePurpose::PickCover => {
            if let Some((false, path)) = &clicked {
                state.selected = Some(path.clone());
            }
            if let Some(sel) = state.selected.clone() {
                if ui.button("Use this image").clicked() {
                    if let Some(bytes) = std::fs::read(&sel).ok() {
                        *done = Some(ChooseDone::Bytes(bytes));
                    } else {
                        app.warn("Couldn't read that file.");
                    }
                }
            } else {
                ui.label("Select an image file to use as cover art.");
            }
        }
        ChoosePurpose::LoadLyrics => {
            if let Some((false, path)) = &clicked {
                state.selected = Some(path.clone());
            }
            if let Some(sel) = state.selected.clone() {
                if ui.button("Use this file").clicked() {
                    if let Some(bytes) = std::fs::read(&sel).ok() {
                        *done = Some(ChooseDone::Bytes(bytes));
                    } else {
                        app.warn("Couldn't read that file.");
                    }
                }
            } else {
                ui.label("Select a .txt or .lrc file with the lyrics.");
            }
        }
        _ => {
            ui.horizontal(|ui| {
                ui.label("File name:");
                ui.text_edit_singleline(&mut state.name);
            });
            ui.horizontal(|ui| {
                if ui.button("Save here").clicked() && !state.name.trim().is_empty() {
                    let mut file = state.dir.join(state.name.trim());
                    match state.purpose {
                        ChoosePurpose::SavePlaylist(_) => {
                            if file.extension().is_none() {
                                file.set_extension("m3u");
                            }
                        }
                        ChoosePurpose::SaveExport => {
                            if file.extension().is_none() {
                                file.set_extension(app.dlg.export_format.ext());
                            }
                        }
                        ChoosePurpose::SaveCover => {
                            if file.extension().is_none() {
                                file.set_extension("jpg");
                            }
                        }
                        ChoosePurpose::SaveLyrics => {
                            if file.extension().is_none() {
                                file.set_extension("txt");
                            }
                        }
                        _ => {}
                    }
                    *done = Some(ChooseDone::SavePath(file));
                }
            });
        }
    }

    // Entering a folder happens on click (single-click navigation).
    if let Some((true, path)) = clicked {
        if path.is_dir() {
            state.dir = path;
            state.selected = None;
        }
    }
}

/// `~` expands to the home folder (the address bar and the chooser's
/// Location row both accept it).
fn expand_tilde(raw: &str) -> PathBuf {
    if let Some(rest) = raw.strip_prefix('~') {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(rest.trim_start_matches('/'));
        }
    }
    PathBuf::from(raw)
}

pub fn default_start_dir(app: &App) -> PathBuf {
    if let Some(folder) = &app.folder {
        if folder.is_dir() {
            return folder.clone();
        }
    }
    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        if home.is_dir() {
            let music = home.join("Music");
            if music.is_dir() {
                return music;
            }
            return home;
        }
    }
    PathBuf::from("/")
}

/// One "pick a tag field" combo row used by the swap/copy dialogs.
fn field_pick(ui: &mut Ui, app: &mut App, label: &str, which: char) {
    ui.horizontal(|ui| {
        ui.label(label);
        let current = if which == 'a' {
            app.dlg.swap_from
        } else {
            app.dlg.swap_to
        };
        let selected = ALL_FIELDS[current.min(ALL_FIELDS.len() - 1)];
        egui::ComboBox::from_id_salt(format!("field_pick_{which}"))
            .selected_text(selected.label())
            .width(240.0)
            .show_ui(ui, |ui| {
                for (i, f) in ALL_FIELDS.iter().enumerate() {
                    if ui.selectable_label(current == i, f.label()).clicked() {
                        if which == 'a' {
                            app.dlg.swap_from = i;
                        } else {
                            app.dlg.swap_to = i;
                        }
                    }
                }
            });
    });
}

fn preview_table(ui: &mut Ui, rows: &[(String, String)], left: &str, right: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(left).color(c_text_dim()).strong());
        ui.add_space(260.0);
        ui.label(RichText::new(right).color(c_text_dim()).strong());
    });
    ui.separator();
    egui::ScrollArea::vertical().show(ui, |ui| {
        egui::Grid::new("preview_grid")
            .num_columns(2)
            .spacing([24.0, 3.0])
            .show(ui, |ui| {
                for (before, after) in rows {
                    ui.label(before);
                    ui.label(RichText::new(after).color(c_ok()));
                    ui.end_row();
                }
            });
    });
}

/// The variables the Mask Editor can insert, with the rename or the
/// file-name-to-tag token set.
fn mask_variables(which: usize) -> Vec<(&'static str, &'static str)> {
    if which == 0 {
        vec![
            ("Artist (%1)", "%1"),
            ("Title (%2)", "%2"),
            ("Album (%3)", "%3"),
            ("Year (%4)", "%4"),
            ("Genre (%5)", "%5"),
            ("Track # (%6)", "%6"),
            ("File name (%7)", "%7"),
            ("File number (%8)", "%8"),
            ("Comment (%9)", "%9"),
            ("Bitrate (%0)", "%0"),
            ("Play time (%t)", "%t"),
            ("BPM (%b)", "%b"),
            ("Disc # (%#)", "%#"),
            ("Album Artist (%aa)", "%aa"),
            ("Conductor (%co)", "%co"),
            ("Composer (%cp)", "%cp"),
            ("Rating (%rt)", "%rt"),
            ("Mood (%md)", "%md"),
            ("Label (%lb)", "%lb"),
            ("Remixed by (%rb)", "%rb"),
            ("Initial Key (%key)", "%key"),
            ("File Ext. (%ext)", "%ext"),
            ("Hz (%khz)", "%khz"),
            ("Bit Depth (%dbit)", "%dbit"),
            ("Encoder (%enc)", "%enc"),
            ("Random (%rnd)", "%rnd"),
        ]
    } else {
        vec![
            ("Artist (%1)", "%1"),
            ("Title (%2)", "%2"),
            ("Album (%3)", "%3"),
            ("Year (%4)", "%4"),
            ("Genre (%5)", "%5"),
            ("Track # (%6)", "%6"),
            ("Comment (%9)", "%9"),
            ("BPM (%b)", "%b"),
            ("Disc # (%#)", "%#"),
            ("Album Artist (%aa)", "%aa"),
            ("Conductor (%co)", "%co"),
            ("Composer (%cp)", "%cp"),
            ("Skip part (%d)", "%d"),
        ]
    }
}

/// Put `insert` where the cursor was left in the mask box, replacing any
/// selected text. With no cursor yet it goes at the end. The cursor then
/// stays right after the inserted text.
fn insert_at_cursor(ctx: &egui::Context, id: egui::Id, text: &mut String, insert: &str) {
    use egui::text::CCursor;
    use egui::text_selection::CCursorRange;
    let len = text.chars().count();
    let mut state = egui::text_edit::TextEditState::load(ctx, id).unwrap_or_default();
    let (start, end) = match state.cursor.char_range() {
        Some(r) => {
            let (p, s) = (usize::from(r.primary.index), usize::from(r.secondary.index));
            let a = p.min(s).min(len);
            let b = p.max(s).min(len);
            (a, b)
        }
        None => (len, len),
    };
    let before: String = text.chars().take(start).collect();
    let after: String = text.chars().skip(end).collect();
    *text = format!("{before}{insert}{after}");
    let pos = start + insert.chars().count();
    state.cursor.set_char_range(Some(CCursorRange::one(CCursor::new(pos))));
    state.store(ctx, id);
    ctx.memory_mut(|m| m.request_focus(id));
}

/// The Mask Editor: the mask box, the variables to insert, saved masks,
/// the result example and OK / Cancel. Returns Some(true) for OK,
/// Some(false) for Cancel, and None while it stays open.
fn mask_editor(ui: &mut Ui, app: &mut App, which: usize) -> Option<bool> {
    let mut mask_text = if which == 0 {
        app.rename_mask.clone()
    } else {
        app.f2t_mask.clone()
    };
    let mut result = None;
    let mut insert: Option<&'static str> = None;
    let mut add_current = false;
    let mut remove_selected = false;
    let mut pick: Option<usize> = None;
    let mut insert_folder = false;
    let variables = mask_variables(which);

    let edit_id = egui::Id::new(("mask_editor_text", which));
    ui.horizontal(|ui| {
        ui.label(RichText::new("File name mask:").color(c_text()).strong());
        ui.add(
            egui::TextEdit::singleline(&mut mask_text)
                .id(edit_id)
                .desired_width(ui.available_width())
                .hint_text("e.g. %6 - %1 - %2"),
        );
    });
    ui.add_space(8.0);

    ui.horizontal_top(|ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new("Variables").color(c_text_dim()).strong());
            egui::Grid::new("mask_vars")
                .num_columns(3)
                .spacing([8.0, 6.0])
                .show(ui, |ui| {
                    for (i, (label, token)) in variables.iter().enumerate() {
                        if ui
                            .add(egui::Button::new(*label).min_size(egui::vec2(168.0, 0.0)))
                            .clicked()
                        {
                            insert = Some(token);
                        }
                        if i % 3 == 2 {
                            ui.end_row();
                        }
                    }
                });
        });
        ui.add_space(14.0);
        ui.vertical(|ui| {
            ui.label(RichText::new("Saved masks").color(c_text_dim()).strong());
            egui::Frame::new()
                .fill(c_panel())
                .stroke(egui::Stroke::new(1.0, c_separator()))
                .inner_margin(egui::Margin::same(6))
                .corner_radius(egui::CornerRadius::same(6))
                .show(ui, |ui| {
                    ui.set_min_size(egui::vec2(230.0, 150.0));
                    egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                        if app.dlg.saved_masks.is_empty() {
                            ui.label(RichText::new("No saved masks yet.").color(c_text_dim()));
                        }
                        for (i, saved) in app.dlg.saved_masks.iter().enumerate() {
                            if ui
                                .selectable_label(app.dlg.saved_mask_sel == Some(i), saved)
                                .clicked()
                            {
                                pick = Some(i);
                            }
                        }
                    });
                });
            ui.horizontal(|ui| {
                if ui.button("Add Current Mask").clicked() {
                    add_current = true;
                }
                let can_remove = app.dlg.saved_mask_sel.is_some();
                if ui
                    .add_enabled(can_remove, egui::Button::new("Remove Mask"))
                    .clicked()
                {
                    remove_selected = true;
                }
            });
        });
    });
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        if ui.button("Insert Folder into Mask").clicked() {
            insert_folder = true;
        }
        ui.label(
            RichText::new("A \\ makes a folder, e.g. %1\\%2 puts each file in an artist folder.")
                .color(c_text_dim()),
        );
    });

    if let Some(tok) = insert {
        insert_at_cursor(ui.ctx(), edit_id, &mut mask_text, tok);
    }
    if insert_folder {
        insert_at_cursor(ui.ctx(), edit_id, &mut mask_text, "\\");
    }
    if let Some(i) = pick {
        mask_text = app.dlg.saved_masks[i].clone();
        app.dlg.saved_mask_sel = Some(i);
    }
    if add_current {
        let trimmed = mask_text.trim().to_string();
        if !trimmed.is_empty() && !app.dlg.saved_masks.contains(&trimmed) {
            app.dlg.saved_masks.push(trimmed);
        }
    }
    if remove_selected {
        if let Some(i) = app.dlg.saved_mask_sel.take() {
            if i < app.dlg.saved_masks.len() {
                app.dlg.saved_masks.remove(i);
            }
        }
    }

    if which == 0 {
        app.rename_mask = mask_text.clone();
    } else {
        app.f2t_mask = mask_text.clone();
    }

    ui.add_space(6.0);
    ui.separator();
    ui.horizontal(|ui| {
        ui.label(RichText::new("Result example:").color(c_text_dim()).strong());
        let example = if which == 0 {
            crate::ui_tabs::rename_example(app, &mask_text)
        } else {
            crate::ui_tabs::f2t_example(app, &mask_text)
        };
        ui.label(RichText::new(example).color(c_ok()).size(15.0));
    });
    ui.separator();
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if primary_button(ui, "OK").clicked() {
                result = Some(true);
            }
            if ui.button("Cancel").clicked() {
                result = Some(false);
            }
        });
    });
    result
}

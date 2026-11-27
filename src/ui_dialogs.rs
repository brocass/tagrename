//! All secondary windows: previews, mask editor, export, auto fill, find,
//! folder/file chooser, options, genre editor, about, confirmations.

use std::path::{Path, PathBuf};

use egui::{RichText, Ui};

use crate::data::{Field, EXPORT_FIELDS, ALL_FIELDS};
use crate::mask;
use crate::native_picker;
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

/// Where a picked cover image should be applied.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CoverPickTarget {
    #[default]
    TagEditor,
    FileCover,
    FolderCover,
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

/// Which mask the editor is editing. Folder renaming needs its own mask
/// (and its own saved value) so F6/F7 never overwrite the file masks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaskKind {
    Rename,
    FileToTag,
    Folder,
}

impl MaskKind {
    fn id(self) -> u8 {
        match self {
            MaskKind::Rename => 0,
            MaskKind::FileToTag => 1,
            MaskKind::Folder => 2,
        }
    }
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
    /// Selection before the editor opened, restored when the editor is cancelled.
    before_theme: usize,
    before_scheme: usize,
    /// Whether an unsaved palette has been pushed into the live preview.
    previewing: bool,
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

    /// Push the current colours into the live preview. An incomplete hex
    /// value leaves the previous preview in place while the person types.
    fn apply_preview(&mut self, ctx: &egui::Context) {
        if let (Ok(dark), Ok(light)) =
            (self.dark_fields.parse("Dark"), self.light_fields.parse("Light"))
        {
            theme::preview(
                ctx,
                &CustomThemeSpec { name: self.name.clone(), dark, light },
                self.dark,
            );
            self.previewing = true;
        }
    }
}

/// One of the three editable colour boxes in the Create theme panel.
#[derive(Clone, Copy)]
struct ColourSlot {
    dark: bool,
    /// 0 = background, 1 = text, 2 = accent.
    field: u8,
}

impl ColourSlot {
    fn field_mut<'a>(self, editor: &'a mut ThemeEditorState) -> &'a mut String {
        let fields = if self.dark { &mut editor.dark_fields } else { &mut editor.light_fields };
        match self.field {
            0 => &mut fields.bg,
            1 => &mut fields.text,
            _ => &mut fields.accent,
        }
    }
}

/// State for the in-app colour picker opened by a hex colour box.
struct ColourPickerState {
    slot: ColourSlot,
    h: f32,
    s: f32,
    v: f32,
    hex: String,
    /// The value when the picker opened, for the old/new preview and Cancel.
    before: String,
    /// Whether the editor was already previewing when the picker opened.
    previewing_before: bool,
}

impl ColourPickerState {
    fn new(slot: ColourSlot, field: &str, previewing_before: bool) -> Self {
        let colour = parse_hex(field).unwrap_or(egui::Color32::GRAY);
        let (h, s, v) = rgb_to_hsv(colour);
        Self {
            slot,
            h,
            s,
            v,
            hex: hex_string(colour),
            before: field.to_string(),
            previewing_before,
        }
    }

    fn colour(&self) -> egui::Color32 {
        hsv_to_colour(self.h, self.s, self.v)
    }

    fn set_hsv(&mut self, h: f32, s: f32, v: f32) {
        self.h = h.clamp(0.0, 1.0);
        self.s = s.clamp(0.0, 1.0);
        self.v = v.clamp(0.0, 1.0);
        self.hex = hex_string(self.colour());
    }

    fn set_colour(&mut self, colour: egui::Color32) {
        let (h, s, v) = rgb_to_hsv(colour);
        self.set_hsv(h, s, v);
    }
}

/// HSV values here use gamma-space RGB bytes, matching the `#RRGGBB` boxes.
fn rgb_to_hsv(colour: egui::Color32) -> (f32, f32, f32) {
    let r = colour.r() as f32 / 255.0;
    let g = colour.g() as f32 / 255.0;
    let b = colour.b() as f32 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let h = if delta == 0.0 {
        0.0
    } else if max == r {
        ((g - b) / delta).rem_euclid(6.0) / 6.0
    } else if max == g {
        ((b - r) / delta + 2.0) / 6.0
    } else {
        ((r - g) / delta + 4.0) / 6.0
    };
    let s = if max == 0.0 { 0.0 } else { delta / max };
    (h, s, max)
}

fn hsv_to_colour(h: f32, s: f32, v: f32) -> egui::Color32 {
    let h = h.clamp(0.0, 1.0).rem_euclid(1.0) * 6.0;
    let sector = h.floor() as u32 % 6;
    let f = h - h.floor();
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    let (r, g, b) = match sector {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    let byte = |x: f32| (x.clamp(0.0, 1.0) * 255.0).round() as u8;
    egui::Color32::from_rgb(byte(r), byte(g), byte(b))
}

pub struct Dialogs {
    pub tag_editor: Option<crate::ui_tageditor::TagEditor>,
    /// The Tag Editor's size when it is not full screen; kept while the
    /// app runs so the next opening starts at the same size.
    pub editor_size: Option<egui::Vec2>,
    pub show_mask_editor: Option<MaskKind>,
    /// Folder whose name F6/F7 will change once the mask is accepted.
    pub folder_rename_target: Option<PathBuf>,
    /// File indices that F6 should rename when the mask is accepted.
    pub file_rename_targets: Vec<usize>,
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
    pub colour_picker: Option<ColourPickerState>,
    pub theme_file: Option<ThemeFileState>,
    pub find_text: String,
    pub find_match_case: bool,
    pub show_about: bool,
    pub show_options: bool,
    pub choose: Option<ChooseState>,
    /// One OS-native picker at a time. `native_theme` says whether its
    /// result belongs to the theme path row or to the general chooser.
    pub native_pick: native_picker::State,
    pub native_theme: bool,
    pub confirm_remove: Option<Vec<usize>>,
    pub confirm_delete: Option<Vec<usize>>,
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
    /// Where a picked cover image should go: the tag editor, the focused
    /// file's embedded art, or the folder cover file.
    pub cover_pick_target: CoverPickTarget,
    results: Vec<crate::ui_tageditor::EditorResult>,
}

impl Default for Dialogs {
    fn default() -> Self {
        Dialogs {
            tag_editor: None,
            editor_size: None,
            show_mask_editor: None,
            folder_rename_target: None,
            file_rename_targets: Vec::new(),
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
            colour_picker: None,
            theme_file: None,
            find_text: String::new(),
            find_match_case: false,
            show_about: false,
            show_options: false,
            choose: None,
            native_pick: native_picker::State::default(),
            native_theme: false,
            confirm_remove: None,
            confirm_delete: None,
            swap_advanced: false,
            copy_field_open: false,
            swap_from: 0,
            swap_to: 0,
            save_cover_bytes: None,
            save_lyrics_text: None,
            pick_cover: false,
            cover_pick_target: CoverPickTarget::default(),
            results: Vec::new(),
        }
    }
}

impl Dialogs {
    pub fn apply_result(&mut self, result: crate::ui_tageditor::EditorResult) {
        self.results.push(result);
    }

    pub fn open_mask_editor(&mut self, which: usize, current: &str) {
        let kind = match which {
            0 => MaskKind::Rename,
            _ => MaskKind::FileToTag,
        };
        self.open_mask_editor_kind(kind, current);
    }

    pub fn open_mask_editor_kind(&mut self, kind: MaskKind, current: &str) {
        self.show_mask_editor = Some(kind);
        self.mask_backup = current.to_string();
        self.saved_mask_sel = None;
    }

    pub fn open_folder_mask_editor(&mut self, current: &str, target: PathBuf) {
        self.folder_rename_target = Some(target);
        self.file_rename_targets.clear();
        self.open_mask_editor_kind(MaskKind::Folder, current);
    }

    pub fn open_file_name_mask_editor(&mut self, current: &str, targets: Vec<usize>) {
        self.folder_rename_target = None;
        self.file_rename_targets = targets;
        self.open_mask_editor_kind(MaskKind::Rename, current);
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
            before_theme: theme::theme_index(),
            before_scheme: theme::scheme_index(),
            previewing: false,
        });
        self.colour_picker = None;
    }

    /// Open the Create theme panel for a brand new theme, using the selected
    /// theme only as a starting palette. Never edits or removes that theme.
    pub fn open_new_theme_editor(&mut self, index: usize) {
        let spec = theme::spec_of(index).unwrap_or_else(theme::default_custom_spec);
        self.theme_editor = Some(ThemeEditorState {
            origin: None,
            name: theme::fresh_theme_name(),
            dark: true,
            dark_fields: ThemeColourFields::from_colours(spec.dark),
            light_fields: ThemeColourFields::from_colours(spec.light),
            before_theme: theme::theme_index(),
            before_scheme: theme::scheme_index(),
            previewing: false,
        });
        self.colour_picker = None;
    }

    /// Open the picker for one of the Create theme colour boxes.
    pub fn open_theme_colour_picker(&mut self, dark: bool, field: usize) {
        let Some(editor) = self.theme_editor.as_ref() else { return };
        let field = field.min(2);
        let fields = if dark { &editor.dark_fields } else { &editor.light_fields };
        let current = match field {
            0 => &fields.bg,
            1 => &fields.text,
            _ => &fields.accent,
        };
        self.colour_picker = Some(ColourPickerState::new(
            ColourSlot { dark, field: field as u8 },
            current,
            editor.previewing,
        ));
    }

    /// Switch which mode the open Create theme panel is editing.
    pub fn set_theme_editor_mode(&mut self, dark: bool) {
        if let Some(editor) = self.theme_editor.as_mut() {
            editor.dark = dark;
        }
    }

    /// Push the open editor's colours into the live preview. Used by the
    /// screenshot-only Dark/Light staging knob.
    pub fn preview_theme_editor(&mut self, ctx: &egui::Context) {
        if let Some(editor) = self.theme_editor.as_mut() {
            editor.apply_preview(ctx);
        }
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
    // Centred on first open and moveable by its title bar — an anchored
    // window cannot be dragged, so place it with default_pos instead.
    let screen = ctx
        .input(|i| i.raw.screen_rect)
        .unwrap_or_else(|| egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1280.0, 800.0)));
    let pos = screen.min + (screen.size() - egui::vec2(540.0, 500.0)) / 2.0;
    egui::Window::new("Appearance")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_width(540.0)
        .default_pos(pos)
        .movable(true)
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
                        theme::clear_preview(ctx);
                        app.dlg.open_new_theme_editor(cur_theme);
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
                        let (picked, edit) = theme_card(ui, w, &t.name, k, cur_theme == k, k >= theme::BUILTIN_COUNT);
                        if edit {
                            theme::clear_preview(ctx);
                            app.dlg.open_theme_editor(k);
                        } else if picked {
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

fn theme_card(
    ui: &mut Ui,
    w: f32,
    name: &str,
    k: usize,
    selected: bool,
    custom: bool,
) -> (bool, bool) {
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

    // Custom themes are selected by the border alone; their right-hand mark
    // is an edit pencil that opens the Create theme panel for editing/removal.
    let edit = if custom {
        let centre = rect.max - egui::vec2(15.0, 13.0);
        let edit_rect = egui::Rect::from_center_size(centre, egui::vec2(22.0, 22.0));
        let edit_resp = ui.interact(edit_rect, ui.id().with(("theme_edit", k)), egui::Sense::click());
        let colour = if edit_resp.hovered() { p.accent } else { p.text_dim };
        let tip = centre + egui::vec2(-5.0, 5.0);
        let tail = centre + egui::vec2(4.0, -4.0);
        ui.painter().line_segment([tip, tail], egui::Stroke::new(2.5, colour));
        ui.painter().line_segment(
            [tip + egui::vec2(1.5, 1.5), tail + egui::vec2(1.5, -1.5)],
            egui::Stroke::new(1.5, colour),
        );
        ui.painter().line_segment(
            [centre + egui::vec2(2.0, -6.5), centre + egui::vec2(6.0, -2.5)],
            egui::Stroke::new(2.5, colour),
        );
        ui.painter().circle_filled(tip, 1.5, colour);
        edit_resp.clicked()
    } else {
        if selected {
            ui.painter().text(
                rect.max - egui::vec2(12.0, 30.0),
                egui::Align2::CENTER_CENTER,
                "✓",
                egui::FontId::proportional(14.0),
                p.accent,
            );
        }
        false
    };
    (resp.clicked(), edit)
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

/// A clickable swatch and a directly editable `#RRGGBB` box.
/// Returns `(hex_changed, picker_requested)`.
fn colour_row(ui: &mut Ui, field: &mut String) -> (bool, bool) {
    let mut changed = false;
    let mut picker_requested = false;
    ui.horizontal(|ui| {
        let colour = parse_hex(field).unwrap_or(egui::Color32::GRAY);
        let (rect, swatch) = ui.allocate_exact_size(egui::vec2(28.0, 20.0), egui::Sense::click());
        ui.painter().rect_filled(rect, egui::CornerRadius::same(5), colour);
        ui.painter().rect_stroke(
            rect,
            egui::CornerRadius::same(5),
            egui::Stroke::new(1.0, c_separator()),
            egui::StrokeKind::Middle,
        );
        if swatch.clicked() {
            picker_requested = true;
        }
        let hex = ui.add(
            egui::TextEdit::singleline(field)
                .desired_width(130.0)
                .hint_text("#RRGGBB")
                .font(egui::TextStyle::Monospace),
        );
        changed = hex.changed();
    });
    (changed, picker_requested)
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
    let mut picker = app.dlg.colour_picker.take();
    enum Action { None, Save, Remove, Cancel, Export }
    let mut action = Action::None;
    let mut open = true;
    // Centred on first open and moveable by its title bar.
    let screen = ctx
        .input(|i| i.raw.screen_rect)
        .unwrap_or_else(|| egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1280.0, 800.0)));
    let pos = screen.min + (screen.size() - egui::vec2(440.0, 560.0)) / 2.0;
    egui::Window::new("Create theme")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_width(440.0)
        .default_pos(pos)
        .movable(true)
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
                    theme::set_preview_mode(ctx, true);
                    editor.apply_preview(ctx);
                }
                if !editor.dark {
                    primary_button(ui, "Light");
                } else if ui.button("Light").clicked() {
                    editor.dark = false;
                    theme::set_preview_mode(ctx, false);
                    editor.apply_preview(ctx);
                }
            });
            ui.add_space(8.0);
            egui::Grid::new("create_theme_colours").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                let (changed, open_picker) = {
                    let field = if editor.dark { &mut editor.dark_fields.bg } else { &mut editor.light_fields.bg };
                    ui.label("Background");
                    colour_row(ui, field)
                };
                if changed { editor.apply_preview(ctx); }
                if open_picker {
                    picker = Some(ColourPickerState::new(
                        ColourSlot { dark: editor.dark, field: 0 },
                        if editor.dark { &editor.dark_fields.bg } else { &editor.light_fields.bg },
                        editor.previewing,
                    ));
                }
                ui.end_row();

                let (changed, open_picker) = {
                    let field = if editor.dark { &mut editor.dark_fields.text } else { &mut editor.light_fields.text };
                    ui.label("Text");
                    colour_row(ui, field)
                };
                if changed { editor.apply_preview(ctx); }
                if open_picker {
                    picker = Some(ColourPickerState::new(
                        ColourSlot { dark: editor.dark, field: 1 },
                        if editor.dark { &editor.dark_fields.text } else { &editor.light_fields.text },
                        editor.previewing,
                    ));
                }
                ui.end_row();

                let (changed, open_picker) = {
                    let field = if editor.dark { &mut editor.dark_fields.accent } else { &mut editor.light_fields.accent };
                    ui.label("Accent");
                    colour_row(ui, field)
                };
                if changed { editor.apply_preview(ctx); }
                if open_picker {
                    picker = Some(ColourPickerState::new(
                        ColourSlot { dark: editor.dark, field: 2 },
                        if editor.dark { &editor.dark_fields.accent } else { &editor.light_fields.accent },
                        editor.previewing,
                    ));
                }
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
                        editor.apply_preview(ctx);
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
    if !open {
        action = Action::Cancel;
    }
    match action {
        Action::None => {}
        Action::Cancel => {
            if editor.previewing {
                theme::select(ctx, editor.before_theme, editor.before_scheme);
            }
            close = true;
        }
        Action::Export => {
            if editor.previewing {
                theme::select(ctx, editor.before_theme, editor.before_scheme);
                editor.previewing = false;
            }
            app.dlg.theme_file = Some(ThemeFileState {
                purpose: ThemeFilePurpose::Export,
                path: theme_json_path(&editor.name),
            });
            picker = None;
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
    if close {
        app.dlg.colour_picker = None;
    } else {
        app.dlg.theme_editor = Some(editor);
        app.dlg.colour_picker = picker;
    }
}

/// The saturation/value square. Dragging chooses saturation (left to right)
/// and brightness (bottom to top) for the current hue.
fn saturation_value_square(ui: &mut Ui, picker: &mut ColourPickerState) -> (egui::Response, bool) {
    let (rect, resp) =
        ui.allocate_exact_size(egui::vec2(218.0, 150.0), egui::Sense::click_and_drag());
    let mut changed = false;
    if resp.clicked() || resp.dragged() {
        if let Some(pos) = resp.interact_pointer_pos() {
            let s = (pos.x - rect.min.x) / rect.width();
            let v = 1.0 - (pos.y - rect.min.y) / rect.height();
            picker.set_hsv(picker.h, s, v);
            changed = true;
        }
    }
    let cols = 56.0_f32;
    let rows = 38.0_f32;
    let cell = egui::vec2(rect.width() / cols, rect.height() / rows);
    for row in 0..rows as u32 {
        let v = 1.0 - (row as f32 + 0.5) / rows;
        for col in 0..cols as u32 {
            let s = (col as f32 + 0.5) / cols;
            let min = rect.min + cell * egui::vec2(col as f32, row as f32);
            let cell_rect = egui::Rect::from_min_size(min, cell);
            ui.painter()
                .rect_filled(cell_rect, egui::CornerRadius::same(0), hsv_to_colour(picker.h, s, v));
        }
    }
    let marker = egui::pos2(
        rect.min.x + picker.s * rect.width(),
        rect.min.y + (1.0 - picker.v) * rect.height(),
    );
    let outline = if picker.v > 0.55 {
        egui::Color32::from_gray(30)
    } else {
        egui::Color32::WHITE
    };
    ui.painter().circle_filled(marker, 6.0, outline);
    ui.painter().circle_stroke(marker, 4.0, egui::Stroke::new(1.5, egui::Color32::WHITE));
    (resp, changed)
}

/// The vertical hue strip on the right of the square, red at the top.
fn hue_strip(ui: &mut Ui, picker: &mut ColourPickerState) -> (egui::Response, bool) {
    let (rect, resp) =
        ui.allocate_exact_size(egui::vec2(24.0, 150.0), egui::Sense::click_and_drag());
    let mut changed = false;
    if resp.clicked() || resp.dragged() {
        if let Some(pos) = resp.interact_pointer_pos() {
            picker.set_hsv((pos.y - rect.min.y) / rect.height(), picker.s, picker.v);
            changed = true;
        }
    }
    let bands = 90.0_f32;
    let band = rect.height() / bands;
    for i in 0..bands as u32 {
        let h = (i as f32 + 0.5) / bands;
        let band_rect = egui::Rect::from_min_size(
            egui::pos2(rect.min.x, rect.min.y + band * i as f32),
            egui::vec2(rect.width(), band),
        );
        ui.painter()
            .rect_filled(band_rect, egui::CornerRadius::same(0), hsv_to_colour(h, 1.0, 1.0));
    }
    let y = rect.min.y + picker.h * rect.height();
    ui.painter().rect_filled(
        egui::Rect::from_min_size(egui::pos2(rect.min.x - 3.0, y - 1.5), egui::vec2(rect.width() + 6.0, 3.0)),
        egui::CornerRadius::same(2),
        egui::Color32::WHITE,
    );
    (resp, changed)
}

fn picker_swatch(ui: &mut Ui, colour: egui::Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(28.0, 20.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, egui::CornerRadius::same(5), colour);
    ui.painter().rect_stroke(
        rect,
        egui::CornerRadius::same(5),
        egui::Stroke::new(1.0, c_separator()),
        egui::StrokeKind::Middle,
    );
}

/// A small Photoshop-style colour picker: saturation/value square, hue strip,
/// hex and RGB fields, old/new preview, and OK/Cancel.
fn colour_picker_window(ctx: &egui::Context, app: &mut App) {
    let Some(mut picker) = app.dlg.colour_picker.take() else { return };
    let Some(mut editor) = app.dlg.theme_editor.take() else {
        app.dlg.colour_picker = Some(picker);
        return;
    };
    enum Action { None, Ok, Cancel }
    let mut action = Action::None;
    let mut open = true;
    // Opens beside the Create theme window (its old anchored offset) and is
    // moveable by its title bar.
    let screen = ctx
        .input(|i| i.raw.screen_rect)
        .unwrap_or_else(|| egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1280.0, 800.0)));
    let pos = screen.center() + egui::vec2(320.0, -30.0) - egui::vec2(430.0, 300.0) / 2.0;
    egui::Window::new("Colour picker")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_width(430.0)
        .default_pos(pos)
        .movable(true)
        .show(ctx, |ui| {
            let mut changed = false;
            ui.horizontal(|ui| {
                let (_, was_changed) = saturation_value_square(ui, &mut picker);
                changed |= was_changed;
                let (_, was_changed) = hue_strip(ui, &mut picker);
                changed |= was_changed;
            });
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.label("Hex");
                let response = ui.add_sized(
                    egui::vec2(110.0, 20.0),
                    egui::TextEdit::singleline(&mut picker.hex).hint_text("#RRGGBB"),
                );
                if response.changed() {
                    if let Some(colour) = parse_hex(&picker.hex) {
                        picker.set_colour(colour);
                        changed = true;
                    }
                }
                ui.add_space(12.0);
                ui.label("RGB");
                let mut rgb = [picker.colour().r(), picker.colour().g(), picker.colour().b()];
                for (index, label) in ["R", "G", "B"].iter().enumerate() {
                    ui.label(*label);
                    let mut value = rgb[index] as i32;
                    let response = ui.add_sized(
                        egui::vec2(54.0, 20.0),
                        egui::DragValue::new(&mut value).speed(1.0).range(0..=255),
                    );
                    if response.changed() {
                        rgb[index] = value.clamp(0, 255) as u8;
                        picker.set_colour(egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]));
                        changed = true;
                    }
                }
            });
            if changed {
                *picker.slot.field_mut(&mut editor) = picker.hex.clone();
                editor.apply_preview(ctx);
            }
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.label("Current");
                picker_swatch(ui, parse_hex(&picker.before).unwrap_or(egui::Color32::GRAY));
                ui.add_space(18.0);
                ui.label("New");
                picker_swatch(ui, picker.colour());
            });
            ui.add_space(10.0);
            ui.separator();
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if primary_button(ui, "OK").clicked() {
                        action = Action::Ok;
                    }
                    if ui.button("Cancel").clicked() {
                        action = Action::Cancel;
                    }
                });
            });
        });

    if !open {
        action = Action::Cancel;
    }
    let mut close = false;
    match action {
        Action::None => {}
        Action::Ok => match parse_hex(&picker.hex) {
            Some(_) => close = true,
            None => app.warn("Enter a #RRGGBB colour"),
        },
        Action::Cancel => {
            *picker.slot.field_mut(&mut editor) = picker.before.clone();
            if picker.previewing_before {
                editor.apply_preview(ctx);
            } else {
                theme::clear_preview(ctx);
                editor.previewing = false;
            }
            close = true;
        }
    }
    app.dlg.theme_editor = Some(editor);
    if close {
        app.dlg.colour_picker = None;
    } else {
        app.dlg.colour_picker = Some(picker);
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
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut state.path)
                        .desired_width(340.0)
                        .hint_text("~/.config/tagrename/theme.json"),
                );
                if ui
                    .add_enabled(app.shot.is_none() && !app.dlg.native_pick.busy(), egui::Button::new("Browse..."))
                    .clicked()
                {
                    let current = expand_tilde(&state.path);
                    let dir = current
                        .parent()
                        .filter(|p| p.is_dir())
                        .map(Path::to_path_buf)
                        .unwrap_or_else(|| default_start_dir(app));
                    let name = current
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .filter(|n| !n.is_empty())
                        .unwrap_or_else(|| "theme.json".to_string());
                    let filters = native_filters(&[("Theme files", &["json"]), ("All files", &["*"])]);
                    let request = match purpose {
                        ThemeFilePurpose::Export => native_picker::Request::SaveFile { dir, name, filters },
                        ThemeFilePurpose::Import => native_picker::Request::OpenFile { dir, filters },
                    };
                    if app.dlg.native_pick.request(request) {
                        app.dlg.native_theme = true;
                    }
                }
            });
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
    poll_native_picker(ctx, app);

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
    if let Some(kind) = app.dlg.show_mask_editor {
        let mut open = true;
        let mut choice = None;
        egui::Window::new("Mask Editor")
            .open(&mut open)
            .collapsible(false)
            .default_width(780.0)
            .show(ctx, |ui| {
                choice = mask_editor(ui, app, kind);
            });
        if choice == Some(false) || (choice.is_none() && !open) {
            let backup = app.dlg.mask_backup.clone();
            match kind {
                MaskKind::Rename => app.rename_mask = backup,
                MaskKind::FileToTag => app.f2t_mask = backup,
                MaskKind::Folder => app.folder_mask = backup,
            }
        }
        if choice == Some(true) {
            match kind {
                MaskKind::Folder => {
                    let target = app
                        .dlg
                        .folder_rename_target
                        .take()
                        .or_else(|| app.folder_rename_target());
                    let mask = app.folder_mask.clone();
                    if let Some(target) = target {
                        app.apply_folder_mask(&target, &mask);
                    } else {
                        app.warn("There is no folder to rename.");
                    }
                }
                MaskKind::Rename if !app.dlg.file_rename_targets.is_empty() => {
                    let targets = std::mem::take(&mut app.dlg.file_rename_targets);
                    let mask = app.rename_mask.clone();
                    app.apply_file_name_mask(&targets, &mask);
                }
                _ => {}
            }
        }
        if choice.is_some() || !open {
            app.dlg.show_mask_editor = None;
            app.dlg.folder_rename_target = None;
            app.dlg.file_rename_targets.clear();
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
        egui::Window::new("Export Tags")
            .open(&mut open)
            .collapsible(false)
            .resizable([false, true])
            .min_size(egui::vec2(480.0, 280.0))
            .default_width(520.0)
            .default_height(430.0)
            .show(ctx, |ui| {
                ui.label("Select what fields to export:");
                egui::ScrollArea::vertical()
                    .id_salt("export_fields")
                    .auto_shrink([false, true])
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
    if app.dlg.colour_picker.is_some() {
        colour_picker_window(ctx, app);
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

    // --- About ---
    if app.dlg.show_about {
        let mut open = true;
        egui::Window::new("About")
            .open(&mut open)
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(6.0);
                    ui.label(RichText::new("Tag:Rename 1.1.4").strong().size(22.0));
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

    // --- Delete confirmation ---
    if let Some(indices) = app.dlg.confirm_delete.clone() {
        let mut open = true;
        egui::Window::new("Delete Files")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_width(420.0)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.label(format!(
                    "Permanently delete {} selected file{}? This cannot be undone.",
                    indices.len(),
                    if indices.len() == 1 { "" } else { "s" }
                ));
                ui.separator();
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("No").clicked() {
                            app.dlg.confirm_delete = None;
                        }
                        if primary_button(ui, "Yes").clicked() {
                            app.delete_files(&indices);
                            app.dlg.confirm_delete = None;
                        }
                    });
                });
            });
        if !open {
            app.dlg.confirm_delete = None;
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
        // Screenshot runs use the old in-app chooser so the pictures stay
        // deterministic and no real OS dialog appears over the window.
        // Everywhere else the desktop's own folder/file dialog is used.
        let use_native = app.shot.is_none();
        if use_native && !app.dlg.native_pick.busy() {
            let _ = app.dlg.native_pick.request(native_request_for(&state, app));
        }
        let native_pending = use_native && app.dlg.native_pick.busy();
        if !native_pending {
            egui::Window::new(purpose.title())
                .default_width(460.0)
                .default_height(400.0)
                .show(ctx, |ui| {
                    chooser_body(ui, app, &mut state, &mut done);
                });
        }
        match done {
            Some(d) => {
                handle_choose(app, purpose, d);
                app.dlg.choose = None;
                app.dlg.pick_cover = false;
                app.dlg.cover_pick_target = CoverPickTarget::TagEditor;
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
            match app.dlg.cover_pick_target {
                CoverPickTarget::TagEditor => {
                    crate::ui_tageditor::stage_cover(app, b);
                }
                CoverPickTarget::FileCover => {
                    if let Some(i) = app.focused {
                        if i < app.files.len() && !app.files[i].is_dir {
                            let path = app.files[i].path.clone();
                            match crate::data::set_cover(&path, b) {
                                Ok(()) => {
                                    if let Some(re) = crate::data::read_file(&path) {
                                        app.files[i] = re;
                                    }
                                    app.say("Cover art updated.");
                                }
                                Err(e) => app.warn(e),
                            }
                        }
                    }
                }
                CoverPickTarget::FolderCover => {
                    let Some(folder) = crate::ui_tabs::folder_cover_dir(app) else { return };
                    let ext = match image::guess_format(&b) {
                        Ok(image::ImageFormat::Png) => "png",
                        _ => "jpg",
                    };
                    let target = folder.join(format!("folder.{ext}"));
                    // Replace any existing folder/cover image files.
                    if let Ok(rd) = std::fs::read_dir(&folder) {
                        for e in rd.flatten() {
                            let name = e.file_name().to_string_lossy().to_lowercase();
                            if matches!(
                                name.as_str(),
                                "folder.jpg"
                                    | "folder.jpeg"
                                    | "folder.png"
                                    | "cover.jpg"
                                    | "cover.jpeg"
                                    | "cover.png"
                            ) {
                                let _ = std::fs::remove_file(e.path());
                            }
                        }
                    }
                    match std::fs::write(&target, &b) {
                        Ok(()) => app.say(format!("Saved {}", target.file_name().unwrap_or_default().to_string_lossy())),
                        Err(e) => app.warn(format!("Can't save cover art: {e}")),
                    }
                }
            }
            app.dlg.cover_pick_target = CoverPickTarget::TagEditor;
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

/// Build the OS picker request from the same state the in-app chooser uses.
/// Keeping this beside `chooser_body` means both routes choose the same
/// starting folder, default name and file types.
fn native_request_for(state: &ChooseState, app: &App) -> native_picker::Request {
    use native_picker::Request;

    let dir = state.dir.clone();
    match &state.purpose {
        ChoosePurpose::OpenFolder | ChoosePurpose::AddRoot => Request::Folder { dir },
        ChoosePurpose::PickCover => Request::OpenFile {
            dir,
            filters: native_filters(&[
                ("Images", &["jpg", "jpeg", "png", "gif", "bmp", "webp"]),
                ("All files", &["*"]),
            ]),
        },
        ChoosePurpose::LoadLyrics => Request::OpenFile {
            dir,
            filters: native_filters(&[("Text files", &["txt", "lrc"]), ("All files", &["*"])]),
        },
        ChoosePurpose::SavePlaylist(_) => Request::SaveFile {
            dir,
            name: with_default_extension(&state.name, "m3u"),
            filters: native_filters(&[("Playlist", &["m3u"])]),
        },
        ChoosePurpose::SaveExport => {
            let ext = app.dlg.export_format.ext();
            Request::SaveFile {
                dir,
                name: with_default_extension(&state.name, ext),
                filters: native_filters(&[(
                    match app.dlg.export_format {
                        ExportFormat::Csv => "CSV files",
                        ExportFormat::Txt => "Text files",
                        ExportFormat::Html => "HTML files",
                        ExportFormat::Xml => "XML files",
                    },
                    &[ext],
                )]),
            }
        }
        ChoosePurpose::SaveCover => Request::SaveFile {
            dir,
            name: with_default_extension(&state.name, "jpg"),
            filters: native_filters(&[
                ("Images", &["jpg", "jpeg", "png", "gif", "bmp"]),
                ("All files", &["*"]),
            ]),
        },
        ChoosePurpose::SaveLyrics => Request::SaveFile {
            dir,
            name: with_default_extension(&state.name, "txt"),
            filters: native_filters(&[("Text files", &["txt"]), ("All files", &["*"])]),
        },
    }
}

fn native_filters(entries: &[(&str, &[&str])]) -> native_picker::Filters {
    entries
        .iter()
        .map(|(name, exts)| {
            (
                (*name).to_string(),
                exts.iter().map(|e| (*e).to_string()).collect(),
            )
        })
        .collect()
}

fn with_default_extension(name: &str, ext: &str) -> String {
    let name = name.trim();
    if name.is_empty() {
        format!("untitled.{ext}")
    } else if PathBuf::from(name).extension().is_none() {
        format!("{name}.{ext}")
    } else {
        name.to_string()
    }
}

fn finalize_saved_path(purpose: &ChoosePurpose, fmt: ExportFormat, mut path: PathBuf) -> PathBuf {
    let ext = match purpose {
        ChoosePurpose::SavePlaylist(_) => "m3u",
        ChoosePurpose::SaveExport => fmt.ext(),
        ChoosePurpose::SaveCover => "jpg",
        ChoosePurpose::SaveLyrics => "txt",
        _ => return path,
    };
    if path.extension().is_none() {
        path.set_extension(ext);
    }
    path
}

/// Turn the OS dialog's result into the existing `ChooseDone` dispatch so
/// no save/load code has to know which picker produced the path.
fn native_choose_done(app: &mut App, purpose: ChoosePurpose, path: PathBuf) {
    let done = match &purpose {
        ChoosePurpose::OpenFolder | ChoosePurpose::AddRoot => ChooseDone::Folder(path),
        ChoosePurpose::PickCover | ChoosePurpose::LoadLyrics => match std::fs::read(&path) {
            Ok(bytes) => ChooseDone::Bytes(bytes),
            Err(err) => {
                app.warn(format!("Couldn't read \"{}\": {err}", path.display()));
                return;
            }
        },
        _ => ChooseDone::SavePath(finalize_saved_path(&purpose, app.dlg.export_format, path)),
    };
    handle_choose(app, purpose, done);
}

/// Poll the one in-flight native dialog, if any, and deliver its result.
fn poll_native_picker(ctx: &egui::Context, app: &mut App) {
    let Some(result) = app.dlg.native_pick.poll() else { return };
    ctx.request_repaint();

    if app.dlg.native_theme {
        app.dlg.native_theme = false;
        if let (Some(path), Some(state)) = (result, app.dlg.theme_file.as_mut()) {
            state.path = path.to_string_lossy().into_owned();
        }
        return;
    }

    match result {
        Some(path) => {
            if let Some(state) = app.dlg.choose.take() {
                native_choose_done(app, state.purpose, path);
            }
        }
        None => app.dlg.choose = None,
    }
    app.dlg.pick_cover = false;
    app.dlg.cover_pick_target = CoverPickTarget::TagEditor;
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

/// The variables the Mask Editor can insert. Folders can be named from
/// the music inside them, so the folder mode uses the same tokens as file
/// rename, but it never inserts backslashes/folder structure.
fn mask_variables(kind: MaskKind) -> Vec<(&'static str, &'static str)> {
    match kind {
        MaskKind::Rename | MaskKind::Folder => {
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
        }
        MaskKind::FileToTag => vec![
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
        ],
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
fn mask_editor(ui: &mut Ui, app: &mut App, kind: MaskKind) -> Option<bool> {
    let mut mask_text = match kind {
        MaskKind::Rename => app.rename_mask.clone(),
        MaskKind::FileToTag => app.f2t_mask.clone(),
        MaskKind::Folder => app.folder_mask.clone(),
    };
    let mut result = None;
    let mut insert: Option<&'static str> = None;
    let mut add_current = false;
    let mut remove_selected = false;
    let mut pick: Option<usize> = None;
    let mut insert_folder = false;
    let variables = mask_variables(kind);

    let edit_id = egui::Id::new(("mask_editor_text", kind.id()));
    let mask_label = if kind == MaskKind::Folder {
        "Folder name mask:"
    } else {
        "File name mask:"
    };
    ui.horizontal(|ui| {
        ui.label(RichText::new(mask_label).color(c_text()).strong());
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
    if kind != MaskKind::Folder {
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
    }

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

    match kind {
        MaskKind::Rename => app.rename_mask = mask_text.clone(),
        MaskKind::FileToTag => app.f2t_mask = mask_text.clone(),
        MaskKind::Folder => app.folder_mask = mask_text.clone(),
    }

    ui.add_space(6.0);
    ui.separator();
    ui.horizontal(|ui| {
        ui.label(RichText::new("Result example:").color(c_text_dim()).strong());
        let example = match kind {
            MaskKind::Rename => crate::ui_tabs::rename_example(app, &mask_text),
            MaskKind::FileToTag => crate::ui_tabs::f2t_example(app, &mask_text),
            MaskKind::Folder => crate::ui_tabs::folder_example(app, &mask_text),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hsv_round_trips_hex_colours() {
        for raw in ["#000000", "#FFFFFF", "#FF0000", "#5B8CFF", "#2F2F2F"] {
            let colour = parse_hex(raw).unwrap();
            let (h, s, v) = rgb_to_hsv(colour);
            assert_eq!(hex_string(hsv_to_colour(h, s, v)), hex_string(colour));
        }
    }

    #[test]
    fn colour_picker_slot_targets_the_requested_field() {
        let spec = theme::default_custom_spec();
        let mut editor = ThemeEditorState {
            origin: None,
            name: "Test".into(),
            dark: true,
            dark_fields: ThemeColourFields::from_colours(spec.dark),
            light_fields: ThemeColourFields::from_colours(spec.light),
            before_theme: 0,
            before_scheme: theme::SCHEME_DARK,
            previewing: false,
        };
        *ColourSlot { dark: false, field: 1 }.field_mut(&mut editor) = "#123456".into();
        assert_eq!(editor.light_fields.text, "#123456");
        *ColourSlot { dark: true, field: 2 }.field_mut(&mut editor) = "#ABCDEF".into();
        assert_eq!(editor.dark_fields.accent, "#ABCDEF");
    }
}

//! The main window: state, layout, menus, toolbars, folder tree,
//! file list and status bar.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use egui::Context;

use crate::data::{self, Field, MusicFile, TagFields, ALL_FIELDS};
use crate::mask;
use crate::tools::{self, ArtistTool, TrackFormat};
use crate::ui_dialogs;

/// The "Filter by" dropdown, in the original's exact order. The original
/// populates ComboBoxFilterBy from the file list's column list at runtime
/// (sub_c9419c @ 0xc9419c: for each of the 51 columns at
/// (form+0xcc0)+0x294+8, Items.Add(column name); the selected index is
/// restored from the Config key `FilterByItemIndex`). The full order is
/// proven twice: the filter engine sub_8413fc @ 0x8413fc switches over
/// the dropdown index (jump table at 0x8414c0, 51 cases) and the column
/// text provider sub_83d600 @ 0x83d600 (jump table at 0x83d64a) reads the
/// same fields in the same order (case addresses ascending = index):
///
/// File Name(+0x18) Artist(+0x8c) Album(+0x94) Title(+0x90) Year(+0x98)
/// Genre(+0x9c) Comment(+0xa0) Track #(+0xa4) Composer(+0xa8)
/// Orig. Artist(+0xac) Bitrate(+0x30) Time(+0x2c) Hz(+0x38) BPM(+0xbc)
/// Cover Art(flag) Copyright(+0xb0) URL(+0xb4) [string +0xb8]
/// Lyrics(+0xc0) Date Modified(+0x40/+0x44) [blank column, always " "]
/// Encoded by(+0x48) Mood(+0xc8) Disc #(+0xcc) Album Artist(+0xd0)
/// Conductor(+0xd4) ISRC #(+0xf8) Catalog #(+0x100) Label(+0xfc)
/// Subfolder(+0x20) Compilation(flag) Remixed by(+0x104)
/// Grouping(+0x108) Subtitle(+0x10c) Rating(+0x110) Lyricist(+0xd8)
/// File Ext.(ext of +0x18) Initial Key(+0x13c) Sort Title(+0x70)
/// Sort Artist(+0x74) Sort Album Artist(+0x78) Sort Album(+0x7c)
/// Sort Composer(+0x80) Sort Show(+0x84) [date +0xc] [date +0x4]
/// Bit Depth(+0x4c) Cover Size(+0x64) Cover Dimensions(+0x68/+0x6c)
/// Cover Format(+0x60) Tag Version(+0x58).
///
/// Corrections from this decompile pass: index 12 is the Hz column
/// (case 0x83d881 reads the SAMPLE RATE +0x38: DSD64/DSD128/DSD256 names
/// for 2822400/5644800/11289600, otherwise IntToStr of the raw rate) —
/// there is no Size column and no field +0x38 is a size anywhere (the
/// rename engine's %khz divides the same +0x38 by 1000); and the column
/// at index 20 (between Date Modified and Encoded by) returns the literal
/// " " (0x83dca0) — no case reads a numeric rating +0xc4 at all, so the
/// old "Rating (number)" entry was a mislabel and is dropped.
///
/// Entries the rebuild's model does not store (the flag fields, the
/// unnamed +0xb8 column, the dates, the blank index-20 column, the
/// cover-art size/dimensions/format, Subfolder, Grouping, Subtitle and
/// the six sort fields) are left out, so the list below is the
/// original's order minus those.
pub const FILTER_FIELDS: &[&str] = &[
    "File name",
    "Artist",
    "Album",   // the original really lists Album before Title (+0x94 before +0x90)
    "Title",
    "Year",
    "Genre",
    "Comment",
    "Track #",
    "Composer",
    "Orig. Artist",
    "Bitrate",
    "Time",
    "Hz", // case 0x83d881: the sample rate, DSD names or the raw number
    "BPM",
    "Copyright",
    "URL",
    "Lyrics",
    "Encoded by",
    "Mood",
    "Disc #",
    "Album Artist",
    "Conductor",
    "ISRC #",
    "Catalog #",
    "Label",
    "Compilation",
    "Remixed by",
    "Rating",
    "Lyricist",
    "File Ext.",
    "Initial Key",
    "Bit Depth",
    "Tag Version",
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    All,
    Id3v2,
    Wma,
    Vorbis,
    Mp4,
    Ape,
}

impl ViewMode {
    pub fn label(self) -> &'static str {
        match self {
            ViewMode::All => "All",
            ViewMode::Id3v2 => "ID3v2",
            ViewMode::Wma => "Wma",
            ViewMode::Vorbis => "Ogg/Flac",
            ViewMode::Mp4 => "Mp4/M4a",
            ViewMode::Ape => "APEv2",
        }
    }

    fn matches(self, f: &MusicFile) -> bool {
        match self {
            ViewMode::All => true,
            ViewMode::Id3v2 => f.tag_kind == "ID3v2" || f.tag_kind == "ID3v1",
            ViewMode::Wma => f.tag_kind == "WMA",
            ViewMode::Vorbis => f.tag_kind == "Vorbis",
            ViewMode::Mp4 => f.tag_kind == "MP4",
            ViewMode::Ape => f.tag_kind == "APEv2",
        }
    }
}

/// Where the folder-tree roots are kept between runs: one path per line
/// in the person's config folder (AppData on Windows, XDG on Linux).
fn roots_file() -> Option<PathBuf> {
    config_file("roots.txt")
}

pub fn config_file(name: &str) -> Option<PathBuf> {
    if cfg!(test) {
        return Some(std::env::temp_dir().join(format!("tagrename_test_{}_{}", name, std::process::id())));
    }
    #[cfg(windows)]
    {
        // Windows has no XDG_CONFIG_HOME. Use the per-user AppData folder so
        // a portable EXE remembers its settings just like an installed one.
        let base = std::env::var_os("APPDATA")
            .or_else(|| std::env::var_os("LOCALAPPDATA"))
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("USERPROFILE")
                    .map(|p| PathBuf::from(p).join("AppData").join("Roaming"))
            })?;
        Some(base.join("Tag-Rename").join(name))
    }
    #[cfg(not(windows))]
    {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
        Some(base.join("tagrename").join(name))
    }
}

/// The file list's columns, in the order the person arranged them, with
/// the width they dragged each one to and whether it is shown.
pub fn load_columns() -> Vec<ColumnState> {
    let defaults = crate::ui_layout::default_columns();
    let Some(file) = config_file("columns.txt") else { return defaults };
    let Ok(text) = std::fs::read_to_string(file) else { return defaults };
    let mut out: Vec<ColumnState> = Vec::new();
    for line in text.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() != 3 {
            continue;
        }
        let (Ok(id), Ok(width), Ok(visible)) =
            (parts[0].parse::<usize>(), parts[1].parse::<f32>(), parts[2].parse::<bool>())
        else {
            continue;
        };
        if id < crate::ui_layout::COLS.len() && !out.iter().any(|c| c.id == id) {
            out.push(ColumnState { id, width: width.clamp(MIN_COL_W, 900.0), visible });
        }
    }
    for d in &defaults {
        if !out.iter().any(|c| c.id == d.id) {
            out.push(*d);
        }
    }
    if !out.iter().any(|c| c.visible) {
        return defaults;
    }
    out
}

pub fn save_columns(cols: &[ColumnState]) {
    let Some(file) = config_file("columns.txt") else { return };
    if let Some(dir) = file.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let body: String = cols
        .iter()
        .map(|c| format!("{} {} {}\n", c.id, c.width, c.visible))
        .collect();
    let _ = std::fs::write(file, body);
}

/// The roots saved by an earlier run, in the order they were added.
pub fn load_roots() -> Vec<PathBuf> {
    let Some(file) = roots_file() else { return Vec::new() };
    let Ok(text) = std::fs::read_to_string(file) else { return Vec::new() };
    let mut roots: Vec<PathBuf> = Vec::new();
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty()) {
        let p = PathBuf::from(line);
        if !roots.contains(&p) {
            roots.push(p);
        }
    }
    roots
}

fn save_roots(roots: &[PathBuf]) {
    let Some(file) = roots_file() else { return };
    if let Some(dir) = file.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let body: String = roots.iter().map(|p| format!("{}\n", p.display())).collect();
    let _ = std::fs::write(file, body);
}

/// The two file name masks and the saved masks, kept between runs in
/// masks.txt: one "rename", one "f2t" and one "saved" line per mask.
/// The mask both file-name tools start with. The original used two
/// different defaults; the person asked for this single one instead.
const DEFAULT_FILE_MASK: &str = "%6 - %1 - %2";
const LEGACY_RENAME_MASK: &str = "%1 - %2";
const LEGACY_F2T_MASK: &str = "%1-%2";

pub fn load_masks() -> Option<(String, String, Vec<String>)> {
    let file = config_file("masks.txt")?;
    let text = std::fs::read_to_string(file).ok()?;
    let mut rename = None;
    let mut f2t = None;
    let mut saved: Vec<String> = Vec::new();
    for line in text.lines() {
        if let Some(v) = line.strip_prefix("rename ") {
            rename = Some(v.to_string());
        } else if let Some(v) = line.strip_prefix("f2t ") {
            f2t = Some(v.to_string());
        } else if let Some(v) = line.strip_prefix("saved ") {
            let v = v.trim().to_string();
            if !v.is_empty() && !saved.contains(&v) {
                saved.push(v);
            }
        }
    }
    // Treat the two old defaults as unset so existing configs pick up the
    // new default too, while any genuinely custom mask is left alone.
    let rename = rename.filter(|m| m != LEGACY_RENAME_MASK && m != LEGACY_F2T_MASK);
    let f2t = f2t.filter(|m| m != LEGACY_RENAME_MASK && m != LEGACY_F2T_MASK);
    Some((
        rename.unwrap_or_else(|| DEFAULT_FILE_MASK.into()),
        f2t.unwrap_or_else(|| DEFAULT_FILE_MASK.into()),
        saved,
    ))
}

fn save_masks(rename: &str, f2t: &str, saved: &[String]) {
    let Some(file) = config_file("masks.txt") else { return };
    if let Some(dir) = file.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let mut body = format!("rename {rename}\nf2t {f2t}\n");
    for s in saved {
        body.push_str(&format!("saved {s}\n"));
    }
    let _ = std::fs::write(file, body);
}

/// One rename batch, for Undo.
pub struct UndoBatch {
    pub pairs: Vec<(PathBuf, PathBuf)>, // (new, old)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Name,
    Title,
    Artist,
    Album,
    Year,
    Track,
    Genre,
    Tag,
    Time,
    Bitrate,
    Size,
    Hz,
}

/// One file-list column: its index in the column table, its width in
/// points, and whether the View > Columns menu shows it.
#[derive(Clone, Copy, PartialEq)]
pub struct ColumnState {
    pub id: usize,
    pub width: f32,
    pub visible: bool,
}

pub const MIN_COL_W: f32 = 30.0;

/// Events from a background folder scan.
pub enum LoadEvent {
    Progress(usize),
    Done(Vec<MusicFile>),
}

/// A folder scan running in the background (the person's library holds
/// ~20,000 files in ~1,000 artist folders — reading it on the UI thread
/// would look like a freeze).
pub struct Loading {
    pub folder: PathBuf,
    pub sub: bool,
    pub seen: usize,
    rx: std::sync::mpsc::Receiver<LoadEvent>,
}

/// The rows Open shows for one folder. `include_subfolders` is the Search
/// Subfolders tick box: when it is off, only music files sitting directly in
/// this folder are listed — a parent/artist folder with no direct songs
/// therefore produces an empty list instead of a row per album.
fn scan_folder_rows(path: &Path, include_subfolders: bool) -> Vec<MusicFile> {
    let mut rows: Vec<MusicFile> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(path) {
        let mut entries: Vec<_> = rd.flatten().collect();
        entries.sort_by_key(|e| e.file_name().to_string_lossy().to_lowercase());
        for e in entries {
            let p = e.path();
            let n = e.file_name().to_string_lossy().into_owned();
            if n.starts_with('.') { continue; }
            if p.is_dir() {
                if include_subfolders {
                    if let Some(m) = data::read_folder_row(&p) { rows.push(m); }
                }
            } else if data::is_audio_file(&p) {
                if let Some(m) = data::read_file(&p) { rows.push(m); }
            }
        }
    }
    rows
}

pub struct App {
    pub files: Vec<MusicFile>,
    pub checked: Vec<bool>,
    pub focused: Option<usize>,
    pub selection_anchor: Option<usize>,
    pub folder: Option<PathBuf>,
    pub search_subfolders: bool,
    pub view_mode: ViewMode,
    pub filter_field: usize,
    pub filter_text: String,
    pub filter_active: bool,
    /// "Hide Selected Files" from the Filters toolbar.
    pub hide_checked: bool,
    pub hide_extensions: bool,
    pub sort: Option<(SortColumn, bool)>,
    /// File list columns in display order (see ColumnState).
    pub columns: Vec<ColumnState>,
    /// The column whose header is being dragged to a new place.
    pub col_drag: Option<usize>,
    /// Expanded folder rows in the main list (path -> loaded children).
    pub expanded_rows: HashMap<PathBuf, Vec<MusicFile>>,

    pub active_tab: usize,
    pub show_tools_bar: bool,
    pub show_folders: bool,

    // Rename Files tab
    pub rename_mask: String,
    pub rename_case_enabled: bool,
    pub rename_case: usize,
    pub replace_spaces: bool,

    // Multi File Tag Editor tab
    pub multi_checked: Vec<(Field, bool)>,
    pub multi_values: TagFields,
    pub multi_case_enabled: bool,
    pub multi_case: usize,

    // Get Tags from File Name tab
    pub f2t_mask: String,
    pub f2t_case_enabled: bool,
    pub f2t_case: usize,
    pub replace_underscores: bool,

    // Clipboard
    pub clip_tag: Option<TagFields>,

    // Folder tree
    pub expanded: HashMap<PathBuf, bool>,
    /// Cached child folders of each expanded tree node. Without this the
    /// tree re-read a thousand directories on every repaint, which made
    /// every click feel slow. Refresh clears it.
    pub folder_children: HashMap<PathBuf, Vec<PathBuf>>,
    /// Width chosen by dragging the folder panel; adding a location does
    /// not change it (the first-load and later states behave the same).
    pub folder_panel_width: f32,
    /// True after a folder is picked in the folder tree, until a file row
    /// is clicked. While it is set, typing a letter jumps the folder tree.
    pub tree_focus: bool,
    /// True after a file row is clicked, until a tree folder is picked.
    /// While it is set, typing a letter jumps the file list.
    pub list_focus: bool,
    /// Folder highlighted in the folder tree. Kept separate from `folder`
    /// so type-ahead can move the tree highlight without loading files.
    pub tree_selected: Option<PathBuf>,
    /// Folder row the tree should scroll into view on the next frame.
    pub tree_scroll_to: Option<PathBuf>,
    /// Row position in the visible list that the file list should scroll to.
    pub scroll_row: Option<usize>,
    /// Folders added as tree roots. Adding a root NEVER reads its music:
    /// the person chooses a folder inside it to load, so a library of
    /// ~20,000 songs is not pulled into the editor just by adding a root.
    pub roots: Vec<PathBuf>,
    /// Locations the person removed from the tree in this session. They are
    /// kept so opening a folder inside one does not restore it as part of an
    /// ancestor chain.
    pub hidden_roots: Vec<PathBuf>,
    /// The address bar's text (above the file tree, like the original's
    /// Address toolbar): paste a location and press Enter or Go.
    pub address: String,

    // Undo
    pub undo_stack: Vec<UndoBatch>,

    // Status toast
    pub toast: Option<(String, Instant, bool)>,

    // Options
    pub remove_leading_zero_disc: bool,
    pub dont_add_leading_zero_track: bool,
    pub update_id3v1_if_exists: bool,

    // Dialogs
    pub dlg: ui_dialogs::Dialogs,
    pub genres: Vec<String>,

    /// The folder scan running in the background, if any.
    pub loading: Option<Loading>,

    // Texture cache for cover art
    pub textures: HashMap<String, egui::TextureHandle>,

    // Screenshot mode
    pub shot: Option<crate::ShotState>,
}

impl App {
    /// Open the per-file Tag Editor window on the focused file (or all
    /// checked files when several are selected). When the selection is
    /// made of folder rows, the audio files inside those folders become
    /// the checked files of the Multi File Tag Editor.
    pub fn open_tag_editor(&mut self) {
        let dirs: Vec<PathBuf> = self
            .checked_indices()
            .into_iter()
            .filter(|i| self.files[*i].is_dir)
            .map(|i| self.files[i].path.clone())
            .collect();
        if !dirs.is_empty() {
            let mut gathered: Vec<MusicFile> = Vec::new();
            for dir in &dirs {
                for entry in walkdir::WalkDir::new(dir)
                    .max_depth(4)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    let p = entry.path();
                    if p.is_file() && data::is_audio_file(p) {
                        if let Some(m) = data::read_file(p) {
                            gathered.push(m);
                        }
                    }
                }
            }
            if !gathered.is_empty() {
                for i in 0..self.checked.len() {
                    if self.files[i].is_dir {
                        self.checked[i] = false;
                    }
                }
                let first = self.files.len();
                self.files.extend(gathered);
                self.checked.extend(std::iter::repeat(true).take(self.files.len() - first));
                self.focused = Some(first);
                self.selection_anchor = Some(first);
                self.say(format!(
                    "Multi File Tag Editor: {} file(s) from {} folder(s).",
                    self.files.len() - first,
                    dirs.len()
                ));
            } else {
                self.warn("The selected folder(s) contain no music files.");
            }
        }
        let ed = crate::ui_tageditor::TagEditor::open(self);
        self.dlg.tag_editor = Some(ed);
    }

    /// Open the Tag Editor for the focused file only. The original's
    /// actFile_EditFileTag (F5, handler 0xca6908) always creates one
    /// editor in single-file mode (the multi flag at form+0x7c1 is
    /// cleared), and the person confirmed the same for a double click:
    /// "double clicking a song opens the Tag Editor" — the clicked song
    /// is edited even when other files are checked.
    pub fn open_tag_editor_focused(&mut self) {
        let ed = crate::ui_tageditor::TagEditor::open_focused(self);
        self.dlg.tag_editor = Some(ed);
    }

    pub fn new() -> Self {
        let (rename_mask, f2t_mask, saved_masks) =
            load_masks().unwrap_or_else(|| (DEFAULT_FILE_MASK.into(), DEFAULT_FILE_MASK.into(), Vec::new()));
        let mut dlg = ui_dialogs::Dialogs::default();
        dlg.saved_masks = saved_masks;
        let files: Vec<MusicFile> = Vec::new();
        let checked = vec![false; files.len()];
        let mut genres: Vec<String> = crate::genres::GENRES.iter().map(|s| s.to_string()).collect();
        for f in &files {
            if !f.fields.genre.is_empty() && !genres.iter().any(|g| g == &f.fields.genre) {
                genres.push(f.fields.genre.clone());
            }
        }
        genres.sort();
        App {
            files,
            checked,
            focused: None,
            selection_anchor: None,
            folder: None,
            search_subfolders: false,
            view_mode: ViewMode::All,
            filter_field: 0,
            filter_text: String::new(),
            filter_active: false,
            hide_checked: false,
            hide_extensions: false,
            sort: None,
            columns: load_columns(),
            col_drag: None,
            expanded_rows: HashMap::new(),
            active_tab: 0,
            show_tools_bar: true,
            show_folders: true,
            rename_mask,
            rename_case_enabled: false,
            rename_case: 0,
            replace_spaces: false,
            // One activation tick box per field, all off on first open.
            // The box is the only way to include a field when saving.
            multi_checked: [
                Field::Title,
                Field::Artist,
                Field::Album,
                Field::Year,
                Field::Genre,
                Field::Comment,
            ]
            .iter()
            .map(|f| (*f, false))
            .collect(),
            multi_values: TagFields::default(),
            multi_case_enabled: false,
            multi_case: 0,
            // The original's file-to-tag default was "%1-%2" (FormCreate
            // sub_c944fc), but both file-name tools now start with the
            // requested "%6 - %1 - %2" mask.
            f2t_mask,
            f2t_case_enabled: false,
            f2t_case: 0,
            replace_underscores: true,
            clip_tag: None,
            expanded: HashMap::new(),
            folder_children: HashMap::new(),
            folder_panel_width: 300.0,
            tree_focus: false,
            list_focus: false,
            tree_selected: None,
            tree_scroll_to: None,
            scroll_row: None,
            roots: load_roots(),
            hidden_roots: Vec::new(),
            address: String::new(),
            undo_stack: Vec::new(),
            toast: None,
            remove_leading_zero_disc: false,
            dont_add_leading_zero_track: false,
            update_id3v1_if_exists: true,
            dlg,
            genres,
            loading: None,
            textures: HashMap::new(),
            shot: None,
        }
    }

    /// Write the masks to masks.txt whenever they differ from the last write.
    pub fn persist_masks(&mut self) {
        let key = format!(
            "{}\n{}\n{}",
            self.rename_mask,
            self.f2t_mask,
            self.dlg.saved_masks.join("\n")
        );
        if key != self.dlg.masks_saved {
            save_masks(&self.rename_mask, &self.f2t_mask, &self.dlg.saved_masks);
            self.dlg.masks_saved = key;
        }
    }

    pub fn say(&mut self, msg: impl Into<String>) {
        self.toast = Some((msg.into(), Instant::now(), false));
    }

    pub fn warn(&mut self, msg: impl Into<String>) {
        self.toast = Some((msg.into(), Instant::now(), true));
    }

    pub fn checked_indices(&self) -> Vec<usize> {
        self.checked
            .iter()
            .enumerate()
            .filter(|(_, c)| **c)
            .map(|(i, _)| i)
            .collect()
    }

    pub fn selected_file_indices(&self) -> Vec<usize> {
        self.checked_indices().into_iter().filter(|i| !self.files[*i].is_dir).collect()
    }

    pub fn select_all_visible(&mut self) {
        let rows = self.visible_rows();
        for c in &mut self.checked {
            *c = false;
        }
        for i in rows {
            self.checked[i] = true;
        }
        self.selection_anchor = self.focused;
    }

    pub fn select_range(&mut self, target: usize) {
        let rows = self.visible_rows();
        let anchor = self.selection_anchor.or(self.focused).unwrap_or(target);
        let Some(a_pos) = rows.iter().position(|r| *r == anchor) else {
            self.checked[target] = true;
            self.selection_anchor = Some(target);
            self.focused = Some(target);
            return;
        };
        let Some(t_pos) = rows.iter().position(|r| *r == target) else { return; };
        let (lo, hi) = if a_pos <= t_pos { (a_pos, t_pos) } else { (t_pos, a_pos) };
        for c in &mut self.checked { *c = false; }
        for r in &rows[lo..=hi] { self.checked[*r] = true; }
        self.focused = Some(target);
    }

    /// Select the next visible file whose name starts with `letter`,
    /// searching after the focused row and wrapping around.
    pub fn jump_to_letter(&mut self, letter: char) {
        let rows = self.visible_rows();
        if rows.is_empty() {
            return;
        }
        let start = self
            .focused
            .and_then(|f| rows.iter().position(|r| *r == f))
            .map(|p| p + 1)
            .unwrap_or(0);
        let letter = letter.to_lowercase().next().unwrap_or(letter);
        let found = (0..rows.len())
            .map(|k| (start + k) % rows.len())
            .find(|p| {
                self.files[rows[*p]]
                    .name
                    .chars()
                    .next()
                    .map(|c| c.to_lowercase().next().unwrap_or(c) == letter)
                    .unwrap_or(false)
            });
        if let Some(pos) = found {
            let i = rows[pos];
            for c in &mut self.checked {
                *c = false;
            }
            self.checked[i] = true;
            self.focused = Some(i);
            self.selection_anchor = Some(i);
            self.scroll_row = Some(pos);
        }
    }

    /// Child folders shown under `path` in the left tree, in display order.
    /// The same cache the tree uses is reused so type-ahead does not re-read
    /// a thousand directories on every keystroke.
    pub fn tree_children(&self, path: &Path) -> Vec<PathBuf> {
        if let Some(saved) = self.folder_children.get(path) {
            return saved.clone();
        }
        if let Some(places) = crate::places::children(path) {
            return places;
        }
        let mut children: Vec<PathBuf> = Vec::new();
        if let Ok(rd) = std::fs::read_dir(path) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    let n = e.file_name().to_string_lossy().into_owned();
                    if !n.starts_with('.') {
                        children.push(p);
                    }
                }
            }
        }
        children.sort();
        children
    }

    /// Type-ahead for the folder tree: move the tree highlight to the next
    /// sibling whose name starts with `letter`, wrapping around. This only
    /// moves the selection — it deliberately does not load any music.
    pub fn jump_tree_letter(&mut self, letter: char) {
        let current = self
            .tree_selected
            .as_deref()
            .or_else(|| self.folder.as_deref());
        let mut roots = self.roots.clone();
        roots.extend(crate::places::initial_roots());
        roots.sort();
        roots.dedup();
        let siblings = match current {
            None => roots,
            Some(p) if roots.iter().any(|r| r.as_path() == p) => self.tree_children(p),
            Some(p) => self.tree_children(p.parent().unwrap_or(p)),
        };
        if siblings.is_empty() {
            return;
        }
        let start = current
            .and_then(|p| siblings.iter().position(|s| s.as_path() == p))
            .map(|i| i + 1)
            .unwrap_or(0);
        let letter = letter.to_lowercase().next().unwrap_or(letter);
        let found = (0..siblings.len())
            .map(|k| (start + k) % siblings.len())
            .find(|p| {
                let sibling = &siblings[*p];
                let fallback = sibling
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| sibling.display().to_string());
                crate::places::display_name(sibling, fallback)
                    .chars()
                    .next()
                    .map(|c| c.to_lowercase().next().unwrap_or(c) == letter)
                    .unwrap_or(false)
            });
        if let Some(pos) = found {
            let path = siblings[pos].clone();
            // Make sure the row is drawn before asking the panel to scroll.
            let parent = path.parent().unwrap_or(path.as_path());
            self.expanded.insert(parent.to_path_buf(), true);
            self.tree_selected = Some(path.clone());
            self.tree_scroll_to = Some(path);
            self.tree_focus = true;
        }
    }

    pub fn toggle_row_check(&mut self, i: usize) {
        self.checked[i] = !self.checked[i];
        self.focused = Some(i);
        self.selection_anchor = Some(i);
    }

    pub fn toggle_row_expand(&mut self, i: usize) {
        let path = self.files[i].path.clone();
        if !self.files[i].is_dir { return; }
        if self.expanded_rows.contains_key(&path) {
            let remove: Vec<PathBuf> = self.expanded_rows.keys().filter(|k| k.starts_with(&path) && **k != path).cloned().collect();
            for k in remove { self.expanded_rows.remove(&k); }
            self.expanded_rows.remove(&path);
            self.rebuild_rows_from_folder();
            return;
        }
        let mut children: Vec<MusicFile> = Vec::new();
        if let Ok(rd) = std::fs::read_dir(&path) {
            let mut entries: Vec<_> = rd.flatten().collect();
            entries.sort_by_key(|e| e.file_name().to_string_lossy().to_lowercase());
            for e in entries {
                let p = e.path();
                let n = e.file_name().to_string_lossy().into_owned();
                if n.starts_with('.') { continue; }
                if p.is_dir() {
                    if let Some(m) = data::read_folder_row(&p) { children.push(m); }
                } else if data::is_audio_file(&p) {
                    if let Some(m) = data::read_file(&p) { children.push(m); }
                }
            }
        }
        self.expanded_rows.insert(path, children);
        self.rebuild_rows_from_folder();
    }

    fn rebuild_rows_from_folder(&mut self) {
        let Some(folder) = self.folder.clone() else { return; };
        fn collect(path: &Path, expanded: &HashMap<PathBuf, Vec<MusicFile>>, out: &mut Vec<MusicFile>) {
            let Some(children) = expanded.get(path) else { return; };
            for c in children {
                let p = c.path.clone();
                let is_dir = c.is_dir;
                out.push(c.clone());
                if is_dir && expanded.contains_key(&p) { collect(&p, expanded, out); }
            }
        }
        let mut rows = Vec::new();
        if let Ok(rd) = std::fs::read_dir(&folder) {
            let mut entries: Vec<_> = rd.flatten().collect();
            entries.sort_by_key(|e| e.file_name().to_string_lossy().to_lowercase());
            for e in entries {
                let p = e.path();
                let n = e.file_name().to_string_lossy().into_owned();
                if n.starts_with('.') { continue; }
                if p.is_dir() {
                    if let Some(m) = data::read_folder_row(&p) {
                        rows.push(m);
                        if self_expanded(&self.expanded_rows, &p) { collect(&p, &self.expanded_rows, &mut rows); }
                    }
                } else if data::is_audio_file(&p) {
                    if let Some(m) = data::read_file(&p) { rows.push(m); }
                }
            }
        }
        let old_paths: Vec<PathBuf> = self.files.iter().map(|f| f.path.clone()).collect();
        let checked_paths: Vec<PathBuf> = self.checked.iter().enumerate().filter(|(_,c)| **c).filter_map(|(i,_)| old_paths.get(i).cloned()).collect();
        let focused_path = self.focused.and_then(|i| old_paths.get(i).cloned());
        self.files = rows;
        self.checked = self.files.iter().map(|f| checked_paths.contains(&f.path)).collect();
        self.focused = focused_path.and_then(|p| self.files.iter().position(|f| f.path == p)).or(if self.files.is_empty(){None}else{Some(0)});
        self.selection_anchor = self.focused;
        fn self_expanded(map: &HashMap<PathBuf, Vec<MusicFile>>, p: &Path) -> bool { map.contains_key(p) }
    }

    /// Add a folder as a root of the folder tree only. This deliberately
    /// does not scan the folder or touch the file list: a root such as
    /// /mnt/pool/Music/Music holds ~20,000 songs in artist subfolders,
    /// and the person asked to add it as a root first, then choose a
    /// folder inside it to load.
    pub fn add_root(&mut self, path: &Path) {
        if !path.is_dir() {
            self.warn(format!("Cannot add \"{}\" — it is not a folder.", path.display()));
            return;
        }
        if !self.roots.iter().any(|p| p == path) {
            self.roots.push(path.to_path_buf());
            save_roots(&self.roots);
        }
        self.hidden_roots.retain(|p| p != path);
        // Adding a location shows the new root only. It must not expand
        // the tree or change the panel width: the person reported that
        // the tree jumped to a fixed size as soon as a root was added.
        self.expanded.entry(path.to_path_buf()).or_insert(false);
        self.folder_children.remove(path);
        self.say(format!(
            "Added {} as a folder root. Select a folder inside it to load music.",
            path.display()
        ));
    }

    /// Remove a folder that was added to the tree by hand. Default roots and
    /// shell places are not in `roots`, so they cannot be removed here.
    pub fn remove_root(&mut self, path: &Path) {
        let Some(pos) = self.roots.iter().position(|p| p.as_path() == path) else {
            return;
        };
        let removed = self.roots.remove(pos);
        save_roots(&self.roots);
        self.hidden_roots.push(removed.clone());
        self.expanded.remove(&removed);
        self.folder_children.remove(&removed);
        if self
            .tree_selected
            .as_deref()
            .is_some_and(|p| p.starts_with(&removed))
        {
            self.tree_selected = None;
        }
        self.say(format!("Removed {} from the folder tree.", removed.display()));
    }

    /// Open a folder: the scan runs in a background thread (a big
    /// library would otherwise freeze the window for the whole read)
    /// and `poll_loading` applies the results as they arrive. With
    /// Search Subfolders off, only this folder's OWN files are read and
    /// its child folders are not listed at all.
    pub fn load_folder(&mut self, path: &Path) {
        // A Windows shell place is a virtual tree node, not a folder that can
        // be scanned. Ignore attempts from the address bar or type-ahead.
        if crate::places::is_place(path) {
            self.tree_selected = Some(path.to_path_buf());
            return;
        }
        // The tick box is captured now so a later click cannot change
        // the rules halfway through a background scan.
        let include_subfolders = self.search_subfolders;
        let (tx, rx) = std::sync::mpsc::channel();
        let scan_path = path.to_path_buf();
        std::thread::spawn(move || {
            let _ = tx.send(LoadEvent::Done(scan_folder_rows(&scan_path, include_subfolders)));
        });
        self.files.clear();
        self.checked.clear();
        self.focused = None;
        self.selection_anchor = None;
        self.folder = Some(path.to_path_buf());
        self.tree_selected = Some(path.to_path_buf());
        self.address = path.display().to_string();
        self.undo_stack.clear();
        self.expanded_rows.clear();
        self.loading = Some(Loading { folder: path.to_path_buf(), sub: false, seen: 0, rx });
        self.say(format!("Reading {}...", path.display()));
    }

    /// Apply background-scan progress and results (called every frame).
    pub fn poll_loading(&mut self) {
        let Some(mut loading) = self.loading.take() else { return };
        loop {
            match loading.rx.try_recv() {
                Ok(LoadEvent::Progress(n)) => loading.seen = n,
                Ok(LoadEvent::Done(files)) => {
                    self.finish_load(loading.folder.clone(), loading.sub, files);
                    return;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    self.loading = Some(loading);
                    return;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    // The scan was replaced or cancelled; nothing to apply.
                    return;
                }
            }
        }
    }

    fn finish_load(&mut self, folder: PathBuf, sub: bool, mut files: Vec<MusicFile>) {
        files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        self.files = files;
        self.checked = vec![false; self.files.len()];
        self.focused = if self.files.is_empty() { None } else { Some(0) };
        self.selection_anchor = self.focused;
        self.folder = Some(folder.clone());
        self.address = folder.display().to_string();
        self.undo_stack.clear();
        // The genre dropdown grows with the library (the original's
        // genre list is user-editable: Config StandardGenresList /
        // AdditionalGenresList).
        for f in &self.files {
            let g = f.fields.genre.trim().to_string();
            if !g.is_empty() && !self.genres.iter().any(|x| x.eq_ignore_ascii_case(&g)) {
                self.genres.push(g);
            }
        }
        self.genres.sort();
        if self.files.is_empty() {
            if sub {
                self.warn(format!("No subfolders or music files in {}", folder.display()));
            } else {
                self.say(format!(
                    "No music files directly in {}. Tick Search Subfolders to list its folders.",
                    folder.display()
                ));
            }
        } else {
            self.say(format!("Read {} files...", self.files.len()));
        }
    }

    /// Resolve the folder typed into the address bar. `~` expands to the
    /// home folder. A path that does not exist (or is outside what the
    /// app is allowed to see) gets a clear message instead of silence.
    fn address_path(&mut self) -> Option<PathBuf> {
        let raw = self.address.trim().to_string();
        if raw.is_empty() {
            return None;
        }
        let path = if let Some(rest) = raw.strip_prefix('~') {
            match std::env::var_os("HOME") {
                Some(home) => PathBuf::from(home).join(rest.trim_start_matches('/')),
                None => PathBuf::from(&raw),
            }
        } else {
            PathBuf::from(&raw)
        };
        if path.is_dir() {
            Some(path)
        } else {
            self.warn(format!(
                "Cannot open \"{}\".\nThe folder does not exist, or it is outside the folders this app is allowed to see.\n\
                 (A mounted drive beyond your home, like /mnt/pool, appears after Craft Anything itself is restarted:\n\
                 save your work, quit it completely — including any tray icon — and start it again.)",
                raw
            ));
            None
        }
    }

    /// Go moves the folder tree to the pasted location: it adds the
    /// folder as a root, expands it and makes it the current tree
    /// selection, but does not read any music into the main list.
    pub fn open_address(&mut self) {
        if let Some(path) = self.address_path() {
            self.add_root(&path);
            for a in path.ancestors() {
                self.expanded.insert(a.to_path_buf(), true);
            }
            self.expanded.insert(path.clone(), true);
            self.say(format!("Moved the folder tree to {}. Press Open to list this folder.", path.display()));
        }
    }

    /// Open reads the typed folder itself (never its whole subtree).
    pub fn load_address(&mut self) {
        if let Some(path) = self.address_path() {
            self.load_folder(&path);
        }
    }

    /// What one filter field holds, per the original's switch
    /// (sub_8413fc @ 0x8413fc): most cases read a string
    /// (FilterValue::Text), the rating case compares numbers and the
    /// two flag cases ignore the pattern.
    fn filter_value(&self, f: &MusicFile) -> FilterValue {
        use Field::*;
        // fields.get() borrows; wrap into an owned Text value.
        let txt = |v: &str| FilterValue::Text(v.to_string());
        match FILTER_FIELDS[self.filter_field.min(FILTER_FIELDS.len() - 1)] {
            "File name" => FilterValue::Text(f.name.clone()),
            "Artist" => txt(f.fields.get(Artist)),
            "Album" => txt(f.fields.get(Album)),
            "Title" => txt(f.fields.get(Title)),
            "Year" => txt(f.fields.get(Year)),
            "Genre" => txt(f.fields.get(Genre)),
            "Comment" => txt(f.fields.get(Comment)),
            "Track #" => FilterValue::Text(with_total(&f.fields.get(Track), &f.fields.get(TrackTotal))),
            "Composer" => txt(f.fields.get(Composer)),
            "Orig. Artist" => txt(f.fields.get(OrigArtist)),
            // case_84167c: "VBR" goes in front of the number when the
            // VBR flag at item+0x34 is set.
            "Bitrate" => FilterValue::Text(if f.is_vbr {
                format!("VBR{}", f.bitrate_kbps)
            } else {
                f.bitrate_kbps.to_string()
            }),
            // case_8416b4: the play time through sub_640954 (h:mm:ss).
            "Time" => FilterValue::Text(play_time_hmmss(f.duration_secs)),
            // case_8416c6 (0x83d881 in the column provider): the sample
            // rate column — DSD64/DSD128/DSD256 for 2822400/5644800/
            // 11289600, otherwise IntToStr of the RAW rate.
            "Hz" => FilterValue::Text(match f.sample_rate {
                2_822_400 => "DSD64".to_string(),
                5_644_800 => "DSD128".to_string(),
                11_289_600 => "DSD256".to_string(),
                r => r.to_string(),
            }),
            "BPM" => txt(f.fields.get(Bpm)),
            "Copyright" => txt(f.fields.get(Copyright)),
            "URL" => txt(f.fields.get(Url)),
            "Lyrics" => txt(f.fields.get(Lyrics)),
            "Encoded by" => txt(f.fields.get(EncodedBy)),
            "Mood" => txt(f.fields.get(Mood)),
            "Disc #" => FilterValue::Text(with_total(&f.fields.get(Disc), &f.fields.get(DiscTotal))),
            "Album Artist" => txt(f.fields.get(AlbumArtist)),
            "Conductor" => txt(f.fields.get(Conductor)),
            "ISRC #" => txt(f.fields.get(Isrc)),
            "Catalog #" => txt(f.fields.get(Catalog)),
            "Label" => txt(f.fields.get(Label)),
            // case_84185d: the compilation flag at item+0x89.
            "Compilation" => {
                let v = f.fields.get(Compilation);
                FilterValue::Flag(!v.trim().is_empty() && v != "0" && !v.eq_ignore_ascii_case("false"))
            }
            "Remixed by" => txt(f.fields.get(RemixedBy)),
            "Rating" => txt(f.fields.get(Rating)),
            "Lyricist" => txt(f.fields.get(Lyricist)),
            "File Ext." => FilterValue::Text(f.extension()),
            "Initial Key" => txt(f.fields.get(InitialKey)),
            "Bit Depth" => FilterValue::Text(f.bit_depth.map(|d| d.to_string()).unwrap_or_default()),
            _ => FilterValue::Text(f.tag_kind.clone()),
        }
    }

    /// Rows after view-mode and filter are applied.
    pub fn visible_rows(&self) -> Vec<usize> {
        let mut rows: Vec<usize> = (0..self.files.len())
            .filter(|i| self.view_mode.matches(&self.files[*i]))
            .filter(|i| !(self.hide_checked && self.checked[*i]))
            .collect();
        if self.filter_active {
            let needle = self.filter_text.clone();
            rows.retain(|i| {
                let f = &self.files[*i];
                match self.filter_value(f) {
                    FilterValue::Text(hay) => filter_match(&hay, &needle),
                    // case_8416e7/0x84185d: the flag fields ignore the text
                    // and keep the file only when the flag is set.
                    FilterValue::Flag(on) => on,
                }
            });
        }
        if let Some((col, asc)) = self.sort {
            let key = |f: &MusicFile| -> String {
                match col {
                    SortColumn::Name => f.name.to_lowercase(),
                    SortColumn::Title => f.fields.title.to_lowercase(),
                    SortColumn::Artist => f.fields.artist.to_lowercase(),
                    SortColumn::Album => f.fields.album.to_lowercase(),
                    SortColumn::Year => f.fields.year.clone(),
                    SortColumn::Track => format!("{:04}", f.fields.track.parse::<u32>().unwrap_or(9999)),
                    SortColumn::Genre => f.fields.genre.to_lowercase(),
                    SortColumn::Tag => f.tag_kind.clone(),
                    SortColumn::Time => format!("{:06}", f.duration_secs as u32),
                    SortColumn::Bitrate => format!("{:05}", f.bitrate_kbps),
                    SortColumn::Size => format!("{:012}", f.size_bytes),
                    SortColumn::Hz => format!("{:08}", f.sample_rate),
                }
            };
            if asc {
                rows.sort_by_cached_key(|a| key(&self.files[*a]));
            } else {
                rows.sort_by_cached_key(|a| std::cmp::Reverse(key(&self.files[*a])));
            }
        }
        rows
    }

    pub fn total_time(&self, only_checked: bool) -> String {
        let secs: f64 = self
            .files
            .iter()
            .enumerate()
            .filter(|(i, _)| !only_checked || self.checked[*i])
            .map(|(_, f)| f.duration_secs)
            .sum();
        let s = secs.round() as u64;
        format!("{}:{:02}:{:02}", s / 3600, (s % 3600) / 60, s % 60)
    }

    pub fn cover_texture(&mut self, ctx: &Context, key: &str, bytes: &[u8]) -> Option<&egui::TextureHandle> {
        if !self.textures.contains_key(key) {
            if let Ok(img) = image::load_from_memory(bytes) {
                let rgba = img.to_rgba8();
                let size = [rgba.width() as usize, rgba.height() as usize];
                let color = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
                let tex = ctx.load_texture(key, color, egui::TextureOptions::default());
                self.textures.insert(key.to_string(), tex);
            } else {
                return None;
            }
        }
        self.textures.get(key)
    }

    /// The Rename Files action (actMainFunctions_Rename).
    pub fn do_rename(&mut self, preview: bool) {
        if !mask::mask_has_variables(&self.rename_mask) {
            self.warn("First fill in the \"File name mask\"!");
            return;
        }
        let indices = self.checked_indices();
        if indices.is_empty() {
            self.warn("Nothing to do!\nSelect (check) the files first.");
            return;
        }
        let opts = mask::RenameOptions {
            case: mask::Case::from_index(self.rename_case),
            case_enabled: self.rename_case_enabled,
            replace_spaces_with_underscores: self.replace_spaces,
            remove_leading_zero_in_disc: self.remove_leading_zero_disc,
        };
        let total = indices.len() as u32;
        let mut batch = UndoBatch { pairs: Vec::new() };
        let mut errors: Vec<String> = Vec::new();
        let mut done = 0usize;

        // Numbered 1..N in list order: sub_cae988 walks the tree with
        // GetFirst/GetNext (0x83ac9c / 0x83aca4) and counts checked nodes.
        for (n, i) in indices.iter().enumerate() {
            let f = &self.files[*i];
            let Some(stem) = mask::build_new_name(f, &self.rename_mask, n as u32 + 1, total, &opts)
            else {
                errors.push(format!("Error in file or file does not contain tag:\r\n{}", f.name));
                continue;
            };
            if stem.trim().is_empty() {
                continue;
            }
            if preview {
                continue;
            }
            // sub_ca0821's final block (0xca0e00-0xca0e21): after the whole
            // name is built and cased, the extension is re-cased on its own —
            // CharLowerBuff (lower) for the checkbox OFF and every radio
            // except "UPPER" (index 1), CharUpperBuff only when the Case
            // checkbox is on AND the UPPER radio is selected.
            let case_ext = |e: &str| {
                if opts.case_enabled && opts.case == mask::Case::Upper {
                    e.to_uppercase()
                } else {
                    e.to_lowercase()
                }
            };
            // Real file: mask may contain '\' to create folder structure.
            let rel = stem.replace('/', "\\");
            let old_path = f.path.clone();
            let ext = case_ext(&f.extension());
            let parent = old_path.parent().unwrap_or(Path::new(".")).to_path_buf();
            let mut new_path = parent.join(format!("{rel}.{ext}"));
            if let Some(dir) = new_path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            if new_path.exists() && new_path != old_path {
                // The original's default (option "Don't rename file if file
                // with this name already exists" off): make "name (1).ext",
                // "name (2).ext" ... via sub_c924a8 with the format string
                // " (%d)" at 0xc9260c, counting from 1.
                let mut n = 1u32;
                while new_path.exists() {
                    new_path = parent.join(format!("{rel} ({n}).{ext}"));
                    n += 1;
                }
            }
            // sub_6d6d8c at 0x6d6d8c (Deep pass 11, built-in lifter): GetFileAttributesW,
            // and when the read-only bit (1) is set it is cleared (SetFileAttributesW with
            // attr ^ 1) before the file is moved.
            if let Ok(meta) = std::fs::metadata(&old_path) {
                let mut perms = meta.permissions();
                if perms.readonly() {
                    perms.set_readonly(false);
                    let _ = std::fs::set_permissions(&old_path, perms);
                }
            }
            match std::fs::rename(&old_path, &new_path) {
                Ok(()) => {
                    batch.pairs.push((new_path.clone(), old_path.clone()));
                    // sub_cae988 after the move (0xcaef9f+): when a sibling
                    // "<stem>.cdg" file exists (literal ".cdg" at 0xcaf364),
                    // it is renamed to follow the music file.
                    if let (Some(old_stem), Some(new_stem)) =
                        (old_path.file_stem(), new_path.file_stem())
                    {
                        let old_cdg = old_path.with_file_name(format!("{}.cdg", old_stem.to_string_lossy()));
                        if old_cdg.exists() {
                            let new_cdg = new_path.with_file_name(format!("{}.cdg", new_stem.to_string_lossy()));
                            if new_cdg != old_cdg {
                                let _ = std::fs::rename(&old_cdg, &new_cdg);
                            }
                        }
                    }
                    self.files[*i].path = new_path.clone();
                    self.files[*i].name = new_path
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    done += 1;
                }
                Err(e) => errors.push(format!(
                    "Can't rename file or folder {}.\r\nError message: {}",
                    f.name, e
                )),
            }
        }

        if preview {
            self.dlg.show_rename_preview = true;
            return;
        }
        if done > 0 {
            self.undo_stack.push(batch);
            self.say(format!("Renamed {done} file(s)."));
        } else if errors.is_empty() {
            self.warn("Nothing to do!");
        }
        if !errors.is_empty() {
            let shown: Vec<String> = errors.iter().take(3).cloned().collect();
            let more = errors.len() - shown.len();
            let mut msg = shown.join("\n");
            if more > 0 {
                msg.push_str(&format!("\n...and {more} more"));
            }
            self.warn(msg);
        }
    }

    pub fn undo_rename(&mut self) {
        match self.undo_stack.pop() {
            Some(mut b) => {
                let mut undone = 0;
                while let Some((new, old)) = b.pairs.pop() {
                    if std::fs::rename(&new, &old).is_ok() {
                        for f in &mut self.files {
                            if f.path == new {
                                f.path = old.clone();
                                f.name = old
                                    .file_name()
                                    .map(|n| n.to_string_lossy().into_owned())
                                    .unwrap_or_default();
                            }
                        }
                        undone += 1;
                    }
                }
                self.say(format!("Undone {undone} rename(s)."));
            }
            None => self.warn("Nothing to undo."),
        }
    }

    /// Save Tags from the Multi File Tag Editor to all checked files.
    pub fn save_multi_tags(&mut self) {
        let indices = self.selected_file_indices();
        if indices.is_empty() {
            self.warn("Before using \"Save tags\" you need to select (check) the files.");
            return;
        }
        let fields_to_write: Vec<Field> = self
            .multi_checked
            .iter()
            .filter(|(_, c)| *c)
            .map(|(f, _)| *f)
            .collect();
        if fields_to_write.is_empty() {
            self.warn("Nothing to do!\nCheck the fields to save first.");
            return;
        }
        let mut values = self.multi_values.clone();
        if self.multi_case_enabled {
            let case = mask::Case::from_index(self.multi_case);
            for f in &fields_to_write {
                let v = case.apply(values.get(*f));
                values.set(*f, &v);
            }
        }
        let mut errors = Vec::new();
        let mut done = 0;
        for i in &indices {
            let f = &self.files[*i];
            match data::save_fields(&f.path, &values, &fields_to_write) {
                Ok(()) => {
                    if let Some(reloaded) = data::read_file(&f.path) {
                        self.files[*i] = reloaded;
                    }
                    done += 1;
                }
                Err(e) => errors.push(e),
            }
        }
        if done > 0 {
            self.say(format!("Saved tags to {done} file(s)."));
        }
        if !errors.is_empty() {
            self.warn(errors.join("\n"));
        }
    }

    /// Remove Tags from every checked file.
    pub fn remove_tags_selected(&mut self) {
        let indices = self.selected_file_indices();
        if indices.is_empty() {
            self.warn("Before using \"Remove tags\" you need to select (check) the files.");
            return;
        }
        self.dlg.confirm_remove = Some(indices);
    }

    /// Copy tag from the focused file.
    pub fn copy_tag(&mut self) {
        if let Some(i) = self.focused {
            self.clip_tag = Some(self.files[i].fields.clone());
            self.say("Tag copied. Use Paste Tag to write it into other files.");
        }
    }

    /// Paste the clipboard tag into the given rows.
    pub fn paste_tag(&mut self, indices: &[usize]) {
        let Some(tag) = self.clip_tag.clone() else {
            self.warn("Nothing to paste!");
            return;
        };
        let mut done = 0;
        let mut errors = Vec::new();
        for i in indices {
            let f = &self.files[*i];
            let fields: Vec<Field> = ALL_FIELDS.to_vec();
            match data::save_fields(&f.path, &tag, &fields) {
                Ok(()) => {
                    if let Some(reloaded) = data::read_file(&f.path) {
                        self.files[*i] = reloaded;
                    }
                    done += 1;
                }
                Err(e) => errors.push(e),
            }
        }
        if done > 0 {
            self.say(format!("Paste Tag to {done} File(s) done."));
        }
        if !errors.is_empty() {
            self.warn(errors.join("\n"));
        }
    }

    /// Copy from Focused File into the multi-file editor.
    pub fn copy_from_focused(&mut self) {
        if let Some(i) = self.focused {
            self.multi_values = self.files[i].fields.clone();
            self.say("Copied the focused file's fields into the editor.");
        }
    }

    /// Get Tags from File Name -> Write Tags.
    pub fn file_to_tag(&mut self, preview: bool) {
        if !mask::mask_has_variables(&self.f2t_mask) {
            self.warn("First fill in the \"File name mask\"!");
            return;
        }
        let indices = self.checked_indices();
        if indices.is_empty() {
            self.warn("Nothing to do!\nSelect (check) the files first.");
            return;
        }
        let opts = mask::ParseOptions {
            case: mask::Case::from_index(self.f2t_case),
            case_enabled: self.f2t_case_enabled,
            replace_underscores_with_spaces: self.replace_underscores,
        };
        let mut errors = Vec::new();
        let mut done = 0;
        for i in &indices {
            let f = &self.files[*i];
            match mask::parse_file_name(&f.file_stem(), &self.f2t_mask, &opts) {
                Ok(pairs) => {
                    if preview {
                        continue;
                    }
                    let mut new_fields = f.fields.clone();
                    let mut any = false;
                    for (field, value) in &pairs {
                        new_fields.set(*field, value);
                        any = true;
                    }
                    if !any {
                        continue;
                    }
                    let checked: Vec<Field> = pairs.iter().map(|(fl, _)| *fl).collect();
                    match data::save_fields(&f.path, &new_fields, &checked) {
                        Ok(()) => {
                            if let Some(reloaded) = data::read_file(&f.path) {
                                self.files[*i] = reloaded;
                            }
                            done += 1;
                        }
                        Err(e) => errors.push(e),
                    }
                }
                Err(e) => errors.push(format!("{}\r\nFile name: {}", e, f.name)),
            }
        }
        if preview {
            self.dlg.show_f2t_preview = true;
            return;
        }
        if done > 0 {
            self.say(format!("Written tags to {done} file(s)."));
        } else if errors.is_empty() {
            self.warn("Nothing to do!");
        }
        if !errors.is_empty() {
            self.warn(errors.join("\n"));
        }
    }

    /// Apply a track # formatting to every checked file.
    pub fn format_track(&mut self, fmt: TrackFormat) {
        let indices = self.checked_indices();
        if indices.is_empty() {
            self.warn("Nothing to do!\nSelect (check) the files first.");
            return;
        }
        let count = indices.len() as u32;
        let mut done = 0;
        let mut errors = Vec::new();
        for i in &indices {
            let f = &self.files[*i];
            let (track, total) = fmt.apply(&f.fields.track, &f.fields.track_total, count);
            // The original saves a file only when the processed value
            // differs from the original (sub_ca9120 compares the result
            // with the saved raw string at the end of each step).
            if track == f.fields.track && total == f.fields.track_total {
                continue;
            }
            let mut new_fields = f.fields.clone();
            new_fields.track = track;
            new_fields.track_total = total;
            let checked = vec![Field::Track, Field::TrackTotal];
            match data::save_fields(&f.path, &new_fields, &checked) {
                Ok(()) => {
                    if let Some(reloaded) = data::read_file(&f.path) {
                        self.files[*i] = reloaded;
                    }
                    done += 1;
                }
                Err(e) => errors.push(e),
            }
        }
        if done > 0 {
            self.say(format!("{}: applied to {done} file(s).", fmt.label()));
        }
        if !errors.is_empty() {
            self.warn(errors.join("\n"));
        }
    }

    /// Auto Fill Track # (and Episode # in the original; we fill track).
    pub fn auto_fill(&mut self, opts: tools::AutoFill) {
        let indices = self.checked_indices();
        if indices.is_empty() {
            self.warn("Auto fill track # for all selected files?\nSelect (check) the files first.");
            return;
        }
        let selected_count = indices.len() as u32;
        let mut done = 0;
        let mut errors = Vec::new();
        // The original's zero width uses the number of CHECKED files in the
        // file's folder (sub_caa614 at 0xcaa614 and sub_caa6a8 at 0xcaa6a8,
        // called from the Yes handler at 0xcab300; both count items whose
        // check flag is set). The dialog captures
        // `start - 1` (caa73c line 281) and the handler increments it before
        // each file (cab300 line 31); when sub_722504 reports the scope
        // ended the counter is zeroed, so later folders restart at 1.
        // The original also filters the count by a visibility test chosen by
        // form+0x8d8 (check state when set; see MECHANICS.md, pass 8). In
        // checkbox mode that test is the check state, so this count matches.
        let mut counter = opts.start.saturating_sub(1);
        let mut last_folder: Option<PathBuf> = None;
        let mut first_file = true;
        for &i in &indices {
            let folder = self.files[i].path.parent().map(Path::to_path_buf);
            if !first_file && folder != last_folder {
                counter = 0;
            }
            first_file = false;
            last_folder = folder.clone();
            counter = counter.saturating_add(1);
            let scope_count = match &folder {
                Some(folder) => indices
                    .iter()
                    .filter(|&&j| self.files[j].path.parent() == Some(folder.as_path()))
                    .count() as u32,
                None => 0,
            };
            let f = &self.files[i];
            let (track, total) =
                tools::auto_fill_track(counter, scope_count, selected_count, &opts);
            let mut new_fields = f.fields.clone();
            new_fields.track = track;
            if opts.use_total {
                new_fields.track_total = total;
            }
            let checked = if opts.use_total {
                vec![Field::Track, Field::TrackTotal]
            } else {
                vec![Field::Track]
            };
            match data::save_fields(&f.path, &new_fields, &checked) {
                Ok(()) => {
                    if let Some(reloaded) = data::read_file(&f.path) {
                        self.files[i] = reloaded;
                    }
                    done += 1;
                }
                Err(e) => errors.push(e),
            }
        }
        if done > 0 {
            self.say(format!("Auto filled track # in {done} file(s)."));
        }
        if !errors.is_empty() {
            self.warn(errors.join("\n"));
        }
    }

    /// Swap two fields in every checked file.
    pub fn swap_selected(&mut self, a: Field, b: Field) {
        if a == b {
            self.warn("Cannot swap tag field with it destination!");
            return;
        }
        let indices = self.checked_indices();
        if indices.is_empty() {
            self.warn("Cannot swap tag field with it destination!\nSelect (check) the files first.");
            return;
        }
        let mut done = 0;
        let mut errors = Vec::new();
        for i in &indices {
            let f = &self.files[*i];
            let mut new_fields = f.fields.clone();
            tools::swap_fields(&mut new_fields, a, b);
            let checked = vec![a, b];
            match data::save_fields(&f.path, &new_fields, &checked) {
                Ok(()) => {
                    if let Some(reloaded) = data::read_file(&f.path) {
                        self.files[*i] = reloaded;
                    }
                    done += 1;
                }
                Err(e) => errors.push(e),
            }
        }
        if done > 0 {
            self.say(format!("Swapped {} <-> {} in {done} file(s).", a.label(), b.label()));
        }
        if !errors.is_empty() {
            self.warn(errors.join("\n"));
        }
    }

    /// Copy Text from One Tag Field to Another, for every checked file.
    pub fn copy_field_selected(&mut self, from: Field, to: Field) {
        if from == to {
            self.warn("Cannot copy tag field to this destination!");
            return;
        }
        let indices = self.checked_indices();
        if indices.is_empty() {
            self.warn("Cannot copy tag field to this destination!\nSelect (check) the files first.");
            return;
        }
        let mut done = 0;
        let mut errors = Vec::new();
        for i in &indices {
            let f = &self.files[*i];
            let mut new_fields = f.fields.clone();
            tools::copy_field(&mut new_fields, from, to);
            let checked = vec![to];
            match data::save_fields(&f.path, &new_fields, &checked) {
                Ok(()) => {
                    if let Some(reloaded) = data::read_file(&f.path) {
                        self.files[*i] = reloaded;
                    }
                    done += 1;
                }
                Err(e) => errors.push(e),
            }
        }
        if done > 0 {
            self.say(format!(
                "Copied {} to {} in {done} file(s).",
                from.label(),
                to.label()
            ));
        }
        if !errors.is_empty() {
            self.warn(errors.join("\n"));
        }
    }

    /// An Artist Field Tool applied to every checked file.
    pub fn artist_tool(&mut self, tool: ArtistTool) {
        let indices = self.checked_indices();
        if indices.is_empty() {
            self.warn("Nothing to do!\nSelect (check) the files first.");
            return;
        }
        let mut done = 0;
        let mut errors = Vec::new();
        for i in &indices {
            let f = &self.files[*i];
            let new_artist = tool.apply(&f.fields.artist);
            if new_artist == f.fields.artist {
                continue;
            }
            let mut new_fields = f.fields.clone();
            new_fields.artist = new_artist;
            match data::save_fields(&f.path, &new_fields, &[Field::Artist]) {
                Ok(()) => {
                    if let Some(reloaded) = data::read_file(&f.path) {
                        self.files[*i] = reloaded;
                    }
                    done += 1;
                }
                Err(e) => errors.push(e),
            }
        }
        if done > 0 {
            self.say(format!("{}: applied to {done} file(s).", tool.label()));
        } else {
            self.say("Nothing to do!");
        }
        if !errors.is_empty() {
            self.warn(errors.join("\n"));
        }
    }

    /// Remove trailing (or all) spaces in every checked file's fields.
    pub fn remove_spaces(&mut self, all: bool) {
        let indices = self.checked_indices();
        if indices.is_empty() {
            self.warn("Nothing to do!\nSelect (check) the files first.");
            return;
        }
        let mut done = 0;
        let mut errors = Vec::new();
        for i in &indices {
            let f = &self.files[*i];
            let mut new_fields = f.fields.clone();
            if all {
                tools::remove_all_spaces(&mut new_fields);
            } else {
                tools::remove_trailing_spaces(&mut new_fields);
            }
            let checked: Vec<Field> = ALL_FIELDS.to_vec();
            match data::save_fields(&f.path, &new_fields, &checked) {
                Ok(()) => {
                    if let Some(reloaded) = data::read_file(&f.path) {
                        self.files[*i] = reloaded;
                    }
                    done += 1;
                }
                Err(e) => errors.push(e),
            }
        }
        if done > 0 {
            self.say(format!("Removed spaces in {done} file(s)."));
        }
        if !errors.is_empty() {
            self.warn(errors.join("\n"));
        }
    }

    /// Save an .m3u playlist of the given rows.
    pub fn save_playlist(&mut self, indices: &[usize], path: &Path) {
        let mut out = String::from("#EXTM3U\n");
        for i in indices {
            let f = &self.files[*i];
            let secs = f.duration_secs.round() as i64;
            let artist_title = if f.fields.artist.is_empty() {
                f.fields.title.clone()
            } else if f.fields.title.is_empty() {
                f.fields.artist.clone()
            } else {
                format!("{} - {}", f.fields.artist, f.fields.title)
            };
            out.push_str(&format!("#EXTINF:{secs},{artist_title}\n{}\n", f.path.display()));
        }
        match std::fs::write(path, out) {
            Ok(()) => self.say(format!("Playlist \"{}\" successfully saved!", path.display())),
            Err(e) => self.warn(format!("Can't save playlist: {e}")),
        }
    }

    /// Export tags (CSV / TXT / HTML / XML).
    pub fn export(&mut self, indices: &[usize], fields: &[Field], path: &Path, fmt: ui_dialogs::ExportFormat) {
        let esc = |s: &str| -> String {
            match fmt {
                ui_dialogs::ExportFormat::Csv => s.replace('"', "\"\""),
                ui_dialogs::ExportFormat::Html => s
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;"),
                ui_dialogs::ExportFormat::Xml => s
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;"),
                _ => s.to_string(),
            }
        };
        let body = match fmt {
            ui_dialogs::ExportFormat::Csv => {
                let mut s = fields.iter().map(|f| f.label()).collect::<Vec<_>>().join(",") + "\n";
                for i in indices {
                    let f = &self.files[*i];
                    let row: Vec<String> = fields
                        .iter()
                        .map(|fl| format!("\"{}\"", esc(f.fields.get(*fl))))
                        .collect();
                    s.push_str(&row.join(","));
                    s.push('\n');
                }
                s
            }
            ui_dialogs::ExportFormat::Txt => {
                let mut s = String::new();
                for i in indices {
                    let f = &self.files[*i];
                    s.push_str(&f.name);
                    s.push('\n');
                    for fl in fields {
                        s.push_str(&format!("  {}: {}\n", fl.label(), f.fields.get(*fl)));
                    }
                    s.push('\n');
                }
                s
            }
            ui_dialogs::ExportFormat::Html => {
                let mut s = String::from("<!DOCTYPE html>\n<html>\n<head><meta charset=\"utf-8\"><title>Tag export</title></head>\n<body>\n<table border=\"1\">\n<tr>");
                for fl in fields {
                    s.push_str(&format!("<th>{}</th>", esc(fl.label())));
                }
                s.push_str("</tr>\n");
                for i in indices {
                    let f = &self.files[*i];
                    s.push_str("<tr>");
                    for fl in fields {
                        s.push_str(&format!("<td>{}</td>", esc(f.fields.get(*fl))));
                    }
                    s.push_str("</tr>\n");
                }
                s.push_str("</table>\n</body>\n</html>\n");
                s
            }
            ui_dialogs::ExportFormat::Xml => {
                let mut s = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<tracks>\n");
                for i in indices {
                    let f = &self.files[*i];
                    s.push_str("  <track>\n");
                    for fl in fields {
                        s.push_str(&format!(
                            "    <{tag}>{value}</{tag}>\n",
                            tag = format!("{:?}", fl).to_lowercase(),
                            value = esc(f.fields.get(*fl))
                        ));
                    }
                    s.push_str("  </track>\n");
                }
                s.push_str("</tracks>\n");
                s
            }
        };
        match std::fs::write(path, body) {
            Ok(()) => self.say(format!(
                "Tags information from {} files successfully exported!",
                indices.len()
            )),
            Err(e) => self.warn(format!("Can't save export file: {e}")),
        }
    }

    /// Open the focused file with the system player.
    pub fn play_file(&mut self, i: usize) {
        let f = &self.files[i];
        #[cfg(unix)]
        let spawned = std::process::Command::new("xdg-open")
            .arg(&f.path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
        #[cfg(windows)]
        let spawned = std::process::Command::new("cmd")
            .args(["/C", "start", ""])
            .arg(&f.path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
        #[cfg(not(any(unix, windows)))]
        let spawned: std::io::Result<std::process::Child> =
            Err(std::io::Error::other("unsupported platform"));
        match spawned {
            Ok(_) => self.say(format!("Play file: {}", f.name)),
            Err(e) => self.warn(format!(
                "Music player not found ({e}).\nPlease select a music player, or open the file yourself:\n{}",
                f.path.display()
            )),
        }
    }

    /// Delete the checked files (with a confirmation, like the original).
    pub fn delete_files(&mut self, indices: &[usize]) {
        let mut done = 0;
        let mut keep = Vec::new();
        for (i, f) in self.files.iter().enumerate() {
            if indices.contains(&i) && std::fs::remove_file(&f.path).is_ok() {
                done += 1;
                continue;
            }
            keep.push(f.clone());
        }
        self.files = keep;
        self.checked = vec![false; self.files.len()];
        self.focused = None;
        self.say(format!("Deleted {done} file(s)."));
    }
}

/// The Filters toolbar's match rule. The original's help file
/// (the original help file, 0210_filters.htm) documents the patterns:
/// "b*" finds all values starting with "b", "*text*" finds values
/// containing "text", "*text" finds values ending with "text".
/// What a filter field holds (see `App::filter_value`). No numeric
/// variant: the decompile proves the engine never reads a numeric
/// rating field (+0xc4 has no case in sub_8413fc or sub_83d600), so
/// every stored field is matched as text.
#[derive(Clone, Debug)]
pub enum FilterValue {
    Text(String),
    Flag(bool),
}

/// "5" + "12" -> "5/12": the original's track and disc fields can carry
/// their total after a '/' (the %6 token drops it at 0xca0386).
fn with_total(value: &str, total: &str) -> String {
    if total.is_empty() {
        value.to_string()
    } else {
        format!("{value}/{total}")
    }
}

/// sub_640954 @ 0x640954: h:mm:ss — hours are omitted when zero, minutes
/// and seconds are always two digits ("00:"/"00"). The rename engine
/// additionally swaps ':' for '-'; the filter does not.
fn play_time_hmmss(secs_f: f64) -> String {
    let secs = secs_f.round() as u64;
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    let mut out = String::new();
    if h != 0 {
        out.push_str(&format!("{h:02}:"));
    }
    out.push_str(&format!("{m:02}:{s:02}"));
    out
}

/// The original's filter matching, decompiled from the tail of
/// sub_8413fc @ 0x8413fc: the field value and the typed text both go
/// through sub_40ae48 (which only replaces a nil string with the empty
/// default) and are handed to **PathMatchSpecW**. That means:
///
/// - `*` and `?` are wildcards; matching is case-insensitive.
/// - A pattern without wildcards must match the WHOLE value — a bare
///   word is an exact match, not "contains".
/// - Several patterns may be given separated with `;` (PathMatchSpec
///   semantics): any one matching is enough.
/// - An empty pattern never reaches PathMatchSpecW: the branch at
///   0x84177x keeps only files whose field is empty (so "Filter!" with
///   an empty box finds files that are missing the selected field).
pub fn filter_match(value: &str, pattern: &str) -> bool {
    let pattern = pattern.trim();
    if pattern.is_empty() {
        return value.trim().is_empty();
    }
    pattern.split(';').any(|spec| glob_match(value, spec))
}

/// One PathMatchSpec spec against one value: whole-value match,
/// case-insensitive, `*` (any run) and `?` (one char).
fn glob_match(value: &str, spec: &str) -> bool {
    let v: Vec<char> = value.to_lowercase().chars().collect();
    let p: Vec<char> = spec.trim().to_lowercase().chars().collect();
    let (mut vi, mut pi) = (0usize, 0usize);
    let (mut star, mut mark) = (usize::MAX, 0usize);
    while vi < v.len() {
        if pi < p.len() && (p[pi] == '?' || p[pi] == v[vi]) {
            vi += 1;
            pi += 1;
        } else if pi < p.len() && p[pi] == '*' {
            star = pi;
            mark = vi;
            pi += 1;
        } else if star != usize::MAX {
            pi = star + 1;
            mark += 1;
            vi = mark;
        } else {
            return false;
        }
    }
    while pi < p.len() && p[pi] == '*' {
        pi += 1;
    }
    pi == p.len()
}

#[cfg(test)]
pub(crate) static ROOTS_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
pub(crate) fn roots_test_lock() -> std::sync::MutexGuard<'static, ()> {
    ROOTS_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

#[cfg(test)]
mod tests {
    use super::filter_match;

    #[test]
    fn filter_documented_examples() {
        // The exact examples from the original's help file.
        assert!(filter_match("beautiful day", "b*"));
        assert!(!filter_match("ambient day", "b*"));
        assert!(filter_match("some text here", "*text*"));
        assert!(!filter_match("some txet here", "*text*"));
        assert!(filter_match("the text", "*text"));
        assert!(!filter_match("the text book", "*text"));
    }

    #[test]
    fn filter_matches_pathmatchspec_semantics() {
        // PathMatchSpecW: a bare word matches the whole value only
        // (sub_8413fc passes the raw pattern through), and matching is
        // case-insensitive.
        assert!(filter_match("beautiful day", "beautiful day"));
        assert!(!filter_match("beautiful day", "day"));
        assert!(filter_match("Beautiful Day", "beautiful day"));
        // '?' is a single-character wildcard.
        assert!(filter_match("day1", "day?"));
        assert!(!filter_match("day12", "day?"));
        // Several specs may be separated with ';'.
        assert!(filter_match("ambient day", "b*;a*"));
        assert!(!filter_match("ambient day", "b*;c*"));
        // An empty pattern selects files whose field is empty
        // (0x84177x compares the trimmed value with '').
        assert!(filter_match("", ""));
        assert!(filter_match("  ", ""));
        assert!(!filter_match("value", ""));
        // '*' alone matches anything non-empty.
        assert!(filter_match("anything", "*"));
        // Multiple stars behave like the usual glob.
        assert!(filter_match("a hard day's night", "a*day*night"));
        assert!(!filter_match("a hard day's night", "a*night*day"));
    }

    /// Wait for a background folder scan to finish and apply it.
    fn settle(app: &mut super::App) {
        let mut guard = 0;
        while app.loading.is_some() {
            app.poll_loading();
            std::thread::sleep(std::time::Duration::from_millis(2));
            guard += 1;
            assert!(guard < 10000, "scan did not settle");
        }
    }

    /// A minimal silent mono 16-bit 44.1 kHz WAV (RIFF), which lofty can
    /// read like any real music file.
    fn silent_wav(secs: u32) -> Vec<u8> {
        let samples = 44_100 * secs;
        let data_len = samples * 2;
        let mut out = Vec::new();
        out.extend_from_slice(b"RIFF");
        out.extend_from_slice(&(36 + data_len).to_le_bytes());
        out.extend_from_slice(b"WAVEfmt ");
        out.extend_from_slice(&16u32.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes()); // PCM
        out.extend_from_slice(&1u16.to_le_bytes()); // mono
        out.extend_from_slice(&44_100u32.to_le_bytes());
        out.extend_from_slice(&88_200u32.to_le_bytes()); // byte rate
        out.extend_from_slice(&2u16.to_le_bytes()); // block align
        out.extend_from_slice(&16u16.to_le_bytes()); // bits
        out.extend_from_slice(b"data");
        out.extend_from_slice(&data_len.to_le_bytes());
        out.resize(out.len() + data_len as usize, 0);
        out
    }

    /// Adding a library root is ONLY a tree operation: no files are read,
    /// no subfolder search is switched on. Selecting a child folder then
    /// loads just that folder's music — /mnt/pool/Music/Music holds
    /// ~20,000 songs in ~1,000 artist folders.
    #[test]
    fn masks_survive_a_restart() {
        super::save_masks("%2 - %1\\%7", "%1.%2", &["%1 - %2".to_string(), "%4 %3".to_string()]);
        let (rename, f2t, saved) = super::load_masks().expect("masks file is read back");
        assert_eq!(rename, "%2 - %1\\%7");
        assert_eq!(f2t, "%1.%2");
        assert_eq!(saved, vec!["%1 - %2".to_string(), "%4 %3".to_string()]);
    }

    #[test]
    fn adding_a_root_does_not_read_music_until_a_child_is_selected() {
        let _roots = super::roots_test_lock();
        let root = std::env::temp_dir().join("tagrename_scan_test");
        let _ = std::fs::remove_dir_all(&root);
        let sub = root.join("Artist One");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(sub.join("song a.wav"), silent_wav(1)).unwrap();
        std::fs::write(sub.join("song b.wav"), silent_wav(2)).unwrap();

        let mut app = super::App::new();
        app.roots.clear();
        app.add_root(&root);
        assert_eq!(app.roots, vec![root.clone()]);
        assert!(super::load_roots().contains(&root), "an added root is saved for the next start");
        assert!(!app.expanded.get(&root).copied().unwrap_or(false), "a new root starts collapsed");
        assert!(!app.search_subfolders, "adding a root must not search subfolders");
        assert!(app.loading.is_none(), "adding a root must not start a scan");
        assert!(app.files.is_empty(), "the file list must be left alone");

        // With the box off, a parent/artist folder with no music of its
        // own must load nothing — no folder rows, no files.
        app.load_folder(&root);
        settle(&mut app);
        assert!(app.files.is_empty(), "an artist folder loads nothing when Search Subfolders is off");
        assert!(!app.search_subfolders, "no automatic recursive reload");
        assert_eq!(app.folder.as_deref(), Some(root.as_path()));

        // Ticking the box lists the album folders directly under it.
        app.search_subfolders = true;
        app.load_folder(&root);
        settle(&mut app);
        assert_eq!(app.files.len(), 1);
        assert!(app.files[0].is_dir);
        assert_eq!(app.files[0].name, "Artist One");

        // With the box off again, an album folder with songs directly in
        // it still loads its own two songs.
        app.search_subfolders = false;
        app.load_folder(&sub);
        settle(&mut app);
        assert_eq!(app.files.len(), 2);
        assert!(app.files.iter().all(|f| !f.is_dir));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn removing_a_root_saves_the_removal() {
        let _roots = super::roots_test_lock();
        let root = std::env::temp_dir().join("tagrename_remove_root_test");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let mut app = super::App::new();
        app.roots.clear();
        app.add_root(&root);
        assert!(super::load_roots().contains(&root));

        app.remove_root(&root);
        assert!(app.roots.is_empty());
        assert!(!super::load_roots().contains(&root), "a removed root must not survive a restart");
        assert!(app.hidden_roots.contains(&root));

        // Adding the same location again makes it visible immediately.
        app.add_root(&root);
        assert!(app.roots.contains(&root));
        assert!(app.hidden_roots.is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }

    /// A folder whose own files sit directly inside it (no subfolders)
    /// must not flip the subfolder search on.
    #[test]
    fn folder_with_direct_files_keeps_the_subfolder_option_off() {
        let root = std::env::temp_dir().join("tagrename_scan_flat");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("song.wav"), silent_wav(1)).unwrap();

        let mut app = super::App::new();
        app.load_folder(&root);
        let mut guard = 0;
        while app.loading.is_some() {
            app.poll_loading();
            std::thread::sleep(std::time::Duration::from_millis(5));
            guard += 1;
            assert!(guard < 4000, "scan did not settle");
        }
        assert!(!app.search_subfolders);
        assert_eq!(app.files.len(), 1);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn tree_type_ahead_moves_the_highlight_without_loading_files() {
        let root = std::env::temp_dir().join("tagrename_tree_jump_test");
        let _ = std::fs::remove_dir_all(&root);
        for name in ["Alpha", "Beta", "Gamma"] {
            std::fs::create_dir_all(root.join(name)).unwrap();
        }
        let mut app = super::App::new();
        app.roots.clear();
        app.tree_selected = Some(root.join("Alpha"));

        app.jump_tree_letter('b');
        assert_eq!(app.tree_selected.as_deref(), Some(root.join("Beta").as_path()));
        assert_eq!(app.tree_scroll_to.as_deref(), Some(root.join("Beta").as_path()));
        assert!(app.tree_focus, "typing in the tree keeps the tree focused");
        assert!(app.files.is_empty(), "jumping the tree must not read music");

        app.jump_tree_letter('a');
        assert_eq!(app.tree_selected.as_deref(), Some(root.join("Alpha").as_path()), "type-ahead wraps");
        let _ = std::fs::remove_dir_all(&root);
    }
}

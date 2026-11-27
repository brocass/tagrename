//! The per-file Tag Editor window (the original's TFORMTAGEDITOR):
//! General / MusicBrainz / ReplayGain / Additional / Lyrics / Sorting
//! tabs, the fields split over two columns. With several files open, every field and the Art box have a
//! tick box that starts off, and Save writes only the ticked ones. A
//! cover-art area with Add / Replace, Save to File and Remove.

use egui::{Layout, RichText, Ui};

use crate::data::{Field, ALL_FIELDS};
use crate::theme::*;
use crate::ui::App;

#[derive(Clone, Debug)]
pub struct EditorResult {
    pub index: usize,
    pub multi: bool,
    pub indices: Vec<usize>,
    pub values: Vec<(Field, String)>,
    /// A compilation-flag change (None = unchanged): the flag is a
    /// boolean, not a string, so it travels separately.
    pub compilation: Option<bool>,
    pub cover_set: Option<(String, std::sync::Arc<Vec<u8>>)>,
    pub cover_removed: bool,
}

/// The persistent editor state (open while the dialog is visible).
pub struct TagEditor {
    /// The row being edited (focused) — single-file mode edits it;
    /// when several files are checked, `multi` is true and changed
    /// fields apply to all of them.
    pub multi: bool,
    pub indices: Vec<usize>,
    pub values: crate::data::TagFields,
    /// The fields the file had when the editor opened. Without the
    /// original's per-field tick boxes, Save writes exactly the fields
    /// whose value differs from these.
    pub original: crate::data::TagFields,
    pub tab: usize,
    /// Staged cover art (not yet saved).
    pub cover_staged: Option<(String, std::sync::Arc<Vec<u8>>)>,
    /// The file's cover when the editor opened (so an untouched cover
    /// is not re-written on Save).
    pub cover_original: Option<(String, std::sync::Arc<Vec<u8>>)>,
    pub cover_removed: bool,
    /// Multi-file mode only: the fields the person ticked to activate.
    /// Every field starts off, and Save writes only the ticked fields.
    pub enabled: Vec<Field>,
    /// Multi-file mode only: the Art tick box. Off by default.
    pub art_enabled: bool,
    /// Whether the window fills the whole app (the top bar's window icon
    /// toggles it). Windowed is the default on every opening.
    pub full: bool,
    /// Changes whenever the window must start again at its remembered
    /// size and centre (each opening, and each switch back from full).
    pub win_gen: u64,
    /// Where the top bar was last dragged to, while it is being dragged.
    pub pos: Option<egui::Pos2>,
    /// How far the window's layout area runs past the frame it draws, on
    /// the right and the bottom. Measured each frame (windowed only), so the
    /// bar, the fields and the button row all end on the drawn frame's edge.
    pub slack: egui::Vec2,
}

/// Height of the custom top bar, and the width of each of its two buttons.
const TITLE_H: f32 = 34.0;
const TITLE_BTN_W: f32 = 30.0;
/// Side inset of the body, and the height of the bottom button band.
const BODY_INSET: f32 = 12.0;
const BUTTON_ROW_H: f32 = 40.0;
/// Room the bottom row keeps from the right edge of a windowed editor, so
/// the resize grip in the bottom-right corner never sits over a button.
const ROW_RIGHT_GAP: f32 = 24.0;

fn next_win_gen() -> u64 {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

impl TagEditor {
    pub fn open(app: &mut App) -> Self {
        let indices = if app.checked_indices().len() > 1 {
            app.checked_indices()
        } else {
            app.focused.into_iter().collect()
        };
        let multi = indices.len() > 1;
        let values = app
            .focused
            .map(|i| app.files[i].fields.clone())
            .unwrap_or_default();
        let cover_original = app.focused.and_then(|i| app.files[i].cover.clone());
        TagEditor {
            multi,
            indices,
            original: values.clone(),
            values,
            tab: 0,
            cover_staged: cover_original.clone(),
            cover_original,
            cover_removed: false,
            enabled: Vec::new(),
            art_enabled: false,
            full: false,
            win_gen: next_win_gen(),
            pos: None,
            slack: egui::vec2(20.0, 5.0),
        }
    }

    /// Single-file mode: edit the focused file, never the checked set.
    /// The original's actFile_EditFileTag handler (0xca6908) always opens
    /// the editor with its multi flag (form+0x7c1) cleared — one song at
    /// a time, which is also what a double click in the file list does.
    pub fn open_focused(app: &mut App) -> Self {
        let mut ed = Self::open(app);
        ed.multi = false;
        ed.indices = app.focused.into_iter().collect();
        ed
    }

    pub fn file_name(&self, app: &App) -> String {
        app.focused
            .map(|i| app.files[i].name.clone())
            .unwrap_or_default()
    }
}

/// "5/12" style merged text for the Track # / Disc # fields (the
/// original keeps the total in the same field).
fn merged_pair(a: &str, b: &str) -> String {
    if b.is_empty() {
        a.to_string()
    } else {
        format!("{a}/{b}")
    }
}

fn split_pair(text: &str) -> (String, String) {
    match text.find('/') {
        Some(p) => (
            text[..p].trim().to_string(),
            text[p + 1..].trim().to_string(),
        ),
        None => (text.trim().to_string(), String::new()),
    }
}

/// One label + widget row (no tick box — the person asked for none).
///
/// The Rating dropdown, drawn outside the grid so its width and position
/// can differ from the other value boxes.
fn rating_combo(ui: &mut Ui, ed: &mut TagEditor, width: f32) {
    let value = ed.values.get(Field::Rating).to_string();
    let stars = crate::tools::stars_from_rating(&value);
    let mut edited = value.clone();
    let mut changed = false;
    egui::ComboBox::from_id_salt("rating_combo")
        .selected_text(crate::tools::rating_from_stars(stars))
        .width(width)
        .show_ui(ui, |ui| {
            for r in crate::tools::RATINGS.iter() {
                // RATINGS is in the original's descending order
                // (index 0 = 5 Stars), so compare through the mapping
                // instead of the loop index.
                if ui.selectable_label(stars == crate::tools::stars_from_rating(r), *r).clicked() {
                    edited = (*r).to_string();
                    changed = true;
                }
            }
        });
    if changed {
        ed.values.set(Field::Rating, &edited);
    }
}

fn field_row(
    ui: &mut Ui,
    ed: &mut TagEditor,
    field: Field,
    width: f32,
) {
    let label = field.label();
    let active = !ed.multi || ed.enabled.contains(&field);
    let _single_col_tab = matches!(ed.tab, 1 | 2 | 4);
    let label_w = if ed.tab == 1 {
        (width * 0.42).clamp(210.0, 250.0)
    } else if ed.tab == 4 {
        (width * 0.38).clamp(130.0, 160.0)
    } else if ed.tab == 2 {
        (width * 0.30).clamp(96.0, 130.0)
    } else {
        (width * 0.30).clamp(96.0, 130.0)
    };
    if ed.multi {
        let mut on = ed.enabled.contains(&field);
        ui.allocate_ui_with_layout(
            egui::vec2(label_w, 20.0),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                let resp = ui.checkbox(&mut on, egui::RichText::new(label).size(13.5));
                if resp.changed() {
                    if on {
                        ed.enabled.push(field);
                    } else {
                        ed.enabled.retain(|f| *f != field);
                    }
                }
            },
        );
    } else {
        ui.allocate_ui_with_layout(
            egui::vec2(label_w, 20.0),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.add(
                    egui::Label::new(egui::RichText::new(label).size(13.5))
                        .selectable(false)
                        .truncate(),
                );
            },
        );
    }
    let widget_w = (width - label_w - 8.0).max(80.0);
    let value = ed.values.get(field).to_string();
    let mut edited = value.clone();
    let mut changed = false;
    let resp = ui.scope(|ui| {
        if !active {
            ui.disable();
        }
        match field {
        Field::Rating => {
            let stars = crate::tools::stars_from_rating(&value);
            let mut changed = false;
            egui::ComboBox::from_id_salt("rating_combo")
                .selected_text(crate::tools::rating_from_stars(stars))
                .width(widget_w)
                .show_ui(ui, |ui| {
                    for r in crate::tools::RATINGS.iter() {
                        // RATINGS is in the original's descending order
                        // (index 0 = 5 Stars), so compare through the mapping
                        // instead of the loop index.
                        if ui.selectable_label(stars == crate::tools::stars_from_rating(r), *r).clicked() {
                            // Store the label ("3 Stars") so the value
                            // round-trips through stars_from_rating.
                            edited = (*r).to_string();
                            changed = true;
                        }
                    }
                });
            if changed {
                ed.values.set(field, &edited);
            }
            None::<egui::Response>
        }
        Field::Comment => {
            // Comment fills roughly three lines and can scroll beyond that.
            let r = egui::ScrollArea::vertical()
                .id_salt("editor_comment_box")
                .max_height(72.0)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut edited)
                            .desired_width(widget_w)
                            .desired_rows(3)
                            .min_size(egui::vec2(widget_w, 56.0))
                            .hint_text(""),
                    )
                })
                .inner;
            if r.changed() {
                ed.values.set(field, &edited);
            }
            Some(r)
        }
        // The single Track # / Disc # fields carry the total after the
        // '/', exactly like the original's fields ("5/12").
        Field::Track | Field::Disc => {
            let (a, b) = if field == Field::Track {
                (&ed.values.track, &ed.values.track_total)
            } else {
                (&ed.values.disc, &ed.values.disc_total)
            };
            let mut text = merged_pair(a, b);
            let r = ui.add_sized(
                [widget_w, 20.0],
                egui::TextEdit::singleline(&mut text).hint_text(""),
            );
            if r.changed() {
                let (v, total) = split_pair(&text);
                if field == Field::Track {
                    ed.values.track = v;
                    ed.values.track_total = total;
                } else {
                    ed.values.disc = v;
                    ed.values.disc_total = total;
                }
            }
            Some(r)
        }
        Field::Compilation => {
            let mut on = ed.values.compilation == Some(true);
            let r = ui.add_sized([18.0, 20.0], egui::Checkbox::new(&mut on, ""));
            if r.changed() {
                ed.values.compilation = Some(on);
            }
            Some(r)
        }
        _ => {
            let r = ui.add_sized(
                [widget_w, 20.0],
                egui::TextEdit::singleline(&mut edited).hint_text(""),
            );
            if r.changed() {
                ed.values.set(field, &edited);
            }
            Some(r)
        }
        }
    })
    .inner;
    let _ = resp;
}

/// Draw the editor window. Returns whether it should stay open: false
/// when the close cross, Cancel or Save closed it. (The caller used to
/// re-insert the editor unconditionally, which silently undid every
/// close — the cross "did nothing".)
pub fn show(ctx: &egui::Context, app: &mut App, ed: &mut TagEditor) -> bool {
    let title = if ed.multi {
        format!("Tag Editor [{} File(s)]", ed.indices.len())
    } else {
        format!("Tag Editor [{}]", ed.file_name(app))
    };
    let mut open = true;
    let ppp = ctx.pixels_per_point().max(0.1);
    let screen = ctx
        .input(|i| i.raw.screen_rect)
        .unwrap_or_else(|| egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1180.0, 800.0)));
    // Windowed: the remembered size (or a default), centred in the app.
    // Full screen: the whole app.
    let default_size = egui::vec2(
        (screen.width() * 0.7).clamp(560.0, 900.0),
        (screen.height() * 0.75).clamp(440.0, 720.0),
    );
    let windowed = app.dlg.editor_size.unwrap_or(default_size).min(screen.size());
    let centred = egui::pos2(
        screen.center().x - windowed.x / 2.0,
        screen.center().y - windowed.y / 2.0,
    );
    let info = app
        .focused
        .and_then(|i| app.files.get(i))
        .map(|f| {
            format!(
                "   {}   ·   {} kbps   ·   {} ch   ·   {} Hz",
                f.play_time(), f.bitrate_kbps, f.channels, f.sample_rate
            )
        })
        .unwrap_or_default();
    let caption = format!("{title}{info}");
    let mut keep = true;
    let mut toggle_full = false;
    // A custom title strip gives the same figure/ground
    // relationship as the original (blue caption over a light body),
    // translated into the requested dark palette. A hand-drawn close
    // button keeps the X working without egui's mid-gray title bar.
    let frame = egui::Frame::new()
        .fill(c_dialog_bg())
        .stroke(egui::Stroke::new(1.0, c_dialog_edge()))
        .corner_radius(egui::CornerRadius::same(WINDOW_RADIUS))
        .shadow(ctx.global_style().visuals.window_shadow)
        .inner_margin(egui::Margin::same(0));
    let mut window = egui::Window::new(title.clone())
        .id(egui::Id::new(("tag_editor", ed.win_gen)))
        .open(&mut open)
        .title_bar(false)
        .collapsible(false)
        .frame(frame)
        .min_size([560.0, 440.0])
        .max_size(screen.size());
    window = {
        let window = match ed.pos {
            Some(at) => window.current_pos(at),
            None => window.default_pos(centred),
        };
        window
            .default_size(windowed)
            .resizable(true)
            .movable(true)
    };
    let mut bar_drag: Option<egui::Vec2> = None;
    let mut content_log = egui::Rect::NOTHING;
    let full = ed.full;
    let win_gen = ed.win_gen;
    let slack = if full { egui::Vec2::ZERO } else { ed.slack };
    let mut draw = |ui: &mut Ui| {
            // The layout area can run past the frame that is drawn (by `slack`
            // in a window). `frame_r` is the drawn frame, and everything below
            // is laid out inside it, so the right edges all meet the frame.
            let area_r = ui.max_rect();
            content_log = area_r;
            let content = egui::Rect::from_min_max(area_r.min, area_r.max - slack);
            // The top bar runs edge to edge; its buttons sit on the frame's edge.
            let bar = egui::Rect::from_min_max(
                content.min,
                egui::pos2(content.max.x, content.min.y + TITLE_H),
            );
            let top = egui::CornerRadius { nw: WINDOW_RADIUS, ne: WINDOW_RADIUS, sw: 0, se: 0 };
            ui.painter().rect_filled(bar, top, c_panel_bright());
            ui.painter().line_segment(
                [bar.left_bottom(), bar.right_bottom()],
                egui::Stroke::new(1.0, c_separator()),
            );
            let drag = ui.interact(bar, ui.id().with("editor_drag"), egui::Sense::drag());
            if drag.dragged() {
                bar_drag = Some(drag.drag_delta());
            }
            let text_area = egui::Rect::from_min_max(
                bar.min,
                egui::pos2(bar.max.x - 2.0 * TITLE_BTN_W, bar.max.y),
            );
            ui.painter().with_clip_rect(text_area).text(
                bar.left_center() + egui::vec2(BODY_INSET, 0.0),
                egui::Align2::LEFT_CENTER,
                &caption,
                egui::FontId::proportional(13.5),
                c_text(),
            );
            let close_rect = egui::Rect::from_min_max(
                egui::pos2(bar.max.x - TITLE_BTN_W, bar.min.y),
                bar.max,
            );
            let close_resp = ui
                .interact(close_rect, ui.id().with("editor_close"), egui::Sense::click())
                .on_hover_text("Close");
            if close_resp.hovered() {
                let corner = egui::CornerRadius { nw: 0, ne: WINDOW_RADIUS, sw: 0, se: 0 };
                ui.painter().rect_filled(close_rect, corner, pal().widget_hover);
            }
            let stroke = egui::Stroke::new(1.3, c_text());
            let x = close_rect.center();
            ui.painter().line_segment([x + egui::vec2(-5.0, -5.0), x + egui::vec2(5.0, 5.0)], stroke);
            ui.painter().line_segment([x + egui::vec2(5.0, -5.0), x + egui::vec2(-5.0, 5.0)], stroke);
            if close_resp.clicked() {
                keep = false;
            }
            let full_rect = close_rect.translate(egui::vec2(-TITLE_BTN_W, 0.0));
            let full_resp = ui
                .interact(full_rect, ui.id().with("editor_full_toggle"), egui::Sense::click())
                .on_hover_text(if ed.full { "Restore window size" } else { "Full screen" });
            if full_resp.hovered() {
                ui.painter().rect_filled(full_rect, egui::CornerRadius::same(0), pal().widget_hover);
            }
            let c = full_rect.center();
            if ed.full {
                let back = egui::Rect::from_center_size(c + egui::vec2(2.0, -2.0), egui::vec2(9.0, 9.0));
                let front = egui::Rect::from_center_size(c + egui::vec2(-2.0, 2.0), egui::vec2(9.0, 9.0));
                ui.painter().rect_stroke(back, egui::CornerRadius::same(0), stroke, egui::StrokeKind::Middle);
                ui.painter().rect_filled(front, egui::CornerRadius::same(0), c_panel_bright());
                ui.painter().rect_stroke(front, egui::CornerRadius::same(0), stroke, egui::StrokeKind::Middle);
            } else {
                let square = egui::Rect::from_center_size(c, egui::vec2(11.0, 11.0));
                ui.painter().rect_stroke(square, egui::CornerRadius::same(0), stroke, egui::StrokeKind::Middle);
            }
            if full_resp.clicked() {
                toggle_full = true;
            }
            // Compact density: the original's 1990s fields were much
            // smaller than the rest of this rebuild's widgets.
            ui.style_mut().spacing.item_spacing = egui::vec2(4.0, 1.0);
            ui.style_mut().spacing.button_padding = egui::vec2(6.0, 1.0);
            ui.style_mut()
                .text_styles
                .insert(egui::TextStyle::Body, egui::FontId::proportional(13.5));
            ui.style_mut()
                .text_styles
                .insert(egui::TextStyle::Button, egui::FontId::proportional(13.5));
            // The body gets the area under the title bar, with the same
            // inset on the left and the right, so the bottom row sits the
            // same distance from each side of the window.
            let below = egui::Rect::from_min_max(
                content.min + egui::vec2(BODY_INSET, TITLE_H + 4.0),
                content.max - egui::vec2(BODY_INSET, 6.0),
            );
            ui.scope_builder(egui::UiBuilder::new().max_rect(below), |ui| {
                body(ui, app, ed, &mut keep, below);
            });
            // Report the whole area as used, so the window keeps its full size
            // (a window shrinks to what its contents report).
            ui.allocate_rect(area_r, egui::Sense::hover());
    };
    let win_rect = if full {
        drop(window);
        Some(
            egui::Area::new(egui::Id::new(("tag_editor_full", win_gen)))
                .order(egui::Order::Middle)
                .fixed_pos(screen.min)
                .constrain(false)
                .show(ctx, |ui| {
                    ui.set_min_size(screen.size());
                    ui.set_max_size(screen.size());
                    ui.painter().rect_filled(
                        egui::Rect::from_min_size(screen.min, screen.size()),
                        egui::CornerRadius::same(0),
                        c_dialog_bg(),
                    );
                    draw(ui);
                })
                .response
                .rect,
        )
    } else {
        window.show(ctx, |ui| draw(ui)).map(|w| w.response.rect)
    };
    if let Some(rect) = win_rect {
        if !ed.full {
            ed.slack = (content_log.max - rect.max).max(egui::Vec2::ZERO);
            app.dlg.editor_size = Some(rect.size());
            let size = rect.size();
            ed.pos = bar_drag.map(|d| {
                let at = rect.min + d;
                egui::pos2(
                    at.x.clamp(screen.min.x, (screen.max.x - size.x).max(screen.min.x)),
                    at.y.clamp(screen.min.y, (screen.max.y - TITLE_H).max(screen.min.y)),
                )
            });
        }
    }
    if let Some(shot) = &app.shot {
        if shot.frame == 21 || shot.frame == 109 {
            if let Some(rect) = win_rect {
                crate::shot_log(&format!(
                    "editor f{} drawn_rect={:?} screen_rect={:?} ppp={} full={} content={:?} windowed={:?} slack={:?}",
                    shot.frame,
                    rect,
                    ctx.input(|i| i.raw.screen_rect),
                    ppp,
                    ed.full,
                    content_log,
                    windowed,
                    ed.slack
                ));
            }
        }
    }
    if toggle_full {
        ed.full = !ed.full;
        ed.win_gen = next_win_gen();
    }
    open && keep
}

fn body(ui: &mut Ui, app: &mut App, ed: &mut TagEditor, keep: &mut bool, below: egui::Rect) {
    let bottom = below.bottom();
    ui.horizontal(|ui| {
        ui.label("File:");
        let mut name = ed.file_name(app);
        ui.add(
            egui::TextEdit::singleline(&mut name)
                .interactive(false)
                // Bounded: on the editor window's first frame
                // available_width can be huge, which used to blow the
                // window up past the screen.
                .desired_width(ui.available_width()),
        );
    });
    // File details live in the caption now, so the compact dialog can
    // show every field without a separate read-only row.
    ui.separator();

    ui.horizontal(|ui| {
        for (i, name) in ["General", "MusicBrainz", "ReplayGain", "Lyrics", "Sorting"]
            .iter()
            .enumerate()
        {
            if ui.selectable_label(ed.tab == i, *name).clicked() {
                ed.tab = i;
            }
        }
        // Cap the available height before the right-to-left scope.
        ui.set_max_height(26.0);
        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .small_button("Clear All")
                .on_hover_text("Empty every field in this editor (the file is not touched until Save)")
                .clicked()
            {
                for f in ALL_FIELDS {
                    ed.values.set(*f, "");
                }
                ed.values.compilation = None;
                ed.values.lyrics.clear();
            }
        });
    });
    ui.add_space(2.0);

    match ed.tab {
        3 => lyrics_tab(ui, app, ed, bottom),
        _ => {
            // Two columns of fields (the person's request), plus the
            // cover-art area on the right. Every column is bounded so
            // nothing can push the editor wider than its window.
            // The fields and the Art box get exact rectangles inside the
            // body, so the Art box ends on the body's right edge (the same
            // edge as the top bar's buttons and the bottom row), in every size.
            let top = ui.cursor().min.y;
            let field_area = egui::Rect::from_min_max(
                egui::pos2(below.min.x, top),
                egui::pos2(below.max.x, (bottom - BUTTON_ROW_H - 6.0).max(top + 1.0)),
            );
            ui.allocate_rect(field_area, egui::Sense::hover());
            let avail = field_area.width();
            let base_art_w = (avail * 0.18).clamp(110.0, 150.0);
            // The art box uses the General tab's larger size on every
            // field tab (MusicBrainz, ReplayGain and Sorting too).
            let art_w = (avail * 0.22).clamp(150.0, 190.0);
            let art_rect = egui::Rect::from_min_max(
                egui::pos2(field_area.max.x - art_w, field_area.min.y),
                field_area.max,
            );
            // General tab: two equal columns + art on right. Keep columns inside
            // the available width so the right column never slides under art.
            let single_col = matches!(ed.tab, 1 | 2 | 4);
            let base_col_gap = 4.0f32;
            let base_art_gap = 6.0f32;
            let base_col_w = ((avail - base_art_w - base_col_gap - base_art_gap) / 2.0).max(160.0);
            let base_label_w = (base_col_w * 0.44).clamp(96.0, 136.0);
            let base_widget_w = (base_col_w - base_label_w - 8.0).max(80.0);

            let rg_label_w = (base_col_w * 0.30).clamp(96.0, 130.0);
            let rg_col_w = (rg_label_w + 8.0 + base_widget_w).min((avail - base_art_w - base_art_gap).max(320.0));

            let art_gap = 6.0f32;
            let (col_w, col_gap, art_gap) = if ed.tab == 1 {
                let label_w = (base_col_w * 0.52).clamp(210.0, 250.0);
                let widget_w = (base_widget_w + 120.0).max(300.0);
                let one_w = (label_w + 8.0 + widget_w).min((avail - art_w - art_gap).max(560.0));
                (one_w, 0.0f32, art_gap)
            } else if matches!(ed.tab, 2 | 4) {
                let label_w = if ed.tab == 4 {
                    (base_col_w * 0.38).clamp(130.0, 160.0)
                } else {
                    rg_label_w
                };
                let one_w = (label_w + 8.0 + base_widget_w).min((avail - art_w - art_gap).max(320.0));
                (one_w, 0.0f32, art_gap)
            } else {
                // General: same column/label/widget sizing as ReplayGain, tight gap.
                let col_gap = 0.0f32;
                let art_gap = 10.0f32;
                let max_each = ((avail - art_w - col_gap - art_gap) / 2.0).max(120.0);
                let col_w = rg_col_w.min(max_each);
                (col_w, col_gap, art_gap)
            };
            {
                // Tabs: General, MusicBrainz, ReplayGain, Lyrics, Sorting.
                let (col_a, col_b): (&[Field], &[Field]) = match ed.tab {
                    0 => (
                        &[
                            Field::Title,
                            Field::Artist,
                            Field::AlbumArtist,
                            Field::Album,
                            Field::Year,
                            Field::Genre,
                            Field::Track,
                            Field::Disc,
                            Field::Catalog,
                            Field::Isrc,
                            Field::Bpm,
                            Field::Mood,
                        ],
                        &[
                            Field::Label,
                            Field::RemixedBy,
                            Field::Composer,
                            Field::OrigArtist,
                            Field::Copyright,
                            Field::Url,
                            Field::EncodedBy,
                            Field::Comment,
                        ],
                    ),
                    1 => (
                        &[
                            Field::MbAlbumType,
                            Field::MbAlbumArtistId,
                            Field::MbAlbumId,
                            Field::MbAlbumStatus,
                            Field::MbArtistId,
                            Field::MbRecordingId,
                            Field::MbReleaseGroupId,
                            Field::MbReleaseTrackId,
                            Field::MbReleaseType,
                            Field::MbTrackId,
                            Field::MbAlbumReleaseCountry,
                            Field::MbWorkId,
                        ],
                        &[],
                    ),
                    2 => (
                        &[
                            Field::RgTrackGain,
                            Field::RgTrackPeak,
                            Field::RgAlbumGain,
                            Field::RgAlbumPeak,
                        ],
                        &[],
                    ),
                    _ => (
                        &[
                            Field::SortTitle,
                            Field::SortArtist,
                            Field::SortAlbumArtist,
                        ],
                        &[
                            Field::SortAlbum,
                            Field::SortComposer,
                            Field::SortShow,
                        ],
                    ),
                };
                if single_col {
                    // MusicBrainz, ReplayGain and Sorting: one column + Art.
                    let one_w = col_w.min((avail - art_w - art_gap).max(120.0));
                    let all: Vec<Field> = col_a.iter().chain(col_b.iter()).copied().collect();
                    let one_rect = egui::Rect::from_min_size(
                        field_area.min,
                        egui::vec2(one_w, field_area.height()),
                    );
                    let mut one = ui.new_child(egui::UiBuilder::new().max_rect(one_rect));
                    one.set_clip_rect(one_rect);
                    one.vertical(|ui| {
                        ui.set_min_width(one_w);
                        ui.set_max_width(one_w);
                        egui::Grid::new("editor_single_col")
                            .num_columns(2)
                            .spacing([8.0, 4.0])
                            .min_col_width(0.0)
                            .show(ui, |ui| {
                                for f in all.iter() {
                                    field_row(ui, ed, *f, one_w);
                                    ui.end_row();
                                }
                            });
                    });
                    let mut art = ui.new_child(egui::UiBuilder::new().max_rect(art_rect));
                    art.set_clip_rect(art_rect);
                    art_panel(&mut art, app, ed, art_w);
                } else {
                    // General: two equal field columns, art on the right.
                    let total_cols_w = (col_w * 2.0 + col_gap).min((avail - art_w - art_gap).max(120.0));
                    let col_w = ((total_cols_w - col_gap) / 2.0).max(100.0);
                    for (n, col) in [col_a, col_b].into_iter().enumerate() {
                        let x = field_area.min.x + n as f32 * (col_w + col_gap);
                        let col_rect = egui::Rect::from_min_size(
                            egui::pos2(x, field_area.min.y),
                            egui::vec2(col_w, field_area.height()),
                        );
                        let mut colui = ui.new_child(egui::UiBuilder::new().max_rect(col_rect));
                        colui.set_clip_rect(col_rect);
                        colui.vertical(|ui| {
                            ui.set_min_width(col_w);
                            ui.set_max_width(col_w);
                            egui::Grid::new(format!("editor_col_{n}"))
                                .num_columns(2)
                                .spacing([6.0, 4.0])
                                .min_col_width(0.0)
                                .show(ui, |ui| {
                                    for f in col.iter() {
                                        field_row(ui, ed, *f, col_w);
                                        ui.end_row();
                                    }
                                });
                            if n == 1 {
                                // Rating and "Part of a compilation" get their
                                // own rows with their own dimensions: the long
                                // compilation title would widen the grid's
                                // shared label column and push every value box
                                // out of the column. Both labels sit left-
                                // aligned, and the dropdown and the tick start
                                // at the same point, clear of the title.
                                ui.add_space(4.0);
                                let title = "Part of a compilation";
                                let title_w = ui
                                    .painter()
                                    .layout(
                                        title.to_owned(),
                                        egui::FontId::proportional(13.5),
                                        c_text(),
                                        10000.0,
                                    )
                                    .size()
                                    .x;
                                // The value column starts after the long title
                                // (a little further in multi mode, where the
                                // title is the enable-tick's own label).
                                let value_x = if ed.multi {
                                    title_w + 32.0
                                } else {
                                    title_w + 10.0
                                };
                                let combo_w = (col_w - value_x - 4.0).clamp(60.0, 90.0);

                                // Rating: label left-aligned, narrow dropdown.
                                let rating_active =
                                    !ed.multi || ed.enabled.contains(&Field::Rating);
                                ui.horizontal(|ui| {
                                    ui.allocate_ui_with_layout(
                                        egui::vec2(value_x, 20.0),
                                        Layout::left_to_right(egui::Align::Center),
                                        |ui| {
                                            if ed.multi {
                                                let mut on =
                                                    ed.enabled.contains(&Field::Rating);
                                                let r = ui.checkbox(
                                                    &mut on,
                                                    egui::RichText::new("Rating").size(13.5),
                                                );
                                                if r.changed() {
                                                    if on {
                                                        ed.enabled.push(Field::Rating);
                                                    } else {
                                                        ed.enabled.retain(|f| *f != Field::Rating);
                                                    }
                                                }
                                            } else {
                                                ui.add(
                                                    egui::Label::new(
                                                        egui::RichText::new("Rating").size(13.5),
                                                    )
                                                    .selectable(false),
                                                );
                                            }
                                        },
                                    );
                                    if !rating_active {
                                        ui.scope(|ui| {
                                            ui.disable();
                                            rating_combo(ui, ed, combo_w);
                                        });
                                    } else {
                                        rating_combo(ui, ed, combo_w);
                                    }
                                });

                                // Part of a compilation: title left-aligned,
                                // tick starting at the same x as the dropdown.
                                let comp_active = !ed.multi
                                    || ed.enabled.contains(&Field::Compilation);
                                ui.horizontal(|ui| {
                                    ui.allocate_ui_with_layout(
                                        egui::vec2(value_x, 20.0),
                                        Layout::left_to_right(egui::Align::Center),
                                        |ui| {
                                            if ed.multi {
                                                let mut on =
                                                    ed.enabled.contains(&Field::Compilation);
                                                let r = ui.checkbox(
                                                    &mut on,
                                                    egui::RichText::new(title).size(13.5),
                                                );
                                                if r.changed() {
                                                    if on {
                                                        ed.enabled.push(Field::Compilation);
                                                    } else {
                                                        ed.enabled
                                                            .retain(|f| *f != Field::Compilation);
                                                    }
                                                }
                                            } else {
                                                ui.add(
                                                    egui::Label::new(
                                                        egui::RichText::new(title).size(13.5),
                                                    )
                                                    .selectable(false),
                                                );
                                            }
                                        },
                                    );
                                    if !comp_active {
                                        ui.scope(|ui| {
                                            ui.disable();
                                            let mut val = ed.values.compilation == Some(true);
                                            let vr = ui.checkbox(&mut val, "");
                                            if vr.changed() {
                                                ed.values.compilation = Some(val);
                                            }
                                        });
                                    } else {
                                        let mut val = ed.values.compilation == Some(true);
                                        let vr = ui.checkbox(&mut val, "");
                                        if vr.changed() {
                                            ed.values.compilation = Some(val);
                                        }
                                    }
                                });
                            }
                        });
                    }
                    let mut art = ui.new_child(egui::UiBuilder::new().max_rect(art_rect));
                    art.set_clip_rect(art_rect);
                    art_panel(&mut art, app, ed, art_w);
                }
            }
        }
    }

    // Push the button band to the bottom of the full-size window.
    let spare = bottom - ui.cursor().min.y - BUTTON_ROW_H;
    if spare > 0.0 {
        ui.add_space(spare);
    }
    ui.separator();

    // Bottom buttons, in the original's order. Both groups share one band
    // of fixed height, so every button is centred on the same line, and the
    // band stands clear of the resize grip in the window's bottom-right corner.
    // The right-hand group stops ROW_RIGHT_GAP short of the right edge.
    let row = egui::Rect::from_min_max(
        egui::pos2(below.min.x, bottom - BUTTON_ROW_H),
        egui::pos2(below.max.x - if ed.full { 0.0 } else { ROW_RIGHT_GAP }, bottom),
    );
    ui.allocate_rect(row, egui::Sense::hover());
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(row)
            .layout(Layout::left_to_right(egui::Align::Center)),
        |ui| {
        if ui.button("Play").clicked() {
            if let Some(i) = app.focused {
                app.play_file(i);
            }
        }
        if ui.button("Copy from ID3v1 Tag").clicked() {
            app.say("This file's tag has no separate ID3v1 part to copy.");
        }
        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Save and Next").clicked() {
                save_current(app, ed);
                move_focus(app, ed, 1);
            }
            if ui.button("Remove Tag").clicked() {
                if let Some(i) = app.focused {
                    let path = app.files[i].path.clone();
                    match crate::data::remove_tags(&path) {
                        Ok(()) => {
                            if let Some(re) = crate::data::read_file(&path) {
                                app.files[i] = re;
                            }
                            app.say("Tag removed.");
                        }
                        Err(e) => app.warn(e),
                    }
                }
            }
            if ui.button("Cancel").clicked() {
                *keep = false;
            }
            if ui.button("Save").clicked() {
                save_current(app, ed);
                *keep = false;
            }
            if ui.button("Save and Prev").clicked() {
                save_current(app, ed);
                move_focus(app, ed, -1);
            }
        });
        },
    );
}

/// The Lyrics tab: the lyrics memo plus the original's four buttons
/// (Load from Text File / Save to Text File / Clear / Remove Leading
/// Spaces — DFM captions).
fn lyrics_tab(ui: &mut Ui, app: &mut App, ed: &mut TagEditor, bottom: f32) {
    let mut active = true;
    if ed.multi {
        let mut on = ed.enabled.contains(&Field::Lyrics);
        if ui.checkbox(&mut on, "Lyrics").changed() {
            if on {
                ed.enabled.push(Field::Lyrics);
            } else {
                ed.enabled.retain(|f| *f != Field::Lyrics);
            }
        }
        active = on;
    }
    ui.label("Lyrics:");
    let mut text = std::mem::take(&mut ed.values.lyrics);
    // Fills the tab down to the button row, like the other tabs fill the window.
    let box_h = (bottom - ui.cursor().min.y - BUTTON_ROW_H - 42.0).max(120.0);
    let r = ui.add_enabled(
        active,
        egui::TextEdit::multiline(&mut text)
            .desired_width(ui.available_width())
            .desired_rows(9)
            .min_size(egui::vec2(ui.available_width(), box_h))
            .hint_text(""),
    );
    ed.values.lyrics = text;
    let _ = r;
    ui.horizontal_wrapped(|ui| {
        if ui.button("Load from Text File...").clicked() {
            app.dlg.choose = Some(crate::ui_dialogs::ChooseState {
                purpose: crate::ui_dialogs::ChoosePurpose::LoadLyrics,
                dir: crate::ui_dialogs::default_start_dir(app),
                selected: None,
                name: String::new(),
                cancelled: false,
                path_edit: String::new(),
            });
        }
        if ui.button("Save to Text File...").clicked() {
            if ed.values.lyrics.trim().is_empty() {
                app.warn("There are no lyrics to save yet.");
            } else {
                app.dlg.save_lyrics_text = Some(ed.values.lyrics.clone());
                app.dlg.choose = Some(crate::ui_dialogs::ChooseState {
                    purpose: crate::ui_dialogs::ChoosePurpose::SaveLyrics,
                    dir: crate::ui_dialogs::default_start_dir(app),
                    selected: None,
                    name: "lyrics.txt".into(),
                    cancelled: false,
                    path_edit: String::new(),
                });
            }
        }
        if ui.button("Clear").clicked() {
            ed.values.lyrics.clear();
        }
        if ui.button("Remove Leading Spaces").clicked() {
            ed.values.lyrics = ed
                .values
                .lyrics
                .lines()
                .map(|l| l.trim_start().to_string())
                .collect::<Vec<_>>()
                .join("\n");
        }
    });
}

/// The cover-art area: the current picture (or "No album art"), its
/// dimensions in the original's `Picture: %d x %d, %s` format
/// (0x9ece3e), and Add / Replace, Save to File, Remove.
fn art_panel(ui: &mut Ui, app: &mut App, ed: &mut TagEditor, art_w: f32) {
    ui.vertical(|ui| {
        ui.set_min_width(art_w);
        ui.set_max_width(art_w);
        let inner = (art_w - 18.0).max(110.0);
        panel_frame(c_panel_bright()).show(ui, |ui| {
            ui.set_min_width(inner);
            ui.set_max_width(inner);
            if ed.multi {
                let mut on = ed.art_enabled;
                if ui.checkbox(&mut on, RichText::new("Art").color(c_text_dim()).size(13.5).strong()).changed() {
                    ed.art_enabled = on;
                }
                if !ed.art_enabled {
                    ui.disable();
                }
            } else {
                ui.add(
                    egui::Label::new(RichText::new("Art").color(c_text_dim()).size(13.5).strong())
                        .selectable(false),
                );
            }
            let img_rect = egui::Rect::from_min_size(
                ui.cursor().min,
                egui::vec2(inner, inner),
            );
            ui.painter().rect_filled(img_rect, egui::CornerRadius::same(6), c_art_card());
            ui.painter().rect_stroke(
                img_rect,
                egui::CornerRadius::same(6),
                egui::Stroke::new(1.0, c_separator()),
                egui::StrokeKind::Middle,
            );
            let staged = ed.cover_staged.clone();
            let tex = staged.as_ref().and_then(|(_, bytes)| {
                let key = format!("editor_cover_{:p}", std::sync::Arc::as_ptr(bytes));
                app.cover_texture(ui.ctx(), &key, bytes).cloned()
            });
            if let Some(tex) = &tex {
                let ts = tex.size_vec2();
                let scale = (img_rect.width() / ts.x).min(img_rect.height() / ts.y);
                let fitted = egui::Rect::from_center_size(img_rect.center(), ts * scale);
                ui.put(
                    fitted,
                    egui::Image::from_texture(tex).fit_to_exact_size(fitted.size()),
                );
            } else if staged.is_none() {
                ui.painter().text(
                    img_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "No album art\nAdd or drop a\npicture here",
                    egui::FontId::proportional(13.0),
                    c_text_dim(),
                );
            }
            ui.allocate_rect(img_rect, egui::Sense::hover());
            let info = match (&staged, &tex) {
                (Some((mime, _)), Some(t)) => {
                    let [w, h] = t.size();
                    format!("{w} x {h}, {mime}")
                }
                (Some((mime, _)), None) => mime.clone(),
                (None, _) => "No album art".to_string(),
            };
            ui.label(RichText::new(info).color(c_text_dim()).size(13.0));
            // The button row wraps, so it can never widen the panel.
            ui.horizontal_wrapped(|ui| {
                if ui
                    .small_button("Add / Replace...")
                    .on_hover_text("Choose a picture file (or drag and drop one onto the window)")
                    .clicked()
                {
                    app.dlg.pick_cover = true;
                    app.dlg.choose = Some(crate::ui_dialogs::ChooseState {
                        purpose: crate::ui_dialogs::ChoosePurpose::PickCover,
                        dir: crate::ui_dialogs::default_start_dir(app),
                        selected: None,
                        name: String::new(),
                        cancelled: false,
                        path_edit: String::new(),
                    });
                }
                if ui.small_button("Save to File...").clicked() {
                    if let Some((mime, bytes)) = &ed.cover_staged {
                        let suggested = if mime == "PNG" { "cover.png" } else { "cover.jpg" };
                        app.dlg.save_cover_bytes = Some(bytes.to_vec());
                        app.dlg.choose = Some(crate::ui_dialogs::ChooseState {
                            purpose: crate::ui_dialogs::ChoosePurpose::SaveCover,
                            dir: crate::ui_dialogs::default_start_dir(app),
                            selected: None,
                            name: suggested.into(),
                            cancelled: false,
                            path_edit: String::new(),
                        });
                    } else {
                        app.warn("There is no album art to save yet.");
                    }
                }
                if ui.small_button("Remove").clicked() {
                    ed.cover_staged = None;
                    ed.cover_removed = true;
                }
            });
        });
    });
}

fn save_current(app: &mut App, ed: &mut TagEditor) {
    // Without the original's per-field tick boxes, Save writes exactly
    // the fields whose value differs from the file's own values.
    // In multi-file mode only the ticked fields (and the ticked Art) are
    // written; everything starts unticked so nothing changes by accident.
    let mut values: Vec<(Field, String)> = Vec::new();
    for f in ALL_FIELDS {
        if *f == Field::Compilation {
            continue;
        }
        if ed.multi && !ed.enabled.contains(f) {
            continue;
        }
        let now = ed.values.get(*f).to_string();
        let was = ed.original.get(*f).to_string();
        if now != was {
            values.push((*f, now));
        }
    }
    let compilation_on = !ed.multi || ed.enabled.contains(&Field::Compilation);
    let compilation = if ed.values.compilation != ed.original.compilation && compilation_on {
        ed.values.compilation
    } else {
        None
    };
    let art_on = !ed.multi || ed.art_enabled;
    let cover_set = if ed.cover_staged != ed.cover_original && art_on {
        ed.cover_staged.clone()
    } else {
        None
    };
    let cover_removed = ed.cover_removed && ed.cover_original.is_some() && art_on;
    if values.is_empty() && compilation.is_none() && cover_set.is_none() && !cover_removed {
        if ed.multi {
            app.warn("Nothing to save. Tick the fields you want to change first (all start off).");
        } else {
            app.warn("Nothing to save — no fields were changed.");
        }
        return;
    }
    let result = EditorResult {
        index: app.focused.unwrap_or(0),
        multi: ed.multi,
        indices: ed.indices.clone(),
        values,
        compilation,
        cover_set,
        cover_removed,
    };
    app.dlg.apply_result(result);
}

fn move_focus(app: &mut App, ed: &mut TagEditor, step: i32) {
    let Some(cur) = app.focused else {
        return;
    };
    let rows = app.visible_rows();
    let pos = rows.iter().position(|r| *r == cur).unwrap_or(0);
    let next = (pos as i32 + step).rem_euclid(rows.len() as i32) as usize;
    let new_focus = rows[next];
    app.focused = Some(new_focus);
    ed.indices = vec![new_focus];
    ed.multi = false;
    ed.values = app.files[new_focus].fields.clone();
    ed.original = app.files[new_focus].fields.clone();
    ed.cover_staged = app.files[new_focus].cover.clone();
    ed.cover_original = app.files[new_focus].cover.clone();
    ed.cover_removed = false;
}

pub fn apply_result(app: &mut App, result: EditorResult) {
    let indices = if result.multi {
        result.indices.clone()
    } else {
        vec![result.index]
    };
    for i in &indices {
        if *i >= app.files.len() {
            continue;
        }
        let f = app.files[*i].clone();
        let mut new_fields = f.fields.clone();
        for (field, value) in &result.values {
            new_fields.set(*field, value);
        }
        if let Some(c) = result.compilation {
            new_fields.compilation = Some(c);
        }
        let mut checked: Vec<Field> = result.values.iter().map(|(fl, _)| *fl).collect();
        if result.compilation.is_some() {
            checked.push(Field::Compilation);
        }
        if !checked.is_empty() {
            if let Err(e) = crate::data::save_fields(&f.path, &new_fields, &checked) {
                app.warn(e);
                continue;
            }
        }
        if let Some((_, bytes)) = &result.cover_set {
            if let Err(e) = crate::data::set_cover(&f.path, bytes.to_vec()) {
                app.warn(e);
            }
        }
        if result.cover_removed {
            if let Err(e) = crate::data::remove_cover(&f.path) {
                app.warn(e);
            }
        }
        if let Some(reloaded) = crate::data::read_file(&f.path) {
            app.files[*i] = reloaded;
        }
    }
    app.say("Saved the changed fields.");
}

/// Used by the file picker to hand a chosen picture to the editor.
pub fn stage_cover(app: &mut App, bytes: Vec<u8>) {
    if let Some(ed) = app.dlg.tag_editor.as_mut() {
        let mime = match image::guess_format(&bytes) {
            Ok(image::ImageFormat::Jpeg) => "JPEG",
            Ok(image::ImageFormat::Png) => "PNG",
            Ok(image::ImageFormat::Gif) => "GIF",
            Ok(image::ImageFormat::Bmp) => "BMP",
            _ => "Picture",
        };
        ed.cover_staged = Some((mime.to_string(), std::sync::Arc::new(bytes)));
        ed.cover_removed = false;
    }
}

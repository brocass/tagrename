//! The "Tools Bar" area with its three tabs: Rename Files (F4),
//! Multi File Tag Editor (Ctrl+F4) and Get Tags from File Name (Shift+F4),
//! plus the two cover-art panels beside them.

use egui::{Grid, RichText, Ui};

/// The cover-art buttons (Delete, Set from file art, Remove, Save to File)
/// are set at 8 pt on screen; the interface zoom is divided out.
const COVER_BUTTON_PT: f32 = 8.0 / UI_ZOOM;

use crate::data::Field;
use crate::mask;
use crate::theme::*;
use crate::ui::App;

pub const TAB_TITLES: [&str; 3] = [
    "Rename Files (F4)",
    "Multi File Tag Editor (Ctrl+F4)",
    "Get Tags from File Name (Shift+F4)",
];

/// The shared "Case:" check radio group (TSftpCheckRadioGroup in the
/// original): a checkbox enables one of the four case options. The four
/// options sit below "Case:" in two columns, as in the original's box, so
/// they never widen the mask box or push the cover art off the window.
fn case_group(ui: &mut Ui, enabled: &mut bool, which: &mut usize) {
    ui.checkbox(enabled, "Case:");
    if *enabled {
        ui.horizontal_top(|ui| {
            ui.add_space(18.0);
            for column in [[0usize, 1], [2, 3]] {
                ui.vertical(|ui| {
                    for i in column {
                        if ui.radio(*which == i, mask::CASE_OPTIONS[i]).clicked() {
                            *which = i;
                        }
                    }
                });
                ui.add_space(12.0);
            }
        });
    }
}

/// An "actions" strip: the tab's buttons in a section of their own
/// (the person's request), separated from the mask and fields boxes.
/// The row wraps instead of ever growing wider than the tools bar.
fn action_section(ui: &mut Ui, title: &str, add: impl FnOnce(&mut Ui)) {
    ui.add_space(4.0);
    panel_frame(c_panel_bright()).show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.add(
                egui::Label::new(
                    RichText::new(title)
                        .color(c_text_dim())
                        .strong()
                        .size(13.5),
                )
                .selectable(false),
            );
            ui.separator();
            add(ui);
        });
    });
}

/// The Rename Files tab.
pub fn rename_tab(ui: &mut Ui, app: &mut App) {
    // Explicit sizes from the bounded tab ui: the mask box and the
    // variables legend each get their own allocation, so nothing can
    // push the row wider than the tools bar (an unbounded legend label
    // here once grew the tools bar over the whole window).
    ui.horizontal_top(|ui| {
        // Left: the mask group box — a vertical scope with an exact width
        // (set_min_width/set_max_width, the one sizing pattern that
        // truly bounds content in a row; unbounded widgets here once
        // grew the tools bar over the whole window).
        ui.vertical(|ui| {
            ui.set_max_width(400.0);
            ui.set_min_width(400.0);
            panel_frame(c_panel_bright()).show(ui, |ui| {
                ui.set_min_height(116.0);
                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("File name mask:").selectable(false));
                    let edit = egui::TextEdit::singleline(&mut app.rename_mask)
                        .desired_width(150.0)
                        .hint_text("e.g. %1 - %2");
                    ui.add(edit);
                    if ui.button("Mask Editor").clicked() {
                        app.dlg.open_mask_editor(0, &app.rename_mask);
                    }
                });
                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("Result preview:").selectable(false));
                    let mut preview = rename_example(app, &app.rename_mask);
                    ui.add(
                        egui::TextEdit::singleline(&mut preview)
                            .interactive(false)
                            .desired_width((ui.available_width() - 4.0).min(260.0)),
                    );
                });
                ui.checkbox(&mut app.replace_spaces, "Replace spaces with underscores");
                case_group(ui, &mut app.rename_case_enabled, &mut app.rename_case);
            });
        });
    });
    // The buttons, in their own section.
    action_section(ui, "Rename actions:", |ui| {
        if primary_button(ui, "Preview").clicked() {
            app.dlg.show_rename_preview = true;
        }
        if primary_button(ui, "Rename").clicked() {
            app.do_rename(false);
        }
        if ui.button("Undo").clicked() {
            app.undo_rename();
        }
    });
}

/// The Get Tags from File Name tab.
pub fn file_to_tag_tab(ui: &mut Ui, app: &mut App) {
    ui.horizontal_top(|ui| {
        // Left: the mask group box.
        ui.vertical(|ui| {
            ui.set_max_width(400.0);
            ui.set_min_width(400.0);
            panel_frame(c_panel_bright()).show(ui, |ui| {
                ui.set_min_height(116.0);
                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("File name mask:").selectable(false));
                    ui.add(
                        egui::TextEdit::singleline(&mut app.f2t_mask)
                            .desired_width(150.0)
                            .hint_text("e.g. %6 %1 - %2"),
                    );
                    if ui.button("Mask Editor").clicked() {
                        app.dlg.open_mask_editor(1, &app.f2t_mask);
                    }
                });
                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("File name preview:").selectable(false));
                    let mut preview = f2t_example(app, &app.f2t_mask);
                    ui.add(
                        egui::TextEdit::singleline(&mut preview)
                            .interactive(false)
                            .desired_width((ui.available_width() - 4.0).min(260.0)),
                    );
                });
                ui.checkbox(&mut app.replace_underscores, "Replace underscores with spaces");
                case_group(ui, &mut app.f2t_case_enabled, &mut app.f2t_case);
            });
        });
    });
    // The buttons, in their own section.
    action_section(ui, "File name to tag actions:", |ui| {
        if primary_button(ui, "Preview").clicked() {
            app.dlg.show_f2t_preview = true;
        }
        if primary_button(ui, "Write Tags").clicked() {
            app.file_to_tag(false);
        }
    });
}

/// The Multi File Tag Editor tab.
pub fn multi_editor_tab(ui: &mut Ui, app: &mut App) {
    ui.horizontal_top(|ui| {
        // Left: the fields grid box.
        ui.vertical(|ui| {
            ui.set_max_width(400.0);
            ui.set_min_width(400.0);
            panel_frame(c_panel_bright()).show(ui, |ui| {
                ui.set_min_height(116.0);
                Grid::new("multi_editor_grid")
                    .num_columns(2)
                    .spacing([10.0, 4.0])
                    .show(ui, |ui| {
                        for slot in app.multi_checked.iter_mut() {
                            let field = slot.0;
                            ui.checkbox(&mut slot.1, field.label());
                            let enabled = slot.1;
                            ui.scope(|ui| {
                                if !enabled {
                                    ui.disable();
                                }
                                let value = app.multi_values.get(field).to_string();
                                let mut edited = value.clone();
                                if field == Field::Genre {
                                    // The original's Multi File Tag Editor genre
                                    // box is an EDITABLE combo (ComboBoxGenres,
                                    // Sorted, no Style property = the Delphi
                                    // default csDropDown): pick from the list,
                                    // type your own, add your own to the list.
                                    let mut set: Option<String> = None;
                                    let mut add: Option<String> = None;
                                    ui.horizontal(|ui| {
                                        let mut text = value.clone();
                                        let r = ui.add(
                                            egui::TextEdit::singleline(&mut text)
                                                .desired_width(206.0)
                                                .hint_text("pick or type"),
                                        );
                                        if r.changed() {
                                            edited = text.clone();
                                        }
                                        let btn = ui.add(egui::Button::new("▾").small());
                                        let popup_id = ui.scope_id().with("multi_genre_popup");
                                        let picked = egui::Popup::from_toggle_button_response(&btn)
                                            .id(popup_id)
                                            .width(230.0)
                                            .show(|ui| {
                                                ui.set_min_width(ui.available_width());
                                                egui::ScrollArea::vertical()
                                                    .max_height(240.0)
                                                    .show(ui, |ui| {
                                                        let cur = text.trim().to_string();
                                                        let mut s: Option<String> = None;
                                                        let mut a: Option<String> = None;
                                                        if !cur.is_empty()
                                                            && !app
                                                                .genres
                                                                .iter()
                                                                .any(|g| g.eq_ignore_ascii_case(&cur))
                                                        {
                                                            if ui
                                                                .selectable_label(
                                                                    false,
                                                                    format!(
                                                                        "Add \u{201c}{cur}\u{201d} to the genre list"
                                                                    ),
                                                                )
                                                                .clicked()
                                                            {
                                                                a = Some(cur.clone());
                                                                s = Some(cur.clone());
                                                            }
                                                            ui.separator();
                                                        }
                                                        for g in app.genres.iter() {
                                                            if ui
                                                                .selectable_label(cur == *g, g.clone())
                                                                .clicked()
                                                            {
                                                                s = Some(g.clone());
                                                            }
                                                        }
                                                        (s, a)
                                                    })
                                                    .inner
                                            });
                                        if let Some(ir) = picked {
                                            let (s, a) = ir.inner;
                                            set = s;
                                            add = a;
                                        }
                                    });
                                    if let Some(v) = set {
                                        edited = v;
                                    }
                                    if let Some(g) = add {
                                        let g = g.trim().to_string();
                                        if !g.is_empty()
                                            && !app.genres.iter().any(|x| x.eq_ignore_ascii_case(&g))
                                        {
                                            app.genres.push(g);
                                            app.genres.sort();
                                        }
                                    }
                                } else {
                                    let r = ui.add(
                                        egui::TextEdit::singleline(&mut edited)
                                            .desired_width(240.0)
                                            .hint_text(""),
                                    );
                                    let _ = r;
                                }
                                if edited != value {
                                    app.multi_values.set(field, &edited);
                                }
                            });
                            ui.end_row();
                        }
                    });
                case_group(ui, &mut app.multi_case_enabled, &mut app.multi_case);
            });
        });
    });
    // The buttons, in their own section.
    action_section(ui, "Tag actions:", |ui| {
        if primary_button(ui, "Save Tags").clicked() {
            app.save_multi_tags();
        }
        if ui.button("Remove Tags").clicked() {
            app.remove_tags_selected();
        }
        if ui.button("Copy from Focused File").clicked() {
            app.copy_from_focused();
        }
        if ui.button("Edit All Supported Tag Fields").clicked() {
            app.open_tag_editor();
        }
    });
}

/// The file the previews show: the focused file, else the first checked
/// file, else the first file in the list.
pub fn preview_file(app: &App) -> Option<usize> {
    app.focused
        .or_else(|| app.checked_indices().first().copied())
        .or_else(|| (!app.files.is_empty()).then_some(0))
}

/// The new file name for the preview file, as it will look after renaming.
/// Without any file it shows the mask with placeholder names.
pub fn rename_example(app: &App, mask_text: &str) -> String {
    let Some(i) = preview_file(app) else {
        return mask::placeholder_name(mask_text);
    };
    let f = &app.files[i];
    let indices = app.checked_indices();
    let n = indices.iter().position(|x| *x == i).unwrap_or(0) as u32 + 1;
    let total = indices.len().max(1) as u32;
    let opts = mask::RenameOptions {
        case: mask::Case::from_index(app.rename_case),
        case_enabled: app.rename_case_enabled,
        replace_spaces_with_underscores: app.replace_spaces,
        remove_leading_zero_in_disc: app.remove_leading_zero_disc,
    };
    mask::build_new_name(f, mask_text, n, total, &opts)
        .map(|stem| mask::with_extension(stem, &f.extension()))
        .unwrap_or_else(|| "(can't build: no tag data)".into())
}

/// The file name the preview file gets from the file-name mask, with its
/// captured values written in place of the variables.
pub fn f2t_example(app: &App, mask_text: &str) -> String {
    let Some(i) = preview_file(app) else {
        return mask::placeholder_name(mask_text);
    };
    let f = &app.files[i];
    let opts = mask::ParseOptions {
        case: mask::Case::from_index(app.f2t_case),
        case_enabled: app.f2t_case_enabled,
        replace_underscores_with_spaces: app.replace_underscores,
    };
    match mask::parse_file_name(&f.file_stem(), mask_text, &opts) {
        Ok(pairs) => mask::with_extension(mask::render_parsed(mask_text, &pairs), &f.extension()),
        Err(e) => e,
    }
}

/// The cover-art panels on the right of the tools bar. Both art boxes are
/// fixed squares: they do not stretch down with the tabs column, and they
/// are a little larger than the panel's inner width. Both panels are
/// hard-bounded to half the covers column each, so no button row or label
/// can ever push a cover box past the right edge of the window.
pub fn cover_panels(ui: &mut Ui, app: &mut App) {
    let spacing = ui.spacing().item_spacing.x;
    let col = COVERS_COLUMN_WIDTH.min(ui.available_width());
    // panel_frame() takes 8 px of inner margin on each side plus a 1 px
    // stroke. The two frames and the gap fill the covers column exactly,
    // so the right frame sits the same distance from the window edge as
    // the rest of the layout.
    let frame_pad = 18.0;
    let outer = ((col - spacing) / 2.0).floor().max(110.0);
    let inner = (outer - frame_pad).max(90.0);
    // The picture fills the frame's content area: 25% larger than the
    // previous 130 px square.
    let img_side = inner;
    ui.add_space(8.0);
    // Side by side like the original's two ~160 px cover panels: a plain
    // top-down ui would stack them, doubling the tools bar's height.
    ui.horizontal_top(|ui| {
        for which in [0usize, 1usize] {
            ui.vertical(|ui| {
                ui.set_min_width(outer);
                ui.set_max_width(outer);
                panel_frame(c_panel_bright()).show(ui, |ui| {
                    ui.set_min_width(inner);
                    ui.set_max_width(inner);
                    // One fixed height for both boxes, whether or not they
                    // hold a picture, so their titles sit on the same line.
                    let body = ui.allocate_exact_size(egui::vec2(inner, img_side + 100.0), egui::Sense::hover()).0;
                    ui.scope_builder(egui::UiBuilder::new().max_rect(body), |ui| {
                    ui.set_min_width(inner);
                    ui.add(
                        egui::Label::new(
                            RichText::new(if which == 0 {
                                "Folder cover art"
                            } else {
                                "File cover art"
                            })
                            .color(c_text_dim())
                            .size(13.5)
                            .strong(),
                        )
                        .selectable(false),
                    );
                    // The picture is decoded once and kept as a texture; the
                    // file is only read again when its texture is not there.
                    let (key, bytes, kind): (String, Option<std::sync::Arc<Vec<u8>>>, String) = if which == 0 {
                        let folder = folder_cover_dir(app)
                            .unwrap_or_else(|| std::path::PathBuf::from("Demo Library"));
                        match find_folder_cover(&folder) {
                            Some(p) => {
                                let ext = p.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
                                let kind = if ext == "png" { "PNG" } else { "JPEG" }.to_string();
                                let stamp = p.metadata().and_then(|m| m.modified()).ok();
                                let key = format!("folder_cover:{}:{:?}", p.display(), stamp);
                                let bytes = if app.textures.contains_key(&key) {
                                    Some(std::sync::Arc::new(Vec::new()))
                                } else {
                                    std::fs::read(&p).ok().map(std::sync::Arc::new)
                                };
                                (key, bytes, kind)
                            }
                            None => (String::new(), None, "No folder cover art".into()),
                        }
                    } else {
                        match app.focused.and_then(|i| app.files[i].cover.clone()) {
                            Some((mime, b)) => {
                                let key = format!("file_cover:{:p}", std::sync::Arc::as_ptr(&b));
                                (key, Some(b), mime)
                            }
                            None => (String::new(), None, "Drag and drop\ncover art here".into()),
                        }
                    };
                    let tex = match &bytes {
                        Some(b) if !key.is_empty() => app.cover_texture(ui.ctx(), &key, b).cloned(),
                        _ => None,
                    };
                    let info = match &tex {
                        Some(t) => {
                            let [w, h] = t.size();
                            format!("{w} x {h}, {kind}")
                        }
                        None => kind.clone(),
                    };
                    let img_rect = egui::Rect::from_min_size(
                        ui.cursor().min,
                        egui::vec2(img_side, img_side),
                    );
                    ui.painter().rect_filled(img_rect, egui::CornerRadius::same(6), c_panel());
                    ui.painter().rect_stroke(img_rect, egui::CornerRadius::same(6), egui::Stroke::new(1.0, c_separator()), egui::StrokeKind::Middle);
                    if let Some(tex) = &tex {
                        ui.put(
                            img_rect,
                            egui::Image::from_texture(tex).fit_to_exact_size(img_rect.size()),
                        );
                    } else if bytes.is_none() {
                        ui.painter().text(
                            img_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            &info,
                            egui::FontId::proportional(13.5),
                            c_text_dim(),
                        );
                    }
                    ui.allocate_rect(img_rect, egui::Sense::hover());
                    // The caption + button row wraps, so a long button
                    // can never widen the panel past its column.
                    // The caption and buttons only appear once a picture is
                    // there; an empty box shows no Delete / Set / Remove text.
                    if bytes.is_some() {
                    if !info.is_empty() {
                        ui.label(RichText::new(&info).color(c_text_dim()).size(13.0));
                    }
                    ui.horizontal_wrapped(|ui| {
                        if which == 0 {
                            if ui.small_button(RichText::new("Delete").size(COVER_BUTTON_PT)).clicked() {
                                if let Some(folder) = folder_cover_dir(app) {
                                    for _ in 0..4 {
                                        let Some(p) = find_folder_cover(&folder) else { break };
                                        let _ = std::fs::remove_file(p);
                                    }
                                    app.say("Deleted folder cover art.");
                                }
                            }
                            if ui.small_button(RichText::new("Set from file art").size(COVER_BUTTON_PT)).clicked() {
                                if let Some(i) = app.focused {
                                    if let Some((_, bytes)) = app.files[i].cover.clone() {
                                        if let Some(folder) = folder_cover_dir(app) {
                                            match std::fs::write(folder.join("folder.jpg"), bytes.as_slice()) {
                                                Ok(()) => app.say("Saved folder.jpg"),
                                                Err(e) => app.warn(format!("Can't save folder.jpg: {e}")),
                                            }
                                        }
                                    }
                                }
                            }
                        } else if let Some(i) = app.focused {
                            if ui.small_button(RichText::new("Remove").size(COVER_BUTTON_PT)).clicked() {
                                let path = app.files[i].path.clone();
                                match data_remove_cover(&path) {
                                    Ok(()) => {
                                        app.files[i].cover = None;
                                        app.say("Removed cover art from the tag.");
                                    }
                                    Err(e) => app.warn(e),
                                }
                            }
                            if ui.small_button(RichText::new("Save to File").size(COVER_BUTTON_PT)).clicked() {
                                if let Some((_, bytes)) = app.files[i].cover.clone() {
                                    app.dlg.save_cover_bytes = Some(bytes.to_vec());
                                    app.dlg.choose = Some(crate::ui_dialogs::ChooseState {
                                        purpose: crate::ui_dialogs::ChoosePurpose::SaveCover,
                                        dir: crate::ui_dialogs::default_start_dir(app),
                                        selected: None,
                                        name: "cover.jpg".into(),
                                        cancelled: false,
                                        path_edit: String::new(),
                                    });
                                }
                            }
                        }
                    });
                    }
                    });
                });
            });
        }
    });
}

/// The folder whose cover the Folder cover art box shows: the focused
/// row when it is a folder (an album row in the list), otherwise the
/// folder currently open.
fn folder_cover_dir(app: &App) -> Option<std::path::PathBuf> {
    if let Some(i) = app.focused {
        if let Some(f) = app.files.get(i) {
            if f.is_dir {
                return Some(f.path.clone());
            }
        }
    }
    app.folder.clone()
}

/// Finds folder art in a folder, ignoring letter case, so `Folder.jpg`,
/// `folder.PNG` and `FOLDER.jpeg` all count. jpg is preferred over png.
fn find_folder_cover(dir: &std::path::Path) -> Option<std::path::PathBuf> {
    let rd = std::fs::read_dir(dir).ok()?;
    let mut found: Vec<(usize, std::path::PathBuf)> = Vec::new();
    for e in rd.flatten() {
        let name = e.file_name().to_string_lossy().to_lowercase();
        let rank = match name.as_str() {
            "folder.jpg" | "folder.jpeg" => 0,
            "folder.png" => 1,
            _ => continue,
        };
        if e.path().is_file() {
            found.push((rank, e.path()));
        }
    }
    found.sort();
    found.into_iter().next().map(|(_, p)| p)
}

fn data_remove_cover(path: &std::path::Path) -> Result<(), String> {
    crate::data::remove_cover(path)
}

/// Two cover frames of 194 px (a 176 px picture plus its 18 px frame) and the gap between them.
const COVERS_COLUMN_WIDTH: f32 = 396.0;

/// Draw the whole tools bar (tabs on the left, covers on the right).
pub fn tools_bar(ui: &mut Ui, app: &mut App) {
    ui.horizontal_top(|ui| {
        let avail = ui.available_width();
        // The covers column: about a third of the bar (the original's
        // two cover panels are 160 px each), clamped so the tabs column
        // keeps room and the covers can never be pushed off the window.
        let covers_width = COVERS_COLUMN_WIDTH;
        let tabs_width = (avail - covers_width - 18.0).max(360.0);
        ui.vertical(|ui| {
            ui.set_max_width(tabs_width);
            ui.set_min_width(tabs_width);
            ui.horizontal(|ui| {
                for (i, title) in TAB_TITLES.iter().enumerate() {
                    let is_active = app.active_tab == i;
                    let btn = egui::Button::new(
                        RichText::new(*title)
                            .color(if is_active { c_text() } else { c_text_dim() })
                            .size(14.0)
                            .strong(),
                    )
                    .fill(if is_active { c_accent_dim() } else { c_panel() })
                    .stroke(if is_active {
                        egui::Stroke::new(1.0, c_accent())
                    } else {
                        egui::Stroke::new(1.0, c_separator())
                    })
                    .corner_radius(egui::CornerRadius::same(7));
                    if ui.add(btn).clicked() {
                        app.active_tab = i;
                    }
                }
            });
            ui.add_space(2.0);
            match app.active_tab {
                0 => rename_tab(ui, app),
                1 => multi_editor_tab(ui, app),
                _ => file_to_tag_tab(ui, app),
            }
            ui.min_rect()
        });
        ui.add_space(6.0);
        ui.vertical(|ui| {
            // Claim the covers column and BOUND it: without the max, a
            // wide button could grow the column past the window's right
            // edge (the file cover art box being half off the screen).
            ui.set_min_width(covers_width);
            ui.set_max_width(covers_width);
            cover_panels(ui, app);
        });
    });
}

//! The main window layout: menu bar, tools bar, the compact action strip
//! above the folder tree, file list, status bar, shortcuts and screenshots.

use egui::{CentralPanel, Context, Key, Layout, RichText, ScrollArea, Sense, Stroke, Ui};

use crate::data::Field;
use crate::theme::*;
use crate::tools::{ArtistTool, TrackFormat};
use crate::ui::{save_columns, App, ColumnState, SortColumn, ViewMode, FILTER_FIELDS, MIN_COL_W};
use crate::ui_dialogs::{self, ChoosePurpose, ChooseState};
use crate::ui_tabs;

pub fn update(app: &mut App, root: &mut Ui) {
    let ctx = root.ctx().clone();
    refresh_theme(&ctx);
    // Repaint only when something is actually changing. egui wakes up on
    // clicks, keys and pointer moves by itself; the old unconditional
    // 16 ms request kept all 60 frames/s busy even while the app was idle,
    // which made every click feel delayed. Screenshot mode still needs a
    // steady frame stream (including while unfocused), and a background
    // folder scan needs a gentle poll so its progress reaches the screen.
    if app.shot.is_some() {
        ctx.request_repaint_after(std::time::Duration::from_millis(16));
    } else if app.loading.is_some() || app.toast.is_some() {
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }

    screenshot_flow(app, &ctx);
    app.persist_masks();
    shortcuts(app, &ctx);
    handle_drops(app, &ctx);
    // Pick up background folder-scan progress / results (the person's
    // library is thousands of files; the read must not freeze the UI).
    app.poll_loading();

    window_chrome(root, &ctx);
    menu_bar(app, root, &ctx);
    if app.show_tools_bar {
        tools_panel(app, root);
    }
    // The refresh/edit row sits directly above the folder tree, not up in
    // the title/menu strip. Its dark frame keeps it visually separate.
    toolbar(app, root, &ctx);
    status_bar(app, root);
    if app.show_folders {
        folders_panel(app, root);
    }
    file_list(app, root);

    // The original dims its main window while the Tag Editor is modal.
    // In dark mode a translucent black scrim does the same job and
    // makes the raised dialog stand out without changing the palette
    // of the main window itself.
    if app.dlg.tag_editor.is_some() {
        root.painter().rect_filled(
            root.max_rect(),
            egui::CornerRadius::same(window_radius(&ctx)),
            egui::Color32::from_black_alpha(96),
        );
    }

    window_edge(root, &ctx);
    ui_dialogs::draw(&ctx, app);
    toast(app, &ctx);
}

fn refresh_theme(ctx: &Context) {
    crate::theme::refresh(ctx);
}

/// The corner radius of the main window. It stays rounded even when a
/// tiling desktop reports the window as maximised.
pub fn window_radius(_ctx: &Context) -> u8 {
    WINDOW_RADIUS
}

const TITLE_BAR_H: f32 = 30.0;

/// The window's own title bar, since the window has no system frame. The
/// bar moves the window when dragged and maximises it on a double click;
/// the buttons on the right minimise, maximise or close it.
fn window_chrome(root: &mut Ui, ctx: &Context) {
    let r = window_radius(ctx);
    let maximized = ctx.input(|i| i.viewport().maximized).unwrap_or(false);
    egui::Panel::top("titlebar")
        .exact_size(TITLE_BAR_H)
        .show_separator_line(false)
        .frame(
            egui::Frame::new()
                .fill(c_header())
                .corner_radius(egui::CornerRadius { nw: r, ne: r, sw: 0, se: 0 }),
        )
        .show(root, |ui| {
            let bar = ui.max_rect();
            let drag = ui.interact(bar, ui.id().with("window_drag"), Sense::click_and_drag());
            if drag.drag_started() {
                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }
            if drag.double_clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
            }
            let caption = egui::FontId::new(13.0, egui::FontFamily::Name("semibold".into()));
            ui.painter().text(
                bar.left_center() + egui::vec2(12.0, 0.0),
                egui::Align2::LEFT_CENTER,
                "Tag:Rename",
                caption,
                c_text(),
            );
            ui.painter().text(
                bar.left_center() + egui::vec2(100.0, 0.0),
                egui::Align2::LEFT_CENTER,
                "1.0.0",
                egui::FontId::proportional(12.0),
                c_text_dim(),
            );

            let w = 46.0;
            let stroke = Stroke::new(1.3, c_text());
            let buttons = [
                ("Minimise", 0),
                ("Maximise", 1),
                ("Close", 2),
            ];
            for (k, (label, which)) in buttons.iter().enumerate() {
                let x0 = bar.max.x - w * (3 - k) as f32;
                let rect = egui::Rect::from_min_max(egui::pos2(x0, bar.min.y), egui::pos2(x0 + w, bar.max.y));
                let resp = ui
                    .interact(rect, ui.id().with(("win_btn", *which)), Sense::click())
                    .on_hover_text(*label);
                let close = *which == 2;
                if resp.hovered() {
                    let fill = if close { c_danger() } else { pal().widget_hover };
                    ui.painter().rect_filled(rect, egui::CornerRadius::same(0), fill);
                }
                let ink = if close && resp.hovered() { egui::Color32::WHITE } else { c_text() };
                let s = Stroke::new(stroke.width, ink);
                let c = rect.center();
                match which {
                    0 => {
                        ui.painter().line_segment([c + egui::vec2(-5.0, 0.0), c + egui::vec2(5.0, 0.0)], s);
                    }
                    1 if maximized => {
                        let back = egui::Rect::from_center_size(c + egui::vec2(2.0, -2.0), egui::vec2(9.0, 9.0));
                        let front = egui::Rect::from_center_size(c + egui::vec2(-2.0, 2.0), egui::vec2(9.0, 9.0));
                        ui.painter().rect_stroke(back, egui::CornerRadius::same(0), s, egui::StrokeKind::Middle);
                        ui.painter().rect_filled(front, egui::CornerRadius::same(0), pal().widget_hover);
                        ui.painter().rect_stroke(front, egui::CornerRadius::same(0), s, egui::StrokeKind::Middle);
                    }
                    1 => {
                        let square = egui::Rect::from_center_size(c, egui::vec2(10.0, 10.0));
                        ui.painter().rect_stroke(square, egui::CornerRadius::same(0), s, egui::StrokeKind::Middle);
                    }
                    _ => {
                        ui.painter().line_segment([c + egui::vec2(-5.0, -5.0), c + egui::vec2(5.0, 5.0)], s);
                        ui.painter().line_segment([c + egui::vec2(5.0, -5.0), c + egui::vec2(-5.0, 5.0)], s);
                    }
                }
                if resp.clicked() {
                    match which {
                        0 => ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true)),
                        1 => ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized)),
                        _ => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
                    }
                }
            }
        });
}

/// Outline of the rounded window, and the edges and corners that resize it
/// (the window has no system frame to grab).
fn window_edge(root: &mut Ui, ctx: &Context) {
    let r = root.max_rect();
    let radius = window_radius(ctx);
    if radius == 0 {
        return;
    }
    root.painter().rect_stroke(
        r,
        egui::CornerRadius::same(radius),
        Stroke::new(1.0, c_dialog_edge()),
        egui::StrokeKind::Inside,
    );
    use egui::{CursorIcon, ResizeDirection as D, Rect, pos2};
    let t = 6.0;
    let c = 16.0;
    let zones = [
        (Rect::from_min_max(pos2(r.min.x, r.min.y + c), pos2(r.min.x + t, r.max.y - c)), D::West, CursorIcon::ResizeHorizontal, "w"),
        (Rect::from_min_max(pos2(r.max.x - t, r.min.y + c), pos2(r.max.x, r.max.y - c)), D::East, CursorIcon::ResizeHorizontal, "e"),
        (Rect::from_min_max(pos2(r.min.x + c, r.max.y - t), pos2(r.max.x - c, r.max.y)), D::South, CursorIcon::ResizeVertical, "s"),
        (Rect::from_min_max(pos2(r.min.x, r.max.y - c), pos2(r.min.x + c, r.max.y)), D::SouthWest, CursorIcon::ResizeNeSw, "sw"),
        (Rect::from_min_max(pos2(r.max.x - c, r.max.y - c), r.max), D::SouthEast, CursorIcon::ResizeNwSe, "se"),
        (Rect::from_min_max(pos2(r.max.x - c, r.min.y), pos2(r.max.x, r.min.y + c)), D::NorthEast, CursorIcon::ResizeNeSw, "ne"),
        (Rect::from_min_max(r.min, pos2(r.min.x + c, r.min.y + c)), D::NorthWest, CursorIcon::ResizeNwSe, "nw"),
    ];
    for (rect, dir, cursor, name) in zones {
        let resp = root.interact(rect, root.id().with(("resize", name)), Sense::drag());
        if resp.hovered() || resp.dragged() {
            ctx.set_cursor_icon(cursor);
        }
        if resp.drag_started() {
            ctx.send_viewport_cmd(egui::ViewportCommand::BeginResize(dir));
        }
    }
}

/// egui's default menu-bar style: plain text labels, no button frames.
/// The separate popup style stays compact, like a standard desktop menu.
fn menu_bar_style(style: &mut egui::Style) {
    egui::containers::menu::menu_style(style);
    style.spacing.button_padding = egui::vec2(9.0, 4.0);
}

/// A menu row that only gets a highlight on hover - not a separate button.
fn menu_item(ui: &mut Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).color(c_text()))
            .frame_when_inactive(false),
    )
}

fn menu_bar(app: &mut App, root: &mut Ui, ctx: &Context) {
    egui::Panel::top("menubar")
        .show_separator_line(false)
        .frame(menu_frame())
        .show(root, |ui| {
            egui::MenuBar::new().style(menu_bar_style).ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if menu_item(ui, "Open Folder...").clicked() {
                        app.dlg.choose = Some(ChooseState {
                            purpose: ChoosePurpose::OpenFolder,
                            dir: ui_dialogs::default_start_dir(app),
                            selected: None,
                            name: String::new(),
                            cancelled: false,
                            path_edit: String::new(),
                        });
                    }
                    if menu_item(ui, "Add Folder to Tree...").clicked() {
                        // Adding a root is just adding a root: no music is read.
                        app.dlg.choose = Some(ChooseState {
                            purpose: ChoosePurpose::AddRoot,
                            dir: ui_dialogs::default_start_dir(app),
                            selected: None,
                            name: String::new(),
                            cancelled: false,
                            path_edit: String::new(),
                        });
                    }
                    if menu_item(ui, "Refresh File List (Ctrl+R)").clicked() {
                        if let Some(folder) = app.folder.clone() {
                            app.load_folder(&folder);
                        }
                    }
                    ui.separator();
                    if menu_item(ui, "Edit File Tag (F5)").clicked() {
                        app.open_tag_editor_focused();
                    }
                    if menu_item(ui, "Edit File/Folder Name (F6)").clicked() {
                        app.say("Rename files with the mask on the \"Rename Files\" tab below.");
                    }
                    if menu_item(ui, "Rename Folder (F7)").clicked() {
                        app.say("Open the folder, then use \"Rename Files\" with a mask like %1\\%2.");
                    }
                    ui.separator();
                    if menu_item(ui, "Play File (Ctrl+P)").clicked() {
                        if let Some(i) = app.focused {
                            app.play_file(i);
                        }
                    }
                    if menu_item(ui, "Play Selected Files").clicked() {
                        let indices = app.checked_indices();
                        if let Some(first) = indices.first() {
                            app.play_file(*first);
                        }
                    }
                    ui.separator();
                    if menu_item(ui, "Copy Tag (Ctrl+Shift+C)").clicked() {
                        app.copy_tag();
                    }
                    if menu_item(ui, "Paste Tag (Ctrl+Shift+V)").clicked() {
                        let i = app.focused.into_iter().collect::<Vec<_>>();
                        app.paste_tag(&i);
                    }
                    if menu_item(ui, "Paste Tag to All Selected Files").clicked() {
                        let indices = app.checked_indices();
                        app.paste_tag(&indices);
                    }
                    ui.separator();
                    if menu_item(ui, "Delete").clicked() {
                        let indices = app.checked_indices();
                        if indices.is_empty() {
                            app.warn("Nothing to delete — check files first.");
                        } else {
                            app.delete_files(&indices);
                        }
                    }
                    ui.separator();
                    if menu_item(ui, "Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("Edit", |ui| {
                    if menu_item(ui, "Select All (Ctrl+A)").clicked() {
                        app.select_all_visible();
                    }
                    if menu_item(ui, "Select None (Ctrl+N)").clicked() {
                        for c in &mut app.checked {
                            *c = false;
                        }
                    }
                    if menu_item(ui, "Invert Selection (Ctrl+I)").clicked() {
                        for c in &mut app.checked {
                            *c = !*c;
                        }
                    }
                    ui.separator();
                    if menu_item(ui, "Find... (Ctrl+F)").clicked() {
                        app.dlg.show_find = true;
                    }
                });

                ui.menu_button("View", |ui| {
                    if menu_item(ui, "Tools Bar (F8)").clicked() {
                        app.show_tools_bar = !app.show_tools_bar;
                    }
                    if menu_item(ui, "Folders panel").clicked() {
                        app.show_folders = !app.show_folders;
                    }
                    ui.separator();
                    if menu_item(ui, "Search Subfolders").clicked() {
                        app.search_subfolders = !app.search_subfolders;
                        if let Some(folder) = app.folder.clone() {
                            app.load_folder(&folder);
                        }
                    }
                    ui.separator();
                    ui.menu_button("Mode", |ui| {
                        for m in [
                            ViewMode::All,
                            ViewMode::Id3v2,
                            ViewMode::Wma,
                            ViewMode::Vorbis,
                            ViewMode::Mp4,
                            ViewMode::Ape,
                        ] {
                            let mark = if app.view_mode == m { "✓ " } else { "   " };
                            if menu_item(ui, &format!("{mark}{}", m.label())).clicked() {
                                app.view_mode = m;
                            }
                        }
                    });
                    ui.menu_button("Columns", |ui| {
                        let shown = app.columns.iter().filter(|c| c.visible).count();
                        for k in 0..app.columns.len() {
                            let c = app.columns[k];
                            let mark = if c.visible { "✓ " } else { "   " };
                            let title = COLS[c.id].0;
                            if menu_item(ui, &format!("{mark}{title}")).clicked()
                                && !(c.visible && shown <= 1)
                            {
                                app.columns[k].visible = !c.visible;
                                save_columns(&app.columns);
                            }
                        }
                        ui.separator();
                        if menu_item(ui, "Reset Columns").clicked() {
                            app.columns = crate::ui_layout::default_columns();
                            save_columns(&app.columns);
                        }
                    });
                    if menu_item(ui, "Appearance").clicked() {
                        app.dlg.show_appearance = true;
                    }
                });

                ui.menu_button("Playlist", |ui| {
                    let selected = app.checked_indices();
                    let all: Vec<usize> = (0..app.files.len()).collect();
                    if menu_item(ui, "Save Selected Files to Playlist").clicked() {
                        app.dlg.choose = Some(ChooseState {
                            purpose: ChoosePurpose::SavePlaylist(selected),
                            dir: ui_dialogs::default_start_dir(app),
                            selected: None,
                            name: "playlist.m3u".into(),
                            cancelled: false,
                            path_edit: String::new(),
                        });
                    }
                    if menu_item(ui, "Save All Files to Playlist").clicked() {
                        app.dlg.choose = Some(ChooseState {
                            purpose: ChoosePurpose::SavePlaylist(all),
                            dir: ui_dialogs::default_start_dir(app),
                            selected: None,
                            name: "playlist.m3u".into(),
                            cancelled: false,
                            path_edit: String::new(),
                        });
                    }
                });

                ui.menu_button("Tools", |ui| {
                    if menu_item(ui, "Auto Fill Track # (Ctrl+T)").clicked() {
                        app.dlg.show_autofill = true;
                    }
                    ui.menu_button("Format Track # to", |ui| {
                        for fmt in [
                            TrackFormat::X,
                            TrackFormat::ZeroX,
                            TrackFormat::ZeroZeroX,
                            TrackFormat::XxXx,
                            TrackFormat::Xx,
                        ] {
                            if menu_item(ui, fmt.label()).clicked() {
                                app.format_track(fmt);
                            }
                        }
                    });
                    ui.separator();
                    if menu_item(ui, "Swap: Artist <-> Title").clicked() {
                        app.swap_selected(Field::Artist, Field::Title);
                    }
                    if menu_item(ui, "Swap: Artist <-> Album").clicked() {
                        app.swap_selected(Field::Artist, Field::Album);
                    }
                    if menu_item(ui, "Swap: Album <-> Title").clicked() {
                        app.swap_selected(Field::Album, Field::Title);
                    }
                    if menu_item(ui, "Advanced... (swap any two fields)").clicked() {
                        app.dlg.swap_advanced = true;
                    }
                    if menu_item(ui, "Copy Text from One Tag Field to Another...").clicked() {
                        app.dlg.copy_field_open = true;
                    }
                    ui.separator();
                    ui.menu_button("Artist Field Tools", |ui| {
                        for tool in [
                            ArtistTool::RemoveLeadingThe,
                            ArtistTool::TheArtistToArtistThe,
                            ArtistTool::ArtistTheToTheArtist,
                            ArtistTool::NameSurnameToSurnameName,
                            ArtistTool::SurnameNameToNameSurname,
                        ] {
                            if menu_item(ui, tool.label()).clicked() {
                                app.artist_tool(tool);
                            }
                        }
                    });
                    ui.separator();
                    if menu_item(ui, "Remove Trailing Spaces from All Tag Fields").clicked() {
                        app.remove_spaces(false);
                    }
                    if menu_item(ui, "Remove Leading and Trailing Spaces from All Tag Fields").clicked() {
                        app.remove_spaces(true);
                    }
                    ui.separator();
                    if menu_item(ui, "Export/Import Tags...").clicked() {
                        app.dlg.show_export = true;
                    }
                });

                ui.menu_button("Options", |ui| {
                    if menu_item(ui, "Program Options...").clicked() {
                        app.dlg.show_options = true;
                    }
                    if menu_item(ui, "Genre List Editor...").clicked() {
                        app.dlg.show_genre_editor = true;
                    }
                });

                ui.menu_button("Help", |ui| {
                    if menu_item(ui, "About...").clicked() {
                        app.dlg.show_about = true;
                    }
                });

                // Cap the available height before the right-to-left scope.
                ui.set_max_height(26.0);
                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    let count = app.files.len();
                    let checked = app.checked_indices().len();
                    ui.label(
                        RichText::new(format!("{count} files, {checked} selected"))
                            .color(c_text_dim())
                            .size(13.5),
                    );
                });
            });
        });
}

fn menu_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(c_header())
        .inner_margin(egui::Margin::symmetric(6, 3))
        .stroke(Stroke::NONE)
}

/// How much smaller the action strip above the tree is than the rest of
/// the interface: a quarter less, as the person asked.
const ACTION_SCALE: f32 = 0.75;

/// Extra vertical gap around the controls. egui adds its own item spacing
/// before the trailing space, so the top gap is 3px smaller when both are
/// equal; these values keep the strip 33px tall but centre the row in it.
const ACTION_PAD_TOP: f32 = 5.5;
const ACTION_PAD_BOTTOM: f32 = 2.5;

fn compact_tool_button(ui: &mut Ui, label: &str, tooltip: &str) -> egui::Response {
    ui.add(egui::Button::new(label)).on_hover_text(tooltip)
}

fn toolbar(app: &mut App, root: &mut Ui, _ctx: &Context) {
    egui::Panel::top("toolbar")
        .show_separator_line(false)
        .frame(
            egui::Frame::new()
                .fill(c_header())
                .inner_margin(egui::Margin::symmetric(5, 1))
                .stroke(Stroke::NONE),
        )
        .show(root, |ui| {
            // Scale the whole strip - text, spacing and buttons - to 75%.
            ui.scope(|ui| {
                let mut style = (**ui.style()).clone();
                for font in style.text_styles.values_mut() {
                    font.size *= ACTION_SCALE;
                }
                style.spacing.item_spacing = style.spacing.item_spacing * ACTION_SCALE;
                style.spacing.button_padding = style.spacing.button_padding * ACTION_SCALE;
                ui.set_style(style);

                ui.add_space(ACTION_PAD_TOP);
                ui.horizontal_wrapped(|ui| {
                    if compact_tool_button(ui, "↻ Refresh", "Refresh file list (Ctrl+R)").clicked() {
                        if let Some(folder) = app.folder.clone() {
                            app.load_folder(&folder);
                        }
                    }
                    ui.separator();
                    if compact_tool_button(ui, "✎ Edit Tag", "Edit the focused file's tag (F5)").clicked() {
                        app.open_tag_editor_focused();
                    }
                    if compact_tool_button(ui, "⧉ Copy Tag", "Copy tag (Ctrl+Shift+C)").clicked() {
                        app.copy_tag();
                    }
                    if compact_tool_button(ui, "⇩ Paste Tag", "Paste tag (Ctrl+Shift+V)").clicked() {
                        let i = app.focused.into_iter().collect::<Vec<_>>();
                        app.paste_tag(&i);
                    }
                    ui.separator();
                    if compact_tool_button(ui, "▶ Play", "Play the focused file (Ctrl+P)").clicked() {
                        if let Some(i) = app.focused {
                            app.play_file(i);
                        }
                    }
                    ui.separator();
                    let sub = &mut app.search_subfolders;
                    if ui.checkbox(sub, "Search Subfolders").changed() {
                        if let Some(folder) = app.folder.clone() {
                            app.load_folder(&folder);
                        }
                    }
                    ui.separator();
                    ui.label(RichText::new("Filter by").color(c_text_dim()));
                    egui::ComboBox::from_id_salt("filter_field")
                        .selected_text(FILTER_FIELDS[app.filter_field])
                        .width(104.0 * ACTION_SCALE)
                        .show_ui(ui, |ui| {
                            for (i, f) in FILTER_FIELDS.iter().enumerate() {
                                if ui.selectable_label(app.filter_field == i, *f).clicked() {
                                    app.filter_field = i;
                                }
                            }
                        });
                    ui.add(
                        egui::TextEdit::singleline(&mut app.filter_text)
                            .desired_width(140.0 * ACTION_SCALE)
                            .hint_text("filter text"),
                    );
                    let filter = egui::Button::new(RichText::new("Filter!").color(c_text()).strong())
                        .fill(c_accent_dim())
                        .stroke(Stroke::new(1.0, c_accent()));
                    if ui.add(filter).clicked() {
                        // The original applies the filter whatever the text:
                        // with an empty box, sub_8413fc keeps only files
                        // whose selected field is empty.
                        app.filter_active = true;
                    }
                    if ui.button("Show All").clicked() {
                        app.filter_active = false;
                        app.filter_text.clear();
                        app.hide_checked = false;
                    }
                    // The original's caption (ToolbarFilters TBXItem57).
                    if ui.button("Hide Selected Files").clicked() {
                        app.hide_checked = true;
                        app.say("Checked files are hidden; use Show All to see everything again.");
                    }
                    // The old trailing "Folder <path>" label is gone: the
                    // tree's address box already shows the location.
                });
                ui.add_space(ACTION_PAD_BOTTOM);
            });
        });
}

fn tools_panel(app: &mut App, root: &mut Ui) {
    let _r = egui::Panel::top("tools_bar")
        .frame(
            egui::Frame::new()
                .fill(c_bg())
                .inner_margin(egui::Margin::symmetric(8, 6))
                .stroke(Stroke::NONE),
        )
        .show(root, |ui| {
            ui_tabs::tools_bar(ui, app);
        });
}

fn status_bar(app: &mut App, root: &mut Ui) {
    let r = window_radius(&root.ctx().clone());
    egui::Panel::bottom("status")
        .frame(menu_frame().corner_radius(egui::CornerRadius { nw: 0, ne: 0, sw: r, se: r }))
        .show(root, |ui| {
            ui.horizontal(|ui| {
                let total = app.files.len();
                let checked = app.checked_indices().len();
                ui.label(
                    RichText::new(format!("Total files: {total}"))
                        .color(c_text_dim())
                        .size(13.5),
                );
                ui.separator();
                ui.label(
                    RichText::new(format!("Selected files: {checked}"))
                        .color(c_text_dim())
                        .size(13.5),
                );
                ui.separator();
                ui.label(
                    RichText::new(format!("Total time: {}", app.total_time(false)))
                        .color(c_text_dim())
                        .size(13.5),
                );
                ui.separator();
                ui.label(
                    RichText::new(format!("Selected time: {}", app.total_time(true)))
                        .color(c_text_dim())
                        .size(13.5),
                );
                // Cap the available height before the right-to-left scope.
                ui.set_max_height(26.0);
                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(loading) = &app.loading {
                        ui.label(
                            RichText::new(format!("Reading: {} files", loading.seen))
                                .color(c_accent())
                                .size(13.5),
                        );
                        ui.separator();
                    }
                    ui.label(
                        RichText::new(format!("Mode: {}", app.view_mode.label()))
                            .color(c_text_dim())
                            .size(13.5),
                    );
                });
            });
        });
}

/// The folder tree on the left (the original's "Folders:" dockable panel),
/// with the address bar above it (the original's Address toolbar).
fn folders_panel(app: &mut App, root: &mut Ui) {
    // The panel always starts at this width and stays draggable. Adding a
    // location only adds a tree root: it never changes the width or
    // expands the tree, so the first-load and later states behave alike.
    // egui's own panel memory could drop the dragged width once the tree
    // content changed, so the width is held by the app and the divider is
    // a drag handle on the panel's right edge that writes it back.
    let width = app.folder_panel_width.clamp(210.0, 900.0);
    let panel = egui::Panel::left("folders")
        .exact_size(width)
        .frame(
            egui::Frame::new()
                .fill(c_panel())
                .inner_margin(egui::Margin::symmetric(8, 6))
                .stroke(Stroke::new(1.0, c_separator())),
        );
    let response = panel.show(root, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Folders:")
                        .color(c_text_dim())
                        .strong()
                        .size(14.0),
                );
                if tool_button(ui, "↻", "Refresh folders").clicked() {
                    app.expanded.clear();
                    app.folder_children.clear();
                }
                if tool_button(ui, "＋", "Add a folder root (no music is read until you select a folder inside it)").clicked() {
                    app.dlg.choose = Some(ChooseState {
                        purpose: ChoosePurpose::AddRoot,
                        dir: ui_dialogs::default_start_dir(app),
                        selected: None,
                        name: String::new(),
                        cancelled: false,
                        path_edit: String::new(),
                    });
                }
                // The divider on the right edge of this panel is draggable;
                // this hint makes that whole section's adjustability clear.
                ui.label(RichText::new("↔").color(c_text_dim()).size(13.0))
                    .on_hover_text("Drag the divider on the right edge to make the folder tree wider or narrower");
            });
            // The address bar: Go moves the tree to the pasted folder;
            // Open reads that folder and lists its subfolders/files in
            // the main window. The buttons are drawn first so the text
            // box can shrink without ever cutting "Open" off.
            let mut go = false;
            let mut open = false;
            ui.horizontal(|ui| {
                let spacing = ui.spacing().item_spacing.x;
                let btn_w = 54.0;
                let edit_w = (ui.available_width() - btn_w * 2.0 - spacing * 3.0).max(80.0);
                let resp = ui.add_sized(
                    [edit_w, 22.0],
                    egui::TextEdit::singleline(&mut app.address)
                        .hint_text("Paste a folder location…"),
                );
                if (resp.lost_focus() && ui.ctx().input(|i| i.key_pressed(Key::Enter)))
                    || ui.add_sized([btn_w, 22.0], egui::Button::new("Go")).clicked()
                {
                    go = true;
                }
                if ui.add_sized([btn_w, 22.0], egui::Button::new("Open")).clicked() {
                    open = true;
                }
            });
            if go {
                app.open_address();
            }
            if open {
                app.load_address();
            }
            ui.separator();
            let roots = folder_roots(app);
            let scroll_target = app.tree_scroll_to.clone();
            let mut open_request: Option<std::path::PathBuf> = None;
            ScrollArea::vertical()
                .id_salt("folder_tree")
                .show(ui, |ui| {
                    for root in roots {
                        tree_node(
                            ui,
                            app,
                            &root,
                            0,
                            &mut open_request,
                            scroll_target.as_deref(),
                        );
                    }
                });
            app.tree_scroll_to = None;
            if let Some(p) = open_request {
                app.load_folder(&p);
            }
        });
    // Remember the width the person dragged to, so it is still theirs
    // after adding or opening a location.
    let rect = response.response.rect;
    let handle = egui::Rect::from_min_max(
        egui::pos2(rect.right() - 3.0, rect.top()),
        egui::pos2(rect.right() + 3.0, rect.bottom()),
    );
    let divider = root.interact(handle, egui::Id::new("folders_divider"), egui::Sense::drag());
    if divider.hovered() || divider.dragged() {
        root.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
    }
    if divider.dragged() {
        app.folder_panel_width = (width + divider.drag_delta().x).clamp(210.0, 900.0);
    }
}

fn paint_folder_icon(painter: &egui::Painter, rect: egui::Rect, current: bool) {
    let body = if current { c_accent() } else { c_text_dim() };
    let tab = egui::Rect::from_min_max(rect.min, egui::pos2(rect.left() + rect.width() * 0.45, rect.top() + rect.height() * 0.35));
    let front = egui::Rect::from_min_max(egui::pos2(rect.left(), rect.top() + rect.height() * 0.2), rect.max);
    let r = egui::CornerRadius::same(2);
    painter.rect_filled(tab, r, body);
    painter.rect_filled(front, r, body);
}

fn folder_roots(app: &App) -> Vec<std::path::PathBuf> {
    // Explicit roots the person added stay in the tree on their own —
    // they never get replaced by the current folder's ancestor chain.
    let mut roots = app.roots.clone();
    // Windows always offers its shell places, even when a user root exists.
    roots.extend(crate::places::initial_roots());
    // Keep the default roots whether or not a folder is open. They used to
    // be added only with no folder selected, so the first click made them
    // vanish and replaced them with the selected folder's ancestor chain.
    #[cfg(not(windows))]
    {
        if let Some(home) = std::env::var_os("HOME") {
            let home = std::path::PathBuf::from(home);
            if home.is_dir() {
                let music = home.join("Music");
                if music.is_dir() {
                    roots.push(music);
                }
                roots.push(home);
            }
        }
        roots.push(std::path::PathBuf::from("/"));
    }
    if let Some(folder) = &app.folder {
        // Only add an ancestor chain when the folder is not already visible
        // under one of the roots. Otherwise opening it duplicates the same
        // path at the bottom of the tree (and, on Windows, creates blank
        // pseudo-folder roots for shell places).
        let visible = crate::places::expand_roots(&roots);
        let covered = !crate::places::is_place(folder)
            && visible.iter().any(|root| folder.starts_with(root));
        if !covered {
            let mut chain: Vec<std::path::PathBuf> = Vec::new();
            for ancestor in folder.ancestors() {
                chain.push(ancestor.to_path_buf());
            }
            chain.reverse();
            roots.extend(chain);
        }
    }
    roots.sort();
    roots.dedup();
    roots
}

fn tree_node(
    ui: &mut Ui,
    app: &mut App,
    path: &std::path::Path,
    depth: usize,
    open_request: &mut Option<std::path::PathBuf>,
    scroll_target: Option<&std::path::Path>,
) {
    let is_open = *app.expanded.entry(path.to_path_buf()).or_insert(false);
    let ordinary_name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string());
    let name = crate::places::display_name(path, ordinary_name);
    let has_children = crate::places::has_children(path).unwrap_or_else(|| path.is_dir());
    ui.horizontal(|ui| {
        ui.add_space((depth as f32) * 14.0);
        let arrow = if is_open && has_children { "▾" } else { "▸" };
        if has_children {
            let arrow_btn =
                egui::Button::new(RichText::new(arrow).color(c_text_dim()).size(12.5)).small();
            if ui.add(arrow_btn).clicked() {
                *app.expanded.entry(path.to_path_buf()).or_insert(false) = !is_open;
            }
        } else {
            ui.label(RichText::new(" ").size(12.5));
        }
        let is_current = match &app.tree_selected {
            Some(p) => p.as_path() == path,
            None => app.folder.as_deref() == Some(path),
        };
        let (icon_rect, _) = ui.allocate_exact_size(egui::vec2(16.0, 13.0), egui::Sense::hover());
        paint_folder_icon(ui.painter(), icon_rect, is_current);
        let label = egui::Button::new(
            RichText::new(name)
                .color(if is_current { c_accent() } else { c_text_dim() })
                .size(14.0),
        )
        .frame(false);
        let label_resp = ui.add(label);
        if label_resp.clicked() {
            if crate::places::is_place(path) {
                // Shell places are navigation nodes, not music folders.
                // Selecting one must not add it as an opened folder.
                if crate::places::has_children(path).unwrap_or(false) {
                    let entry = app.expanded.entry(path.to_path_buf()).or_insert(false);
                    *entry = !*entry;
                }
                app.tree_selected = Some(path.to_path_buf());
                app.tree_focus = true;
                app.list_focus = false;
            } else {
                let target = crate::places::resolve(path);
                *open_request = Some(target.clone());
                app.tree_selected = Some(target);
                app.tree_focus = true;
                app.list_focus = false;
            }
        }
        if scroll_target == Some(path) {
            ui.scroll_to_cursor(Some(egui::Align::Center));
        }
        label_resp.context_menu(|ui| {
            if !crate::places::is_place(path) {
                if let Some(msg) = crate::open_with::menu(ui, path) {
                    app.say(msg);
                }
            }
        });
    });
    if is_open && has_children {
        // A library can hold a thousand artist folders. Reading them on
        // every repaint made expanding the tree feel sluggish, so the
        // child list is read once and reused until Refresh is pressed.
        let children = app.tree_children(path);
        if !app.folder_children.contains_key(path) {
            app.folder_children.insert(path.to_path_buf(), children.clone());
        }
        for c in children {
            tree_node(ui, app, &c, depth + 1, open_request, scroll_target);
        }
    }
}

pub const COLS: [(&str, f32, SortColumn); 12] = [
    ("File Name", 250.0, SortColumn::Name),
    ("Artist", 140.0, SortColumn::Artist),
    ("Title", 170.0, SortColumn::Title),
    ("Album", 150.0, SortColumn::Album),
    ("Year", 46.0, SortColumn::Year),
    ("Track #", 56.0, SortColumn::Track),
    ("Genre", 90.0, SortColumn::Genre),
    // "Tag Version" is the original's column name (UTF-16 constant at
    // 0xdf39a2 in the payload's file-list column-name table).
    ("Tag Version", 72.0, SortColumn::Tag),
    ("Time", 52.0, SortColumn::Time),
    ("Bitrate", 60.0, SortColumn::Bitrate),
    ("Size", 62.0, SortColumn::Size),
    ("Hz", 52.0, SortColumn::Hz),
];

/// The default column layout: every column shown, in the table's order.
pub fn default_columns() -> Vec<ColumnState> {
    COLS.iter()
        .enumerate()
        .map(|(id, c)| ColumnState { id, width: c.1, visible: true })
        .collect()
}

/// Space left of the first column for the tick box and the tree triangle.
const HEAD_PAD: f32 = 30.0;

fn file_list(app: &mut App, root: &mut Ui) {
    CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(c_bg())
                .inner_margin(egui::Margin::same(6)),
        )
        .show(root, |ui| {
            // Loading banner (a folder scan is running in the background).
            if let Some(loading) = &app.loading {
                panel_frame(c_panel()).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("Reading files…")
                                .color(c_accent())
                                .strong()
                                .size(14.0),
                        );
                        ui.label(
                            RichText::new(format!(
                                "{} — {} files so far",
                                loading.folder.display(),
                                loading.seen
                            ))
                            .color(c_text_dim())
                            .size(13.5),
                        );
                    });
                });
                ui.add_space(4.0);

            }
            let rows = app.visible_rows();
            let visible: Vec<ColumnState> = app.columns.iter().copied().filter(|c| c.visible).collect();
            let total_w = HEAD_PAD + visible.iter().map(|c| c.width).sum::<f32>();
            // Wider than the panel: the list scrolls sideways, so a column
            // dragged wider never pushes the others out of reach.
            ScrollArea::horizontal()
                .id_salt("file_list_cols")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let width = total_w.max(ui.clip_rect().width());
                    ui.set_min_width(width);
                    file_list_body(app, ui, &rows, &visible, width);
                });
        });
}

fn file_list_body(app: &mut App, ui: &mut Ui, rows: &[usize], visible: &[ColumnState], width: f32) {
    let ctx = ui.ctx().clone();
    let (header_rect, _) = ui.allocate_at_least(egui::vec2(width, 18.0), Sense::hover());
    let mut sort_request: Option<Option<(SortColumn, bool)>> = None;
    // Each visible column's id, left edge and width, in display order.
    let mut heads: Vec<(usize, f32, f32)> = Vec::new();
    {
        let painter = ui.painter().clone();
        let mut x = header_rect.left() + HEAD_PAD;
        for cs in visible {
            let (title, _, col) = COLS[cs.id];
            let sorted = app.sort == Some((col, true));
            let sorted_desc = app.sort == Some((col, false));
            let arrow = if sorted {
                " ↑"
            } else if sorted_desc {
                " ↓"
            } else {
                ""
            };
            painter.text(
                egui::pos2(x + 2.0, header_rect.center().y),
                egui::Align2::LEFT_CENTER,
                format!("{title}{arrow}"),
                egui::FontId::proportional(13.5),
                c_text_dim(),
            );
            // Dragging the header body moves the column; a plain click
            // sorts by it. The strip on the right edge resizes it.
            let head_rect = egui::Rect::from_min_max(
                egui::pos2(x + 2.0, header_rect.top()),
                egui::pos2(x + cs.width - 4.0, header_rect.bottom()),
            );
            let hit = ui.interact(head_rect, ui.id().with(("col_head", cs.id)), Sense::click_and_drag());
            if hit.clicked() {
                sort_request = Some(match app.sort {
                    Some((c, true)) if c == col => Some((col, false)),
                    _ => Some((col, true)),
                });
            }
            if hit.drag_started() {
                app.col_drag = Some(cs.id);
            }
            let grip = egui::Rect::from_min_max(
                egui::pos2(x + cs.width - 4.0, header_rect.top()),
                egui::pos2(x + cs.width + 2.0, header_rect.bottom()),
            );
            let grip_resp = ui.interact(grip, ui.id().with(("col_grip", cs.id)), Sense::drag());
            if grip_resp.hovered() || grip_resp.dragged() {
                ctx.set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
            }
            if grip_resp.dragged() {
                if let Some(c) = app.columns.iter_mut().find(|c| c.id == cs.id) {
                    c.width = (c.width + grip_resp.drag_delta().x).clamp(MIN_COL_W, 900.0);
                }
            }
            if grip_resp.drag_stopped() {
                save_columns(&app.columns);
            }
            painter.line_segment(
                [
                    egui::pos2(x + cs.width + 0.5, header_rect.top() + 3.0),
                    egui::pos2(x + cs.width + 0.5, header_rect.bottom() - 3.0),
                ],
                Stroke::new(1.0, c_text_dim().gamma_multiply(0.35)),
            );
            heads.push((cs.id, x, cs.width));
            x += cs.width;
        }
        painter.line_segment(
            [header_rect.left_bottom(), header_rect.right_bottom()],
            Stroke::new(1.0, c_text_dim().gamma_multiply(0.35)),
        );
    }
    if let Some(s) = sort_request {
        app.sort = s;
    }

    if let Some(drag_id) = app.col_drag {
        let pointer = ui.input(|i| i.pointer.interact_pos());
        let down = ui.input(|i| i.pointer.primary_down());
        // The slot the column would land in: before the first column whose
        // middle lies to the right of the pointer.
        let target = pointer.map(|p| {
            heads
                .iter()
                .position(|(_, hx, hw)| p.x < hx + hw / 2.0)
                .unwrap_or(heads.len())
        });
        if down {
            if let Some(t) = target {
                let line_x = match heads.get(t) {
                    Some((_, hx, _)) => hx - 1.0,
                    None => heads.last().map(|(_, hx, hw)| hx + hw).unwrap_or(header_rect.left()),
                };
                ui.painter().line_segment(
                    [
                        egui::pos2(line_x, header_rect.top()),
                        egui::pos2(line_x, header_rect.bottom()),
                    ],
                    Stroke::new(3.0, c_accent()),
                );
            }
        } else {
            let from = heads.iter().position(|(id, _, _)| *id == drag_id);
            if let (Some(t), Some(from)) = (target, from) {
                if t != from && t != from + 1 {
                    let before = heads.get(t).map(|(id, _, _)| *id);
                    if let Some(idx) = app.columns.iter().position(|c| c.id == drag_id) {
                        let item = app.columns.remove(idx);
                        let to = match before {
                            Some(id) => app
                                .columns
                                .iter()
                                .position(|c| c.id == id)
                                .unwrap_or(app.columns.len()),
                            None => app.columns.len(),
                        };
                        app.columns.insert(to, item);
                        save_columns(&app.columns);
                    }
                }
            }
            app.col_drag = None;
        }
    }
    ui.add_space(1.0);

    if rows.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            let msg = if app.loading.is_some() {
                "Reading files…"
            } else if app.files.is_empty() {
                "There is no files in program file list!"
            } else {
                "No files match the current filter."
            };
            ui.label(RichText::new(msg).color(c_text_dim()));
        });
        return;
    }
    let row_h = 20.0;
    let mut rows_scroll = ScrollArea::vertical()
        .id_salt("rows")
        .max_width(width)
        .auto_shrink([false, false]);
    if let Some(pos) = app.scroll_row.take() {
        let step = row_h + ui.spacing().item_spacing.y;
        let view_h = ui.available_height();
        let offset = (pos as f32 * step - (view_h - row_h) / 2.0).max(0.0);
        rows_scroll = rows_scroll.vertical_scroll_offset(offset);
    }
    rows_scroll.show_rows(ui, row_h, rows.len(), |ui, range| {
            for r in range {
                let i = rows[r];
                let focused = app.focused == Some(i);
                let name = if app.hide_extensions {
                    app.files[i].file_stem()
                } else {
                    app.files[i].name.clone()
                };
                let track_text = {
                    let t = &app.files[i].fields.track;
                    if app.files[i].fields.track_total.is_empty() {
                        t.clone()
                    } else {
                        format!("{}/{}", t, app.files[i].fields.track_total)
                    }
                };
                let is_dir = app.files[i].is_dir;
                let bitrate_text = if is_dir && app.files[i].bitrate_kbps == 0 {
                    String::new()
                } else {
                    format!("{} kbps", app.files[i].bitrate_kbps)
                };
                let hz_text = if is_dir {
                    String::new()
                } else {
                    app.files[i].sample_rate.to_string()
                };
                let cells: [String; 12] = [
                    name,
                    app.files[i].fields.artist.clone(),
                    app.files[i].fields.title.clone(),
                    app.files[i].fields.album.clone(),
                    app.files[i].fields.year.clone(),
                    track_text,
                    app.files[i].fields.genre.clone(),
                    if app.files[i].tag_kind.is_empty() {
                        "no tag".into()
                    } else {
                        app.files[i].tag_kind.clone()
                    },
                    app.files[i].play_time(),
                    bitrate_text,
                    app.files[i].size_text(),
                    hz_text,
                ];
                let bg = if focused {
                    c_selected()
                } else if r % 2 == 1 {
                    c_panel()
                } else {
                    c_bg()
                };
                let (rect, resp) =
                    ui.allocate_at_least(egui::vec2(width, row_h), Sense::click());
                ui.painter().rect_filled(rect, 0.0, bg);
                if focused {
                    ui.painter().rect_stroke(
                        rect,
                        0.0,
                        Stroke::new(1.0, c_accent_dim()),
                        egui::StrokeKind::Middle,
                    );
                }
                let y = rect.center().y;
                let depth = row_depth(app, &app.files[i].path);
                let indent = depth as f32 * 14.0;
                // Own columns: triangle, tick box, then the name.
                // The old layout started the name at x + 7 while
                // the box covered x - 2 .. x + 16, so long names
                // were drawn on top of the tick (and vice versa).
                let check_left = rect.left() + if is_dir { 20.0 } else { 8.0 } + indent;
                // The first column moves right past the indent, so the
                // tree shows nested rows; the other columns keep their place.
                let row_shift = indent + if is_dir { 12.0 } else { 0.0 };
                let mut tri_clicked = false;
                if is_dir {
                    // Folder rows drop down to show the files they
                    // contain; the triangle makes that discoverable.
                    let tri_rect = egui::Rect::from_min_size(
                        egui::pos2(rect.left() + 2.0 + indent, y - 7.0),
                        egui::vec2(14.0, 14.0),
                    );
                    let expanded = app.expanded_rows.contains_key(&app.files[i].path);
                    ui.painter().text(
                        egui::pos2(tri_rect.center().x, y),
                        egui::Align2::CENTER_CENTER,
                        if expanded { "▾" } else { "▸" },
                        egui::FontId::proportional(13.5),
                        c_text_dim(),
                    );
                    tri_clicked = ui
                        .interact(
                            tri_rect,
                            ui.scope_id().with(("expand_row", app.files[i].path.clone())),
                            Sense::click(),
                        )
                        .clicked();
                }
                // The per-row checkbox (the original selects files
                // with these before tag operations). It now has a
                // dedicated 16 px cell with a gap before the name.
                let check_rect = egui::Rect::from_min_size(
                    egui::pos2(check_left, y - 8.0),
                    egui::vec2(16.0, 16.0),
                );
                let mut checked = app.checked[i];
                let check_resp = ui.put(
                    check_rect,
                    egui::Checkbox::new(&mut checked, ""),
                );
                if check_resp.clicked() {
                    app.checked[i] = checked;
                }
                let mut col_x = rect.left() + HEAD_PAD;
                for (pos, cs) in visible.iter().enumerate() {
                    let shift = if pos == 0 { row_shift } else { 0.0 };
                    let avail = (cs.width - shift - 6.0).max(24.0);
                    ui.painter().text(
                        egui::pos2(col_x + 2.0 + shift, y),
                        egui::Align2::LEFT_CENTER,
                        truncate(&cells[cs.id], avail),
                        egui::FontId::proportional(14.0),
                        c_text(),
                    );
                    col_x += cs.width;
                }
                if resp.secondary_clicked() && !app.checked[i] {
                    // Right-click picks the row first, as in a file manager.
                    for c in &mut app.checked {
                        *c = false;
                    }
                    app.checked[i] = true;
                    app.focused = Some(i);
                    app.selection_anchor = Some(i);
                }
                if resp.clicked() || resp.secondary_clicked() {
                    app.tree_focus = false;
                    app.list_focus = true;
                }
                let row_path = app.files[i].path.clone();
                resp.context_menu(|ui| {
                    if let Some(msg) = crate::open_with::menu(ui, &row_path) {
                        app.say(msg);
                    }
                });
                if tri_clicked {
                    app.toggle_row_expand(i);
                } else if resp.double_clicked() {
                    // Double clicking a folder drops it down; double
                    // clicking a song opens the Tag Editor (the
                    // original's actFile_EditFileTag handler).
                    if is_dir {
                        app.toggle_row_expand(i);
                    } else {
                        app.open_tag_editor_focused();
                    }
                } else if resp.clicked() {
                    let mods = ui.ctx().input(|inp| inp.modifiers);
                    if mods.shift {
                        app.select_range(i);
                    } else if mods.ctrl || mods.command {
                        app.toggle_row_check(i);
                    } else {
                        for c in &mut app.checked {
                            *c = false;
                        }
                        app.checked[i] = true;
                        app.focused = Some(i);
                        app.selection_anchor = Some(i);
                    }
                }
            }
        });
}

/// How far a row is nested under the folder that was opened in the main
/// list. Used to indent expanded folder children.
fn row_depth(app: &App, path: &std::path::Path) -> usize {
    let Some(folder) = &app.folder else { return 0 };
    let mut depth = 0;
    let mut parent = path.parent();
    while let Some(p) = parent {
        if p == folder {
            break;
        }
        depth += 1;
        parent = p.parent();
    }
    depth
}

fn truncate(s: &str, width: f32) -> String {
    let max_chars = (width / 7.2) as usize;
    if s.chars().count() > max_chars {
        let cut: String = s.chars().take(max_chars.saturating_sub(1)).collect();
        format!("{cut}…")
    } else {
        s.to_string()
    }
}

/// Cover art dragged in from a file manager: with the Tag Editor open
/// the picture becomes its Art (staged until Save); otherwise it
/// replaces the focused file's cover art directly — the drop target the
/// cover panels' "Drag and drop cover art here" hints promise.
fn handle_drops(app: &mut App, ctx: &Context) {
    let drops = ctx.input_mut(|i| std::mem::take(&mut i.raw.dropped_files));
    for drop in drops {
        let path = drop.path();
        let is_image = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| {
                matches!(
                    e.to_ascii_lowercase().as_str(),
                    "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp"
                )
            })
            .unwrap_or(false);
        if !is_image {
            continue;
        }
        let bytes = drop.bytes().ok().or_else(|| std::fs::read(path).ok());
        let Some(bytes) = bytes else { continue };
        if app.dlg.tag_editor.is_some() {
            crate::ui_tageditor::stage_cover(app, bytes);
        } else if let Some(i) = app.focused {
            let path = app.files[i].path.clone();
            match crate::data::set_cover(&path, bytes) {
                Ok(()) => {
                    if let Some(re) = crate::data::read_file(&path) {
                        app.files[i] = re;
                    }
                    app.say("Cover art saved to the file's tag.");
                }
                Err(e) => app.warn(e),
            }
        }
        break;
    }
}

/// Gap kept between the notification box and the status bar at the bottom.
const TOAST_ABOVE_STATUS: f32 = 44.0;

fn toast(app: &mut App, ctx: &Context) {
    if let Some((msg, at, is_err)) = app.toast.clone() {
        let age = at.elapsed().as_millis() as u64;
        if age > 7000 {
            app.toast = None;
            return;
        }
        let id = egui::Id::unique("toast");
        let layer = egui::Order::Foreground;
        let avail = ctx
            .input(|i| i.raw.screen_rect)
            .unwrap_or_else(|| egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1180.0, 800.0)));
        let width = 380.0;
        let painter = ctx.layer_painter(egui::LayerId::new(layer, id));
        let galley = painter.layout(msg, egui::FontId::proportional(14.0), c_text(), width - 24.0);
        let height = galley.size().y + 24.0;
        let rect = egui::Rect::from_min_max(
            egui::pos2(avail.right() - width - 14.0, avail.bottom() - TOAST_ABOVE_STATUS - height),
            egui::pos2(avail.right() - 14.0, avail.bottom() - TOAST_ABOVE_STATUS),
        );
        painter.rect_filled(rect, 8.0, c_panel_bright());
        painter.rect_stroke(
            rect,
            8.0,
            Stroke::new(1.0, if is_err { c_danger() } else { c_accent() }),
            egui::StrokeKind::Middle,
        );
        painter.galley(rect.min + egui::vec2(12.0, 12.0), galley, c_text());
        // The toast fades after seven seconds; a gentle poll is enough,
        // not a full-speed repaint for its whole lifetime.
        ctx.request_repaint_after(std::time::Duration::from_millis(200));
    }
}

fn shortcuts(app: &mut App, ctx: &Context) {
    let tab = ctx.input(|i| {
        if i.key_pressed(Key::F4) && i.modifiers.ctrl {
            Some(1)
        } else if i.key_pressed(Key::F4) && i.modifiers.shift {
            Some(2)
        } else if i.key_pressed(Key::F4) {
            Some(0)
        } else {
            None
        }
    });
    if let Some(t) = tab {
        app.active_tab = t;
    }
    let ctrl = ctx.input(|i| i.modifiers.ctrl);
    let shift = ctx.input(|i| i.modifiers.shift);
    if ctrl && ctx.input(|i| i.key_pressed(Key::A)) {
        app.select_all_visible();
    }
    if ctrl && !shift && ctx.input(|i| i.key_pressed(Key::N)) {
        for c in &mut app.checked {
            *c = false;
        }
    }
    if ctrl && ctx.input(|i| i.key_pressed(Key::I)) {
        for c in &mut app.checked {
            *c = !*c;
        }
    }
    if ctrl && ctx.input(|i| i.key_pressed(Key::F)) {
        app.dlg.show_find = true;
    }
    if ctrl && ctx.input(|i| i.key_pressed(Key::P)) {
        if let Some(i) = app.focused {
            app.play_file(i);
        }
    }
    if ctrl && ctx.input(|i| i.key_pressed(Key::R)) {
        if let Some(folder) = app.folder.clone() {
            app.load_folder(&folder);
        }
    }
    if ctrl && ctx.input(|i| i.key_pressed(Key::T)) {
        app.dlg.show_autofill = true;
    }
    if ctrl && shift && ctx.input(|i| i.key_pressed(Key::C)) {
        app.copy_tag();
    }
    if ctrl && shift && ctx.input(|i| i.key_pressed(Key::V)) {
        let i = app.focused.into_iter().collect::<Vec<_>>();
        app.paste_tag(&i);
    }
    if ctx.input(|i| i.key_pressed(Key::F5)) {
        app.open_tag_editor_focused();
    }
    if ctx.input(|i| i.key_pressed(Key::F8)) {
        app.show_tools_bar = !app.show_tools_bar;
    }
    if (app.tree_focus || app.list_focus)
        && app.dlg.tag_editor.is_none()
        && !ctx.egui_wants_keyboard_input()
    {
        let typed = ctx.input(|i| {
            if i.modifiers.ctrl || i.modifiers.alt || i.modifiers.command {
                return None;
            }
            i.events.iter().find_map(|e| match e {
                egui::Event::Text(t) => t.chars().next().filter(|c| c.is_alphabetic()),
                _ => None,
            })
        });
        if let Some(c) = typed {
            if app.tree_focus {
                app.jump_tree_letter(c);
            } else {
                app.jump_to_letter(c);
            }
        }
    }
}

fn screenshot_flow(app: &mut App, ctx: &Context) {
    // Stage the scene the person's picture of the original shows
    // (reference/picture-1.png): the Tag Editor open over the main window
    // after a double click on a song, just below the middle of the
    // window (the person now wants it kept at the window's full width).
    let frame = app.shot.as_ref().map_or(0, |s| s.frame);
    // TAGRENAME_NO_EDITOR is a local verification knob: it lets a picture
    // of the main screen (with the cover boxes) be taken. The official
    // check does not set it, so its normal picture still shows the editor.
    let staged_editor = std::env::var_os("TAGRENAME_NO_EDITOR").is_none();
    // Verification knob: TAGRENAME_MULTI=1 checks three rows and opens the
    // multi-file Tag Editor, to picture its tick boxes.
    if frame == 20 && app.dlg.tag_editor.is_none() && std::env::var_os("TAGRENAME_MULTI").is_some() {
        for i in 0..3.min(app.checked.len()) {
            app.checked[i] = true;
        }
        app.open_tag_editor();
    }
    if staged_editor && frame == 20 && app.dlg.tag_editor.is_none() && app.focused.is_some()
        && std::env::var_os("TAGRENAME_MULTI").is_none()
    {
        app.open_tag_editor_focused();
    }
    // Verification knob: TAGRENAME_EDITOR_FULL=1 opens the Tag Editor full screen.
    if frame == 20 && std::env::var_os("TAGRENAME_EDITOR_FULL").is_some() {
        if let Some(ed) = app.dlg.tag_editor.as_mut() {
            ed.full = true;
        }
    }
    // Verification knob: TAGRENAME_EDITOR_TAB=0..5 picks the editor tab to picture.
    if let Ok(t) = std::env::var("TAGRENAME_EDITOR_TAB") {
        if let (Ok(t), Some(ed)) = (t.trim().parse::<usize>(), app.dlg.tag_editor.as_mut()) {
            ed.tab = t.min(4);
        }
    }
    let Some(shot) = &mut app.shot else {
        return;
    };
    if shot.frame == 1 {
        crate::shot_log("event loop running, first update");
    }
    shot.frame += 1;
    // Ask for the screenshot after about 110 frames.
    if shot.frame == 110 && !shot.requested {
        shot.requested = true;
        crate::shot_log("requesting screenshot");
        let slot = std::sync::Arc::new(egui::mutex::Mutex::new(None));
        shot.slot = Some(slot.clone());
        ctx.request_screenshot(move |image: std::sync::Arc<egui::ColorImage>| {
            // Note: this callback may be called from another thread.
            *slot.lock() = Some(image);
        });
    }
    // Pick up the result when the callback has fired.
    let mut got: Option<std::sync::Arc<egui::ColorImage>> = None;
    if let Some(slot) = &shot.slot {
        got = slot.lock().take();
    }
    if let Some(image) = got {
        save_png(&shot.path.clone(), &image);
        crate::SHOT_SAVED.store(true, std::sync::atomic::Ordering::SeqCst);
        crate::shot_log(&format!("saved {}", shot.path.display()));
        eprintln!("screenshot: saved to {}", shot.path.display());
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
    // Watchdog: never hang forever.
    if shot.frame > 600 {
        crate::shot_log("frame watchdog: giving up");
        eprintln!("screenshot: gave up waiting for the frame after 600 frames");
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
}

fn save_png(path: &std::path::Path, image: &egui::ColorImage) {
    // The target folder may not exist yet (a checker can pass a fresh
    // directory), so make sure it does before writing.
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                crate::shot_log(&format!("mkdir for {} failed: {e}", path.display()));
            }
        }
    }
    if let Err(e) = image::save_buffer(
        path,
        image.as_raw(),
        image.width() as u32,
        image.height() as u32,
        image::ColorType::Rgba8,
    ) {
        eprintln!("screenshot save failed: {e}");
        crate::shot_log(&format!("save to {} failed: {e}", path.display()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_roots_stay_when_a_folder_is_selected() {
        let Some(home) = std::env::var_os("HOME").map(std::path::PathBuf::from) else {
            return;
        };
        if !home.is_dir() {
            return;
        }
        let mut before = crate::ui::App::new();
        before.roots.clear();
        let expected = folder_roots(&before);
        let mut after = crate::ui::App::new();
        after.roots.clear();
        after.folder = Some(home);
        assert_eq!(
            folder_roots(&after),
            expected,
            "selecting Home must not replace the default roots with an ancestor chain"
        );
    }

    #[test]
    fn added_root_stays_in_the_tree_after_a_child_is_loaded() {
        let root = std::env::temp_dir().join("tagrename_root_persist_test");
        let child = root.join("Artist One");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&child).unwrap();
        let mut app = crate::ui::App::new();
        app.add_root(&root);
        app.folder = Some(child.clone());
        let roots = folder_roots(&app);
        assert!(
            roots.contains(&root),
            "the library root disappeared after selecting a folder inside it"
        );
        assert!(
            !roots.contains(&child),
            "a folder already under a root must not become a second tree root"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    fn tiny_wav(path: &std::path::Path) {
        // A valid 1-sample 8 kHz mono PCM WAV is enough for lofty to
        // recognise the file while keeping the test independent of any
        // sample audio shipped with the project.
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&36u32.to_le_bytes());
        bytes.extend_from_slice(b"WAVEfmt ");
        bytes.extend_from_slice(&16u32.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes()); // PCM
        bytes.extend_from_slice(&1u16.to_le_bytes()); // mono
        bytes.extend_from_slice(&8000u32.to_le_bytes());
        bytes.extend_from_slice(&8000u32.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&8u16.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&1u32.to_le_bytes());
        bytes.push(128);
        std::fs::write(path, bytes).unwrap();
    }

    /// Open reads the folder on a background thread: wait until it lands.
    fn settle(app: &mut crate::ui::App) {
        let mut guard = 0;
        while app.loading.is_some() {
            app.poll_loading();
            std::thread::sleep(std::time::Duration::from_millis(2));
            guard += 1;
            assert!(guard < 10000, "scan did not settle");
        }
    }

    #[test]
    fn open_lists_folders_and_expanding_shows_their_files() {
        let root = std::env::temp_dir().join("tagrename_folder_rows_test");
        let child = root.join("Artist One");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&child).unwrap();
        tiny_wav(&child.join("track.wav"));
        let mut app = crate::ui::App::new();
        app.load_folder(&root);
        settle(&mut app);
        assert!(app.files.is_empty(), "an artist folder loads nothing with the box off");
        app.search_subfolders = true;
        app.load_folder(&root);
        settle(&mut app);
        assert_eq!(app.files.len(), 1, "Open should list the subfolder");
        assert!(app.files[0].is_dir);
        assert_eq!(app.files[0].fields.track, "1", "folder shows its file count");

        app.toggle_row_expand(0);
        assert_eq!(app.files.len(), 2, "expanding reveals the folder's file");
        assert!(!app.files[1].is_dir);
        assert_eq!(row_depth(&app, &app.files[1].path), 1);

        app.select_all_visible();
        assert!(app.checked.iter().all(|c| *c), "Ctrl+A selects folder and file");
        app.toggle_row_expand(0);
        assert_eq!(app.files.len(), 1, "collapsing hides the children again");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn shift_range_and_multi_editor_from_folder_selection() {
        let root = std::env::temp_dir().join("tagrename_multi_folder_test");
        for name in ["A", "B", "C"] {
            let dir = root.join(name);
            std::fs::create_dir_all(&dir).unwrap();
            tiny_wav(&dir.join("song.wav"));
        }
        let mut app = crate::ui::App::new();
        app.search_subfolders = true;
        app.load_folder(&root);
        settle(&mut app);
        app.checked[0] = true;
        app.focused = Some(0);
        app.selection_anchor = Some(0);
        app.select_range(2);
        assert_eq!(app.checked, vec![true, true, true], "Shift selects the whole range");

        app.open_tag_editor();
        let ed = app.dlg.tag_editor.as_ref().expect("the multi editor opens");
        assert!(ed.multi, "selecting several folders opens the multi editor");
        assert!(
            ed.indices.iter().all(|i| !app.files[*i].is_dir),
            "the gathered files, not the folder rows, are edited"
        );
        let _ = std::fs::remove_dir_all(&root);
    }
}

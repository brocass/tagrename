# Latest pass: version 1.1.1 Linux package

-76. Clicking either a hex colour box or its swatch in Create theme now opens an in-app Photoshop-style Colour picker: saturation/value square, vertical hue strip, hex and RGB fields, old/new swatches, and OK/Cancel. HSV conversions round-trip every `#RRGGBB` value, and the picker targets the exact dark/light field it was opened from (ui_dialogs.rs).
-77. Custom theme cards no longer draw the enabled tick — the accent border already shows selection. Custom cards instead have a small pencil at the top right; clicking it opens that theme in Create theme, where it can be edited, exported or removed. Built-in cards keep the tick (ui_dialogs.rs).
-78. The app icon was replaced from /home/steven/Documents/tagrename2.svg: tagrename.svg plus 16/32/48/64/128/256/512 PNGs and a six-size ICO were regenerated in assets/. The installed 256 PNG and scalable SVG match those assets byte for byte.
-79. Version bumped to 1.1.1 in the title bar/About dialog, Cargo files, package READMEs, PKGBUILD, package-windows.bat and tagrename-installer.iss. cargo test passes 44/44. Linux package rebuilt and installed at the person's request: dist/tagrename-1.1.1-linux-x86_64.tar.gz SHA-256 8790d06d19b425be37b654831c0a754d880221606e7bc0121ed99a8a66a228fa; installed /home/steven/.local/bin/tagrename SHA-256 ba5fc4122370085ecd2879c5909f89fd2cd7d2695d871105d77ca7927b901b3f. The binary contains "Tag:Rename 1.1.1", and a screenshot run opened the colour picker over the Create theme panel.
-80. Windows packages remain at 1.1.0 as requested (not rebuilt). The Windows build tree and host rebuild/verify scripts were synced to 1.1.1 for a later VM rebuild.

# Earlier pass: version 1.1.0 Linux package

-73. Version bumped to 1.1.0 for the custom-theme feature and Slab light-text change. Updated Cargo.toml/Cargo.lock, the title bar and About dialog, both package READMEs, PKGBUILD, package-windows.bat, tagrename-installer.iss and the Windows host rebuild/verify scripts. cargo test passes 42/42.
-74. Linux package rebuilt and installed at the person's request: dist/tagrename-1.1.0-linux-x86_64.tar.gz SHA-256 0fd3aa856be4b981696e9a35acd7ca1cdb135068f340a8cd924c26627743cfff; installed /home/steven/.local/bin/tagrename SHA-256 4c35862abe49dce6d8138c9a41fc2aefbb29dd8be8d198a4e2433c698fe161a4. The binary contains "Tag:Rename 1.1.0", and a screenshot run opened the Appearance window showing the Import and Create theme buttons.
-75. Windows packages were intentionally left at 1.0.1. The Windows build source tree and its host scripts were synced to 1.1.0 so a later VM rebuild can produce matching packages.

# Earlier pass: custom themes and Slab light text

-69. Slab's light text is now #2F2F2F (was #504945); the dark palette is unchanged. Updated the Slab colour test (theme.rs).
-70. The Appearance window now has right-aligned Import and Create theme buttons beside the Theme heading. Clicking Create theme with a custom theme selected edits it; with a built-in selected it starts a new theme from those colours. A custom theme card says "Custom" instead of "Built in" (ui_dialogs.rs).
-71. The Create theme panel has a Name field, Dark/Light buttons showing which mode is being edited, Background/Text/Accent rows with a colour swatch beside each #RRGGBB box, a "Generate dark colours from light" / "Generate Light colours from Dark" button that builds a complementary opposing palette, and a bottom row with Remove theme on the left and Export / Cancel / Save theme on the right (ui_dialogs.rs, theme.rs).
-72. Custom themes are appended after the eleven built-ins, so saved numeric theme indices keep their meaning. They persist in ~/.config/tagrename/themes.json (or %APPDATA%/Tag-Rename/themes.json on Windows). Export/Import use small in-app path panels defaulting to that config folder; exported files contain one theme as JSON, and Import also accepts a themes.json-style array. cargo test passes 42/42.

# Earlier pass: Slab theme

-57. Added Slab to View > Appearance, between GitHub and High Contrast, as the ninth theme (index 8, so existing saved choices keep their meaning). Light: background #8b8b8b, contrasting surface #818181, text #504945, accent #f37135. Dark: background #403d3d, contrasting surface #4f4c4c, text #d5c4a1, accent #f37135. The remaining dim text and accent/shadow colours are sampled from the supplied screenshots (theme.rs).
-58. Checked with local screenshots shots/slab-dark.png and shots/slab-light.png; their dominant colours match the requested backgrounds and contrasting surfaces. cargo test passes 35/35. Packages were rebuilt afterwards; see the packaging pass below.

# Latest pass: Format Track # xx/xx round trip

-62. Fixed the reported case where Format Track # to 0x skipped files whose track displayed as "01/14". A reader splits that into track "01" plus total "14"; the old comparison only looked at the two halves and the old 0x rule kept the total, so the result looked unchanged and no file was written. x, 0x and 00x now clear the embedded total, and the unchanged check compares the displayed "01/14" pair (tools.rs apply, ui.rs format_track).
-63. xx/xx -> 0x -> xx/xx now round-trips at the `apply()` level: "01" -> "01/14" -> "01" -> "01/14". Tests: 37 passed, including track_formats_round_trip_between_plain_and_xx_xx and clearing_track_total_removes_the_slash_from_mp3 (which checks the raw ID3v2 TRCK frame). A second save-path bug was found before packaging; see -64.

# Latest pass: Format Track # xx/xx save

-64. The remaining 9-of-14 symptom came from the MP3 writer: after lofty wrote TrackNumber and TrackTotal, write_id3v2_track_text rewrote TRCK with only the padded number ("01"), silently dropping "/14". So xx/xx never persisted on padded tracks and a later 0x had no total to remove. The raw writer now includes "/total" when Field::TrackTotal is checked. Added saving_xx_xx_keeps_the_total_in_mp3; 38 tests pass.
-65. Linux package rebuilt and installed with both fixes: dist/tagrename-1.0.0-linux-x86_64.tar.gz SHA-256 7ace6869e589e10f71274d72aa93dc70e2ca3239ced836ed3b59d42a9fc6e727; installed /home/steven/.local/bin/tagrename SHA-256 91bdfedafd847def54db708f5426c7bb968a2206938d850c8bb189fd402e8cfa. An app that was already open keeps the old inode, so it must be restarted to use the fix. Windows packages still need one more rebuild with this save fix.

# Latest pass: version 1.0.1

-66. Version bumped to 1.0.1 so the track-format fixes are visible. Updated Cargo.toml/Cargo.lock, the title bar and About dialog, both package READMEs, PKGBUILD, package-windows.bat, tagrename-installer.iss and the Windows rebuild/verify helper scripts. cargo test passes 38/38.
-67. Linux 1.0.1 package rebuilt and installed: dist/tagrename-1.0.1-linux-x86_64.tar.gz SHA-256 a4366389e738d027243448d6ef80771f5ce5879f795a6c38dabeacb03fe54986; installed /home/steven/.local/bin/tagrename SHA-256 da88cd44f44018bd667d52f2a78e03f13da747da775533ab66747378089251e6. The binary contains "Tag:Rename 1.0.1". The user confirmed the xx/xx -> 0x fix after closing and reopening the app.
-68. Windows 1.0.1 packages rebuilt in the VM: Tag-Rename-1.0.1-windows-x64.zip SHA-256 13e2f45fdb9bc292228dc197729a8723b9b3b0fa629f47ab76754d302750d088 and Tag-Rename-1.0.1-setup.exe SHA-256 7ba4115eac5a9043ff837c7abd3a62dd557c4e7c7f79406c66205e449c9e47c1. Installer, Start Menu shortcut, config folder and app launch verified; shell label "Open with Tag:Rename", command and all 22 extensions verified. The sample WAV launch loaded 2 files with 1 selected, and the EXE contains "Tag:Rename 1.0.1". A Windows Open File security warning for the new setup.exe was accepted during verification.

# Earlier pass: package shell menus and Slab theme

-59. Linux package rebuilt and installed: dist/tagrename-1.0.0-linux-x86_64.tar.gz SHA-256 abc35c87e2c616dfbdb5ee9a56e8ac2a00190e6cdfc2a91ea5bca71f1076edd6; installed /home/steven/.local/bin/tagrename SHA-256 04e5c563baf475ad85935c3687dd370609f72231b4c188239628cd7b3827085b. Desktop entry has Exec=... %f and the music MIME types; opening one WAV loaded both files and selected the clicked one.
-60. Windows packages rebuilt in the VM from commit ebca12e: Tag-Rename-1.0.0-windows-x64.zip SHA-256 f4cdc817fac386779d9634dbd51a65e2102bc46d90a8dcf0fe5be40010ee132a and Tag-Rename-1.0.0-setup.exe SHA-256 3fa47470c5d134dd180b98a5366701e03e1e60a7b4164ee1c7ed0d3a95d90378. Installer, Start Menu shortcut, config folder and app launch all verified.
-61. Windows shell menu verified in the installed registry: HKCU Directory\shell\TagRename has label "Open with Tag:Rename" and command "{app}\Tag-Rename.exe" "%1"; all 22 music extensions are present with the same command. Opening a sample WAV loaded its whole folder (2 files, 1 selected) and showed the Slab dark palette. The host-only rebuild script now cd's to Z:\tagrename-windows-build before invoking cargo, because PowerShell's PSDrive location alone was not inherited by cmd.exe.

# Earlier pass: modern dark mode, cover art buttons

-55. Modern dark mode is the default: the Default theme's dark palette is a charcoal (#0f1115 background, #171a21 panels, blue #5b8cff accent) and the colour scheme starts on Dark instead of System (theme.rs, main.rs). Light and the other themes are unchanged. Checked in app/shots/dark-now.png.
-56. Cover art boxes: the size text (e.g. "920 x 549, PNG") sits on its own line, and Delete / Set from file art / Remove / Save to File sit on the row under it (ui_tabs.rs cover_panels). Checked in app/shots/dark-cover.png. mechanics.json values are unchanged.

-52. The notification box (toast, ui_layout.rs toast) sizes to its text and sits above the status bar (TOAST_ABOVE_STATUS), instead of over it. Checked in app/shots/round-43b.png.
-53. Removed the "Rename variables" legend from the Rename Files tab, the "File name variables" legend from the Get Tags from File Name tab, and the note in the Multi File Tag Editor tab (ui_tabs.rs).
-54. Folder / File cover art buttons (Delete, Set from file art, Remove, Save to File) are 8 pt (COVER_BUTTON_PT).
-55. Cover art is larger: the two frames are 194 px wide (176 px picture), so the boxes reach down to the line under the tools bar (COVERS_COLUMN_WIDTH 396). Checked in app/shots/round-43.png.
-56. mechanics.json and MECHANICS.md are unchanged (layout and text only). rebuild_check and the Craft Anything question tools were not in this session.

# Latest pass: Appearance window, ten themes, rounded window, Inter font

-44. View > Theme is replaced by View > Appearance. It opens a window with two sections: Colour scheme (System, Light, Dark) and Theme (Default, Everforest, Gruvbox, Nord, Tokyo Night, Dracula, Solarized, GitHub, High Contrast, Spotty). Each theme has a light and a dark version; the choice is saved to ~/.config/tagrename/appearance.txt as "<theme> <scheme>" (ui_dialogs.rs appearance_window, theme.rs).
-45. Colours for the seven named themes were sampled from the person's screenshots (attachments/picture-43 to 58) and checked against the usual palettes for those themes. Default is the Aurral theme (black and white); High Contrast and Spotty keep their old dark colours and gain a light version. Catppuccin, Rosé Pine and Flexoki (in the screenshots, not on the list) are not included. White, Midnight, Sepia and Modern Dark are gone.
-46. The window is borderless and see-through outside its rounded corners (main.rs with_decorations(false), with_transparent(true), clear_color). The app draws its own 30 px title bar (drag to move, double-click to maximise, minimise / maximise / close buttons) and resize zones on the edges and corners (ui_layout.rs window_chrome, window_edge). The status bar has rounded bottom corners. The corners stay rounded even when the tiling desktop reports the window as maximised.
-47. Windows use a 12 px corner radius: the Tag Editor frame and its title strip (ui_tageditor.rs), the dialogs (egui window radius), the notification box.
-48. The app font is Inter (app/fonts, SIL Open Font License, OFL-Inter-LICENSE.txt), used for all text including the Monospace family (theme.rs install_fonts).
-49. Colour scheme System follows the operating system's light or dark setting when it reports one; otherwise it uses dark.
-50. Screenshot knobs: TAGRENAME_THEME=<index> and TAGRENAME_SCHEME=<0 System, 1 Light, 2 Dark>; TAGRENAME_APPEARANCE=1 opens the Appearance window. Checked with app/shots/appearance-dark.png, appearance-light.png, now.png and editor-rounded.png; corner pixels are transparent.
-51. mechanics.json and MECHANICS.md are unchanged: this pass changes only the look. rebuild_check and the Craft Anything evidence and question tools were not in this session, so nothing was checked through them.

# Latest pass: track numbers keep their leading zero

-41. "01" track numbers no longer turn into "1" on save. lofty stores the track as a number, so an MP3 saved as "03" read back as "3". After the normal save, save_fields (data.rs) now writes the ID3v2 TRCK frame as text when the track has a leading zero (write_id3v2_track_text). Test saved_track_keeps_its_leading_zero_in_mp3 passes; cargo test 28/28.
-42. Dark mode is the start-up theme (Modern Dark). Screenshot app/shots/dark-track.png checked. It has no files loaded, so the track column was not seen on screen; the fix is covered by the test.
-43. mechanics.json and MECHANICS.md are unchanged. rebuild_check was not available in this session.

# Latest pass: Format Track # rules from the person's guide

-38. Tools > Format Track # now follows the person's guide (attachments/picture-39.png): x gives a plain number ("01" -> "1"), 0x two digits ("1" -> "01"), 00x three digits ("1" -> "001"), xx/xx "01/01" with the checked-file count, and xx two digits with the total removed. x, 0x and 00x keep the total (tools.rs apply, pad_number). Unit test format_track_rules_follow_the_guide passes. mechanics.json "track formats" now cites the guide (estimated: true).
-39. Screenshot check: app/shots/now.png shows the dark-mode Tag:Rename window (not the original).
-40. Not run this pass: rebuild_check and the Craft Anything evidence tools were not in this session. The Format Track menu itself was not opened in a screenshot.

# Latest pass: Windows packaging path

-34. Windows packaging: app/package-windows.bat builds the release program and writes dist/Tag-Rename-1.0.0-windows-x64.zip (program, package/README-windows.txt). It has to run on a Windows PC with Rust and the Visual Studio build tools.
-35. Windows compile fix: the Play File fallback in ui.rs (play_file) now opens files with the Windows shell and has an explicit type for other platforms; the Windows target compiled past it.
-36. Cross-compiling from this Linux sandbox did not produce a .exe: the Windows standard library installed, but the final link failed (raw-dylib system libraries not found through the zig linker, and no mingw dlltool). Not solved; the .bat on Windows is the working route.
-37. Linux build passes (app/build.sh), screenshot run saved app/shots/now.png. mechanics.json is unchanged.

# Earlier pass: modern dark theme, faster Open, deployable package

-29. Modern Dark is the default theme (theme.rs). Darker dialog and panel surfaces, a softer accent and rounder corners. The old "Dark" palette is renamed; View > Theme keeps the same six entries.
-30. Open reads the folder on a background thread (ui.rs load_folder, scan_folder_rows). The list fills when the scan lands. Measured: window-thread wait 32 ms to 0 ms on a 1,200-file folder. Tests now wait for the scan (ui.rs and ui_layout.rs settle).
-31. Release build tuned (Cargo.toml): level 3, thin LTO, one codegen unit, stripped. Debug builds use level 1 with optimised dependencies. A cold release build takes about 5.5 minutes.
-32. Deployable package: app/package.sh writes app/dist/tagrename-1.0.0-linux-x86_64.tar.gz with a checksum. It holds the program, icon, menu entry, install.sh (per user, --uninstall available) and README.txt (package/).
-33. MECHANICS.md and mechanics.json are unchanged: the decompile tools were not in this session, so nothing was re-read (see MECHANICS.md, Pass 12). rebuild_check was not available either.

# Earlier pass: full-screen Tag Editor alignment

-27. Full-screen Tag Editor lines up on the right. The window's own frame was 21 px narrower and 6 px shorter than the screen, so the top bar, fields, Art box and bottom row stopped at different edges. Full screen is now an area that covers the whole app (ui_tageditor.rs show); windowed keeps the window and measures the gap between its layout area and its drawn frame (TagEditor.slack). The body's fields and Art box are placed in exact rectangles (body), so the Art box, the File box, Clear All, the divider and the bottom row all end on the same right edge, with a 12 px margin on each side. The top bar runs edge to edge with its caption at the same 12 px margin. The bottom row keeps its 24 px clear of the resize grip only when windowed.
-28. Checked with screenshots: app/shots/full-after4.png (General), full-tab1.png (MetaBrainz), full-tab4.png (Lyrics), multi-full.png (multi-file), win-dbg2.png (windowed). Screenshot knob: TAGRENAME_EDITOR_FULL=1 opens the editor full screen. mechanics.json and MECHANICS.md are unchanged (layout only). The decompiler and rebuild check tools were not available in this session, so rebuild_check was not run.

# Earlier pass: dark start, Tag Editor top bar and bottom row

-23. The app starts in a dark palette: a saved light palette (theme.txt, Sepia) is no longer applied at start; View > Theme still offers all six (main.rs).
-24. Tag Editor top bar ends on the window's drawn right edge, aligned with the body, so the close and full-screen buttons are no longer 11 px past it (ui_tageditor.rs show).
-25. Tag Editor bottom buttons: a 40 px band with the buttons centred in it, and the right-hand group pulled 24 px in so Save and Next is clear of the resize grip (ui_tageditor.rs body). Checked with app/shots/now-r34.png.
-26. mechanics.json is unchanged; the decompile tools were not in this session, so no function was re-read (see MECHANICS.md, Pass 14).

# Earlier pass: Tag Editor bar, bottom buttons, speed

-17. Tag Editor speed: the picture boxes (Tag Editor Art, Folder cover art, File cover art) no longer re-read and fully decode the picture on every redraw. Each picture is decoded once into a texture; the size shown under it comes from that texture (ui_tageditor.rs art_panel, ui_tabs.rs cover_panels). Each texture is keyed by its data, so a newly added or replaced picture shows at once (before, the editor kept the first picture it drew for a file). Checked with a --screenshot run on a test album with folder.jpg (app/shots/now.png).
-18. Tag Editor top bar: runs edge to edge across the window; the caption and the full-screen and close buttons sit at the drawn right edge (ui_tageditor.rs show, RIGHT_OVERHANG).
-19. Tag Editor bottom buttons: both groups sit in one fixed 30 px row, so the left and right buttons share a centre line (ui_tageditor.rs body). The right group is pulled in from the window edge to match the left (ROW_RIGHT_GAP). The body has a 12 px side inset on both sides (BODY_INSET). The Lyrics box stops above the button row (lyrics_tab).
-20. The window's layout area runs about 9 px past the frame it draws (measured in the screenshot log). The fixed RIGHT_OVERHANG accounts for it. Measuring the frame each frame was tried and made the window shrink, so it was dropped.
-21. mechanics.json and MECHANICS.md are unchanged: this pass changes layout and speed only. The decompiler and rebuild check tools were not available in this session, so no function was re-decompiled and rebuild_check was not run.
-22. Not done yet: the file list still sorts and filters every row on each redraw (ui.rs visible_rows). It is a smaller cost than the picture decoding but grows with the library size.

# Earlier pass: cover art, Case options, Tag Editor bar

-13. Cover art shows again. The image decoder was built without JPEG support, so the Folder and File cover boxes and the Tag Editor's Art box showed the size text but no picture. Cargo.toml now enables the JPEG, GIF and BMP decoders as well as PNG. Checked with a screenshot of a test album with a folder.jpg and embedded JPEG art (app/shots/v2-editor.png, app/shots/v2-main.png).
-14. Rename Files, Multi File Tag Editor and Get Tags from File Name: the "Case:" options now sit under the checkbox, in two columns (lower / UPPER, then Capitalize First Letter / Capitalize first word), as in the original's Case box. They no longer run to the right, so the cover art stays on screen. Checked with TAGRENAME_CASE=1 (app/shots/v3-case.png). Screenshot knob: TAGRENAME_CASE=1 ticks "Case:".
-15. Tag Editor top bar: the bar now has the same 8 px side inset as the fields below it, so its ends, the caption and the X line up with the body instead of running 8 px past it (ui_tageditor.rs show).
-16. mechanics.json and MECHANICS.md are unchanged: none of these changes touch a game-like value from the original, and no function was re-decompiled this pass (the decompiler tools were not available in this session).

-9. File list columns can be changed: drag a header's right edge to resize a column (30 px minimum), drag a header to move it to a new place (a blue line shows where it lands), and the list scrolls sideways when the columns are wider than the window. View > Columns lists all twelve columns with a tick for each shown one (the last shown column cannot be hidden), and Reset Columns restores the default order, widths and visibility. The layout is saved to ~/.config/tagrename/columns.txt (ui.rs load_columns/save_columns; ui_layout.rs file_list_body). Checked with a --screenshot run (app/shots/cols-2.png); the drag, resize and menu clicks were not driven by a test, so they are unverified in a live window. mechanics.json is unchanged: the original's default column set and order are not in the binary (see MECHANICS.md, File list columns).

# What the latest pass changed

-12. Tag Editor top bar matches the theme: the bar was a fixed blue (the old Windows caption colour) and now uses the active palette's panel colour, with the caption, window icon and X in the theme's text colour, a hover tint from the palette and a thin separator line under the bar (ui_tageditor.rs show). Built, and checked with a screenshot run that opens the editor on a folder (app/shots/topbar-editor.png). The drag, edge resize and icon click are still not driven by a test.

-11. Tag Editor top bar: holding and dragging the bar moves the window (ui_tageditor.rs, the bar's drag handle and TagEditor.pos; the window is pinned there while dragged and stays where it was dropped). The caption and the full-screen and close buttons are centred on the bar's 26 px line, and the close cross is drawn as two lines, so nothing sits high or low (checked with app/shots/after-bar.png; the drag itself was not driven by a test). The View > Theme choice is saved to ~/.config/tagrename/theme.txt and read back at the next start (theme.rs saved_index / save_choice, unit test the_chosen_theme_is_remembered_for_the_next_start). Screenshot runs keep the Dark default. mechanics.json is unchanged; no function was re-decompiled this pass.

-10. Tag Editor window (single and multi-file): it can be moved (drag its body) and resized from any edge, with a minimum size of 560 x 440. The top bar has a window icon that toggles full screen and back, next to the X. Each opening is centred in the app at the size the window was last left at (ui_tageditor.rs show, Dialogs.editor_size in ui_dialogs.rs). Built and checked with a --screenshot run (app/shots/resize-check.png); dragging, edge resizing and the icon click were not driven by a test.

-8. Cover art boxes keep one size: the Folder and File cover frames now reserve a fixed-height body (picture height + 100 px, ui_tabs.rs, cover_panels), so a box with a picture, a caption and buttons no longer grows taller than an empty one. Checked with --screenshot runs of an empty pair and of a folder with a picture (app/shots/empty-both.png, folder-filled.png). The Dark theme is still the default. mechanics.json is unchanged.

-7. Tag Editor, MetaBrainz, ReplayGain and Sorting tabs: every field is in one single column, each with its label, the Art box stays on the right (ui_tageditor.rs, the `matches!(ed.tab, 1 | 2 | 5)` branch). Checked with TAGRENAME_EDITOR_TAB=1 and 5 screenshots (app/shots/tab1-single.png, tab5-single.png). Test files were copies in /tmp/opencode/tagtest. mechanics.json is unchanged.
-6. Folder cover art card on the main screen: the red "folder.jpg" name under the picture is gone, and the placeholder no longer says "(folder.jpg)". The card now shows the picture's size and format under it ("1200 x 1200, JPEG", or PNG), the same line the File cover art card shows, with the same button row below (ui_tabs.rs, cover_panels). Checked with TAGRENAME_FOLDER and TAGRENAME_NO_EDITOR=1 screenshots (app/shots/folder-now.png). The Dark theme is still the default. mechanics.json is unchanged.

-5. Tag Editor fills the screen: it spans from the left edge to the right edge and down to the bottom of the window (no gap, no status bar strip). Its padding sits inside that area, the bottom buttons sit on the bottom edge, and a long Comment scrolls inside a fixed-height box instead of pushing the fields down (ui_tageditor.rs).
-5. MusicBrainz and ReplayGain tabs: all four fields of each section are in one row, each with its label, the Art box on the right (ui_tageditor.rs).
-5. Scale: the whole interface is drawn at 80% of the previous size (theme.rs, UI_ZOOM = 0.8). The "Rename variables" and the tab notes stay at 9 pt on screen (ui_tabs.rs, LEGEND_PT = 9 / UI_ZOOM). The mechanics sheet is unchanged; this pass changed layout only, no function was re-decompiled.

-4. Variables text is 9 pt on all three tabs: the "Rename variables:" and "File name variables:" headings, the variable lists and the notes all use one size, `LEGEND_PT` = 9.0 (ui_tabs.rs). The Multi File Tag Editor note, which was 19 pt, now uses the same 9 pt. Checked with the Rename Files, Multi File Tag Editor and Get Tags from File Name screenshots. mechanics.json values are unchanged.

-3. (Superseded by -4) Rename and File name variables text is back to the original's size: the "Rename variables:" and "File name variables:" headings are 11 pt, the variable lists 8.5 pt and the notes 9.5 pt (ui_tabs.rs). The variable lines sit about 10 px apart, as in the original. Checked with the Rename Files and Get Tags from File Name screenshots. mechanics.json values are unchanged.

-2. (Superseded by -3) Tools bar text was made twice as large: the "Rename variables:" and "File name variables:" headings are 22 pt (were 11), the variable lists 17 pt (were 8.5) and the notes on all three tabs 19 pt (were 9.5). The Rename Files variables sit in a grid of three columns that splits into two rows; the File name tab keeps its three columns on one row. Checked with TAGRENAME_TAB=0/1/2 screenshots.

-1. Folder tree: folder names have no box any more; each one has a small folder icon after the expanding arrow (the current folder is shown in the accent colour).
-1. Main screen cover art: the Folder and File cover pictures are 25% larger (130 px to 162 px). The two cover frames fill their column exactly, so the right frame sits a fixed small margin from the window edge, and the row sits a little lower.
-1. Tag Editor Art box: the box is square and the picture keeps its proportions and fills the box, so it shows larger instead of small in a wide box.
-1. Not re-decompiled this pass: these are layout changes only; mechanics.json values are unchanged.

0. The Dark theme is a bit darker (backgrounds, panels, dialog and widget greys lowered). A new "Green Night" theme (black and dark grey surfaces, green accent, white and grey text) is the sixth entry in View > Theme. Screenshot knob: TAGRENAME_THEME=5 picks it.
1. The whole interface is drawn at 80% of its normal size: fonts, spacing and panels all shrink by 20% (theme.rs, UI_ZOOM = 0.8).
2. View > Theme offers five palettes: Dark (default), White, Midnight, Sepia and High Contrast. The choice applies at once. Every colour now comes from the active palette in theme.rs.
3. Screenshot-only knob: TAGRENAME_THEME=<index> picks a palette for a check (0 Dark, 1 White, 2 Midnight, 3 Sepia, 4 High Contrast).


1. Tag Editor (single and multi-file): empty fields are blank. The grey "unchanged" hint is gone from every field, and the "e.g. 5/12" hint is gone from Track # and Disc #. The multi-file help line now says "Unchecked fields of tag are left as they are."
2. Speed: cover art is shared instead of copied, so the cover panels and the Tag Editor no longer copy whole pictures on every frame.
3. Speed: the file list sorts with precomputed keys instead of rebuilding them for every comparison.
4. Speed: debug builds use optimisation level 1; release builds use level 3.
5. Not done yet: opening a large folder still reads every tag and cover on the window thread, so very big folders can pause while loading. Moving that to a background thread is the next step.
6. Not re-checked this pass: the function-level notes in MECHANICS.md were not re-decompiled (the decompiler tools were not available in this session).

# What the earlier pass fixed

1. The folder tree keeps the width you drag it to, also after adding a root folder. The divider is a drag handle on its right edge.
2. The Tag Editor stays attached to the bottom of the window (just above the status bar) when the window is resized.
3. The Folder cover art box finds Folder.jpg / Folder.png in any letter case, and follows the album row you select in the list.
4. In the multi-file Tag Editor, each field and the Art box has a tick box, all off by default. Save writes only the ticked fields.
5. Verification knobs (screenshot mode only): TAGRENAME_MULTI=1 opens the multi-file editor with three rows checked.

## Round: text size, Mask Editor, name previews (2026-11-23)

- Text: the 80% interface zoom is gone (`theme::UI_ZOOM` = 1.0) and every
  font size was raised; the variables legend was 8.5–9.5 pt and is now 13 pt.
- Covers column is 300 pt wide and capped in `cover_panels`, so both cover
  boxes fit at the default window size.
- Mask Editor: the window was opened with a close flag that hid it at once.
  It now opens with the variables, saved masks (session only), a live result
  example and OK / Cancel (Cancel and the window cross restore the mask).
  Variable labels read like the original's, e.g. "Artist (%1)".
- Result preview (Rename tab, Rename preview dialog) and File name preview
  (Get Tags tab, its preview dialog) show the whole new name with the file's
  extension, using the focused file, else the first checked file, else the
  first file. Without files they show the mask with names, e.g.
  "<Artist> - <Title>.mp3".
- Screenshot knob: TAGRENAME_MASK=0|1 opens the Mask Editor for a picture.
- Open: the Mask Editor's saved masks are not saved to disk yet.

## Round: folder tree defaults and removable roots (2026-11-25)

- Linux tree defaults are now just `/` and the home folder; `~/Music` is no
  longer added unless the person adds it as a location.
- A manually added root can be removed by right-clicking it and choosing
  "Remove from tree"; default roots and built-in places do not offer this.
- Right-clicking folder-tree rows and file-list rows no longer offers
  "Open with".

## Round: shell "Open with Tag:Rename" (2026-11-25)

- The Windows installer now registers a per-user "Open with Tag:Rename"
  verb for folders and 22 music extensions, including .spx, .aifc and .mp+.
- The Linux desktop entry advertises the same action to file managers for
  music files and folders.
- Launching the app with a file path loads that file's whole folder and
  checks/scrolls to the file, so the rest of the folder stays visible.

# Tag:Rename dark build: behaviour sheet

Source of truth for this build. The original program's code could not be
decompiled in this session (the Craft Anything decompile tools were not
available), so every value below comes from the three reference pictures, the
program's release notes (WhatsNew.txt) and its menus. Anything not checked
against code is marked **estimated**. No function addresses are listed because
none were examined.

## What the original does
- Lists the audio files of the folder chosen in the folder tree (or Address bar).
- Rename Files: a file name mask built from variables (`%1` artist ... `%co`
  conductor), applied to selected files; a preview shows the new name; Rename
  applies it, Undo reverts it.
- Edit Tag: a Tag Editor window with fields for the selected file and Save,
  Save and Prev, Save and Next, Cancel.
- Multi-File Tag Editor: writes the same tag fields to every selected file.
- Shows file size and total time in the status bar.

## Values (see mechanics.json)
- Mask default `%1 - %2`, extension kept (picture-1).
- Variables %1 Artist, %2 Title, %3 Album, %4 Year, %5 Genre, %6 Track #,
  %7 File name, %8 File number, %9 Comment, %0 Bitrate, %t Play time, %b BPM,
  %# Disc #, %aa Album Artist, %co Conductor (picture-1).
- Case modes: None, Upper, Lower, Capitalize First Letter, Capitalize First Word
  (picture-1). The exact rule for the two capitalise modes is **estimated**.
- Time as mm:ss, bitrate as 320k, total size in MB (picture-2).
- Shortcuts F4, Ctrl+F4, F8, F9 (picture-1, picture-2 menus).

## New request: folder tree letter jump
- With the folder tree focused (click a folder), typing a letter or number moves
  to the first visible folder whose name starts with it (case-insensitive).
- Typing the same letter again moves to the next folder starting with it.
- Up/Down move between folders, Right/Left expand and collapse (**estimated**).

## Known gaps
- "Get Tags from Internet" tab is shown but disabled: not in this version.
- Mask Editor, Lyrics, and extra tag tabs are not built.
- Ratings, play-count and the Explorer shell integration are not built.
- Dark colours are a new palette; the exact original colours were not measured.
- The macOS window setting for --screenshot is written but was not compiled on a Mac.

//! "Open with" for the right-click menus of folders and music files.
//!
//! The applications offered are the ones the desktop has registered for
//! the file's type: their .desktop entries (in the user's and the system's
//! applications folders) whose MimeType list covers the type. The chosen
//! program is started directly, without a shell, with the path as its
//! argument, so folder and file names with spaces or quotes are safe.

use std::collections::HashSet;
use std::io;
use std::path::{Path, PathBuf};

use egui::Ui;

#[derive(Clone, Debug)]
pub struct OpenApp {
    pub name: String,
    pub exec: String,
}

/// The MIME type used to look up applications: a folder, or a music file
/// by its extension.
pub fn mime_for(path: &Path) -> &'static str {
    if path.is_dir() {
        return "inode/directory";
    }
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "mp3" | "mp2" => "audio/mpeg",
        "flac" => "audio/flac",
        "ogg" | "oga" | "spx" => "audio/ogg",
        "opus" => "audio/opus",
        "m4a" | "m4b" | "m4p" | "mp4" => "audio/mp4",
        "aac" => "audio/aac",
        "wav" => "audio/x-wav",
        "wma" | "asf" => "audio/x-ms-wma",
        "aif" | "aiff" | "aifc" => "audio/x-aiff",
        "ape" => "audio/x-ape",
        "wv" => "audio/x-wavpack",
        "mpc" | "mp+" => "audio/x-musepack",
        _ => "application/octet-stream",
    }
}

/// The folders that hold .desktop entries: the user's data folder first,
/// so its entries override the system ones with the same file name.
fn applications_dirs() -> Vec<PathBuf> {
    let mut data = Vec::new();
    let home_data = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")));
    data.extend(home_data);
    let system = std::env::var("XDG_DATA_DIRS").unwrap_or_else(|_| "/usr/local/share:/usr/share".into());
    data.extend(system.split(':').filter(|s| !s.is_empty()).map(PathBuf::from));
    data.into_iter().map(|d| d.join("applications")).collect()
}

/// Every application that can open this path, sorted by name.
pub fn apps_for(path: &Path) -> Vec<OpenApp> {
    let mime = mime_for(path);
    let mut seen: HashSet<String> = HashSet::new();
    let mut apps: Vec<OpenApp> = Vec::new();
    for dir in applications_dirs() {
        let Ok(rd) = std::fs::read_dir(&dir) else { continue };
        let mut entries: Vec<PathBuf> = rd
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == "desktop"))
            .collect();
        entries.sort();
        for entry in entries {
            let id = entry.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            if !seen.insert(id) {
                continue;
            }
            if let Some(app) = parse_desktop(&entry, mime) {
                apps.push(app);
            }
        }
    }
    apps.sort_by_key(|a| a.name.to_lowercase());
    apps
}

fn parse_desktop(path: &Path, mime: &str) -> Option<OpenApp> {
    let text = std::fs::read_to_string(path).ok()?;
    let mut in_entry = false;
    let mut name = None;
    let mut exec = None;
    let mut types = String::new();
    let mut kind = String::new();
    let mut hidden = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_entry = line == "[Desktop Entry]";
            continue;
        }
        if !in_entry {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else { continue };
        let value = value.trim();
        match key.trim() {
            "Name" if name.is_none() => name = Some(value.to_string()),
            "Exec" => exec = Some(value.to_string()),
            "MimeType" => types = value.to_string(),
            "Type" => kind = value.to_string(),
            "NoDisplay" | "Hidden" => hidden |= value == "true",
            _ => {}
        }
    }
    if hidden || kind != "Application" {
        return None;
    }
    let supported = types.split(';').map(str::trim).any(|t| {
        t == mime
            || t.strip_suffix("/*")
                .is_some_and(|prefix| mime.starts_with(&format!("{prefix}/")))
    });
    if !supported {
        return None;
    }
    Some(OpenApp { name: name?, exec: exec? })
}

/// Split a .desktop Exec line into program and arguments (quotes group
/// words; a backslash inside quotes escapes the next character).
fn split_exec(exec: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    let mut has_word = false;
    let mut chars = exec.chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => {
                in_quotes = !in_quotes;
                has_word = true;
            }
            '\\' if in_quotes => {
                if let Some(n) = chars.next() {
                    cur.push(n);
                }
            }
            c if c.is_whitespace() && !in_quotes => {
                if has_word {
                    out.push(std::mem::take(&mut cur));
                    has_word = false;
                }
            }
            c => {
                cur.push(c);
                has_word = true;
            }
        }
    }
    if has_word {
        out.push(cur);
    }
    out
}

/// Start an application on a path. The path replaces the %f/%F/%u/%U
/// field codes; deprecated codes are dropped; without a code the path is
/// appended as the last argument.
pub fn launch(app: &OpenApp, path: &Path) -> io::Result<()> {
    let words = split_exec(&app.exec);
    let Some((program, rest)) = words.split_first() else {
        return Err(io::Error::other("the application has no command"));
    };
    let target = path.display().to_string();
    let mut args: Vec<String> = Vec::new();
    let mut used_path = false;
    for word in rest {
        let mut arg = word.clone();
        for code in ["%f", "%F", "%u", "%U"] {
            if arg.contains(code) {
                arg = arg.replace(code, &target);
                used_path = true;
            }
        }
        for code in ["%i", "%c", "%k", "%d", "%D", "%n", "%N", "%v", "%m"] {
            arg = arg.replace(code, "");
        }
        if !arg.is_empty() {
            args.push(arg);
        }
    }
    if !used_path {
        args.push(target);
    }
    std::process::Command::new(program)
        .args(&args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map(|_| ())
}

/// Open with the desktop's default handler (xdg-open).
fn open_default(path: &Path) -> io::Result<()> {
    std::process::Command::new("xdg-open")
        .arg(path)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map(|_| ())
}

/// The "Open with" submenu for a right-click menu. Returns a message for
/// the status toast when an application was started or failed to start.
pub fn menu(ui: &mut Ui, path: &Path) -> Option<String> {
    let mut message = None;
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string());
    ui.menu_button("Open with", |ui| {
        if ui.button("System default").clicked() {
            message = Some(match open_default(path) {
                Ok(()) => format!("Opened {name} with the system default."),
                Err(e) => format!("Cannot open {name}: {e}"),
            });
            ui.close();
        }
        let apps = apps_for(path);
        if apps.is_empty() {
            ui.label("No other applications registered for this type");
        }
        ui.separator();
        for app in &apps {
            if ui.button(&app.name).clicked() {
                message = Some(match launch(app, path) {
                    Ok(()) => format!("Opened {name} with {}.", app.name),
                    Err(e) => format!("Cannot start {}: {e}", app.name),
                });
                ui.close();
            }
        }
    });
    message
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exec_line_keeps_quoted_words_together() {
        assert_eq!(
            split_exec(r#"/usr/bin/app --name "My App" %U"#),
            vec!["/usr/bin/app", "--name", "My App", "%U"]
        );
    }

    #[test]
    fn music_and_folders_get_their_mime_types() {
        assert_eq!(mime_for(Path::new("song.MP3")), "audio/mpeg");
        assert_eq!(mime_for(Path::new("song.flac")), "audio/flac");
        assert_eq!(mime_for(&std::env::temp_dir()), "inode/directory");
    }

    #[test]
    fn a_desktop_entry_is_matched_by_mime_type() {
        let dir = std::env::temp_dir().join(format!("tagrename_open_with_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("player.desktop");
        std::fs::write(
            &file,
            "[Desktop Entry]\nType=Application\nName=Test Player\nExec=test-player %U\nMimeType=audio/*;inode/x;\n",
        )
        .unwrap();
        let app = parse_desktop(&file, "audio/flac").expect("audio/* covers flac");
        assert_eq!(app.name, "Test Player");
        assert!(parse_desktop(&file, "inode/directory").is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }
}

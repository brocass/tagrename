//! Windows-only shell places for the folder tree.
//!
//! The tree on Linux shows real paths, but a Windows user expects the
//! familiar Desktop / Libraries / This PC / Network roots. Those are not
//! ordinary directories, so they are represented by reserved extended paths
//! and expanded by the helpers here. On other platforms every function is a
//! no-op, keeping the Linux tree exactly as it was.

use std::path::{Path, PathBuf};

const PREFIX: &str = r"\\?\tagrename\";
const DESKTOP: &str = "Desktop";
const LIBRARIES: &str = "Libraries";
const THIS_PC: &str = "This PC";
const NETWORK: &str = "Network";

fn place(name: &str) -> PathBuf {
    PathBuf::from(format!("{PREFIX}{name}"))
}

fn place_name(path: &Path) -> Option<String> {
    let text = path.to_string_lossy();
    text.strip_prefix(PREFIX).map(|s| s.to_string())
}

pub fn is_place(path: &Path) -> bool {
    place_name(path).is_some()
}

/// The roots shown on Windows when the person has not added their own.
pub fn initial_roots() -> Vec<PathBuf> {
    #[cfg(windows)]
    {
        vec![
            desktop_dir(),
            place(LIBRARIES),
            place(THIS_PC),
            place(NETWORK),
        ]
        .into_iter()
        .filter(|p| p.is_dir() || is_place(p))
        .collect()
    }
    #[cfg(not(windows))]
    {
        let _ = (DESKTOP, LIBRARIES, THIS_PC, NETWORK);
        Vec::new()
    }
}

/// Display name for a tree row. `fallback` is the ordinary file name.
pub fn display_name(path: &Path, fallback: String) -> String {
    if let Some(name) = place_name(path) {
        return name;
    }
    #[cfg(windows)]
    {
        // Show drive roots the way Explorer does: "Local Disk (C:)".
        let text = path.to_string_lossy();
        if text.len() == 3 && text.as_bytes()[1] == b':' && text.ends_with('\\') {
            return format!("Local Disk ({})", &text[..2]);
        }
    }
    fallback
}

/// Whether the tree row can be expanded. `None` means "not a shell place".
pub fn has_children(path: &Path) -> Option<bool> {
    let name = place_name(path)?;
    Some(match name.as_str() {
        DESKTOP => desktop_dir().is_dir(),
        LIBRARIES | THIS_PC | NETWORK => true,
        _ => false,
    })
}

/// Child rows for a shell place. `None` means "not a shell place".
pub fn children(path: &Path) -> Option<Vec<PathBuf>> {
    let name = place_name(path)?;
    let mut out = match name.as_str() {
        DESKTOP => Vec::new(),
        LIBRARIES => library_folders(),
        THIS_PC => drives(),
        NETWORK => network_servers(),
        _ => Vec::new(),
    };
    out.sort();
    out.dedup();
    Some(out)
}

/// The real folders a set of tree roots stands for. This lets the tree tell
/// whether an opened folder is already visible under one of its roots, so it
/// does not add a second ancestor chain for it.
pub fn expand_roots(paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    for path in paths {
        if is_place(path) {
            let children = children(path).unwrap_or_default();
            if children.is_empty() {
                out.push(resolve(path));
            } else {
                out.extend(children);
            }
        } else {
            out.push(path.clone());
        }
    }
    out.sort();
    out.dedup();
    out
}

/// A shell place that stands for a real folder resolves to that folder when
/// the person clicks it. Virtual places expand instead of loading music.
pub fn resolve(path: &Path) -> PathBuf {
    if place_name(path).as_deref() == Some(DESKTOP) {
        return desktop_dir();
    }
    path.to_path_buf()
}

#[cfg(windows)]
fn profile_dir() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOMEPATH"))
        .map(PathBuf::from)
}

#[cfg(windows)]
fn desktop_dir() -> PathBuf {
    profile_dir()
        .map(|p| p.join("Desktop"))
        .unwrap_or_else(|| PathBuf::from(r"C:\Users\steven\Desktop"))
}

#[cfg(not(windows))]
fn desktop_dir() -> PathBuf {
    PathBuf::new()
}

#[cfg(windows)]
fn library_folders() -> Vec<PathBuf> {
    let Some(profile) = profile_dir() else { return Vec::new() };
    ["Documents", "Music", "Pictures", "Videos"]
        .iter()
        .map(|n| profile.join(n))
        .filter(|p| p.is_dir())
        .collect()
}

#[cfg(windows)]
fn drives() -> Vec<PathBuf> {
    ('A'..='Z')
        .map(|c| PathBuf::from(format!("{}:\\", c)))
        .filter(|p| p.exists())
        .collect()
}

#[cfg(windows)]
fn network_servers() -> Vec<PathBuf> {
    let Ok(rd) = std::fs::read_dir(r"\\") else { return Vec::new() };
    rd.flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect()
}

#[cfg(not(windows))]
fn library_folders() -> Vec<PathBuf> {
    Vec::new()
}

#[cfg(not(windows))]
fn drives() -> Vec<PathBuf> {
    Vec::new()
}

#[cfg(not(windows))]
fn network_servers() -> Vec<PathBuf> {
    Vec::new()
}

//! OS-native file and folder pickers.
//!
//! Linux uses the XDG Desktop Portal (so the desktop's own GTK/KDE file
//! chooser appears) and Windows uses the native common item dialog. The
//! dialog runs on its own thread because `rfd`'s synchronous API blocks
//! until the person closes the dialog; the egui frame loop polls for the
//! result instead of freezing.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, TryRecvError};

pub type Filters = Vec<(String, Vec<String>)>;

#[derive(Clone)]
pub enum Request {
    Folder { dir: PathBuf },
    OpenFile { dir: PathBuf, filters: Filters },
    SaveFile { dir: PathBuf, name: String, filters: Filters },
}

fn start_dir(dir: &Path) -> PathBuf {
    if dir.is_dir() {
        dir.to_path_buf()
    } else {
        dir.parent()
            .filter(|p| p.is_dir())
            .map(Path::to_path_buf)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")))
    }
}

impl Request {
    fn run(self) -> Option<PathBuf> {
        match self {
            Request::Folder { dir } => {
                rfd::FileDialog::new().set_directory(start_dir(&dir)).pick_folder()
            }
            Request::OpenFile { dir, filters } => {
                let mut dialog = rfd::FileDialog::new().set_directory(start_dir(&dir));
                for (name, extensions) in filters {
                    dialog = dialog.add_filter(name, &extensions);
                }
                dialog.pick_file()
            }
            Request::SaveFile { dir, name, filters } => {
                let mut dialog = rfd::FileDialog::new()
                    .set_directory(start_dir(&dir))
                    .set_file_name(name);
                for (name, extensions) in filters {
                    dialog = dialog.add_filter(name, &extensions);
                }
                dialog.save_file()
            }
        }
    }
}

/// The one picker allowed to be open at a time, plus its result channel.
#[derive(Default)]
pub struct State {
    rx: Option<Receiver<Option<PathBuf>>>,
}

impl State {
    pub fn busy(&self) -> bool {
        self.rx.is_some()
    }

    /// Start a native dialog. Returns false when one is already open.
    pub fn request(&mut self, request: Request) -> bool {
        if self.rx.is_some() {
            return false;
        }
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(request.run());
        });
        self.rx = Some(rx);
        true
    }

    /// Take a completed result. `Some(None)` means the person cancelled.
    pub fn poll(&mut self) -> Option<Option<PathBuf>> {
        let Some(rx) = self.rx.as_ref() else { return None };
        let result = match rx.try_recv() {
            Ok(path) => Some(path),
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => Some(None),
        };
        self.rx = None;
        result
    }
}

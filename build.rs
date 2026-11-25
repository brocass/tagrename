fn main() {
    #[cfg(target_os = "windows")]
    {
        // Embed the icon into Tag-Rename.exe itself, so Explorer, the taskbar
        // and the Start menu show it even when the ZIP is copied elsewhere.
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/tagrename.ico");
        if let Err(err) = res.compile() {
            eprintln!("warning: could not embed the Windows icon: {err}");
        }
    }
}

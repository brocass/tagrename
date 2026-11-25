Tag:Rename 1.0.1 - modern dark edition (Linux, 64-bit)
======================================================

What is in this folder
  tagrename          the program
  icons/             the app icon in several sizes
  tagrename.desktop  the app-menu entry
  install.sh         installs the program for your user account
  README.txt         this file

Install
  1. Open a terminal in this folder.
  2. Run:  ./install.sh
  3. Start "Tag:Rename" from your app menu, or type  tagrename  in a terminal.

Remove
  Run:  ./install.sh --uninstall
  Your settings (in ~/.config/tagrename) are kept.

Notes
  - No administrator rights are needed; everything goes into ~/.local.
  - The program needs a desktop session (Wayland or X11) to open its window.
  - Use View > Appearance to pick a colour scheme (System, Light or Dark) and a theme. Dark is the starting scheme.
  - Right-click a music file (or folder) in your file manager and choose
    Tag:Rename under "Open With" to load that file's whole folder.
  - On Arch-based systems (such as Omarchy) there is also a package file, tagrename-1.0.1-1-x86_64.pkg.tar.zst,
    installed with:  sudo pacman -U tagrename-1.0.1-1-x86_64.pkg.tar.zst

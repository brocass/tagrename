#define MyAppName "Tag:Rename"
#define MyAppVersion "1.0.0"
#define MyAppPublisher "Steven Davis"
#define MyAppExeName "tagrename.exe"

[Setup]
AppId={{8C5A4F4E-3B2C-4A6D-9E1F-2D3B4C5A6E7F}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
DefaultDirName={userpf}\Tag-Rename
DefaultGroupName=Tag-Rename
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
UninstallDisplayIcon={app}\{#MyAppExeName}
WizardStyle=modern
Compression=lzma2
SolidCompression=yes
OutputDir=dist
OutputBaseFilename=Tag-Rename-1.0.0-setup

[Files]
Source: "target\release\{#MyAppExeName}"; DestDir: "{app}"; DestName: "Tag-Rename.exe"; Flags: ignoreversion
Source: "package\README-windows.txt"; DestDir: "{app}"; DestName: "README.txt"; Flags: ignoreversion

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional shortcuts:"

[Icons]
Name: "{group}\Tag-Rename"; Filename: "{app}\Tag-Rename.exe"
Name: "{autodesktop}\Tag-Rename"; Filename: "{app}\Tag-Rename.exe"; Tasks: desktopicon

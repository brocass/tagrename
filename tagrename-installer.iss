#define MyAppName "Tag:Rename"
#define MyAppVersion "1.1.2"
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
SetupIconFile=assets\tagrename.ico
UninstallDisplayIcon={app}\tagrename.ico
WizardStyle=modern
Compression=lzma2
SolidCompression=yes
OutputDir=dist
OutputBaseFilename=Tag-Rename-1.1.2-setup

[Files]
Source: "target\release\{#MyAppExeName}"; DestDir: "{app}"; DestName: "Tag-Rename.exe"; Flags: ignoreversion
Source: "package\README-windows.txt"; DestDir: "{app}"; DestName: "README.txt"; Flags: ignoreversion
Source: "assets\tagrename.ico"; DestDir: "{app}"; Flags: ignoreversion

[Registry]
; Per-user shell verbs: no administrator rights are needed.
Root: HKCU; Subkey: "Software\Classes\Directory\shell\TagRename"; ValueType: none; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\Directory\shell\TagRename"; ValueType: string; ValueName: ""; ValueData: "Open with Tag:Rename"
Root: HKCU; Subkey: "Software\Classes\Directory\shell\TagRename"; ValueType: string; ValueName: "Icon"; ValueData: "{app}\tagrename.ico"
Root: HKCU; Subkey: "Software\Classes\Directory\shell\TagRename\command"; ValueType: string; ValueName: ""; ValueData: """{app}\Tag-Rename.exe"" ""%1"""

Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.mp3\shell\TagRename"; ValueType: none; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.mp3\shell\TagRename"; ValueType: string; ValueName: ""; ValueData: "Open with Tag:Rename"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.mp3\shell\TagRename"; ValueType: string; ValueName: "Icon"; ValueData: "{app}\tagrename.ico"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.mp3\shell\TagRename\command"; ValueType: string; ValueName: ""; ValueData: """{app}\Tag-Rename.exe"" ""%1"""

Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.mp2\shell\TagRename"; ValueType: none; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.mp2\shell\TagRename"; ValueType: string; ValueName: ""; ValueData: "Open with Tag:Rename"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.mp2\shell\TagRename"; ValueType: string; ValueName: "Icon"; ValueData: "{app}\tagrename.ico"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.mp2\shell\TagRename\command"; ValueType: string; ValueName: ""; ValueData: """{app}\Tag-Rename.exe"" ""%1"""

Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.flac\shell\TagRename"; ValueType: none; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.flac\shell\TagRename"; ValueType: string; ValueName: ""; ValueData: "Open with Tag:Rename"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.flac\shell\TagRename"; ValueType: string; ValueName: "Icon"; ValueData: "{app}\tagrename.ico"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.flac\shell\TagRename\command"; ValueType: string; ValueName: ""; ValueData: """{app}\Tag-Rename.exe"" ""%1"""

Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.ogg\shell\TagRename"; ValueType: none; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.ogg\shell\TagRename"; ValueType: string; ValueName: ""; ValueData: "Open with Tag:Rename"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.ogg\shell\TagRename"; ValueType: string; ValueName: "Icon"; ValueData: "{app}\tagrename.ico"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.ogg\shell\TagRename\command"; ValueType: string; ValueName: ""; ValueData: """{app}\Tag-Rename.exe"" ""%1"""

Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.oga\shell\TagRename"; ValueType: none; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.oga\shell\TagRename"; ValueType: string; ValueName: ""; ValueData: "Open with Tag:Rename"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.oga\shell\TagRename"; ValueType: string; ValueName: "Icon"; ValueData: "{app}\tagrename.ico"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.oga\shell\TagRename\command"; ValueType: string; ValueName: ""; ValueData: """{app}\Tag-Rename.exe"" ""%1"""

Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.opus\shell\TagRename"; ValueType: none; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.opus\shell\TagRename"; ValueType: string; ValueName: ""; ValueData: "Open with Tag:Rename"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.opus\shell\TagRename"; ValueType: string; ValueName: "Icon"; ValueData: "{app}\tagrename.ico"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.opus\shell\TagRename\command"; ValueType: string; ValueName: ""; ValueData: """{app}\Tag-Rename.exe"" ""%1"""

Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.spx\shell\TagRename"; ValueType: none; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.spx\shell\TagRename"; ValueType: string; ValueName: ""; ValueData: "Open with Tag:Rename"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.spx\shell\TagRename"; ValueType: string; ValueName: "Icon"; ValueData: "{app}\tagrename.ico"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.spx\shell\TagRename\command"; ValueType: string; ValueName: ""; ValueData: """{app}\Tag-Rename.exe"" ""%1"""

Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.m4a\shell\TagRename"; ValueType: none; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.m4a\shell\TagRename"; ValueType: string; ValueName: ""; ValueData: "Open with Tag:Rename"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.m4a\shell\TagRename"; ValueType: string; ValueName: "Icon"; ValueData: "{app}\tagrename.ico"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.m4a\shell\TagRename\command"; ValueType: string; ValueName: ""; ValueData: """{app}\Tag-Rename.exe"" ""%1"""

Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.m4b\shell\TagRename"; ValueType: none; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.m4b\shell\TagRename"; ValueType: string; ValueName: ""; ValueData: "Open with Tag:Rename"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.m4b\shell\TagRename"; ValueType: string; ValueName: "Icon"; ValueData: "{app}\tagrename.ico"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.m4b\shell\TagRename\command"; ValueType: string; ValueName: ""; ValueData: """{app}\Tag-Rename.exe"" ""%1"""

Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.m4p\shell\TagRename"; ValueType: none; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.m4p\shell\TagRename"; ValueType: string; ValueName: ""; ValueData: "Open with Tag:Rename"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.m4p\shell\TagRename"; ValueType: string; ValueName: "Icon"; ValueData: "{app}\tagrename.ico"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.m4p\shell\TagRename\command"; ValueType: string; ValueName: ""; ValueData: """{app}\Tag-Rename.exe"" ""%1"""

Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.mp4\shell\TagRename"; ValueType: none; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.mp4\shell\TagRename"; ValueType: string; ValueName: ""; ValueData: "Open with Tag:Rename"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.mp4\shell\TagRename"; ValueType: string; ValueName: "Icon"; ValueData: "{app}\tagrename.ico"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.mp4\shell\TagRename\command"; ValueType: string; ValueName: ""; ValueData: """{app}\Tag-Rename.exe"" ""%1"""

Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.aac\shell\TagRename"; ValueType: none; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.aac\shell\TagRename"; ValueType: string; ValueName: ""; ValueData: "Open with Tag:Rename"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.aac\shell\TagRename"; ValueType: string; ValueName: "Icon"; ValueData: "{app}\tagrename.ico"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.aac\shell\TagRename\command"; ValueType: string; ValueName: ""; ValueData: """{app}\Tag-Rename.exe"" ""%1"""

Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.wav\shell\TagRename"; ValueType: none; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.wav\shell\TagRename"; ValueType: string; ValueName: ""; ValueData: "Open with Tag:Rename"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.wav\shell\TagRename"; ValueType: string; ValueName: "Icon"; ValueData: "{app}\tagrename.ico"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.wav\shell\TagRename\command"; ValueType: string; ValueName: ""; ValueData: """{app}\Tag-Rename.exe"" ""%1"""

Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.wma\shell\TagRename"; ValueType: none; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.wma\shell\TagRename"; ValueType: string; ValueName: ""; ValueData: "Open with Tag:Rename"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.wma\shell\TagRename"; ValueType: string; ValueName: "Icon"; ValueData: "{app}\tagrename.ico"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.wma\shell\TagRename\command"; ValueType: string; ValueName: ""; ValueData: """{app}\Tag-Rename.exe"" ""%1"""

Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.asf\shell\TagRename"; ValueType: none; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.asf\shell\TagRename"; ValueType: string; ValueName: ""; ValueData: "Open with Tag:Rename"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.asf\shell\TagRename"; ValueType: string; ValueName: "Icon"; ValueData: "{app}\tagrename.ico"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.asf\shell\TagRename\command"; ValueType: string; ValueName: ""; ValueData: """{app}\Tag-Rename.exe"" ""%1"""

Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.aif\shell\TagRename"; ValueType: none; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.aif\shell\TagRename"; ValueType: string; ValueName: ""; ValueData: "Open with Tag:Rename"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.aif\shell\TagRename"; ValueType: string; ValueName: "Icon"; ValueData: "{app}\tagrename.ico"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.aif\shell\TagRename\command"; ValueType: string; ValueName: ""; ValueData: """{app}\Tag-Rename.exe"" ""%1"""

Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.aiff\shell\TagRename"; ValueType: none; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.aiff\shell\TagRename"; ValueType: string; ValueName: ""; ValueData: "Open with Tag:Rename"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.aiff\shell\TagRename"; ValueType: string; ValueName: "Icon"; ValueData: "{app}\tagrename.ico"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.aiff\shell\TagRename\command"; ValueType: string; ValueName: ""; ValueData: """{app}\Tag-Rename.exe"" ""%1"""

Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.aifc\shell\TagRename"; ValueType: none; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.aifc\shell\TagRename"; ValueType: string; ValueName: ""; ValueData: "Open with Tag:Rename"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.aifc\shell\TagRename"; ValueType: string; ValueName: "Icon"; ValueData: "{app}\tagrename.ico"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.aifc\shell\TagRename\command"; ValueType: string; ValueName: ""; ValueData: """{app}\Tag-Rename.exe"" ""%1"""

Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.ape\shell\TagRename"; ValueType: none; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.ape\shell\TagRename"; ValueType: string; ValueName: ""; ValueData: "Open with Tag:Rename"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.ape\shell\TagRename"; ValueType: string; ValueName: "Icon"; ValueData: "{app}\tagrename.ico"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.ape\shell\TagRename\command"; ValueType: string; ValueName: ""; ValueData: """{app}\Tag-Rename.exe"" ""%1"""

Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.wv\shell\TagRename"; ValueType: none; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.wv\shell\TagRename"; ValueType: string; ValueName: ""; ValueData: "Open with Tag:Rename"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.wv\shell\TagRename"; ValueType: string; ValueName: "Icon"; ValueData: "{app}\tagrename.ico"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.wv\shell\TagRename\command"; ValueType: string; ValueName: ""; ValueData: """{app}\Tag-Rename.exe"" ""%1"""

Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.mpc\shell\TagRename"; ValueType: none; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.mpc\shell\TagRename"; ValueType: string; ValueName: ""; ValueData: "Open with Tag:Rename"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.mpc\shell\TagRename"; ValueType: string; ValueName: "Icon"; ValueData: "{app}\tagrename.ico"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.mpc\shell\TagRename\command"; ValueType: string; ValueName: ""; ValueData: """{app}\Tag-Rename.exe"" ""%1"""

Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.mp+\shell\TagRename"; ValueType: none; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.mp+\shell\TagRename"; ValueType: string; ValueName: ""; ValueData: "Open with Tag:Rename"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.mp+\shell\TagRename"; ValueType: string; ValueName: "Icon"; ValueData: "{app}\tagrename.ico"
Root: HKCU; Subkey: "Software\Classes\SystemFileAssociations\.mp+\shell\TagRename\command"; ValueType: string; ValueName: ""; ValueData: """{app}\Tag-Rename.exe"" ""%1"""

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional shortcuts:"

[Icons]
Name: "{group}\Tag-Rename"; Filename: "{app}\Tag-Rename.exe"; IconFilename: "{app}\tagrename.ico"
Name: "{autodesktop}\Tag-Rename"; Filename: "{app}\Tag-Rename.exe"; Tasks: desktopicon; IconFilename: "{app}\tagrename.ico"

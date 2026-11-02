; Rustcord's per-user installer for Windows, built
; with Inno Setup 6 by the release workflow:
;
;   ISCC /DVersion=0.1.0 packaging\windows\rustcord.iss
;
; (/DProgram=<rustcord.exe> names another build; /DLoader=<WebView2Loader.dll>
; adds the loader a MinGW build needs beside it, which MSVC links in). It
; installs into
; %LOCALAPPDATA%\Programs\Rustcord without administrator rights, so Rustcord can
; replace itself: its updater runs this installer with /VERYSILENT
; /SUPPRESSMSGBOXES /NORESTART /CLOSEAPPLICATIONS once its window has
; closed, and the installer starts Rustcord again. The person's data lives
; elsewhere (%LOCALAPPDATA%\Rustcord, %APPDATA%\Rustcord): updating and
; uninstalling never touch it.

#ifndef Version
  #error Name the version: ISCC /DVersion=x.y.z rustcord.iss
#endif
#ifndef Program
  #define Program "..\..\target\x86_64-pc-windows-msvc\release\rustcord.exe"
#endif

[Setup]
; Never change it: an update or the uninstaller finds the installed Rustcord by it.
AppId={{9A354FE5-8128-4442-8261-E11202381361}
AppName=Rustcord
AppVersion={#Version}
AppVerName=Rustcord {#Version}
UninstallDisplayName=Rustcord
UninstallDisplayIcon={app}\rustcord.exe
; For this person only, never elevated.
PrivilegesRequired=lowest
; The updater recognizes Rustcord only here (rustcord_update::install::Install).
DefaultDirName={localappdata}\Programs\Rustcord
UsePreviousAppDir=yes
DisableDirPage=yes
DisableProgramGroupPage=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
; Toasts and the rest of rustcord-platform's Windows services need Windows 10.
MinVersion=10.0
; A Rustcord still running is closed, also when silent: after an update it is
; already on its way out (the updater exits as it starts this); by hand,
; the wizard asks first. Forced, in case Rustcord does not quit when the
; Restart Manager asks. [Run] starts it again.
CloseApplications=force
RestartApplications=no
SetupIconFile=..\..\assets\icons\rustcord.ico
; Explorer is told to refresh what it shows once installed: an update
; with a new icon kept showing the old one on the Start menu's shortcut
; and the taskbar from Windows' icon cache.
ChangesAssociations=yes
WizardStyle=modern
Compression=lzma2/max
SolidCompression=yes
OutputDir=..\..\target
OutputBaseFilename=rustcord-x86_64-windows-setup
; The language of Windows when it is one of these, English otherwise.
ShowLanguageDialog=no
; A silent update shows nothing: what it did stays in
; %TEMP%\Setup Log <date> #<n>.txt.
SetupLogging=yes

[Languages]
Name: "en"; MessagesFile: "compiler:Default.isl"
Name: "es"; MessagesFile: "compiler:Languages\Spanish.isl"

[InstallDelete]
; The notices of the version before, which may have dropped one.
Type: filesandordirs; Name: "{app}\licenses"
; The Start menu shortcut under the app's old name.
Type: files; Name: "{autoprograms}\RUNE.lnk"

[Files]
; ignoreversion: a release always replaces the program, even one whose
; version information is the same (0.2.0-rc.1 and 0.2.0 are both 0.2.0.0).
Source: "{#Program}"; DestDir: "{app}"; DestName: "rustcord.exe"; Flags: ignoreversion
; Older startup entries and shortcuts may still launch rune.exe, so the program is
; installed under that name too.
Source: "{#Program}"; DestDir: "{app}"; DestName: "rune.exe"; Flags: ignoreversion
#ifdef Loader
Source: "{#Loader}"; DestDir: "{app}"; DestName: "WebView2Loader.dll"; Flags: ignoreversion
#endif
; The notices beside the program.
Source: "..\..\THIRD_PARTY_NOTICES.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\LICENSE-MIT"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\LICENSE-APACHE"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\assets\licenses\*"; DestDir: "{app}\licenses"; Flags: ignoreversion

[Icons]
; The application ID Rustcord's toasts are shown under (rustcord-platform's
; notifications): Windows wants a Start menu shortcut carrying it.
Name: "{autoprograms}\Rustcord"; Filename: "{app}\rustcord.exe"; AppUserModelID: "Rustcord.Desktop"

[Registry]
; What Rustcord itself writes as it runs, so that uninstalling leaves nothing
; behind: the toasts' name, and starting with the session.
Root: HKCU; Subkey: "Software\Classes\AppUserModelId\Rustcord.Desktop"; ValueType: string; ValueName: "DisplayName"; ValueData: "Rustcord"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: none; ValueName: "Rustcord"; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run"; ValueType: none; ValueName: "Rustcord"; Flags: uninsdeletevalue dontcreatekey

[Run]
; No skipifsilent: after an update the installer is what starts Rustcord again.
Filename: "{app}\rustcord.exe"; Description: "{cm:LaunchProgram,Rustcord}"; Flags: postinstall nowait

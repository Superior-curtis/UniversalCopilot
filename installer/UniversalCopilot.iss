; Inno Setup Script for UniversalCopilot + Ollama
; Requires Inno Setup 6+ (iscc.exe)

[Setup]
AppName=UniversalCopilot
AppVersion=1.0.0
AppPublisher=UniversalCopilot
DefaultDirName={pf}\UniversalCopilot
DefaultGroupName=UniversalCopilot
DisableDirPage=yes
DisableProgramGroupPage=yes
OutputDir=dist
OutputBaseFilename=install
Compression=lzma
SolidCompression=yes
ArchitecturesInstallIn64BitMode=x64
PrivilegesRequired=admin

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Files]
; Main application
Source: "..\\target\\release\\universal_copilot.exe"; DestDir: "{app}"; Flags: ignoreversion

; Optional bundled Ollama installer (place it at installer/ollama/OllamaSetup.exe)
Source: "ollama\\OllamaSetup.exe"; DestDir: "{tmp}"; Flags: deleteafterinstall; Check: OllamaInstallerExists

[Icons]
Name: "{group}\\UniversalCopilot"; Filename: "{app}\\universal_copilot.exe"; WorkingDir: "{app}"
Name: "{group}\\Uninstall UniversalCopilot"; Filename: "{uninstallexe}"

[Run]
; Install Ollama silently if bundled
Filename: "{tmp}\\OllamaSetup.exe"; Parameters: "/S"; StatusMsg: "Installing Ollama..."; Flags: runhidden waituntilterminated; Check: OllamaInstallerExists

; Try to pre-pull a default model (optional)
Filename: "{cmd}"; Parameters: "/c \"\"{code:GetOllamaExe}\" run mistral\""; StatusMsg: "Preparing Ollama model (mistral)..."; Flags: runhidden waituntilterminated; Check: OllamaExeExists

; Launch app after install (optional)
Filename: "{app}\\universal_copilot.exe"; Description: "Run UniversalCopilot"; Flags: nowait postinstall skipifsilent

[UninstallRun]
; Attempt to uninstall Ollama silently if found
Filename: "{cmd}"; Parameters: "/c \"\"{code:GetOllamaUninstall}\" /S\""; Flags: runhidden waituntilterminated; Check: OllamaUninstallerExists

[Code]
function OllamaInstallerExists(): Boolean;
begin
  Result := FileExists(ExpandConstant('{src}\\ollama\\OllamaSetup.exe'));
end;

function GetOllamaExe(Param: string): string;
var
  Path1, Path2: string;
begin
  { Common install paths }
  Path1 := ExpandConstant('{localappdata}\\Programs\\Ollama\\ollama.exe');
  Path2 := ExpandConstant('{pf}\\Ollama\\ollama.exe');
  if FileExists(Path1) then
    Result := Path1
  else if FileExists(Path2) then
    Result := Path2
  else
    Result := 'ollama'; { fallback to PATH }
end;

function OllamaExeExists(): Boolean;
begin
  Result := FileExists(GetOllamaExe(''));
end;

function GetOllamaUninstall(Param: string): string;
var
  Un1, Un2: string;
begin
  { Guessed uninstaller names/locations }
  Un1 := ExpandConstant('{localappdata}\\Programs\\Ollama\\Uninstall Ollama.exe');
  Un2 := ExpandConstant('{pf}\\Ollama\\Uninstall Ollama.exe');
  if FileExists(Un1) then
    Result := Un1
  else if FileExists(Un2) then
    Result := Un2
  else
    Result := '';
end;

function OllamaUninstallerExists(): Boolean;
var S: string;
begin
  S := GetOllamaUninstall('');
  Result := (S <> '') and FileExists(S);
end;

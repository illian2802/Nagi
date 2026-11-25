#define MyAppName "Nagi"
#define MyAppVersion "0.5.0"
#define MyAppPublisher "Nagi"
#define MyAppExeName "nagi-browser.exe"

[Setup]
AppId={{CC0B9F62-4F78-4D11-A6CA-2E4BB1980061}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
DefaultDirName={localappdata}\Programs\Nagi
DefaultGroupName=Nagi
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64
OutputDir=..
OutputBaseFilename=Nagi-Setup-0.5.0
SetupIconFile=assets\nagi.ico
UninstallDisplayIcon={app}\nagi.ico
WizardStyle=modern
Compression=lzma2/max
SolidCompression=yes
DisableWelcomePage=no
Uninstallable=yes
UseSetupLdr=no
ShowLanguageDialog=yes
LanguageDetectionMethod=none

[Languages]
Name: "en"; MessagesFile: "assets\Default.isl"
Name: "fr"; MessagesFile: "assets\French.isl"
Name: "es"; MessagesFile: "assets\Spanish.isl"
Name: "de"; MessagesFile: "assets\German.isl"
Name: "it"; MessagesFile: "assets\Italian.isl"
Name: "ptbr"; MessagesFile: "assets\BrazilianPortuguese.isl"
Name: "nl"; MessagesFile: "assets\Dutch.isl"
Name: "pl"; MessagesFile: "assets\Polish.isl"
Name: "tr"; MessagesFile: "assets\Turkish.isl"
Name: "ru"; MessagesFile: "assets\Russian.isl"
Name: "ja"; MessagesFile: "assets\Japanese.isl"
Name: "zhcn"; MessagesFile: "assets\ChineseSimplified.isl"
Name: "zhtw"; MessagesFile: "assets\ChineseTraditional.isl"
Name: "ar"; MessagesFile: "assets\Arabic.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:DesktopIconDesc}"; GroupDescription: "{cm:ShortcutGroup}"; Flags: unchecked

[Files]
Source: "..\nagi-browser.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "assets\nagi.ico"; DestDir: "{app}"; Flags: ignoreversion
Source: "assets\MicrosoftEdgeWebview2Setup.exe"; DestDir: "{tmp}"; Flags: deleteafterinstall
Source: "..\README.md"; DestDir: "{app}"; DestName: "Lisez-moi.txt"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\Nagi"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\nagi.ico"
Name: "{autodesktop}\Nagi"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\nagi.ico"; Tasks: desktopicon

[Run]
Filename: "{tmp}\MicrosoftEdgeWebview2Setup.exe"; Parameters: "/silent /install"; StatusMsg: "{cm:WebView2Status}"; Flags: runhidden waituntilterminated; Check: not IsWebView2Installed
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchNagi}"; Parameters: "--language={language}"; Flags: nowait postinstall skipifsilent

[CustomMessages]
fr.DesktopIconDesc=Créer un raccourci sur le bureau
fr.ShortcutGroup=Raccourcis :
fr.LaunchNagi=Lancer Nagi
fr.WebView2Status=Installation du composant WebView2...
en.DesktopIconDesc=Create a desktop shortcut
en.ShortcutGroup=Shortcuts:
en.LaunchNagi=Launch Nagi
en.WebView2Status=Installing the WebView2 component...
es.DesktopIconDesc=Crear un acceso directo en el escritorio
es.ShortcutGroup=Accesos directos:
es.LaunchNagi=Iniciar Nagi
es.WebView2Status=Instalando el componente WebView2...
de.DesktopIconDesc=Desktop-Verknüpfung erstellen
de.ShortcutGroup=Verknüpfungen:
de.LaunchNagi=Nagi starten
de.WebView2Status=WebView2-Komponente wird installiert...
it.DesktopIconDesc=Crea un collegamento sul desktop
it.ShortcutGroup=Collegamenti:
it.LaunchNagi=Avvia Nagi
it.WebView2Status=Installazione del componente WebView2...
ptbr.DesktopIconDesc=Criar atalho na área de trabalho
ptbr.ShortcutGroup=Atalhos:
ptbr.LaunchNagi=Iniciar Nagi
ptbr.WebView2Status=Instalando o componente WebView2...
nl.DesktopIconDesc=Snelkoppeling op het bureaublad maken
nl.ShortcutGroup=Snelkoppelingen:
nl.LaunchNagi=Nagi starten
nl.WebView2Status=WebView2-onderdeel installeren...
pl.DesktopIconDesc=Utwórz skrót na pulpicie
pl.ShortcutGroup=Skróty:
pl.LaunchNagi=Uruchom Nagi
pl.WebView2Status=Instalowanie składnika WebView2...
tr.DesktopIconDesc=Masaüstü kısayolu oluştur
tr.ShortcutGroup=Kısayollar:
tr.LaunchNagi=Nagi'yi başlat
tr.WebView2Status=WebView2 bileşeni yükleniyor...
ru.DesktopIconDesc=Создать ярлык на рабочем столе
ru.ShortcutGroup=Ярлыки:
ru.LaunchNagi=Запустить Nagi
ru.WebView2Status=Установка компонента WebView2...
ja.DesktopIconDesc=デスクトップにショートカットを作成
ja.ShortcutGroup=ショートカット:
ja.LaunchNagi=Nagi を起動
ja.WebView2Status=WebView2 コンポーネントをインストール中...
zhcn.DesktopIconDesc=创建桌面快捷方式
zhcn.ShortcutGroup=快捷方式:
zhcn.LaunchNagi=启动 Nagi
zhcn.WebView2Status=正在安装 WebView2 组件...
zhtw.DesktopIconDesc=建立桌面捷徑
zhtw.ShortcutGroup=捷徑:
zhtw.LaunchNagi=啟動 Nagi
zhtw.WebView2Status=正在安裝 WebView2 元件...
ar.DesktopIconDesc=إنشاء اختصار على سطح المكتب
ar.ShortcutGroup=الاختصارات:
ar.LaunchNagi=تشغيل Nagi
ar.WebView2Status=جارٍ تثبيت مكوّن WebView2...

[Code]
function IsWebView2Installed: Boolean;
var
  Version: String;
begin
  Result := False;
  if IsWin64 then
    Result := RegQueryStringValue(HKLM, 'SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}', 'pv', Version);
  if (not Result) and IsWin64 then
    Result := RegQueryStringValue(HKCU, 'SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}', 'pv', Version);
  if (not Result) and (not IsWin64) then
    Result := RegQueryStringValue(HKLM, 'SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}', 'pv', Version);
  Result := Result and (Version <> '') and (Version <> '0.0.0.0');
end;






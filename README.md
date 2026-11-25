# Nagi 0.5.0 — a lightweight desktop browser

Nagi is built with Rust and Wry. It has tabs, search, bookmarks, history, downloads and a new-tab page.

- **Windows:** uses Microsoft WebView2 (Chromium).
- **macOS:** uses the built-in WebKit engine. The app is built for Apple silicon and Intel, but is not yet signed or notarized and is therefore not included in public releases.
- **Linux:** uses WebKitGTK 4.1. The first downloadable Linux build targets x86_64 Ubuntu 24.04 and compatible distributions.

Nagi does not bundle a second full browser engine. Memory use depends on the sites you open and is not capped at 500 MB.

## Install

### Windows

Run `Nagi-Installer.exe`. It installs Nagi for your Windows account and offers to install the Microsoft WebView2 runtime if it is missing.

### Linux

Download `Nagi-Linux-x86_64.tar.gz`, extract it, then run `./install.sh` from the extracted directory. The script installs Nagi for your user and adds it to the applications menu. On Ubuntu 24.04, install the WebKitGTK runtime if it is missing:

```sh
sudo apt install libwebkit2gtk-4.1-0
```

Other distributions need WebKitGTK 4.1 and GTK 3. This archive is for x86_64 Linux, not ARM devices.

For the Proton VPN controls in Nagi, install the official Proton VPN Linux CLI and sign in first. Open **Settings → Proton VPN**, choose a country, then select **Connect VPN**. The CLI manages the device-wide VPN and cannot run alongside Proton's Linux desktop app. Country availability depends on your Proton plan. See [Proton's Linux CLI guide](https://protonvpn.com/support/use-linux-cli).

### macOS

A universal Apple silicon and Intel build is prepared for validation. It will be downloadable after the app is signed and notarized.

## Features and platform limits

- Tabs, back and forward, reload and keyboard shortcuts
- New-tab page with search and quick links
- Persistent bookmarks, history, preferences and downloads
- Dark mode and 14 interface languages
- Restore the last page on startup, with a setting to disable it
- Reopen recently closed tabs with `Ctrl+Shift+T` on Windows/Linux or `⌘+Shift+T` on macOS
- Basic filtering of known advertising and tracking domains on Windows
- WebView2 password-save prompts on Windows; Nagi does not currently provide a separate password manager on macOS or Linux
- Proton VPN controls: on Linux, connect or disconnect with the official Proton VPN CLI and select a country; on Windows and macOS, open the official Proton VPN app. Linux VPN connections route traffic for the whole device and require the Proton CLI to be installed and signed in.

On Windows and macOS, the Proton VPN control opens Proton's official app; connection state and region are managed there. Nagi does not collect or store Proton credentials.

The Windows installer offers 14 languages: English, French, Spanish, German, Italian, Brazilian Portuguese, Dutch, Polish, Turkish, Russian, Japanese, Simplified Chinese, Traditional Chinese and Arabic. English is selected by default. Change it during setup or later in Settings. Arabic uses a right-to-left interface.

## Build

From `rust/app`:

```sh
cargo run --release
cargo run --release -- https://www.wikipedia.org
```

For Linux builds, install the WebKitGTK 4.1 development packages first. Wry uses GTK/WebKitGTK on Linux and WebKit on macOS ([Wry platform documentation](https://docs.rs/wry/0.57.3/wry/#platform-considerations)).

Browser profiles:

- Windows: `%LOCALAPPDATA%/NagiBrowser`
- macOS: `~/Library/Application Support/NagiBrowser`
- Linux: `$XDG_DATA_HOME/NagiBrowser` or `~/.local/share/NagiBrowser`

## Releases and updates

Version tags build a Windows installer and a Linux x86_64 archive. GitHub Actions also compiles macOS Apple silicon and Intel builds and packages a universal app for validation; the unsigned app is not attached to public releases. Nagi checks GitHub Releases for its platform's matching asset. Updates open in the default browser and are installed by the user; they are never applied silently.

The Linux release currently targets Ubuntu 24.04 or compatible x86_64 desktops with WebKitGTK 4.1. The macOS build remains unavailable for public download until Apple Developer signing and notarization are configured.

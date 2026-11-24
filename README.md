# Nagi — lightweight browser for Windows and macOS

Nagi is a lightweight desktop browser built with Rust and Wry. It provides tabs, a new tab page, search, bookmarks and downloads.

On Windows, sites are rendered by WebView2, the Chromium engine supplied by Microsoft Edge. On macOS, Nagi uses the system WebKit engine. Nagi does not bundle a second full browser engine. Memory use depends on the pages you open and is not capped at 500 MB.

## Install

- **Windows:** run `Nagi-Installer.exe`. It installs Nagi for your Windows account and offers to install the Microsoft WebView2 runtime if it is missing.
- **macOS 12 or later:** a universal Apple silicon and Intel build is prepared for validation. It will be downloadable from GitHub Releases after it has been signed and notarized.

The macOS build is not signed or notarized yet. macOS may therefore require approval to open it in Privacy & Security. A signed and notarized release requires an Apple Developer signing identity.

Nagi keeps bookmarks, history, downloads and preferences in your local user profile. On Windows, WebView2 provides the password save prompt. On macOS, password saving and autofill depend on WebKit and macOS password settings; Nagi does not currently provide a separate password manager.

## Features

- Tabs, back and forward, reload, and keyboard shortcuts
- New tab page with search and quick links
- Persistent bookmarks, history, preferences and downloads
- Dark mode and 14 interface languages
- Restore the last page on startup, with a setting to disable it
- Reopen recently closed tabs with `Ctrl+Shift+T` on Windows or `⌘+Shift+T` on macOS
- Basic filtering of known ad and tracking domains on Windows
- WebView2 password save prompts on Windows

The Windows installer offers 14 languages: English, French, Spanish, German, Italian, Brazilian Portuguese, Dutch, Polish, Turkish, Russian, Japanese, Simplified Chinese, Traditional Chinese and Arabic. English is selected by default. Change it during setup or later in Settings. Arabic uses a right-to-left interface.

## Build

From `rust/app`:

```sh
cargo run --release
cargo run --release -- https://www.wikipedia.org
```

The Windows profile is stored under `%LOCALAPPDATA%/NagiBrowser`; WebView2 data is in its `WebView2` subfolder. On macOS, the profile is stored in `~/Library/Application Support/NagiBrowser`; WebKit data is in its `WebKit` subfolder.

Each stable version tag publishes `Nagi-Installer.exe` for Windows. The macOS workflow builds native Apple silicon and Intel binaries and packages a universal app for validation, but the disk image is not attached to public releases until it can be signed and notarized. See [DEPLOYMENT.md](DEPLOYMENT.md) for GitHub Releases and Vercel setup.

## Limitations

The Windows ad filter uses a built-in domain list. It cannot remove all ads, especially those served from the same domain as a video or page. macOS currently has no request-filtering support. Windows uses WebView2's password prompts; macOS password autofill depends on native WebKit and system settings and has not been verified as equivalent.

Memory usage depends heavily on websites and is not guaranteed to stay under 500 MB. The included `mesure-memoire.ps1` measures Nagi and its WebView2 child processes on Windows.

## Updates

GitHub builds include the source repository. At startup, Nagi checks for a newer stable release and offers the matching installer or disk image. The download opens in the default browser; the user launches the installer or copies the app to update. Nagi does not install updates silently.

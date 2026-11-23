# Nagi — A Rust Browser for Windows

Nagi is a Windows browser built with Rust and WebView2. It features a tabbed interface, a new tab page, web search, bookmarks, and lightweight browsing.

Web pages are rendered by WebView2, the Chromium engine provided by Microsoft Edge. Nagi does not bundle a second full Chromium engine. Low memory usage is a design goal, not a guaranteed limit for every website.

## Install Nagi

Run `Nagi-Installer.exe`. This single-file installer contains the setup program and its data. It extracts temporary files and starts the Nagi installation. The setup creates a Start menu shortcut, offers to create a desktop shortcut, and installs the Microsoft WebView2 Runtime if it is missing.

Nagi is installed for your Windows account only. Uninstalling removes the application and its shortcuts. Your bookmarks, passwords, and browsing data remain in your Nagi profile.

You can also run `nagi-browser.exe` directly without installing Nagi.

## Features

- Tabs, back and forward navigation, refresh, and keyboard shortcuts
- New tab page with search, quick links, and subtle transitions
- Animations that respect the Windows animation setting
- Persistent settings for dark mode, search engine, and ad blocking
- Clear browsing history from Settings
- Option to restore the last page at startup
- Reopen the last ten closed tabs with `Ctrl+Shift+T`
- Page loading progress indicator
- Persistent bookmarks, browsing history, and download manager
- Downloads saved to `%USERPROFILE%\Downloads\Nagi`
- Basic filtering of known advertising domains
- Password saving and autofill through WebView2 prompts

Shortcuts: `Ctrl+L` address bar, `Ctrl+T` new tab, `Ctrl+Shift+T` reopen a tab, `Ctrl+W` close tab, `Ctrl+H` history, `Ctrl+J` downloads, `Ctrl+D` bookmark, and `Ctrl+,` settings.

## Build and Run

From `rust/app`:

```powershell
cargo run --release
```

You can open a URL or search directly:

```powershell
cargo run --release -- https://www.wikipedia.org
cargo run --release -- "pancake recipes"
```

Browsing data is stored in `%LOCALAPPDATA%\NagiBrowser\WebView2`. Bookmarks, history, downloads, and preferences are stored in `%LOCALAPPDATA%\NagiBrowser`.

The compiled Windows binary is `nagi-browser.exe` in this directory and can be launched directly. The Nagi logo is shown in the app window, and the release binary does not open a PowerShell window.

## Ad Blocking

Nagi blocks requests to a built-in list of known advertising and tracking domains. This basic list is not a replacement for a full content-filtering extension. Ads served from the same domain as a video or web page, including some YouTube ads, may still get through.

## Memory Usage

For an accurate measurement, include the Nagi process and all associated `msedgewebview2.exe` processes. Websites control much of their own memory usage, so complex pages may exceed the 500 MB target.

From this directory, run `.\mesure-memoire.ps1`. If Nagi is already open on the video, the script measures that instance after 20 seconds. Otherwise, it opens three lightweight pages. The script adds together the working sets of the Nagi host and its WebView2 child processes. It leaves the browser open after measuring.

## Languages

The installer offers 14 languages: French, English, Spanish, German, Italian, Brazilian Portuguese, Dutch, Polish, Turkish, Russian, Japanese, Simplified Chinese, Traditional Chinese, and Arabic.

English is selected by default. You can choose another language during installation. Nagi uses that choice on first launch, and you can change the language later in **Settings**. Arabic uses a right-to-left interface.

## GitHub Updates

Builds created by the GitHub workflow automatically include the repository they were built from. At startup, Nagi checks for a newer stable release. If one is available, Settings opens with a button to download the official release installer.

The installer opens in your default browser. You must run it to apply the update; Nagi does not install updates silently.

The GitHub repository must be public so Nagi can check its releases without embedding a secret access token in the app. Local builds made without `NAGI_GITHUB_REPOSITORY` are not linked to a repository and do not check for updates.

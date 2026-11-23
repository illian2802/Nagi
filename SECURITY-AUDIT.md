# Nagi security and reliability review

Review scope: the Windows Rust/WebView2 app, toolbar update flow, static download site, Cargo dependency lockfile, and GitHub release workflow. This is a source review plus targeted tests; it is not a formal penetration test or a guarantee that every possible defect has been found.

## Fixed in this revision

- Updated `cookie` from 0.18.2 to 0.18.3 and `time` from 0.3.45 to 0.3.47 to address RustSec advisories RUSTSEC-2026-0594 and RUSTSEC-2026-0009. The minimum Rust version is now 1.88, required by the patched `time` release.
- The updater checks on startup, opens Settings when a newer stable release is found, rejects malformed or non-GitHub installer URLs, times out network requests, clears stale installer URLs, and lets the user retry a check after an “up to date” result.
- The Rust side now permits opening only the exact HTTPS asset path for the configured GitHub release repository.
- The release workflow now builds with read-only repository permission and publishes in a separate write-only job. Checkout no longer persists credentials, actions are pinned to full commit SHAs, and the release build uses the committed dependency lockfile.
- Added a Content Security Policy and a restrictive Permissions Policy to the static site.
- Added automated checks for release URL validation, script-scheme search input, and ad-block host boundaries.

## Audit results

- RustSec audit of the Windows dependency set: **0 vulnerabilities** after the lockfile updates.
- The all-target lockfile still reports `proc-macro-error` as unmaintained and `glib` as having an unsound iterator. Neither appears in the Windows target dependency tree used by Nagi; they are present for other platforms in Cargo’s cross-platform lockfile.
- The updater and toolbar UI were exercised with mocked GitHub releases: newer release notification, safe download action, retry after “up to date,” stale-URL clearing, and invalid asset rejection all passed.
- All 4 Rust unit tests passed. `cargo fmt -- --check` and Clippy with warnings denied passed. The GitHub Actions workflow could not be executed against a real repository because no GitHub repository exists yet.

## Remaining setup and limitations

- No GitHub repository exists yet. The release workflow embeds the actual `owner/repository` automatically when GitHub builds a tagged release, but the current local executable and the website config cannot be connected to a nonexistent repository. The site’s `site/config.js` still needs the real public repository slug after one is created.
- Installers already on users’ computers do not gain the updater retroactively. Users must install the first release built by GitHub Actions; later stable releases will be detected at startup.
- Updates are offered for user confirmation and opened through the default browser. Nagi does not download or install them silently, and it currently does not verify a downloaded installer’s Authenticode signature or checksum.
- The installer and browser are unsigned. A code-signing certificate and a signed release process would reduce Windows SmartScreen warnings and strengthen publisher verification.
- The ad blocker is a small domain list, not a full content-filter engine. It cannot remove every ad, especially first-party ads.
- Password saving is delegated to WebView2’s profile; Nagi does not provide its own encrypted password-vault format or sync service.

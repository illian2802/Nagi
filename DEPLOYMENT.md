# GitHub Releases and the Nagi website

This project does not push code to GitHub or deploy to Vercel. A pushed version tag starts GitHub Actions and publishes that release automatically.

## Repository and website

Keep the GitHub repository public so Nagi can check releases without a token embedded in the app. The website reads the repository name from `site/config.js`:

```js
window.NAGI_SITE_CONFIG = { githubRepository: "illian2802/Nagi" };
```

Import the repository into Vercel and set `site` as the Root Directory. It is a static site and needs no database or secret environment variables. The page detects Windows, Linux and macOS and only enables a download when the latest stable GitHub release has the matching asset.

## Build verification

`.github/workflows/check.yml` builds the Linux x86_64 app and macOS Apple silicon and Intel apps on pushes to `main` and pull requests. These checks do not publish releases.

## Version 0.4.0 release

The latest public release is `v0.3.3`. After pushing the updated source to `main` and confirming the build checks pass, create and push the `v0.4.0` tag:

```powershell
git tag v0.4.0
git push origin main
git push origin v0.4.0
```

The tag starts `.github/workflows/release.yml`. It creates a GitHub Release with `Nagi-Installer.exe` for Windows and `Nagi-Linux-x86_64.tar.gz` for Linux. Linux targets x86_64 and Ubuntu 24.04 or compatible desktops with WebKitGTK 4.1. macOS Apple silicon and Intel binaries are compiled and packaged for validation, but the unsigned Mac app is not attached to public releases.

## Updates

GitHub builds embed the repository name in Nagi. At startup, Nagi checks the latest stable release and offers the matching platform asset: Windows installer or Linux archive. The download opens in the default browser; the user runs the Windows installer or extracts the Linux archive and runs `install.sh`. Nagi does not install updates silently. Locally built executables without `NAGI_GITHUB_REPOSITORY` cannot check this repository.

## macOS distribution note

The universal app is not signed or notarized and is omitted from public releases. Before offering it to Mac users, sign the app and disk image with an Apple Developer ID certificate and notarize them with Apple's notary service. This requires membership in the Apple Developer Program and protected signing credentials in GitHub Actions.

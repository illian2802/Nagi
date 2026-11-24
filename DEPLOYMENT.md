# GitHub Releases and the Nagi website

This project does not push code to GitHub or deploy to Vercel. Those remain manual actions by the repository owner. A pushed version tag starts GitHub Actions and publishes that release automatically.

## Repository and website

Keep the GitHub repository public so Nagi can check releases without a token embedded in the app. The website reads the repository name from `site/config.js`:

```js
window.NAGI_SITE_CONFIG = { githubRepository: "illian2802/Nagi" };
```

Import the repository into Vercel and set `site` as the Root Directory. It is a static site and needs no database or secret environment variables. The download page checks the latest stable GitHub release and picks the Windows installer or macOS disk image based on the visitor's platform.

## Build verification

`.github/workflows/check.yml` builds the macOS app on Apple silicon and Intel runners when code is pushed to `main` or a pull request is opened. This check does not publish a release.

## Create a release

The existing public release is `v0.3.3`. After adding these changes to the repository, make a new stable version, for example `v0.3.4`, then push the branch and tag:

```powershell
git tag v0.3.4
git push origin main
git push origin v0.3.4
```

The tag starts `.github/workflows/release.yml`. GitHub Actions builds `Nagi-Installer.exe` for Windows and `Nagi-macOS-universal.dmg` for macOS. The macOS app combines native Apple silicon and Intel builds. It then creates a GitHub Release with both files.

## Updates

GitHub builds embed the repository name in Nagi. At startup, Nagi checks the latest stable release and offers the asset for its operating system. The download opens in the default browser; the user runs the Windows installer or copies the Mac app into Applications. Nagi does not install updates silently. Locally built executables without `NAGI_GITHUB_REPOSITORY` cannot check this repository.

## macOS distribution note

The workflow currently packages an unsigned and unnotarized Mac app. It is suitable for build validation and controlled testing, but Gatekeeper may prevent normal opening after download. Before offering it to the public, sign the app and DMG with an Apple Developer ID certificate and notarize them with Apple's notary service. GitHub Actions will need the corresponding protected signing credentials. See [Apple Developer ID](https://developer.apple.com/developer-id/) and [Apple's notarization guide](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution/).

#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Linux" || "$(uname -m)" != "x86_64" ]]; then
  echo "This Nagi package requires an x86_64 Linux system." >&2
  exit 1
fi

archive_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
app_dir="$HOME/.local/opt/nagi-browser"
bin_dir="$HOME/.local/bin"
data_home="${XDG_DATA_HOME:-$HOME/.local/share}"
applications_dir="$data_home/applications"

if command -v ldd >/dev/null 2>&1; then
  missing_libraries="$(ldd "$archive_dir/nagi-browser" | grep 'not found' || true)"
  if [[ -n "$missing_libraries" ]]; then
    echo "Nagi needs system libraries that are missing:" >&2
    echo "$missing_libraries" >&2
    echo "On Ubuntu 24.04, install WebKitGTK with: sudo apt install libwebkit2gtk-4.1-0" >&2
    exit 1
  fi
fi

install -d "$app_dir" "$bin_dir" "$applications_dir"
install -m 755 "$archive_dir/nagi-browser" "$app_dir/nagi-browser"
install -m 644 "$archive_dir/logo.svg" "$app_dir/logo.svg"
ln -sfn "$app_dir/nagi-browser" "$bin_dir/nagi-browser"

cat > "$applications_dir/nagi-browser.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=Nagi Browser
Comment=A lightweight browser for everyday browsing
Exec="$app_dir/nagi-browser" %u
Icon=$app_dir/logo.svg
Terminal=false
Categories=Network;WebBrowser;
MimeType=text/html;x-scheme-handler/http;x-scheme-handler/https;
DESKTOP

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "$applications_dir" >/dev/null 2>&1 || true
fi

echo "Nagi is installed for this user. Find it in your applications menu as Nagi Browser."
echo "You can also launch it with: $bin_dir/nagi-browser"

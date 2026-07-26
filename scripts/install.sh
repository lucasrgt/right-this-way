#!/usr/bin/env sh
set -eu

case "$(uname -s)-$(uname -m)" in
  Linux-x86_64) target="x86_64-unknown-linux-gnu" ;;
  Linux-aarch64|Linux-arm64) target="aarch64-unknown-linux-gnu" ;;
  Darwin-arm64) target="aarch64-apple-darwin" ;;
  Darwin-x86_64) target="x86_64-apple-darwin" ;;
  *) echo "Unsupported platform: $(uname -s)-$(uname -m)" >&2; exit 1 ;;
esac

destination="${RTW_INSTALL_DIR:-$HOME/.local/bin}"
archive="${TMPDIR:-/tmp}/rtw-$target.zip"
mkdir -p "$destination"
curl -fsSL "https://github.com/lucasrgt/right-this-way/releases/latest/download/rtw-$target.zip" -o "$archive"
unzip -jo "$archive" "*/rtw" -d "$destination"
chmod +x "$destination/rtw"
rm -f "$archive"
echo "Installed rtw to $destination/rtw"

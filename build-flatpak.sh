#!/bin/bash
set -e

echo "=== Rusic Flatpak Builder ==="

echo "[0/4] Cleaning up previous builds..."
rm -rf .flatpak-builder build-dir dist
rm -f target/dx/rusic/release/linux/app/rusic
flatpak uninstall --user -y com.temidaradev.rusic || true
flatpak uninstall --system -y com.temidaradev.rusic || true
rm -f ~/.local/bin/rusic
rm -f ~/.local/share/applications/rusic.desktop
rm -f ~/.local/share/applications/com.temidaradev.rusic.desktop
update-desktop-database ~/.local/share/applications || true

echo "[1/4] Building with dx cli..."
npx @tailwindcss/cli -i ./tailwind.css -o ./rusic/assets/tailwind.css --content './rusic/**/*.rs,./components/**/*.rs,./pages/**/*.rs,./hooks/**/*.rs,./player/**/*.rs,./reader/**/*.rs'
dx build --release --package rusic

echo "[2/4] Verifying binary..."
BINARY_PATH="target/dx/rusic/release/linux/app/rusic"
if [ ! -f "$BINARY_PATH" ]; then
    echo "❌ Error: Binary not found at $BINARY_PATH"
    exit 1
fi

echo "[3/4] Packaging Flatpak..."
flatpak-builder --user --install --force-clean build-dir com.temidaradev.rusic.json

echo "[4/4] Creating bundle file..."
mkdir -p dist
flatpak build-bundle ~/.local/share/flatpak/repo dist/rusic.flatpak com.temidaradev.rusic

echo
echo "✅ Flatpak build complete!"
echo "Run with: flatpak run com.temidaradev.rusic"
echo "Bundle file: dist/rusic.flatpak"


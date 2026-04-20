# Maintainer: temidaradev <temidaradev@proton.me>
pkgname=rusic
pkgver=0.4.0
pkgrel=1
pkgdesc="A modern music player"
arch=('x86_64')
url="https://github.com/temidaradev/rusic"
license=('MIT')
depends=(
    'webkit2gtk-4.1'
    'gtk3'
    'libsoup3'
    'glib-networking'
    'alsa-lib'
    'openssl'
    'xdotool'
    'dbus'
)
makedepends=(
    'rust'
    'cargo'
    'git'
)
# dioxus-cli must be installed manually or from AUR at version matching dioxus 0.7.x:
#   cargo install dioxus-cli --version "^0.7"
source=("$pkgname-$pkgver.tar.gz::https://github.com/temidaradev/rusic/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('5f71a49f5e43a499af7d09c44653e687eef5bc41879b91e83d8fa4b2760cc87e')

build() {
    cd "$pkgname-$pkgver"

    export CARGO_HOME="$srcdir/cargo-home"
    export RUSTUP_TOOLCHAIN=stable

    # tailwind.css is pre-built and included in the release tarball
    dx build --release --platform desktop -p rusic
}

package() {
    cd "$pkgname-$pkgver"

    local appdir="target/dx/rusic/release/linux/app"

    install -Dm755 "$appdir/rusic" "$pkgdir/usr/bin/rusic"

    # Install bundled assets alongside binary
    if [[ -d "$appdir/assets" ]]; then
        cp -r "$appdir/assets" "$pkgdir/usr/bin/"
    fi

    install -Dm644 "data/com.temidaradev.rusic.desktop" \
        "$pkgdir/usr/share/applications/com.temidaradev.rusic.desktop"
    sed -i "s|Exec=rusic|Exec=/usr/bin/rusic|" \
        "$pkgdir/usr/share/applications/com.temidaradev.rusic.desktop"

    install -Dm644 "rusic/assets/logo.png" \
        "$pkgdir/usr/share/icons/hicolor/256x256/apps/com.temidaradev.rusic.png"

    install -Dm644 "data/com.temidaradev.rusic.metainfo.xml" \
        "$pkgdir/usr/share/metainfo/com.temidaradev.rusic.metainfo.xml"

    install -Dm644 "LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}

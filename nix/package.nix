{ lib
, rustPlatform
, pkg-config
, wrapGAppsHook3
, webkitgtk_4_1
, gtk3
, libsoup_3
, glib-networking
, alsa-lib
, openssl
, xdotool
, wayland
, dbus
, tailwindcss_4
, dioxus-cli
, src
}:

rustPlatform.buildRustPackage {
  pname = "rusic";
  version = "0.3.2";

  inherit src;

  cargoLock = {
    lockFile = ../Cargo.lock;
  };

  nativeBuildInputs = [
    pkg-config
    wrapGAppsHook3
    tailwindcss_4
    dioxus-cli
  ];

  buildInputs = [
    webkitgtk_4_1
    gtk3
    libsoup_3
    glib-networking
    alsa-lib
    openssl
    xdotool
    wayland
    dbus
  ];

  doCheck = false;

  buildPhase = ''
    runHook preBuild

    tailwindcss -i tailwind.css -o rusic/assets/tailwind.css --minify

    dx build --release --platform desktop -p rusic --offline --frozen

    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall

    mkdir -p $out/bin
    cp -r target/dx/rusic/release/linux/app/* $out/bin/

    install -Dm644 data/com.temidaradev.rusic.desktop \
      $out/share/applications/com.temidaradev.rusic.desktop
    substituteInPlace $out/share/applications/com.temidaradev.rusic.desktop \
      --replace-fail "Exec=rusic" "Exec=$out/bin/rusic"

    install -Dm644 data/com.temidaradev.rusic.metainfo.xml \
      $out/share/metainfo/com.temidaradev.rusic.metainfo.xml

    install -Dm644 rusic/assets/logo.png \
      $out/share/icons/hicolor/256x256/apps/com.temidaradev.rusic.png

    runHook postInstall
  '';

  preFixup = ''
    gappsWrapperArgs+=(--chdir $out/bin)
  '';

  meta = with lib; {
    description = "Rusic - A modern music player";
    license = licenses.mit;
    platforms = platforms.linux;
    mainProgram = "rusic";
  };
}

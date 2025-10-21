{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
    buildInputs = [
        pkgs.git
        pkgs.rustc
        pkgs.cargo
        pkgs.gcc
        pkgs.alsa-lib
        pkgs.pkg-config
        pkgs.wayland
        pkgs.wayland-utils
        pkgs.wayland-protocols
        pkgs.libxkbcommon
        pkgs.xorg.libX11
    ];

    shellHook = ''
    '';
}

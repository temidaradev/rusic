{
  description = "Rusic development environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" "rustfmt" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            pkg-config
            rustToolchain
            dioxus-cli
          ];

          buildInputs = with pkgs; [
            openssl
            glib
            glib-networking
            libsoup_3
            nodejs
          ] ++ lib.optionals stdenv.isLinux [
            gtk3
            webkitgtk_4_1
            xdotool
            alsa-lib
          ] ++ lib.optionals stdenv.isDarwin [
            pkgs.apple-sdk_15
          ];

          LD_LIBRARY_PATH = with pkgs; lib.makeLibraryPath (
            [
              openssl
              glib
              glib-networking
              libsoup_3
            ] ++ lib.optionals stdenv.isLinux [
              gtk3
              webkitgtk_4_1
              xdotool
            ]
          );

          GIO_MODULE_DIR = "${pkgs.glib-networking}/lib/gio/modules/";
        };
      }
    );
}

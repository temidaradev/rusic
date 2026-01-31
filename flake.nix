{
  description = "Rusic - A Dioxus-based music application";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        buildInputs = with pkgs; [
          webkitgtk_4_1
          gtk3
          libsoup_3
          glib-networking
          wayland
          alsa-lib
          xdotool
          openssl
        ];

        nativeBuildInputs = with pkgs; [
          pkg-config
          clang
          lld
          mold
        ];

        shellEnv = {
          RUSTFLAGS = "-C link-arg=-fuse-ld=lld";
          GIO_MODULE_DIR = "${pkgs.glib-networking}/lib/gio/modules/";
          PKG_CONFIG_PATH = pkgs.lib.makeSearchPath "lib/pkgconfig" buildInputs;
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
          WEBKIT_DISABLE_COMPOSITING_MODE = "1";
        };
      in
      {
        devShells.default = pkgs.mkShell {
          inherit buildInputs;

          nativeBuildInputs = nativeBuildInputs ++ (with pkgs; [
            rustToolchain
            dioxus-cli
            nodejs_22
            nodePackages.npm
          ]);

          shellHook = ''
            export RUSTFLAGS="${shellEnv.RUSTFLAGS}"
            export GIO_MODULE_DIR="${shellEnv.GIO_MODULE_DIR}"
            export LD_LIBRARY_PATH="${shellEnv.LD_LIBRARY_PATH}:$LD_LIBRARY_PATH"
            export WEBKIT_DISABLE_COMPOSITING_MODE="${shellEnv.WEBKIT_DISABLE_COMPOSITING_MODE}"
          '';
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "rusic";
          version = "0.1.0";
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          inherit buildInputs nativeBuildInputs;

          CARGO_BUILD_RUSTFLAGS = "-C link-arg=-fuse-ld=lld";
          GIO_MODULE_DIR = "${pkgs.glib-networking}/lib/gio/modules/";
          doCheck = false;

          meta = with pkgs.lib; {
            description = "A Dioxus-based music application";
            license = licenses.mit;
            maintainers = [ ];
          };
        };
      }
    );
}

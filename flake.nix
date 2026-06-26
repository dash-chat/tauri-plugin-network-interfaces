{
  description = "tauri-plugin-network-interfaces development flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.11";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f system);
    in {
      devShells = forAllSystems (system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs { inherit system overlays; };
          rust = pkgs.rust-bin.stable.latest.default;

          tauriLibraries = with pkgs; [
            webkitgtk_4_1
            gtk3
            cairo
            gdk-pixbuf
            glib
            dbus
            openssl
            librsvg
            libsoup_3
            libayatana-appindicator
          ];
        in {
          default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              rust
              pkg-config
            ];
            buildInputs = pkgs.lib.optionals pkgs.stdenv.isLinux tauriLibraries;
            shellHook = pkgs.lib.optionalString pkgs.stdenv.isLinux ''
              export LD_LIBRARY_PATH=${
                pkgs.lib.makeLibraryPath tauriLibraries
              }:$LD_LIBRARY_PATH
            '';
          };
        }
      );
    };
}

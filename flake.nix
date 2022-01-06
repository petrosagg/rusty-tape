{
  description = "Kasetophono client written in rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay }: {
    defaultPackage.x86_64-linux =
      with import nixpkgs {
        system = "x86_64-linux";
        overlays = [ rust-overlay.overlay ];
      };

      rustPlatform.buildRustPackage rec {
        name = "rusty-tape";
        src = self;

        nativeBuildInputs = [
          breakpointHook
          # This is not technically correct, but taped currently looks for mpv in the path
          mpv
          pkg-config
          (rust-bin.stable.latest.default.override {
            targets = [ "wasm32-unknown-unknown" ];
          })
          trunk
          wasm-bindgen-cli
        ];

        buildInputs = [ mpv openssl ];
        doCheck = false;

        cargoBuildFlags = [ "--bin taped" ];

        preBuild = ''
          cd src/webui

          # trunk attempts to create these directories
          export XDG_CONFIG_HOME=$TMPDIR/.config
          export XDG_CACHE_HOME=$TMPDIR/.cache
          trunk build --release --dist ../taped/assets
          cd ../../
        '';

        cargoLock = {
          lockFile = ./Cargo.lock;
        };
      };
  };
}

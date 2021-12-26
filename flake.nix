{
  description = "Kasetophono client written in rust";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  inputs.rust-overlay.url = "github:oxalica/rust-overlay";

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
          pkg-config
          (rust-bin.stable.latest.default.override {
            targets = [ "wasm32-unknown-unknown" ];
          })
          trunk
          wasm-bindgen-cli
        ];

        buildInputs = [ openssl ];
        buildType = "debug";
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

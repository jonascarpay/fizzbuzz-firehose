{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    rust-overlay.url = "github:oxalica/rust-overlay";
    mkflake.url = "github:jonascarpay/mkflake";
  };

  outputs = { nixpkgs, mkflake, rust-overlay, ... }: mkflake.lib.mkflake {
    perSystem = system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            rust-overlay.overlays.default
            (final: prev: {
              PKGNAME = final.callPackage PKGNAME-pkg { };
            })
          ];
        };
        rust-env = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-analyzer"
            "clippy"
            "rustfmt"
            "rust-src"
          ];
          targets = [
            "x86_64-unknown-linux-musl"
          ];
        };

        PKGNAME-pkg = { rustPlatform }: rustPlatform.buildRustPackage {
          pname = "PKGNAME";
          version = "0.1";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
        };

      in
      {
        devShells.default = pkgs.mkShell {
          packages = [
            rust-env
            pkgs.cargo-show-asm
            pkgs.cargo-flamegraph
            pkgs.flamegraph
            pkgs.pv
            pkgs.python3
            pkgs.pyright
            pkgs.ruff
          ];
        };
        packages = rec {
          default = PKGNAME;
          PKGNAME = pkgs.PKGNAME;
          PKGNAME-static = pkgs.pkgsStatic.PKGNAME;
        };
      };
  };
}

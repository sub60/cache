{
  inputs = {
    crane.url = "github:ipetkov/crane";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      crane,
      rust-overlay,
      ...
    }:
    let
      forEachSystem =
        f:
        nixpkgs.lib.genAttrs [
          "aarch64-darwin"
          "aarch64-linux"
          "x86_64-darwin"
          "x86_64-linux"
        ] (system: f nixpkgs.legacyPackages.${system});

      mkToolchain =
        pkgs: (rust-overlay.lib.mkRustBin { } pkgs).fromRustupToolchainFile ./rust-toolchain.toml;
    in
    {
      packages = forEachSystem (pkgs: {
        prod = pkgs.callPackage ./nix/prod.nix rec {
          toolchain = mkToolchain pkgs;
          craneLib = (crane.mkLib pkgs).overrideToolchain (_: toolchain);
        };
      });

      devShells = forEachSystem (
        pkgs:
        let
          toolchain = (mkToolchain pkgs).override {
            extensions = [
              "clippy"
              "rust-analyzer"
              "rust-src"
              "rustfmt"
            ];
          };
        in
        {
          default = pkgs.mkShell {
            nativeBuildInputs = [
              toolchain
              pkgs.worker-build
            ];
          };
        }
      );
    };
}

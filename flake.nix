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
      self,
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

      # The name of the R2 bucket (set in the Cloudflare dashboard) that stores
      # all NARs and NARInfo files.
      r2BucketName = "cache";

      # Internal binding name for the R2 bucket in the Worker environment.
      r2BucketBindingName = "${nixpkgs.lib.toUpper r2BucketName}_BUCKET";
    in
    {
      packages = forEachSystem (pkgs: rec {
        prod = pkgs.callPackage ./nix/prod.nix rec {
          toolchain = mkToolchain pkgs;
          craneLib = (crane.mkLib pkgs).overrideToolchain (_: toolchain);
          inherit r2BucketBindingName;
        };
        deploy = pkgs.callPackage ./nix/deploy.nix {
          inherit prod r2BucketName r2BucketBindingName;
          workerName = "cache";
          workerCompatibilityDate = "2026-03-11";
          gitShortRev = self.shortRev;
        };
        workerd = pkgs.callPackage ./nix/workerd.nix { };
      });

      apps = forEachSystem (pkgs: {
        deploy = {
          type = "app";
          program = pkgs.lib.getExe self.packages.${pkgs.stdenv.hostPlatform.system}.deploy;
        };
      });

      devShells = forEachSystem (pkgs: {
        default = pkgs.mkShell {
          nativeBuildInputs = [
            ((mkToolchain pkgs).override {
              extensions = [
                "clippy"
                "rust-analyzer"
                "rust-src"
                "rustfmt"
              ];
            })
            self.packages.${pkgs.stdenv.hostPlatform.system}.workerd
          ];
          env.R2_BUCKET_BINDING_NAME = r2BucketBindingName;
        };
      });
    };
}

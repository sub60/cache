{
  inputs = {
    cache-db.url = "git+ssh://git@github.com/sub60/cache-db.git";
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
      cache-db,
      crane,
      nixpkgs,
      rust-overlay,
      ...
    }:
    let
      forEachSystem =
        f:
        nixpkgs.lib.genAttrs [
          "aarch64-darwin"
          "aarch64-linux"
          "x86_64-linux"
        ] (system: f system nixpkgs.legacyPackages.${system});

      mkToolchain =
        pkgs: (rust-overlay.lib.mkRustBin { } pkgs).fromRustupToolchainFile ./rust-toolchain.toml;
    in
    {
      lib = {
        cloudflareAccountId = "0218f60551cfad9349da6d43dbb1add3";
        cloudflareWorkerName = "cache";
        r2BucketName = "cache";
      };

      packages = forEachSystem (
        system: pkgs:
        let
          toolchain = mkToolchain pkgs;
          craneLib = (crane.mkLib pkgs).overrideToolchain (_: toolchain);
        in
        {
          deploy = pkgs.callPackage ./nix/deploy.nix {
            inherit self;
          };
          miniflare-server = pkgs.callPackage ./nix/miniflare-server.nix {
            inherit self;
          };
          prod = pkgs.callPackage ./nix/prod.nix {
            inherit self craneLib toolchain;
          };
          prod-sqlx = pkgs.callPackage ./nix/prod-sqlx.nix {
            inherit cache-db craneLib;
            prod = self.packages.${system}.prod;
          };
          update-sqlx = pkgs.callPackage ./nix/update-sqlx.nix {
            prod-sqlx = self.packages.${system}.prod-sqlx;
          };
        }
      );

      apps = forEachSystem (
        system: _pkgs: {
          deploy = {
            type = "app";
            program = nixpkgs.lib.getExe self.packages.${system}.deploy;
          };
          update-sqlx = {
            type = "app";
            program = nixpkgs.lib.getExe self.packages.${system}.update-sqlx;
          };
        }
      );

      devShells = forEachSystem (
        system: pkgs: {
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
              pkgs.wrangler
            ];
            env = self.packages.${system}.prod.devShellEnv;
          };
        }
      );
    };
}

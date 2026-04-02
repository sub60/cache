{
  inputs = {
    cache-db.url = "git+ssh://git@github.com/sub60/cache-db.git";
    crane.url = "github:ipetkov/crane";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
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
      treefmt-nix,
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
        pkgs:
        (rust-overlay.lib.mkRustBin { } pkgs).fromRustupToolchainFile
          ./rust-toolchain.toml;

      mkTreefmt =
        pkgs:
        let
          toolchain = mkToolchain pkgs;
        in
        treefmt-nix.lib.evalModule pkgs {
          projectRootFile = "flake.nix";

          programs.nixfmt = {
            enable = true;
            width = 80;
          };

          programs.rustfmt = {
            enable = true;
            package = toolchain;
          };

          settings.formatter.rustfmt.options = [
            "--config-path"
            "${./rustfmt.toml}"
          ];
        };
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
            inherit self cache-db;
          };
          miniflare-server = pkgs.callPackage ./nix/miniflare-server.nix {
            inherit self;
          };
          prod = pkgs.callPackage ./nix/prod.nix {
            inherit
              self
              cache-db
              craneLib
              toolchain
              ;
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
        system: pkgs:
        {
          deploy = {
            type = "app";
            program = nixpkgs.lib.getExe self.packages.${system}.deploy;
          };
          update-sqlx = {
            type = "app";
            program = nixpkgs.lib.getExe self.packages.${system}.update-sqlx;
          };
        }
        # Workaround for https://github.com/NixOS/nix/issues/8881 so that we
        # can run individual checks with `nix run .#check-<foo>`.
        // (nixpkgs.lib.mapAttrs' (name: check: {
          name = "check-${name}";
          value = {
            type = "app";
            program =
              (pkgs.writeShellScript "check-${name}" ''
                # Force evaluation of ${check}.
                echo -e "\033[1;32m✓\033[0m Check '${name}' passed"
              '').outPath;
          };
        }) self.checks.${system})
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

      formatter = forEachSystem (
        _system: pkgs: (mkTreefmt pkgs).config.build.wrapper
      );

      checks = forEachSystem (
        _system: pkgs: {
          formatting = (mkTreefmt pkgs).config.build.check self;
        }
      );
    };
}

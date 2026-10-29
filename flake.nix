{
  inputs = {
    nix-jettison.url = "git+ssh://git@github.com/noib3/nix-jettison.git";
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
      nix-jettison,
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

      workspaces = forEachSystem (
        _system: pkgs:
        nix-jettison.lib.loadWorkspace {
          inherit pkgs;
          src = self;
          toolchain = mkToolchain pkgs;
        }
      );
    in
    {
      # Workaround for https://github.com/NixOS/nix/issues/8881 so that we
      # can run individual checks with `nix run .#check-<foo>`.
      apps = forEachSystem (
        system: pkgs:
        (nixpkgs.lib.mapAttrs' (name: check: {
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
            ];
          };
        }
      );

      formatter = forEachSystem (
        _system: pkgs: (mkTreefmt pkgs).config.build.wrapper
      );

      checks = forEachSystem (
        system: pkgs: {
          clippy = workspaces.${system}.clippy.override {
            allFeatures = true;
            allTargets = true;
            clippyArgs = [
              "-D"
              "warnings"
            ];
          };
          formatting = (mkTreefmt pkgs).config.build.check self;
        }
      );
    };

  nixConfig = {
    extra-substituters = [
      "https://cache.sub60.dev/noib3/nix-jettison"
    ];
    extra-trusted-public-keys = [
      "cache.sub60.dev/noib3/nix-jettison-1:yFjuL2ZV5kv1FDUlU76T6yEh6/i7Ud3qF+QxgdZt0Ko="
    ];
  };
}

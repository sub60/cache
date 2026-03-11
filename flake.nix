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
        nixpkgs.lib.genAttrs
          [
            "aarch64-darwin"
            "aarch64-linux"
            "x86_64-darwin"
            "x86_64-linux"
          ]
          (
            system:
            f (
              import nixpkgs {
                inherit system;
                overlays = [ (import rust-overlay) ];
              }
            )
          );

      mkToolchain = pkgs: pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
    in
    {
      packages = forEachSystem (
        pkgs:
        let
          toolchain = mkToolchain pkgs;
          craneLib = (crane.mkLib pkgs).overrideToolchain (_: toolchain);
          src = craneLib.cleanCargoSource ./.;

          commonArgs = {
            inherit src;
            strictDeps = true;
            pname = "server-prod";
            CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
            cargoExtraArgs = "-p server-prod";
          };

          cargoArtifacts = craneLib.buildDepsOnly (commonArgs // { doInstall = false; });

          wasm = craneLib.buildPackage (
            commonArgs
            // {
              inherit cargoArtifacts;
              doCheck = false;
              doInstallCargoArtifacts = false;
              installPhaseCommand = ''
                mkdir -p $out
                cp target/wasm32-unknown-unknown/release/*.wasm $out/
                cargo metadata --format-version 1 > $out/metadata.json
              '';
            }
          );

          # Problem: worker-build (from cloudflare/workers-rs) is poorly
          # designed and it couples compilation and bundling into a single tool
          # with no flag to only bundle. We want to handle compilation ourselves
          # (for more granular builds and better caching), but we don't want to
          # reimplement the bundling logic that produces a valid Cloudflare
          # Workers deploy artifact.
          #
          # Solution: worker-build shells out to `cargo` and `rustc`, so we
          # create shims for both that only handle the CLI arguments used by
          # worker-build by either doing nothing or replaying the outputs from
          # the wasm derivation.
          cargoShim = pkgs.writeShellScriptBin "cargo" ''
            case "$1" in
              build)
                mkdir -p "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release"
                cp ${wasm}/*.wasm "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/"
                ;;
              metadata)
                old=$(${pkgs.lib.getExe pkgs.jq} -r .workspace_root ${wasm}/metadata.json)
                ${pkgs.lib.getExe pkgs.gnused} "s|$old|$PWD|g" ${wasm}/metadata.json
                ;;
              *) echo "unexpected subcommand: $1" >&2; exit 1 ;;
            esac
          '';

          rustcShim = pkgs.writeShellScriptBin "rustc" ''
            case "$*" in
              --version) echo "rustc ${toolchain.version}" ;;
              *target-libdir*) echo /tmp ;;
              *sysroot*) echo /tmp ;;
            esac
          '';

          # The version of the wasm-bindgen CLI must exactly match the one of
          # the wasm-bindgen crate in Cargo.lock (because they share an unstable
          # metadata format), so we read the latter from Cargo.lock and build
          # the CLI at that exact version.
          wasm-bindgen-cli =
            let
              cargoLock = builtins.fromTOML (builtins.readFile ./Cargo.lock);
              wasmBindgenVersion =
                (builtins.head (builtins.filter (p: p.name == "wasm-bindgen") cargoLock.package)).version;
            in
            pkgs.buildWasmBindgenCli rec {
              src = pkgs.fetchCrate {
                pname = "wasm-bindgen-cli";
                version = wasmBindgenVersion;
                hash = "sha256-xrCym+rFY6EUQFWyWl6OPA+LtftpUAE5pIaElAIVqW0=";
              };

              cargoDeps = pkgs.rustPlatform.fetchCargoVendor {
                inherit src;
                inherit (src) pname version;
                hash = "sha256-Z8+dUXPQq7S+Q7DWNr2Y9d8GMuEdSnq00quUR0wDNPM=";
              };
            };
        in
        {
          default = pkgs.stdenv.mkDerivation {
            name = "server-prod-worker";
            inherit src;

            nativeBuildInputs = [
              cargoShim
              rustcShim
              pkgs.worker-build
            ];

            env = {
              WASM_BINDGEN_BIN = pkgs.lib.getExe wasm-bindgen-cli;
              ESBUILD_BIN = pkgs.lib.getExe pkgs.esbuild;
              WASM_OPT_BIN = pkgs.lib.getExe' pkgs.binaryen "wasm-opt";
            };

            buildPhase = ''
              export CARGO_TARGET_DIR=''${CARGO_TARGET_DIR:-$PWD/target}
              worker-build --release crates/server-prod
            '';

            installPhase = ''
              mv crates/server-prod/build $out
              rm $out/.gitignore
            '';
          };
        }
      );

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

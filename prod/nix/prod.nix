{
  lib,
  stdenv,
  buildWasmBindgenCli,
  binaryen,
  esbuild,
  fetchCrate,
  gnused,
  jq,
  rustPlatform,
  worker-build,
  writeShellScriptBin,
  # --
  self,
  cache-db,
  craneLib,
  toolchain,
}:

let
  packageName = "sub60-cache-prod";
  packagePathInWorkspace = "prod";

  worker-build_0_8_1 = worker-build.overrideAttrs (_: rec {
    version = "0.8.1";

    src = fetchCrate {
      pname = "worker-build";
      inherit version;
      hash = "sha256-Df87FvodwFeI3UNdFSPqO2oDbnlEg0gWB8ZA1tk/NHo=";
    };

    cargoDeps = rustPlatform.fetchCargoVendor {
      inherit src version;
      pname = "worker-build";
      hash = "sha256-YGt+D7f5VUVnftc9TAeSjbh7a5YwXdFgm+XiaQwoiDA=";
    };

    buildAndTestSubdir = null;
  });

  src = lib.cleanSourceWith {
    src = ../../.;
    filter =
      path: type:
      (craneLib.filterCargoSources path type)
      || (
        type == "regular" && builtins.match ".*/\.sqlx/query-.*\.json$" path != null
      );
  };

  commonArgs = {
    inherit src;
    strictDeps = true;
    pname = packageName;
    cargoExtraArgs = "--locked -p ${packageName}";
    env.CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
  };

  cargoArtifacts = craneLib.buildDepsOnly (commonArgs // { doInstall = false; });

  devShellEnv = {
    CLOUDFLARE_ACCOUNT_ID = self.lib.cloudflareAccountId;
    D1_DB_BINDING_NAME = cache-db.lib.cloudflareD1DatabaseBindingName;
    R2_BUCKET_NAME = self.lib.r2BucketName;
    R2_BUCKET_BINDING_NAME = "${lib.toUpper self.lib.r2BucketName}_BUCKET";
  };

  buildEnv = commonArgs.env // devShellEnv;

  wasm = craneLib.buildPackage (
    commonArgs
    // {
      inherit cargoArtifacts;
      env = buildEnv;
      doCheck = false;
      doInstallCargoArtifacts = false;
      installPhaseCommand = ''
        mkdir -p $out
        cp target/wasm32-unknown-unknown/release/*.wasm $out/
        cargo metadata --format-version 1 > $out/metadata.json
      '';
    }
  );

  # Problem: worker-build (from cloudflare/workers-rs) is poorly designed and it
  # couples compilation and bundling into a single tool with no flag to only
  # bundle. We want to handle compilation ourselves (for more granular builds
  # and better caching), but we don't want to reimplement the bundling logic
  # that produces a valid Cloudflare Workers deploy artifact.
  #
  # Solution: worker-build shells out to `cargo` and `rustc`, so we create shims
  # for both that only handle the CLI arguments used by it by either doing
  # nothing or replaying the outputs from the wasm derivation.
  cargoShim = writeShellScriptBin "cargo" ''
    case "$1" in
      build)
        mkdir -p "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release"
        cp ${wasm}/*.wasm "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/"
        ;;
      metadata)
        old=$(${lib.getExe jq} -r .workspace_root ${wasm}/metadata.json)
        ${lib.getExe gnused} "s|$old|$PWD|g" ${wasm}/metadata.json
        ;;
      *) echo "unexpected subcommand: $1" >&2; exit 1 ;;
    esac
  '';

  rustcShim = writeShellScriptBin "rustc" ''
    case "$*" in
      --version) echo "rustc ${toolchain.version}" ;;
      *target-libdir*) echo /tmp ;;
      *sysroot*) echo /tmp ;;
    esac
  '';

  # The wasm-bindgen CLI must be the exact same version as the wasm-bindgen
  # crate in Cargo.lock (they share an unstable metadata format), so we read it
  # from Cargo.lock and build the CLI to match.
  wasm-bindgen-cli =
    let
      cargoLock = builtins.fromTOML (builtins.readFile ../../Cargo.lock);
      wasmBindgenVersion =
        (builtins.head (
          builtins.filter (p: p.name == "wasm-bindgen") cargoLock.package
        )).version;
    in
    buildWasmBindgenCli rec {
      src = fetchCrate {
        pname = "wasm-bindgen-cli";
        version = wasmBindgenVersion;
        hash = "sha256-ve783oYH0TGv8Z8lIPdGjItzeLDQLOT5uv/jbFOlZpI=";
      };

      cargoDeps = rustPlatform.fetchCargoVendor {
        inherit src;
        inherit (src) pname version;
        hash = "sha256-EYDfuBlH3zmTxACBL+sjicRna84CvoesKSQVcYiG9P0=";
      };
    };
in
stdenv.mkDerivation {
  inherit src;

  name = "${packageName}-worker";

  nativeBuildInputs = [
    cargoShim
    rustcShim
    worker-build_0_8_1
  ];

  env = {
    ESBUILD_BIN = lib.getExe esbuild;
    WASM_BINDGEN_BIN = lib.getExe wasm-bindgen-cli;
    WASM_OPT_BIN = lib.getExe' binaryen "wasm-opt";
  };

  buildPhase = ''
    export CARGO_TARGET_DIR=''${CARGO_TARGET_DIR:-$PWD/target}
    worker-build --release ${packagePathInWorkspace}
  '';

  installPhase = ''
    mkdir -p $out
    mv ${packagePathInWorkspace}/build/index.js $out/
    mv ${packagePathInWorkspace}/build/index_bg.wasm $out/
  '';

  passthru = {
    inherit
      src
      buildEnv
      cargoArtifacts
      devShellEnv
      packageName
      ;
  };
}

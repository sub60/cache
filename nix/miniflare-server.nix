# Packages a small Node.js script that uses Miniflare to serve a Cloudflare
# Worker with R2 support locally.
#
# Takes no arguments. Prints {"event":"listen","port":<port>} to stdout when
# ready.
{
  lib,
  stdenvNoCC,
  buildNpmPackage,
  nodejs,
  cacert,
  writeShellApplication,
  writeText,
  prod,
  r2BucketBindingName,
  workerCompatibilityDate,
}:

let
  version = "4.20260312.0";

  # A minimal package.json requesting miniflare as the sole dependency.
  packageJson = writeText "package.json" (
    builtins.toJSON {
      name = "miniflare-server";
      version = "0.0.0";
      private = true;
      type = "module";
      dependencies.miniflare = version;
    }
  );

  # FOD that generates a package-lock.json from the package.json above.
  # `npm install --package-lock-only` is deterministic for a given input.
  packageLock = stdenvNoCC.mkDerivation {
    name = "miniflare-${version}-package-lock.json";

    outputHash = "sha256-67ivBdagl34S1xGhnc96Igewp5BXbrvLQOIX//NawAY=";
    outputHashAlgo = "sha256";
    outputHashMode = "flat";

    nativeBuildInputs = [
      nodejs
      cacert
    ];

    dontUnpack = true;

    buildPhase = ''
      export HOME=$TMPDIR
      cp ${packageJson} package.json
      npm install --package-lock-only --ignore-scripts
    '';

    installPhase = ''
      cp package-lock.json $out
    '';
  };

  # Use buildNpmPackage to deterministically install the node_modules tree
  # from the generated lockfile.
  node_modules = buildNpmPackage {
    pname = "miniflare-server-deps";
    inherit version;

    src = stdenvNoCC.mkDerivation {
      name = "miniflare-server-src";
      dontUnpack = true;
      installPhase = ''
        mkdir -p $out
        cp ${packageJson} $out/package.json
        cp ${packageLock} $out/package-lock.json
      '';
    };

    npmDepsHash = "sha256-1M9kjWslvkoDdlFHEDSs10AVUcha8CT6BxyA8ob2CCU=";

    dontNpmBuild = true;

    installPhase = ''
      mkdir -p $out
      cp -r node_modules $out/
    '';
  };

  script = writeText "miniflare-server.mjs" ''
    import { Miniflare } from "miniflare";

    const mf = new Miniflare({
      scriptPath: "${prod}/index.js",
      modules: true,
      modulesRules: [
        { type: "CompiledWasm", include: ["**/*.wasm"], fallthrough: true },
      ],
      compatibilityDate: "${workerCompatibilityDate}",
      r2Buckets: ["${r2BucketBindingName}"],
      port: 0,
      host: "127.0.0.1",
    });

    const url = await mf.ready;

    console.log(JSON.stringify({ event: "listen", port: parseInt(url.port) }));
  '';
in
writeShellApplication {
  name = "miniflare-server";
  runtimeInputs = [ nodejs ];
  text = ''
    export NODE_PATH=${node_modules}/node_modules
    exec node --input-type=module ${script}
  '';
}

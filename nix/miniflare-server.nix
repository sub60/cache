# Packages a small Node.js script that uses Miniflare to serve a Cloudflare
# Worker with R2 support locally.
#
# The resulting binary accepts:
#   miniflare-server <script-path>
#
# and prints {"event":"listen","port":<port>} to stdout when ready.
{
  lib,
  stdenvNoCC,
  nodejs,
  writeShellApplication,
  writeText,
  r2BucketBindingName,
  workerCompatibilityDate,
}:

let
  version = "4.20260312.0";

  # FOD that runs `npm install miniflare` and captures the resulting
  # node_modules tree. The output hash must be updated when `version` changes.
  node_modules = stdenvNoCC.mkDerivation {
    name = "miniflare-${version}-node_modules";

    outputHash = lib.fakeHash;
    outputHashAlgo = "sha256";
    outputHashMode = "recursive";

    nativeBuildInputs = [ nodejs ];

    dontUnpack = true;

    buildPhase = ''
      export HOME=$TMPDIR
      npm install --no-audit --no-fund --ignore-scripts miniflare@${version}
    '';

    installPhase = ''
      mv node_modules $out
    '';
  };

  script = writeText "miniflare-server.mjs" ''
    import { Miniflare } from "miniflare";

    const scriptPath = process.argv[2];

    if (!scriptPath) {
      console.error("Usage: miniflare-server <script-path>");
      process.exit(1);
    }

    const mf = new Miniflare({
      scriptPath,
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
    export NODE_PATH=${node_modules}
    exec node --input-type=module ${script} "$@"
  '';
}

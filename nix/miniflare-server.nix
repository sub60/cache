# Packages a small script that uses Miniflare to serve a Cloudflare Worker
# with R2 support locally.
#
# Usage:
#   miniflare-server <state-dir>
#
# Prints the listening port to stdout when ready.
{
  stdenvNoCC,
  bun,
  cacert,
  writeShellApplication,
  writeText,
  prod,
  r2BucketBindingName,
  workerCompatibilityDate,
}:

let
  version = "4.20260312.0";

  # FOD that runs `bun install miniflare` and captures the resulting
  # node_modules tree.
  node_modules = stdenvNoCC.mkDerivation {
    name = "miniflare-${version}-node_modules";

    outputHash = "sha256-OrMv6HlU2QFHXi4Qc8GOyQ9NdNTfBowlwPqRveCAsEM=";
    outputHashAlgo = "sha256";
    outputHashMode = "recursive";

    nativeBuildInputs = [
      bun
      cacert
    ];

    dontUnpack = true;

    buildPhase = ''
      bun install --no-save miniflare@${version}
    '';

    installPhase = ''
      rm -rf node_modules/.cache
      mv node_modules $out
    '';
  };

  script = writeText "miniflare-server.ts" ''
    import { Miniflare } from "miniflare";

    const stateDir = process.argv[2];
    if (!stateDir) {
      console.error("Usage: miniflare-server <state-dir>");
      process.exit(1);
    }

    // Change to the state directory so any files miniflare/workerd creates
    // (like .wrangler/) end up there instead of the caller's cwd.
    process.chdir(stateDir);

    const mf = new Miniflare({
      scriptPath: "${prod}/index.js",
      modulesRoot: "${prod}",
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

    console.log(url.port);
  '';
in
writeShellApplication {
  name = "miniflare-server";
  runtimeInputs = [ bun ];
  runtimeEnv.NODE_PATH = "${node_modules}";
  text = ''
    exec bun run ${script} "$@"
  '';
}

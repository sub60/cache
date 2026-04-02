{
  stdenv,
  sqlite,
  # --
  cache-db,
  craneLib,
  prod,
}:

let
  sqlxDir = "crates/server-prod/.sqlx";
in
craneLib.mkCargoDerivation {
  inherit (prod) src cargoArtifacts;

  pname = "${prod.packageName}-sqlx";

  nativeBuildInputs = [
    cache-db.packages.${stdenv.hostPlatform.system}.print-schema
    sqlite
  ];

  env = prod.buildEnv // {
    inherit sqlxDir;
    miniflareDir = ".wrangler/state/v3/d1/miniflare-D1DatabaseObject";
  };

  doCheck = false;
  doInstallCargoArtifacts = false;
  strictDeps = true;

  buildPhaseCargoCommand = ''
    mkdir -p "$miniflareDir" "$sqlxDir"
    rm -f "$sqlxDir"/*.json
    print-schema | sqlite3 "$miniflareDir/cache.sqlite"
    cargo check --locked -p ${prod.packageName}
  '';

  installPhaseCommand = ''
    mkdir -p $out
    cp -R "$sqlxDir"/. $out/
  '';

  passthru = {
    inherit sqlxDir;
  };
}

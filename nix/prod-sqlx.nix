{
  stdenv,
  sqlite,
  # --
  cache-db,
  craneLib,
  prod,
}:

craneLib.mkCargoDerivation {
  inherit (prod) src cargoArtifacts;

  pname = "${prod.packageName}-sqlx";

  nativeBuildInputs = [
    cache-db.packages.${stdenv.hostPlatform.system}.print-schema
    sqlite
  ];

  env = prod.buildEnv // {
    miniflareDir = ".wrangler/state/v3/d1/miniflare-D1DatabaseObject";
    sqlxDir = "crates/server-prod/.sqlx";
  };

  doCheck = false;
  doInstallCargoArtifacts = false;
  strictDeps = true;

  buildPhaseCargoCommand = ''
    mkdir -p $miniflareDir
    rm -f $sqlxDir/*
    print-schema | sqlite3 $miniflareDir/cache.sqlite
    cargo check --locked -p ${prod.packageName}
  '';

  installPhaseCommand = ''
    mkdir -p $out
    cp -R $sqlxDir/. $out/
  '';
}

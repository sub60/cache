# Fetches a prebuilt `workerd` binary from the npm registry.
#
# Cloudflare distributes workerd as platform-specific npm packages under the
# `@cloudflare/workerd-<platform>-<arch>` scope. Each package contains a single
# prebuilt binary.
{
  lib,
  stdenv,
  fetchurl,
  autoPatchelfHook,
  glibc,
}:

let
  version = "1.20260313.1";

  # Map Nix system identifiers to npm package names and SRI hashes.
  platforms = {
    "aarch64-darwin" = {
      npmPkg = "workerd-darwin-arm64";
      hash = "sha512-exIuImgO3hHGli9BKb4Lri35oKVbbV0/O4ZZ0sxblCNq/MHQKJT41w5s9g9vvTnf51+0UCzuXk8137ehVA61hw==";
    };
    "x86_64-darwin" = {
      npmPkg = "workerd-darwin-64";
      hash = "sha512-BfiPuTdn3UzIz+tSWDbvIxi0amhZASLteeRdkRhjry3UieB47Z17fHiebFJ/gAaGU8zDhhQwVzOaJgtnsqceHw==";
    };
    "x86_64-linux" = {
      npmPkg = "workerd-linux-64";
      hash = "sha512-1riikTl9uMb/B7lkTilnrp6+RZlDoEgG6BvDBYG/F0MOlOjV8KPWA4bwYw0WqcRWtbzFNrEAaYotKkd/VSp+pQ==";
    };
    "aarch64-linux" = {
      npmPkg = "workerd-linux-arm64";
      hash = "sha512-M6H97XVICxVImZNcnlfMkCS5tthAw8Hicp0PUkO/m4T4KSzpD7ZPVRiyuhP9r9cT/+s28phpvjvy/agGy+rL5g==";
    };
  };

  platform =
    platforms.${stdenv.hostPlatform.system}
      or (throw "workerd: unsupported system ${stdenv.hostPlatform.system}");

  src = fetchurl {
    url = "https://registry.npmjs.org/@cloudflare/${platform.npmPkg}/-/${platform.npmPkg}-${version}.tgz";
    hash = platform.hash;
  };
in
stdenv.mkDerivation {
  pname = "workerd";
  inherit version src;

  sourceRoot = ".";

  nativeBuildInputs = lib.optionals stdenv.hostPlatform.isLinux [
    autoPatchelfHook
  ];

  buildInputs = lib.optionals stdenv.hostPlatform.isLinux [
    glibc
  ];

  installPhase = ''
    mkdir -p $out/bin
    cp package/bin/workerd $out/bin/workerd
    chmod +x $out/bin/workerd
  '';

  meta = {
    description = "The JavaScript/Wasm runtime that powers Cloudflare Workers";
    homepage = "https://github.com/cloudflare/workerd";
    license = lib.licenses.asl20;
    platforms = builtins.attrNames platforms;
    mainProgram = "workerd";
  };
}

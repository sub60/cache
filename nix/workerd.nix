# Fetches a prebuilt `workerd` binary from GitHub releases.
#
# To update, run:
#   nix run .#update-workerd
{
  lib,
  stdenv,
  fetchurl,
  autoPatchelfHook,
  glibc,
  writeShellApplication,
  nix,
  curl,
  jq,
}:

let
  inherit (stdenv.hostPlatform) isDarwin isAarch64 system;

  os = if isDarwin then "darwin" else "linux";
  arch = if isAarch64 then "arm64" else "64";
  asset = "workerd-${os}-${arch}";

  hashes = {
    "workerd-darwin-arm64" = "sha256-5dzCpBnMiIcJ0qbsnjQ8L0MS9NQDHgGgD+HBGtiMb+M=";
    "workerd-darwin-64" = "sha256-ry991b/K8PpvoSYoSTs06l1ZEN09BB3q2Bw8IwGJob0=";
    "workerd-linux-64" = "sha256-xh3aZqPWcPscbXwrt+uTfvoT2hCfyrLyDYyI0q1yfpI=";
    "workerd-linux-arm64" = "sha256-ibDH6EGSJs5oGD29wB1ptmenmtgogtVwqGEzrSBnmQs=";
  };
in
stdenv.mkDerivation (finalAttrs: {
  pname = "workerd";
  version = "1.20260313.1";

  src = fetchurl {
    url = "https://github.com/cloudflare/workerd/releases/download/v${finalAttrs.version}/${asset}.gz";
    hash = hashes.${asset} or (throw "workerd: unsupported system ${system}");
  };

  nativeBuildInputs = lib.optionals stdenv.hostPlatform.isLinux [
    autoPatchelfHook
  ];

  buildInputs = lib.optionals stdenv.hostPlatform.isLinux [
    glibc
  ];

  unpackPhase = ''
    gzip -dc $src > workerd
    chmod +x workerd
  '';

  installPhase = ''
    mkdir -p $out/bin
    mv workerd $out/bin/
  '';

  passthru.updateScript = writeShellApplication {
    name = "update-workerd";
    runtimeInputs = [
      curl
      jq
      nix
    ];
    text = ''
      nix_file="nix/workerd.nix"

      # Find the latest release that has assets.
      latest=$(curl -sL "https://api.github.com/repos/cloudflare/workerd/releases" \
        | jq -r '[.[] | select(.assets | length > 0)][0].tag_name')

      version="''${latest#v}"
      echo "Latest workerd version with assets: $version"

      for name in ${lib.concatStringsSep " " (builtins.attrNames hashes)}; do
        url="https://github.com/cloudflare/workerd/releases/download/v''${version}/''${name}.gz"
        echo "Prefetching $name..."
        raw_hash=$(nix-prefetch-url --type sha256 "$url" 2>/dev/null)
        sri_hash=$(nix hash convert --hash-algo sha256 --to sri "$raw_hash")
        echo "  $name: $sri_hash"

        # Replace the hash for this asset in the Nix file.
        sed -i.bak "/$name/s|\"sha256-.*\"|\"$sri_hash\"|" "$nix_file"
      done

      rm -f "''${nix_file}.bak"

      # Update the version.
      sed -i.bak "s|version = \".*\";|version = \"$version\";|" "$nix_file"
      rm -f "''${nix_file}.bak"

      echo "Updated $nix_file to workerd $version"
    '';
  };

  meta.mainProgram = "workerd";
})

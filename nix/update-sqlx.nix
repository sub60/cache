{
  coreutils,
  git,
  writeShellApplication,
  # --
  prod-sqlx,
}:

writeShellApplication {
  name = "update-sqlx";
  runtimeInputs = [
    coreutils
    git
  ];
  text = ''
    repo_root="$(git rev-parse --show-toplevel)"
    sqlx_dir="$repo_root/${prod-sqlx.sqlxDir}"

    mkdir -p "$sqlx_dir"
    rm -f "$sqlx_dir"/query-*.json

    for file in "${prod-sqlx}"/query-*.json; do
      cp "$file" "$sqlx_dir"/
      chmod 0644 "$sqlx_dir"/"$(basename "$file")"
    done
  '';
}

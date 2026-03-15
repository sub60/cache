#![allow(missing_docs)]

use std::env;
use std::path::PathBuf;
use std::process::Command;

const NIX_IN_PATH_CFG: &str = "nix_in_path";

fn main() {
    println!("cargo::rustc-check-cfg=cfg({NIX_IN_PATH_CFG})");

    // We only build the `miniflare-server` via `nix build` when the
    // `server-prod` feature is active and `nix` is in $PATH.
    if env::var("CARGO_FEATURE_SERVER_PROD").is_err()
        || find_in_path("nix").is_none()
    {
        return;
    }

    println!("cargo:rustc-cfg={NIX_IN_PATH_CFG}");

    build_miniflare();
}

fn build_miniflare() {
    let manifest_dir =
        env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set");

    let flake_root = PathBuf::from(&manifest_dir)
        .parent()
        .expect("tests/ has a parent directory")
        .to_path_buf();

    let output = Command::new("nix")
        .args(["build", ".#miniflare-server", "--no-link", "--print-out-paths"])
        .current_dir(&flake_root)
        .output()
        .expect("failed to run `nix build`");

    if !output.status.success() {
        panic!(
            "`nix build .#miniflare-server` failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let store_path = String::from_utf8(output.stdout)
        .expect("`nix build` output is valid UTF-8")
        .trim()
        .to_owned();

    let bin_path = format!("{store_path}/bin/miniflare-server");

    println!("cargo:rustc-env=MINIFLARE_SERVER_BIN={bin_path}");

    // Re-run if Nix inputs change.
    println!("cargo:rerun-if-changed=../flake.lock");
    println!("cargo:rerun-if-changed=../flake.nix");
    println!("cargo:rerun-if-changed=../nix/miniflare-server.nix");
    println!("cargo:rerun-if-changed=../nix/prod.nix");
}

/// Checks if a binary is available in `$PATH`.
fn find_in_path(binary: &str) -> Option<PathBuf> {
    env::var_os("PATH").and_then(|path| {
        env::split_paths(&path)
            .map(|dir| dir.join(binary))
            .find(|candidate| candidate.is_file())
    })
}

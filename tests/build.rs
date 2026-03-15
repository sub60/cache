#![allow(missing_docs)]

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    setup_nix();
    setup_miniflare();
}

/// Checks if `nix` is in `$PATH` and enables `cfg(nix_in_path)`.
fn setup_nix() {
    println!("cargo::rustc-check-cfg=cfg(nix_in_path)");

    if find_in_path("nix").is_some() {
        println!("cargo:rustc-cfg=nix_in_path");
    }
}

/// Checks if `miniflare-server` is in `$PATH` and enables
/// `cfg(miniflare_in_path)`.
///
/// When the `server-prod` feature is active and both `nix` and
/// `miniflare-server` are in `$PATH`, this also runs `nix build .#prod` to
/// compile the WASM worker and exports its store path via
/// `PROD_WORKER_PATH`.
fn setup_miniflare() {
    println!("cargo::rustc-check-cfg=cfg(miniflare_in_path)");

    if find_in_path("miniflare-server").is_some() {
        println!("cargo:rustc-cfg=miniflare_in_path");
    }

    if env::var("CARGO_FEATURE_SERVER_PROD").is_ok()
        && find_in_path("nix").is_some()
        && find_in_path("miniflare-server").is_some()
    {
        build_prod_worker();
    }
}

/// Builds the `server-prod` WASM worker via `nix build .#prod` and exports
/// the Nix store path containing `index.js` and `index_bg.wasm`.
fn build_prod_worker() {
    let manifest_dir =
        env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");

    let flake_root = PathBuf::from(&manifest_dir)
        .parent()
        .expect("tests/ has a parent directory")
        .to_path_buf();

    let output = Command::new("nix")
        .args(["build", ".#prod", "--no-link", "--print-out-paths"])
        .current_dir(&flake_root)
        .output()
        .expect("failed to run `nix build`");

    if !output.status.success() {
        panic!(
            "`nix build .#prod` failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let prod_store_path = String::from_utf8(output.stdout)
        .expect("`nix build` output is valid UTF-8")
        .trim()
        .to_owned();

    println!("cargo:rustc-env=PROD_WORKER_PATH={prod_store_path}");

    // Re-run if the prod Nix derivation inputs change.
    println!("cargo:rerun-if-changed=../flake.nix");
    println!("cargo:rerun-if-changed=../flake.lock");
    println!("cargo:rerun-if-changed=../nix/prod.nix");
    println!("cargo:rerun-if-changed=../crates/server-prod/src");
    println!("cargo:rerun-if-changed=../crates/cache/src");
    println!("cargo:rerun-if-env-changed=R2_BUCKET_BINDING_NAME");
}

/// Checks if a binary is available in `$PATH`.
fn find_in_path(binary: &str) -> Option<PathBuf> {
    env::var_os("PATH").and_then(|path| {
        env::split_paths(&path)
            .map(|dir| dir.join(binary))
            .find(|candidate| candidate.is_file())
    })
}

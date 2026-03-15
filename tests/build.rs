#![allow(missing_docs)]

use std::env;
use std::path::PathBuf;

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
fn setup_miniflare() {
    println!("cargo::rustc-check-cfg=cfg(miniflare_in_path)");

    if find_in_path("miniflare-server").is_some() {
        println!("cargo:rustc-cfg=miniflare_in_path");
    }
}

/// Checks if a binary is available in `$PATH`.
fn find_in_path(binary: &str) -> Option<PathBuf> {
    env::var_os("PATH").and_then(|path| {
        env::split_paths(&path)
            .map(|dir| dir.join(binary))
            .find(|candidate| candidate.is_file())
    })
}

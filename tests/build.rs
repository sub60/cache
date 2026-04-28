#![allow(missing_docs)]

use std::env;

const NIX_IN_PATH_CFG: &str = "nix_in_path";

fn main() {
    println!("cargo::rustc-check-cfg=cfg({NIX_IN_PATH_CFG})");

    if is_in_path("nix") {
        println!("cargo:rustc-cfg={NIX_IN_PATH_CFG}");
    }
}

/// Checks if a binary is available in `$PATH`.
fn is_in_path(binary: &str) -> bool {
    env::var_os("PATH").is_some_and(|path| {
        env::split_paths(&path)
            .map(|dir| dir.join(binary))
            .any(|candidate| candidate.is_file())
    })
}

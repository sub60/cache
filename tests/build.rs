#![allow(missing_docs)]

use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

fn main() {
    setup_nix();
    setup_workerd();
}

/// Checks if `nix` is in `$PATH` and enables `cfg(nix_in_path)`.
///
/// When enabled, the build script also runs `nix build .#prod` to compile the
/// `server-prod` WASM worker, generates a Cap'n Proto config file for
/// `workerd`, and exports its path via `WORKERD_TEST_CONFIG`.
fn setup_nix() {
    println!("cargo::rustc-check-cfg=cfg(nix_in_path)");

    if find_in_path("nix").is_none() {
        return;
    }

    println!("cargo:rustc-cfg=nix_in_path");

    // Only build the WASM and generate the config if workerd is also
    // available — otherwise the config would be unused.
    if find_in_path("workerd").is_none() {
        return;
    }

    build_workerd_config();
}

/// Checks if `workerd` is in `$PATH` and enables `cfg(workerd_in_path)`.
fn setup_workerd() {
    println!("cargo::rustc-check-cfg=cfg(workerd_in_path)");

    if find_in_path("workerd").is_some() {
        println!("cargo:rustc-cfg=workerd_in_path");
    }
}

/// Builds the `server-prod` WASM worker via `nix build .#prod` and generates a
/// Cap'n Proto config file that `workerd` can serve.
fn build_workerd_config() {
    let manifest_dir =
        env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");

    // The flake root is the parent of the `tests/` directory.
    let flake_root = PathBuf::from(&manifest_dir)
        .parent()
        .expect("tests/ has a parent directory")
        .to_path_buf();

    // Build the prod worker via Nix. The result symlink points to a directory
    // containing `index.js` and `index_bg.wasm`.
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

    // Read R2_BUCKET_BINDING_NAME from the environment (set by the Nix dev
    // shell).
    let r2_binding_name = env::var("R2_BUCKET_BINDING_NAME")
        .expect("R2_BUCKET_BINDING_NAME must be set");

    // The compatibility date must match what's used in deploy.nix.
    let compat_date = "2026-03-11";

    let index_js_path = format!("{prod_store_path}/index.js");
    let index_wasm_path = format!("{prod_store_path}/index_bg.wasm");

    // Generate the Cap'n Proto config file.
    let config = format!(
        r#"using Workerd = import "/workerd/workerd.capnp";

const config :Workerd.Config = (
  services = [
    (name = "cache", worker = .cacheWorker),
  ],
  sockets = [
    (name = "http", address = "*:0", http = (), service = "cache"),
  ],
);

const cacheWorker :Workerd.Worker = (
  modules = [
    (name = "index.js", esModule = embed "{index_js_path}"),
    (name = "index_bg.wasm", wasm = embed "{index_wasm_path}"),
  ],
  compatibilityDate = "{compat_date}",
  bindings = [
    (name = "{r2_binding_name}", r2Bucket = "test-bucket"),
  ],
);
"#
    );

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let config_path = PathBuf::from(&out_dir).join("workerd-test.capnp");
    fs::write(&config_path, &config).expect("failed to write capnp config");

    // Export the config path so tests can find it.
    println!("cargo:rustc-env=WORKERD_TEST_CONFIG={}", config_path.display());

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

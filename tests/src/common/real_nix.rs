use core::convert::Infallible;
use core::iter;
use core::net::{SocketAddr, SocketAddrV4, SocketAddrV6};
use core::str::FromStr;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::{env, fs, process};

use axum::body::Bytes;
use either::Either;
use nix_types::{NarInfo, StoreName, StorePath};

use crate::common::{CopyOptions, NixClient, NixStore};

/// TODO: docs.
pub struct RealNixClient {
    store: RealNixStore,
}

/// TODO: docs.
pub struct RealNixStore {
    canonicalized_path: PathBuf,
    #[expect(unused)]
    dir: tempfile::TempDir,
}

#[derive(Debug)]
pub struct CacheHost(url::Url);

impl RealNixClient {
    /// TODO: docs.
    #[track_caller]
    pub fn create() -> Self {
        Self { store: RealNixStore::create() }
    }

    async fn run(
        &self,
        args: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> process::Output {
        run_nix_command(&self.store.canonicalized_path, args).await
    }
}

impl RealNixStore {
    #[track_caller]
    fn create() -> Self {
        let dir =
            tempfile::tempdir().expect("failed to create temporary directory");

        let canonicalized_path = fs::canonicalize(dir.path())
            .expect("failed to canonicalize temporary directory path");

        Self { canonicalized_path, dir }
    }

    /// Runs a `nix` subcommand against this store.
    async fn run(
        &self,
        args: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> process::Output {
        run_nix_command(&self.canonicalized_path, args).await
    }
}

impl NixClient for RealNixClient {
    type Store = RealNixStore;
    type CacheHost = CacheHost;
    type CopyError = Infallible;

    fn store(&self) -> &Self::Store {
        &self.store
    }

    async fn copy_impl(
        &self,
        paths: Vec<StorePath>,
        options: CopyOptions<true, Self>,
    ) -> Result<(), Self::CopyError> {
        self.run(
            iter::once(Either::Right("copy".to_owned()))
                .chain(options.into_cli_args().map(Either::Left))
                .chain(paths.iter().map(|p| p.to_string()).map(Either::Right)),
        )
        .await;

        Ok(())
    }
}

impl NixStore for RealNixStore {
    async fn add_file(
        &self,
        store_name: &str,
        file_contents: &[u8],
    ) -> StorePath {
        if let Err(err) = store_name.parse::<StoreName>() {
            panic!("invalid store name {store_name:?}: {err}");
        }

        let mut tempfile = tempfile::NamedTempFile::new()
            .expect("failed to create temporary file");

        use std::io::Write as _;
        tempfile
            .write_all(file_contents)
            .expect("failed to write file contents to tempfile");

        let file_path =
            tempfile.path().to_str().expect("tempfile's path is valid UTF-8");

        let output = self
            .run(["store", "add-file", "--name", store_name, file_path])
            .await;

        str::from_utf8(&output.stdout)
            .expect("`nix store add-file` prints valid UTF-8")
            .trim()
            .parse()
            .expect("`nix store add-file` prints a Nix store path")
    }

    async fn delete(&self, store_path: &StorePath) {
        let path_str = store_path.to_string();
        self.run(["store", "delete", "--ignore-liveness", &path_str]).await;
    }

    async fn nar_info_and_bytes(
        &self,
        store_path: &StorePath,
    ) -> (NarInfo<String>, Bytes) {
        // To get the NARInfo and NAR bytes we copy the store path to a
        // temporary `file://` cache and read the contents of the files produced
        // by `nix`.

        let cache_dir =
            tempfile::tempdir().expect("failed to create temp cache dir");

        let cache_dir_path = fs::canonicalize(cache_dir.path())
            .expect("failed to canonicalize temp cache dir");

        let cache_url = format!("file://{}", cache_dir_path.display());

        let store_path_str = store_path.to_string();

        self.run(["copy", "--to", &cache_url, &store_path_str]).await;

        let narinfo_path =
            cache_dir_path.join(format!("{}.narinfo", store_path.hash()));

        let narinfo_text = fs::read_to_string(&narinfo_path)
            .expect("nix copy should have written a .narinfo file");

        let narinfo = narinfo_text
            .parse::<NarInfo<String>>()
            .expect("nix-produced .narinfo should be valid");

        let nar_path = cache_dir_path.join(&narinfo.url);

        let nar_bytes =
            Bytes::from(fs::read(&nar_path).expect("NAR file should exist"));

        (narinfo.with_url(String::new()), nar_bytes)
    }

    async fn contains(&self, store_path: &StorePath) -> bool {
        let mut cmd = nix_command(&self.canonicalized_path);
        cmd.args(["path-info", &store_path.to_string()]);
        let mut cmd = tokio::process::Command::from(cmd);

        let output = cmd
            .output()
            .await
            .unwrap_or_else(|err| panic!("failed to run `{cmd:?}`: {err}"));

        // `nix path-info` prints the path to stdout if it exists,
        // and prints nothing (with a non-zero exit) if it doesn't.
        !output.stdout.is_empty()
    }
}

impl From<SocketAddrV4> for CacheHost {
    #[track_caller]
    fn from(addr: SocketAddrV4) -> Self {
        SocketAddr::V4(addr).into()
    }
}

impl From<SocketAddrV6> for CacheHost {
    #[track_caller]
    fn from(addr: SocketAddrV6) -> Self {
        SocketAddr::V6(addr).into()
    }
}

impl From<SocketAddr> for CacheHost {
    #[track_caller]
    fn from(addr: SocketAddr) -> Self {
        assert!(
            addr.ip().is_loopback(),
            "expected {addr:?} to be a loopback address"
        );
        Self(format!("http://{addr}").parse().expect("valid cache URI"))
    }
}

impl From<url::Url> for CacheHost {
    fn from(url: url::Url) -> Self {
        Self(url)
    }
}

impl TryFrom<&str> for CacheHost {
    type Error = <url::Url as FromStr>::Err;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse::<url::Url>().map(Self)
    }
}

impl From<CacheHost> for url::Url {
    fn from(CacheHost(url): CacheHost) -> Self {
        url
    }
}

fn nix_command(store_path: &Path) -> process::Command {
    let Some(nix_binary) =
        env::var_os("NIX").map(PathBuf::from).or_else(|| {
            env::var_os("PATH").and_then(|path| {
                env::split_paths(&path)
                    .map(|dir| dir.join("nix"))
                    .find(|candidate| candidate.is_file())
            })
        })
    else {
        panic!("failed to find `nix` in $PATH; set $NIX to an absolute path");
    };

    let mut cmd = process::Command::new(nix_binary);

    cmd
        // Clear all environment variables for reproducibility.
        .env_clear()
        // Prevent Nix from reading any configuration files for
        // reproducibility.
        .env("NIX_CONF_DIR", "/dev/null")
        .env("NIX_USER_CONF_FILES", "/dev/null")
        // Enable the `nix` command and flakes.
        .env("NIX_CONFIG", "experimental-features = nix-command flakes")
        // Point Nix to use a temporary directory as the store to avoid
        // interfering with other concurrently-running tests and with the
        // user's Nix store.
        .arg("--store")
        .arg(store_path);

    cmd
}

async fn run_nix_command(
    store_path: &Path,
    args: impl IntoIterator<Item = impl AsRef<str>>,
) -> process::Output {
    /// A `AsRef<str>` -> `AsRef<OsStr>` adapter for command arguments.
    struct Arg<T>(T);

    impl<T: AsRef<str>> AsRef<OsStr> for Arg<T> {
        fn as_ref(&self) -> &OsStr {
            OsStr::new(self.0.as_ref())
        }
    }

    let mut cmd = {
        let mut cmd = nix_command(store_path);
        cmd.args(args.into_iter().map(Arg));
        tokio::process::Command::from(cmd)
    };

    let output = match cmd.output().await {
        Ok(output) => output,
        Err(err) => panic!("failed to run `{:?}`: {err}", cmd),
    };

    if !output.status.success() {
        panic!(
            "`{cmd:?}` failed with status {}: stdout:\n{}\nstderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    output
}

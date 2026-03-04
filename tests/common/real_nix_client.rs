use core::convert::Infallible;
use core::iter;
use core::net::{SocketAddr, SocketAddrV4, SocketAddrV6};
use core::ops::Deref;
use core::str::FromStr;
use std::borrow::Cow;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;
use std::{env, fs, process};

use cache::types::{NixStoreBaseName, NixStorePath};
use either::Either;

use crate::common::{CopyDirection, CopyOptions, NixClient};

/// TODO: docs.
pub struct RealNixClient {
    nix_store: tempfile::TempDir,
}

#[derive(Debug)]
pub struct CacheHost(url::Url);

#[derive(Debug)]
pub struct Installable(String);

impl RealNixClient {
    #[track_caller]
    pub fn add_file(
        &self,
        _file_name: &str,
        _file_contents: &[u8],
    ) -> NixStorePath {
        // let source = tempfile::NamedTempFile::new().unwrap();
        // std::fs::write(source.path(), b"hello world").unwrap();
        todo!()
    }

    #[track_caller]
    pub fn new() -> Self {
        Self {
            nix_store: tempfile::tempdir()
                .expect("failed to create temporary directory"),
        }
    }

    fn command(&self) -> process::Command {
        let Some(nix_binary) =
            env::var_os("NIX").map(PathBuf::from).or_else(|| {
                env::var_os("PATH").and_then(|path| {
                    env::split_paths(&path)
                        .map(|dir| dir.join("nix"))
                        .find(|candidate| candidate.is_file())
                })
            })
        else {
            panic!(
                "failed to find `nix` in $PATH; set $NIX to an absolute path"
            );
        };

        let mut cmd = process::Command::new(nix_binary);

        let nix_store_path = fs::canonicalize(self.nix_store.path())
            .expect("failed to canonicalize nix store path");

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
            .arg(nix_store_path);

        cmd
    }

    fn run(
        &self,
        args: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> process::Output {
        /// A `AsRef<str>` -> `AsRef<OsStr>` adapter for command arguments.
        struct Arg<T>(T);

        impl<T: AsRef<str>> AsRef<OsStr> for Arg<T> {
            fn as_ref(&self) -> &OsStr {
                OsStr::new(self.0.as_ref())
            }
        }

        let mut cmd = self.command();
        cmd.args(args.into_iter().map(Arg));

        let output = match cmd.output() {
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
}

impl NixClient for RealNixClient {
    type CacheHost = CacheHost;
    type Installable = Installable;
    type CopyError = Infallible;

    async fn copy_impl(
        &self,
        installables: Vec<Self::Installable>,
        options: CopyOptions<true, Self>,
    ) -> Result<(), Self::CopyError> {
        self.run(
            iter::once(Either::Right("copy"))
                .chain(options.into_cli_args().map(Either::Left))
                .chain(installables.iter().map(|s| &**s).map(Either::Right)),
        );

        Ok(())
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

impl From<NixStorePath> for Installable {
    fn from(path: NixStorePath) -> Self {
        Self(path.to_string())
    }
}

impl Deref for Installable {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

use core::net::SocketAddr;
use std::io::{BufRead, BufReader};
use std::process;

/// Path to the `miniflare-server` binary (set by `build.rs` via `nix build`).
const MINIFLARE_SERVER_BIN: &str = env!("MINIFLARE_SERVER_BIN");

pub(super) struct MiniflareServer {
    pub(super) addr: SocketAddr,
    child: process::Child,
    #[expect(
        dead_code,
        reason = "kept alive so the directory is cleaned up on drop"
    )]
    state_dir: tempfile::TempDir,
}

impl MiniflareServer {
    /// Spawns a `miniflare-server` process which gets killed when the
    /// returned [`MiniflareServer`] is dropped.
    pub(super) fn spawn() -> Self {
        let state_dir = tempfile::tempdir()
            .expect("failed to create temp dir for miniflare state");

        let mut child = process::Command::new(MINIFLARE_SERVER_BIN)
            .arg(state_dir.path())
            .stdin(process::Stdio::null())
            .stdout(process::Stdio::piped())
            .stderr(process::Stdio::piped())
            .spawn()
            .expect("failed to spawn miniflare-server");

        // miniflare-server prints the listening port to stdout when ready.
        let stdout = child.stdout.take().expect("stdout was piped");
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();

        reader
            .read_line(&mut line)
            .expect("failed to read from miniflare-server stdout");

        let port: u16 = line.trim().parse().unwrap_or_else(|err| {
            let stderr = child.stderr.take().map(|s| {
                let mut buf = String::new();
                BufReader::new(s).read_line(&mut buf).ok();
                buf
            });
            panic!(
                "failed to parse miniflare-server port from {line:?}: \
                 {err}\nstderr: {stderr:?}"
            )
        });

        Self {
            addr: SocketAddr::from(([127, 0, 0, 1], port)),
            child,
            state_dir,
        }
    }
}

impl Drop for MiniflareServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

use core::net::SocketAddr;
use std::io::{BufRead, BufReader};
use std::process;

pub(super) struct MiniflareServer {
    pub(super) addr: SocketAddr,
    child: process::Child,
}

impl MiniflareServer {
    /// Spawns a `miniflare-server` process which gets killed when the
    /// returned [`MiniflareServer`] is dropped.
    pub(super) fn spawn() -> Self {
        let mut child = process::Command::new("miniflare-server")
            .stdin(process::Stdio::null())
            .stdout(process::Stdio::piped())
            .stderr(process::Stdio::piped())
            .spawn()
            .expect("failed to spawn miniflare-server");

        // miniflare-server prints a JSON line to stdout when ready:
        //   {"event":"listen","port":12345}
        let stdout = child.stdout.take().expect("stdout was piped");
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();

        reader
            .read_line(&mut line)
            .expect("failed to read from miniflare-server stdout");

        let msg: serde_json::Value = serde_json::from_str(line.trim())
            .unwrap_or_else(|err| {
                let stderr = child.stderr.take().map(|s| {
                    let mut buf = String::new();
                    BufReader::new(s).read_line(&mut buf).ok();
                    buf
                });
                panic!(
                    "failed to parse miniflare-server output {line:?}: \
                     {err}\nstderr: {stderr:?}"
                )
            });

        assert_eq!(
            msg["event"].as_str(),
            Some("listen"),
            "unexpected miniflare-server message: {msg}"
        );

        let port = msg["port"].as_u64().unwrap_or_else(|| {
            panic!("missing port in miniflare-server message: {msg}")
        }) as u16;

        Self { addr: SocketAddr::from(([127, 0, 0, 1], port)), child }
    }
}

impl Drop for MiniflareServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

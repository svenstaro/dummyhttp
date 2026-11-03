use assert_cmd::prelude::*;
use port_check::{free_local_port, is_port_reachable};
use std::ffi::OsStr;
use std::process::{Child, Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

/// Error type used by tests
pub type Error = Box<dyn std::error::Error>;

#[derive(Debug)]
pub struct DummyhttpProcess {
    pub child: Child,
    pub url: String,
}

impl Drop for DummyhttpProcess {
    fn drop(&mut self) {
        if let Err(e) = self.child.kill() {
            eprintln!("WARN: {}", e);
        }
    }
}

impl DummyhttpProcess {
    /// Get a Dummyhttp instance on a free port.
    pub fn new<I, S>(args: I) -> Result<Self, Error>
    where
        I: IntoIterator<Item = S> + Clone + std::fmt::Debug,
        S: AsRef<OsStr> + PartialEq + From<&'static str>,
    {
        let mut last_err = None;
        for _ in 0..3 {
            let port = free_local_port()
                .expect("Couldn't find a free local port")
                .to_string();

            let mut child = Command::cargo_bin("dummyhttp")?
                .arg("-p")
                .arg(&port)
                .args(args.clone())
                .stdout(Stdio::piped())
                .spawn()?;

            // Wait a max of 1s for the port to become available.
            let start_wait = Instant::now();
            let mut reachable = false;
            while start_wait.elapsed().as_secs() < 1 {
                if is_port_reachable(format!("127.0.0.1:{}", port)) {
                    reachable = true;
                    break;
                }
                // If the child exited early (e.g. port bind failure), stop waiting.
                if let Ok(Some(status)) = child.try_wait() {
                    last_err = Some(format!("child exited early with status: {}", status));
                    break;
                }
                sleep(Duration::from_millis(50));
            }

            if reachable {
                let proto = if args.clone().into_iter().any(|x| x == "--tls-cert".into()) {
                    "https".to_string()
                } else {
                    "http".to_string()
                };
                let url = format!("{proto}://127.0.0.1:{port}", proto = proto, port = port);

                return Ok(Self { child, url });
            }

            let _ = child.kill();
        }

        Err(format!(
            "Failed to start dummyhttp after 3 attempts. Last error: {:?}",
            last_err
        )
        .into())
    }
}

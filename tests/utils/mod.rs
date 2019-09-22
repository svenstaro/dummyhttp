use assert_cmd::prelude::*;
use port_check::free_local_port;
use std::ffi::OsStr;
use std::process::{Child, Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

/// Error type used by tests
pub type Error = Box<dyn std::error::Error>;

pub struct DummyhttpProcess {
    pub child: Child,
    pub port: String,
    pub url: String,
}

impl Drop for DummyhttpProcess {
    fn drop(&mut self) {
        if let Err(e) = self.child.kill() {
            eprintln!("WARN: {}", e);
        }
    }
}

#[allow(dead_code)]
impl DummyhttpProcess {
    /// Get a Dummyhttp instance on a free port.
    pub fn new<I, S>(args: I) -> Result<DummyhttpProcess, Error>
    where
        I: IntoIterator<Item = S> + Clone + std::fmt::Debug,
        S: AsRef<OsStr> + PartialEq + From<&'static str>,
    {
        let port = free_local_port()
            .expect("Couldn't find a free local port")
            .to_string();

        let child = Command::cargo_bin("dummyhttp")?
            .arg("-p")
            .arg(&port)
            .args(args.clone())
            .stdout(Stdio::piped())
            .spawn()?;

        // This isn't a great way of making sure that Dummyhttp is ready to serve connections
        // but I can't be bothered to do this properly right now.
        sleep(Duration::from_secs(1));

        let proto = if args.into_iter().any(|x| x == "--cert".into()) {
            "https".to_string()
        } else {
            "http".to_string()
        };
        let url = format!("{proto}://localhost:{port}", proto = proto, port = port);

        Ok(DummyhttpProcess { child, port, url })
    }
}

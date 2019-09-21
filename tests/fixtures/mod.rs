use port_check::free_local_port;
use rstest::fixture;

/// Error type used by tests
pub type Error = Box<dyn std::error::Error>;

/// Get a free port.
#[fixture]
#[allow(dead_code)]
pub fn port() -> u16 {
    free_local_port().expect("Couldn't find a free local port")
}

mod utils;

use http::StatusCode;
use utils::{DummyhttpProcess, Error};

/// We can connect to a secured connection.
#[test]
fn tls_works() -> Result<(), Error> {
    let dh = DummyhttpProcess::new(vec![
        "--cert",
        "tests/data/cert.pem",
        "--key",
        "tests/data/key.pem",
    ])?;

    let client = reqwest::ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .build()?;
    let mut resp = client.get(&dh.url).send()?;

    assert_eq!(resp.text()?, "dummyhttp\n");
    assert_eq!(resp.status(), StatusCode::OK);

    Ok(())
}

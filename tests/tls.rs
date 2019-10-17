mod utils;

use assert_cmd::prelude::*;
use http::StatusCode;
use std::process::Command;
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

/// Wrong paths for cert or key throw errors.
#[test]
fn wrong_path() -> Result<(), Error> {
    Command::cargo_bin("dummyhttp")?
        .args(&["--cert", "wrong", "--key", "tests/data/key.pem"])
        .assert()
        .failure()
        .stdout("lol");

    Ok(())
}

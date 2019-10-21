mod utils;

use assert_cmd::prelude::*;
use http::StatusCode;
use std::process::Command;
use utils::{DummyhttpProcess, Error};
use predicates::str::contains;

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

/// Wrong path for cert throws error.
#[test]
fn wrong_path_cert() -> Result<(), Error> {
    Command::cargo_bin("dummyhttp")?
        .args(&["--cert", "wrong", "--key", "tests/data/key.pem"])
        .assert()
        .failure()
        .stderr(contains("Error: Failed to load certificate file 'wrong'"))
        .stderr(contains("No such file or directory"));

    Ok(())
}

/// Wrong paths for key throws errors.
#[test]
fn wrong_path_key() -> Result<(), Error> {
    Command::cargo_bin("dummyhttp")?
        .args(&["--cert", "tests/data/cert.pem", "--key", "wrong"])
        .assert()
        .failure()
        .stderr(contains("Error: Failed to load key file 'wrong'"))
        .stderr(contains("No such file or directory"));

    Ok(())
}

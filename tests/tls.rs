mod utils;

use assert_cmd::prelude::*;
use http::StatusCode;
use predicates::str::contains;
use reqwest::blocking::ClientBuilder;
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

    let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .build()?;
    let resp = client.get(&dh.url).send()?;

    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(resp.text()?, "dummyhttp\n");

    Ok(())
}

/// Wrong path for cert throws error.
#[test]
fn wrong_path_cert() -> Result<(), Error> {
    Command::cargo_bin("dummyhttp")?
        .args(&["--cert", "wrong", "--key", "tests/data/key.pem"])
        .assert()
        .failure()
        .stderr(contains("Error: Failed to load certificate file 'wrong'"));

    Ok(())
}

/// Wrong paths for key throws errors.
#[test]
fn wrong_path_key() -> Result<(), Error> {
    Command::cargo_bin("dummyhttp")?
        .args(&["--cert", "tests/data/cert.pem", "--key", "wrong"])
        .assert()
        .failure()
        .stderr(contains("Error: Failed to load key file 'wrong'"));

    Ok(())
}

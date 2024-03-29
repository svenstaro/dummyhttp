mod utils;

use assert_cmd::prelude::*;
use clap::{crate_name, crate_version};
use reqwest::blocking::Client;
use rstest::rstest;
use std::io::Read;
use std::process::Command;
use utils::{DummyhttpProcess, Error};

/// Show help and exit.
#[test]
fn help_shows() -> Result<(), Error> {
    Command::cargo_bin("dummyhttp")?
        .arg("--help")
        .assert()
        .success();

    Ok(())
}

/// Show version and exit.
#[test]
fn version_shows() -> Result<(), Error> {
    Command::cargo_bin("dummyhttp")?
        .arg("-V")
        .assert()
        .success()
        .stdout(format!("{} {}\n", crate_name!(), crate_version!()));

    Ok(())
}

/// If provided with no options, we're shown some basic information on stdout.
#[test]
fn has_some_output_by_default() -> Result<(), Error> {
    let mut dh = DummyhttpProcess::new(Vec::<String>::new())?;

    reqwest::blocking::get(&dh.url)?.error_for_status()?;

    dh.child.kill()?;
    let mut output = String::new();
    dh.child
        .stdout
        .as_mut()
        .unwrap()
        .read_to_string(&mut output)?;

    assert!(output.contains("dummyhttp"));
    assert!(output.contains("GET"));

    Ok(())
}

/// If we pass --quiet, we get no output.
#[test]
fn has_quiet_output() -> Result<(), Error> {
    let mut dh = DummyhttpProcess::new(vec!["--quiet"])?;

    reqwest::blocking::get(&dh.url)?.error_for_status()?;

    dh.child.kill()?;
    let mut output = String::new();
    dh.child
        .stdout
        .as_mut()
        .unwrap()
        .read_to_string(&mut output)?;

    dbg!(&output);
    assert!(output.is_empty());

    Ok(())
}

/// If we pass -v/--verbose, we get a ton of pretty output.
#[rstest(flag, case::v("-v"), case::verbose("--verbose"))]
fn has_verbose_output(flag: &'static str) -> Result<(), Error> {
    let mut dh = DummyhttpProcess::new(vec![flag, "-b", "teststring"])?;

    let client = Client::new();
    client
        .post(&dh.url)
        .body("some body")
        .send()?
        .error_for_status()?;

    dh.child.kill()?;
    let mut output = String::new();
    dh.child
        .stdout
        .as_mut()
        .unwrap()
        .read_to_string(&mut output)?;

    assert!(output.contains("Incoming request"));
    assert!(output.contains("Outgoing response"));
    assert!(!output.contains("teststring"));
    assert!(!output.contains("some body"));

    Ok(())
}

/// If we pass -vv, we also get body output.
#[test]
fn has_very_verbose_output() -> Result<(), Error> {
    let mut dh = DummyhttpProcess::new(vec!["-vv", "-b", "teststring"])?;

    let client = Client::new();
    client
        .post(&dh.url)
        .body("some body")
        .send()?
        .error_for_status()?;

    dh.child.kill()?;
    let mut output = String::new();
    dh.child
        .stdout
        .as_mut()
        .unwrap()
        .read_to_string(&mut output)?;

    assert!(output.contains("Incoming request"));
    assert!(output.contains("Outgoing response"));
    assert!(output.contains("teststring"));
    assert!(output.contains("some body"));

    Ok(())
}

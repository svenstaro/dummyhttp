mod utils;

use assert_cmd::prelude::*;
use std::io::Read;
use std::process::Command;
use structopt::clap::{crate_name, crate_version};
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

    reqwest::get(&dh.url)?.error_for_status()?;

    dh.child.kill()?;
    let mut output = String::new();
    dh.child
        .stdout
        .as_mut()
        .unwrap()
        .read_to_string(&mut output)?;

    dbg!(&output);
    assert!(output.find("Starting server").is_some());
    assert!(output.find("Connection from").is_some());

    Ok(())
}

/// If we pass --quiet, we get no output.
#[test]
fn has_quiet_output() -> Result<(), Error> {
    let mut dh = DummyhttpProcess::new(vec!["--quiet"])?;

    reqwest::get(&dh.url)?.error_for_status()?;

    dh.child.kill()?;
    let mut output = String::new();
    dh.child
        .stdout
        .as_mut()
        .unwrap()
        .read_to_string(&mut output)?;

    assert!(output.is_empty());

    Ok(())
}

/// If we pass --verbose, we get a ton of pretty output.
#[test]
fn has_verbose_output() -> Result<(), Error> {
    let mut dh = DummyhttpProcess::new(vec!["--verbose"])?;

    reqwest::get(&dh.url)?.error_for_status()?;

    dh.child.kill()?;
    let mut output = String::new();
    dh.child
        .stdout
        .as_mut()
        .unwrap()
        .read_to_string(&mut output)?;

    assert!(output.find("Incoming request").is_some());
    assert!(output.find("Outgoing response").is_some());

    Ok(())
}

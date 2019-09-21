mod fixtures;

use assert_cmd::prelude::*;
use structopt::clap::{crate_name, crate_version};
use fixtures::Error;
use std::process::Command;

#[test]
/// Show help and exit.
fn help_shows() -> Result<(), Error> {
    Command::cargo_bin("dummyhttp")?
        .arg("--help")
        .assert()
        .success();

    Ok(())
}

#[test]
/// Show version and exit.
fn version_shows() -> Result<(), Error> {
    Command::cargo_bin("dummyhttp")?
        .arg("-V")
        .assert()
        .success()
        .stdout(format!("{} {}\n", crate_name!(), crate_version!()));

    Ok(())
}

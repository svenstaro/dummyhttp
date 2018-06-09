# dummyhttp [![Build Status](https://travis-ci.com/svenstaro/dummyhttp.svg?branch=master)](https://travis-ci.com/svenstaro/dummyhttp) [![Crates.io](https://img.shields.io/crates/v/dummyhttp.svg)](https://crates.io/crates/dummyhttp) [![license](http://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/svenstaro/dummyhttp/blob/master/LICENSE)

**A super simple HTTP server that replies a fixed body with a fixed response code**

This is a simple, small, self-contained, cross-platform CLI tool for debugging
and testing. It allows you to return arbitrary HTTP responses.

## How to use

### Running with no arguments always returns 200 on all interfaces at port 8080

    dummyhttp
    curl localhost:8080
    # < HTTP/1.1 200 OK
    # < content-length: 10
    # < date: Sat, 09 Jun 2018 13:56:14 GMT
    # <
    # dummyhttp

### Always emit 400 Bad Request

    dummyhttp -c 400
    curl localhost:8080
    # < HTTP/1.1 400 Bad Request
    # < content-length: 10
    # < date: Sat, 09 Jun 2018 13:57:53 GMT
    # <
    # dummyhttp

### Always return a certain string

    dummyhttp -b "Hello World"
    curl localhost:8080
    < HTTP/1.1 200 OK
    < content-length: 12
    < date: Sat, 09 Jun 2018 13:58:57 GMT
    <
    Hello World

## How to install

**On Linux**: Download `dummyhttp-linux` from [the releases page](https://github.com/svenstaro/dummyhttp/releases) and run

    chmod +x dummyhttp-linux
    ./dummyhttp-linux

**On OSX**: Download `dummyhttp-osx` from [the releases page](https://github.com/svenstaro/dummyhttp/releases) and run

    chmod +x dummyhttp-osx
    ./dummyhttp-osx

**On Windows**: Download `dummyhttp-win.exe` from [the releases page](https://github.com/svenstaro/dummyhttp/releases) and run

    dummyhttp-win.exe

**With Cargo**: If you have a somewhat recent version of Rust and Cargo installed, you can run

    cargo install dummyhttp
    dummyhttp

## Full options

    dummyhttp 0.1.0
    Sven-Hendrik Haase <svenstaro@gmail.com>
    Super simple HTTP server that replies a fixed body with a fixed response code

    USAGE:
        dummyhttp [FLAGS] [OPTIONS]

    FLAGS:
        -h, --help       Prints help information
        -q, --quiet      Be quiet
        -V, --version    Prints version information

    OPTIONS:
        -b, --body <body>       HTTP body to send [default: dummyhttp]
        -c, --code <code>       HTTP status code to send [default: 200]
        -i, --if <interface>    Interface to listen on [default: 0.0.0.0]
        -p, --port <port>       Port to use [default: 8080]

## Releasing

This is mostly a note for me on how to release this thing:

- Update version in `Cargo.toml`.
- `git commit` and `git tag -s`, `git push`.
- `cargo publish`
- Releases will automatically be deployed by Travis.

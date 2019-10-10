# dummyhttp

[![GitHub Actions Workflow](https://github.com/svenstaro/dummyhttp/workflows/Build/badge.svg)](https://github.com/svenstaro/dummyhttp/actions)
[![Docker Cloud Build Status](https://img.shields.io/docker/cloud/build/svenstaro/dummyhttp)](https://cloud.docker.com/repository/docker/svenstaro/dummyhttp/)
[![AUR](https://img.shields.io/aur/version/dummyhttp.svg)](https://aur.archlinux.org/packages/dummyhttp/)
[![Crates.io](https://img.shields.io/crates/v/dummyhttp.svg)](https://crates.io/crates/dummyhttp)
[![dependency status](https://deps.rs/repo/github/svenstaro/dummyhttp/status.svg)](https://deps.rs/repo/github/svenstaro/dummyhttp)
[![license](http://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/svenstaro/dummyhttp/blob/master/LICENSE)

**A super simple HTTP server that replies with a fixed body and a fixed response code**

This is a simple, small, self-contained, cross-platform CLI tool for debugging
and testing. It allows you to return arbitrary HTTP responses and log incoming request data.

## How to use

### Log all incoming request data

    dummyhttp --verbose
    curl -X POST localhost:8080 -d hi
    # ┌─Incoming request
    # │ POST / HTTP/1.1
    # │ Accept: */*
    # │ Content-Length: 2
    # │ Host: localhost:8080
    # │ User-Agent: curl/7.66.0
    # │ Content-Type: application/x-www-form-urlencoded
    # │ Body:
    # hi

Example with color:

![Pretty log](pretty_log.png)

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
    # < HTTP/1.1 200 OK
    # < content-length: 12
    # < date: Sat, 09 Jun 2018 13:58:57 GMT
    # <
    # Hello World

### Return a specific header

    dummyhttp -b "Hello World" -H application/json
    curl localhost:8080
    # < HTTP/1.1 200 OK
    # < content-length: 10
    # < content-type: application/json
    # < date: Thu, 14 Jun 2018 11:10:14 GMT
    # <
    # Hello World

## How to install

**On Linux**: Download `dummyhttp-linux-amd64` from [the releases page](https://github.com/svenstaro/dummyhttp/releases) and run

    chmod +x dummyhttp-linux-amd64
    ./dummyhttp-linux-amd64

**On macOS**: Download `dummyhttp-macos-amd64` from [the releases page](https://github.com/svenstaro/dummyhttp/releases) and run

    chmod +x dummyhttp-macos-amd64
    ./dummyhttp-macos-amd64

**On Windows**: Download `dummyhttp-windows-amd64.exe` from [the releases page](https://github.com/svenstaro/dummyhttp/releases) and run

    dummyhttp-windows-amd64.exe

**With Cargo**: If you have a somewhat recent version of Rust and Cargo installed, you can run

    cargo install dummyhttp
    dummyhttp

## Full options

    dummyhttp 0.4.0
    Sven-Hendrik Haase <svenstaro@gmail.com>
    Super simple HTTP server that replies with a fixed body and a fixed response code

    USAGE:
        dummyhttp [FLAGS] [OPTIONS]

    FLAGS:
            --help       Prints help information
        -q, --quiet      Be quiet (log nothing)
        -V, --version    Prints version information
        -v, --verbose    Be verbose (log data of incoming and outgoing requests)

    OPTIONS:
        -b, --body <body>                   HTTP body to send [default: dummyhttp]
        -c, --code <code>                   HTTP status code to send [default: 200]
        -h, --headers <headers>...          Headers to send (format: key:value)
        -i, --interfaces <interfaces>...    Interface to bind to [default: 0.0.0.0]
        -p, --port <port>                   Port on which to listen [default: 8080]
            --cert <tls-cert>               TLS cert to use
            --key <tls-key>                 TLS key to use

## Releasing

This is mostly a note for me on how to release this thing:

- Update version in `Cargo.toml` and `README.md`
- `git commit` and `git tag -s`, `git push`
- Run `cargo publish`
- Releases will be built and deployed automatically by GitHub Actions
- Update AUR package

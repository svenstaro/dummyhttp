extern crate actix_web;
extern crate simplelog;
#[macro_use] extern crate clap;

use actix_web::http::StatusCode;
use actix_web::{Responder, server, App, middleware, HttpRequest, HttpResponse, Result};
use simplelog::{TermLogger, LevelFilter, Config};
use std::net::IpAddr;

#[derive(Clone, Debug)]
pub struct DummyhttpConfig {
    quiet: bool,
    port: u16,
    code: u16,
    body: String,
    interface: IpAddr,
}

fn is_valid_port(port: String) -> Result<(), String> {
    port.parse::<u16>().and(Ok(())).or_else(|e| Err(e.to_string()))
}

fn is_valid_status_code(code: String) -> Result<(), String> {
    StatusCode::from_bytes(code.as_bytes()).and(Ok(())).or_else(|e| Err(e.to_string()))
}

fn is_valid_interface(interface: String) -> Result<(), String> {
    interface.parse::<IpAddr>().and(Ok(())).or_else(|e| Err(e.to_string()))
}

pub fn parse_args() -> DummyhttpConfig {
    use clap::{App, Arg};

    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("quiet")
                .short("q")
                .long("quiet")
                .help("Be quiet"),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .help("Port to use")
                .validator(is_valid_port)
                .required(false)
                .default_value("8080")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("code")
                .short("c")
                .long("code")
                .help("HTTP status code to send")
                .validator(is_valid_status_code)
                .required(false)
                .default_value("200")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("body")
                .short("b")
                .long("body")
                .help("HTTP body to send")
                .required(false)
                .default_value("dummyhttp")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("interface")
                .short("i")
                .long("if")
                .help("Interface to listen on")
                .validator(is_valid_interface)
                .required(false)
                .default_value("0.0.0.0")
                .takes_value(true),
        )
        .get_matches();

    let quiet = matches.is_present("quiet");
    let port = matches.value_of("port").unwrap().parse().unwrap();
    let code = matches.value_of("code").unwrap().parse().unwrap();
    let body = matches.value_of("body").unwrap().parse().unwrap();
    let interface = matches.value_of("interface").unwrap().parse().unwrap();

    DummyhttpConfig {
        quiet,
        port,
        code,
        body,
        interface,
    }
}

fn respond(req: HttpRequest<DummyhttpConfig>) -> impl Responder {
    let status_code = StatusCode::from_u16(req.state().code).unwrap();
    HttpResponse::with_body(status_code, format!("{}\n", req.state().body))
}

fn main() {
    let dummyhttp_config = parse_args();

    if !dummyhttp_config.quiet {
        let _ = TermLogger::init(LevelFilter::Info, Config::default());
    }

    let inside_config = dummyhttp_config.clone();
	let server =server::new(
		move || App::with_state(inside_config.clone())
            .middleware(middleware::Logger::default())
            .default_resource(|r| r.f(respond)))
		.bind(format!("{}:{}", &dummyhttp_config.interface, dummyhttp_config.port)).expect("Couldn't bind server")
		.shutdown_timeout(0);

    server.run();
}

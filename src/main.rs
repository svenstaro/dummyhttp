extern crate actix_web;
extern crate simplelog;
#[macro_use]
extern crate clap;

#[macro_use]
extern crate log;
extern crate chrono;

use actix_web::http::{header, StatusCode};
use actix_web::middleware::{Finished, Middleware, Started};
use actix_web::{middleware, server, App, HttpRequest, HttpResponse, Responder, Result};
use chrono::prelude::*;
use simplelog::{Config, LevelFilter, TermLogger};
use std::net::IpAddr;

#[derive(Clone, Debug)]
pub struct DummyhttpConfig {
    quiet: bool,
    verbose: bool,
    port: u16,
    headers: header::HeaderMap,
    code: u16,
    body: String,
    interface: IpAddr,
}

fn is_valid_port(port: String) -> Result<(), String> {
    port.parse::<u16>()
        .and(Ok(()))
        .or_else(|e| Err(e.to_string()))
}

fn is_valid_status_code(code: String) -> Result<(), String> {
    StatusCode::from_bytes(code.as_bytes())
        .and(Ok(()))
        .or_else(|e| Err(e.to_string()))
}

fn is_valid_interface(interface: String) -> Result<(), String> {
    interface
        .parse::<IpAddr>()
        .and(Ok(()))
        .or_else(|e| Err(e.to_string()))
}

fn is_valid_header(header: String) -> Result<(), String> {
    let header: Vec<&str> = header.split(':').collect();
    if header.len() != 2 {
        return Err("Wrong header format".to_string());
    }

    let (header_name, header_value) = (header[0], header[1]);

    let hn = header::HeaderName::from_lowercase(header_name.to_lowercase().as_bytes())
        .map(|_| ())
        .map_err(|e| e.to_string());

    let hv = header::HeaderValue::from_str(header_value)
        .map(|_| ())
        .map_err(|e| e.to_string());

    hn.and(hv)
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
                .help("Be quiet (log nothing)"),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .conflicts_with("quiet")
                .help("Be verbose (log everything)"),
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
            Arg::with_name("header")
                .short("H")
                .long("header")
                .help("Header to send (format: key:value)")
                .validator(is_valid_header)
                .required(false)
                .multiple(true)
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
    let verbose = matches.is_present("verbose");
    let port = matches.value_of("port").unwrap().parse().unwrap();
    let headers = if matches.is_present("header") {
        let headers_raw = matches.values_of("header").unwrap();

        let mut headers = header::HeaderMap::new();
        for header in headers_raw {
            let header_parts: Vec<String> = header.split(':').map(|x| x.to_string()).collect();
            headers.append(
                header::HeaderName::from_lowercase(header_parts[0].to_lowercase().as_bytes())
                    .expect("Invalid header name"),
                header_parts[1].parse().expect("Invalid header value"),
            );
        }
        headers
    } else {
        header::HeaderMap::new()
    };
    let code = matches.value_of("code").unwrap().parse().unwrap();
    let body = matches.value_of("body").unwrap().parse().unwrap();
    let interface = matches.value_of("interface").unwrap().parse().unwrap();

    DummyhttpConfig {
        quiet,
        verbose,
        port,
        headers,
        code,
        body,
        interface,
    }
}

fn respond(req: HttpRequest<DummyhttpConfig>) -> impl Responder {
    let status_code = StatusCode::from_u16(req.state().code).unwrap();
    let mut resp = HttpResponse::with_body(status_code, format!("{}\n", req.state().body));
    resp.headers_mut().extend(req.state().headers.clone());
    resp
}

fn configure_app(app: App<DummyhttpConfig>) -> App<DummyhttpConfig> {
    if app.state().verbose {
        app.middleware(VerboseLogger)
    } else {
        app.middleware(middleware::Logger::default())
    }
}

struct VerboseLogger;
struct StartTime(DateTime<Local>);

impl<S> Middleware<S> for VerboseLogger {
    fn start(&self, req: &mut HttpRequest<S>) -> Result<Started> {
        req.extensions_mut().insert(StartTime(Local::now()));
        Ok(Started::Done)
    }

    fn finish(&self, req: &mut HttpRequest<S>, _resp: &HttpResponse) -> Finished {
        let remote = req.connection_info().remote().unwrap_or("unknown");
        let entry_time = if let Some(entry_time) = req.extensions().get::<StartTime>() {
            entry_time.0.format("[%d/%b/%Y:%H:%M:%S %z]").to_string()
        } else {
            "unknown time".to_string()
        };
        let method_path_line = if req.query_string().is_empty() {
            format!("{} {} {:?}", req.method(), req.path(), req.version())
        } else {
            format!(
                "{} {}?{} {:?}",
                req.method(),
                req.path(),
                req.query_string(),
                req.version()
            )
        };
        let mut incoming_headers = String::new();
        for (hk, hv) in req.clone().headers_mut() {
            incoming_headers.push_str(&format!(
                "> {}: {}\n",
                hk.as_str(),
                hv.to_str().unwrap_or("<unprintable>")
            ));
        }

        let incoming_info = format!(
            "> {method_path_line}\n{headers}",
            method_path_line = method_path_line,
            headers = incoming_headers
        );

        info!(
            "Connection from {remote} at {entry_time}\n{incoming_info}",
            remote = remote,
            entry_time = entry_time,
            incoming_info = incoming_info,
        );
        Finished::Done
    }
}

fn main() {
    let dummyhttp_config = parse_args();

    if !dummyhttp_config.quiet {
        let _ = TermLogger::init(LevelFilter::Info, Config::default());
    }

    let inside_config = dummyhttp_config.clone();
    let server = server::new(move || {
        App::with_state(inside_config.clone())
            .configure(configure_app)
            .default_resource(|r| r.f(respond))
    }).bind(format!(
        "{}:{}",
        &dummyhttp_config.interface, dummyhttp_config.port
    ))
        .expect("Couldn't bind server")
        .shutdown_timeout(0);

    server.run();
}

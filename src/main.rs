use actix_http::httpmessage::HttpMessage;
use actix_service::Service;
use actix_web::http::header::HeaderMap;
use actix_web::http::{header, StatusCode};
use actix_web::web::{self};
use actix_web::App as ActixApp;
use actix_web::HttpResponse;
use actix_web::HttpServer;
use chrono::prelude::*;
use futures::Future;
use log::info;
use simplelog::{Config, LevelFilter, TermLogger, TerminalMode};
use std::io::{Error, ErrorKind};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt)]
#[structopt(
    name = "dummyhttp",
    author,
    about,
    global_settings = &[structopt::clap::AppSettings::ColoredHelp]
)]
pub struct DummyhttpConfig {
    /// Be quiet (log nothing)
    #[structopt(short, long)]
    quiet: bool,

    /// Be verbose (log data of incoming and outgoing requests)
    #[structopt(short, long)]
    verbose: bool,

    /// Port on which to listen
    #[structopt(short, long, default_value = "8080")]
    port: u16,

    /// Headers to send (format: key:value)
    #[structopt(short, long, parse(try_from_str = parse_header))]
    headers: Vec<HeaderMap>,

    /// HTTP status code to send
    #[structopt(short, long, default_value = "200")]
    code: u16,

    /// HTTP body to send
    #[structopt(short, long, default_value = "dummyhttp")]
    body: String,

    /// Interface to bind to
    #[structopt(
        short = "i",
        long = "interfaces",
        parse(try_from_str = parse_interface),
        number_of_values = 1,
        default_value = "0.0.0.0"
    )]
    interfaces: Vec<IpAddr>,
}

/// Checks wether an interface is valid, i.e. it can be parsed into an IP address
fn parse_interface(src: &str) -> Result<IpAddr, std::net::AddrParseError> {
    src.parse::<IpAddr>()
}

fn parse_header(header: &str) -> Result<HeaderMap, String> {
    let header: Vec<&str> = header.split(':').collect();
    if header.len() != 2 {
        return Err("Wrong header format".to_string());
    }

    let (header_name, header_value) = (header[0], header[1]);

    let hn = header::HeaderName::from_lowercase(header_name.to_lowercase().as_bytes())
        .map_err(|e| e.to_string())?;

    let hv = header::HeaderValue::from_str(header_value).map_err(|e| e.to_string())?;

    let mut map = HeaderMap::new();
    map.insert(hn, hv);
    Ok(map)
}

fn default_response(data: web::Data<DummyhttpConfig>) -> HttpResponse {
    let status_code = StatusCode::from_u16(data.code).unwrap();
    let mut resp = HttpResponse::with_body(status_code, format!("{}\n", data.body).into());

    let mut headers = header::HeaderMap::new();
    for header in &data.headers {
        // There should only be a single Header in each HeaderMap that we parsed from the command
        // line arguments.
        let val = header.iter().next().unwrap();
        headers.insert(val.0.clone(), val.1.clone());
    }

    *resp.headers_mut() = headers;
    resp
}

struct StartTime(DateTime<Local>);

fn main() -> Result<(), std::io::Error> {
    let dummyhttp_config = DummyhttpConfig::from_args();

    if !dummyhttp_config.quiet {
        let _ = TermLogger::init(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::default(),
        );
    }

    let interfaces = dummyhttp_config
        .interfaces
        .iter()
        .map(|&interface| {
            if interface == IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)) {
                // If the interface is 0.0.0.0, we'll change it to 127.0.0.1 so that clicking the link will
                // also work on Windows. Why can't Windows interpret 0.0.0.0?
                "127.0.0.1".to_string()
            } else if interface.is_ipv6() {
                // If the interface is IPv6 then we'll print it with brackets so that it is clickable.
                format!("[{}]", interface)
            } else {
                format!("{}", interface)
            }
        })
        .collect::<Vec<String>>();

    let socket_addresses = interfaces
        .iter()
        .map(|interface| {
            format!(
                "{interface}:{port}",
                interface = &interface,
                port = dummyhttp_config.port,
            )
            .parse::<SocketAddr>()
        })
        .collect::<Result<Vec<SocketAddr>, _>>()
        .map_err(|e| Error::new(ErrorKind::Other, e))?;

    let dummyhttp_config_cloned = dummyhttp_config.clone();
    let server = HttpServer::new(move || {
        ActixApp::new()
            .data(dummyhttp_config_cloned.clone())
            .wrap_fn(|req, srv| {
                req.extensions_mut().insert(StartTime(Local::now()));
                srv.call(req).map(|res| {
                    let req_ = res.request();
                    let app_state: &DummyhttpConfig =
                        req_.app_data().expect("There should be data here");
                    if app_state.verbose {
                        let conn_info = req_.connection_info();
                        let remote = conn_info.remote().unwrap_or("unknown");
                        let entry_time =
                            if let Some(entry_time) = req_.extensions().get::<StartTime>() {
                                entry_time.0.format("[%d/%b/%Y:%H:%M:%S %z]").to_string()
                            } else {
                                "unknown time".to_string()
                            };
                        let method_path_line = if req_.query_string().is_empty() {
                            format!("{} {} {:?}", req_.method(), req_.path(), req_.version())
                        } else {
                            format!(
                                "{} {}?{} {:?}",
                                req_.method(),
                                req_.path(),
                                req_.query_string(),
                                req_.version()
                            )
                        };
                        let mut incoming_headers = String::new();
                        for (hk, hv) in req_.headers() {
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
                    }
                    res
                })
            })
            .default_service(web::route().to(default_response))
    })
    .bind(socket_addresses.as_slice())
    .expect("Couldn't bind server")
    .shutdown_timeout(0);

    server.run()
}

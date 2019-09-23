use actix_service::{Service, Transform};
use actix_web::error::PayloadError;
use actix_web::http::{header, StatusCode};
use actix_web::web::{self};
use actix_web::App as ActixApp;
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error};
use actix_web::{HttpMessage, HttpResponse, HttpServer};
use bytes::BytesMut;
use chrono::prelude::*;
use colored_json::prelude::*;
use futures::future::{ok, FutureResult};
use futures::stream::Stream;
use futures::{Future, Poll};
use inflector::Inflector;
use log::info;
use rustls::{NoClientAuth, ServerConfig};
use simplelog::{Config, LevelFilter, TermLogger, TerminalMode};
use std::cell::RefCell;
use std::io::Error as IoError;
use std::io::ErrorKind as IoErrorKind;
use std::net::SocketAddr;
use std::rc::Rc;
use structopt::StructOpt;
use yansi::Paint;

use crate::args::DummyhttpConfig;
use crate::tls_util::{load_cert, load_private_key};

mod args;
mod tls_util;

/// dummyhttp only has a single response and this is it :)
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

pub struct Logging;

impl<S, B> Transform<S> for Logging
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = LoggingMiddleware<S>;
    type Future = FutureResult<Self::Transform, Self::InitError>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(LoggingMiddleware {
            service: Rc::new(RefCell::new(service)),
        })
    }
}

pub struct LoggingMiddleware<S> {
    service: Rc<RefCell<S>>,
}

impl<S, B> Service for LoggingMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Box<dyn Future<Item = Self::Response, Error = Self::Error>>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        self.service.poll_ready()
    }

    fn call(&mut self, mut req: ServiceRequest) -> Self::Future {
        req.extensions_mut().insert(StartTime(Local::now()));
        let mut svc = self.service.clone();

        Box::new(
            req.take_payload()
                .fold(BytesMut::new(), move |mut body, chunk| {
                    body.extend_from_slice(&chunk);
                    Ok::<_, PayloadError>(body)
                })
                .map_err(|e| e.into())
                .and_then(move |bytes| {
                    svc.call(req).and_then(move |res| {
                        let req_ = res.request();
                        let app_state: &DummyhttpConfig =
                            req_.app_data().expect("There should be data here");

                        let conn_info = req_.connection_info().clone();
                        let remote = conn_info.remote().unwrap_or("unknown");
                        let entry_time =
                            if let Some(entry_time) = req_.extensions().get::<StartTime>() {
                                entry_time.0.format("[%d/%b/%Y:%H:%M:%S %z]").to_string()
                            } else {
                                "unknown time".to_string()
                            };
                        if app_state.verbose {
                            let path_query = if req_.query_string().is_empty() {
                                req_.path().to_string()
                            } else {
                                format!("{path}?{query}",
                                    path = req_.path(),
                                    query = req_.query_string(),
                                )
                            };
                            let method_path_version_line = format!("{method} {path_query} {http}/{version}",
                                method = Paint::green(req_.method()),
                                path_query = Paint::cyan(path_query).underline(),
                                http = Paint::blue("HTTP"),
                                version = Paint::blue(format!("{:?}", req_.version()).split('/').nth(1).unwrap_or("unknown")),
                            );

                            let mut incoming_headers_vec = vec![];
                            for (hk, hv) in req_.headers() {
                                incoming_headers_vec.push(format!(
                                    "{deco} {key}: {value}",
                                    deco = Paint::green("│").bold(),
                                    key = Paint::cyan(Inflector::to_train_case(hk.as_str())),
                                    value = hv.to_str().unwrap_or("<unprintable>")
                                ));
                            }
                            incoming_headers_vec.sort();
                            let incoming_headers = incoming_headers_vec.join("\n");

                            let req_info = format!(
                                "{deco} {method_path_line}\n{headers}",
                                deco = Paint::green("│").bold(),
                                method_path_line = method_path_version_line,
                                headers = incoming_headers
                            );

                            let body = String::from_utf8_lossy(&bytes);
                            let body_text_req = if body.is_empty() {
                                "".to_string()
                            } else {
                                let body_formatted = if let Some(content_type) = req_.headers().get(header::CONTENT_TYPE) {
                                    if content_type == header::HeaderValue::from_static("application/json") {
                                        serde_json::from_str::<serde_json::Value>(&body)
                                            .and_then(|loaded_json| serde_json::to_string_pretty(&loaded_json))
                                            .and_then(|pretty_json| pretty_json.to_colored_json_auto())?
                                    } else {
                                        body.to_string()
                                    }
                                } else {
                                    body.to_string()
                                };
                                let body_formatted = body_formatted
                                    .lines()
                                    .map(|line| format!("{deco} {line}", deco = Paint::green("│").bold(), line = line))
                                    .collect::<Vec<_>>()
                                    .join("\n");
                                format!(
                                    "\n{deco} {body}\n{body_formatted}",
                                    deco = Paint::green("│").bold(),
                                    body = Paint::yellow("Body:"),
                                    body_formatted = body_formatted,
                                )
                            };

                            let status_line = format!(
                                "{http}/{version} {status_code} {status_text}",
                                http = Paint::blue("HTTP"),
                                version = Paint::blue(
                                    format!("{:?}", res.response().head().version)
                                    .split('/')
                                    .nth(1)
                                    .unwrap_or("unknown")
                                ),
                                status_code = Paint::blue(res.status().as_u16()),
                                status_text = Paint::cyan(
                                    res.status().canonical_reason().unwrap_or("")
                                ),
                            );

                            let mut outgoing_headers_vec = vec![];
                            for (hk, hv) in res.headers() {
                                outgoing_headers_vec.push(format!(
                                        "{deco} {key}: {value}",
                                        deco = Paint::red("│").bold(),
                                        key =
                                        Paint::cyan(Inflector::to_train_case(hk.as_str())),
                                        value = hv.to_str().unwrap_or("<unprintable>")
                                ));
                            }
                            outgoing_headers_vec.sort();
                            let outgoing_headers = outgoing_headers_vec.join("\n");

                            let res_info = format!(
                                "{deco} {status_line}\n{headers}",
                                deco = Paint::red("│").bold(),
                                status_line = status_line,
                                headers = outgoing_headers
                            );

                            info!(
                                "Connection from {remote} at {entry_time}\n{req_banner}\n{req_info}{body_text_req}\n{res_banner}\n{res_info}",
                                req_banner = Paint::green("┌─Incoming request").bold(),
                                remote = remote,
                                entry_time = entry_time,
                                req_info = req_info,
                                body_text_req = body_text_req,
                                res_banner = Paint::red("┌─Outgoing response").bold(),
                                res_info = res_info,
                            );
                        } else {
                            info!(
                                "Connection from {remote} at {entry_time}",
                                remote = remote,
                                entry_time = entry_time,
                            );
                        }

                        Ok(res)
                    })
                }),
        )
    }
}

fn main() -> std::io::Result<()> {
    #[cfg(windows)]
    Paint::enable_windows_ascii();
    #[cfg(windows)]
    colored_json::enable_ansi_support();

    let args = DummyhttpConfig::from_args();

    if !args.quiet {
        let _ = TermLogger::init(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::default(),
        );
    }

    let interfaces = args
        .interfaces
        .iter()
        .map(|&interface| {
            if interface.is_ipv6() {
                // If the interface is IPv6 then we'll print it with brackets so that it is
                // clickable and also because for some reason, actix-web won't it otherwise.
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
                port = args.port,
            )
            .parse::<SocketAddr>()
        })
        .collect::<Result<Vec<SocketAddr>, _>>()
        .map_err(|e| IoError::new(IoErrorKind::Other, e))?;

    let dummyhttp_config_cloned = args.clone();
    let mut server = HttpServer::new(move || {
        ActixApp::new()
            .data(dummyhttp_config_cloned.clone())
            .wrap(Logging)
            .default_service(web::route().to(default_response))
    });
    // TODO: This conditional is kinda dirty but it'll have to do until we have stable if let chains.
    if args.tls_cert.is_some() && args.tls_key.is_some() {
        let tls_cert = args.tls_cert.unwrap();
        let tls_key = args.tls_key.unwrap();

        let mut config = ServerConfig::new(NoClientAuth::new());
        let cert_file = load_cert(&tls_cert)?;
        let key_file = load_private_key(&tls_key)?;
        config
            .set_single_cert(cert_file, key_file)
            .map_err(|e| IoError::new(IoErrorKind::Other, e.to_string()))?;
        server = server.bind_rustls(socket_addresses.as_slice(), config)?;
    } else {
        server = server.bind(socket_addresses.as_slice())?;
    }
    server.system_exit().run()
}

use std::{collections::HashMap, net::SocketAddr};

#[cfg(not(feature = "tls"))]
use anyhow::Result;
#[cfg(feature = "tls")]
use anyhow::{Context, Result};

use axum::{
    body::{Body, Bytes},
    extract::ConnectInfo,
    handler::Handler,
    http::{HeaderValue, Request, StatusCode, Uri},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    Extension, Router,
};

#[cfg(feature = "tls")]
use axum_server::tls_rustls::RustlsConfig;
use chrono::Local;
use clap::{crate_version, Parser};
use colored::*;
use colored_json::ToColoredJson;
use hyper::{header::CONTENT_TYPE, HeaderMap};
use inflector::Inflector;

use crate::args::Args;

mod args;

pub fn template_uuid(_args: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
    Ok(tera::to_value(uuid::Uuid::new_v4().to_string()).unwrap())
}

pub fn template_lorem(args: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
    let n_words = args
        .get("words")
        .and_then(|w| w.as_u64())
        .ok_or_else(|| tera::Error::from("Failed to template lorem"))?;
    Ok(tera::to_value(lipsum::lipsum(n_words as usize)).unwrap())
}

/// dummyhttp only has a single response and this is it :)
async fn dummy_response(_uri: Uri, Extension(args): Extension<Args>) -> impl IntoResponse {
    let status_code = StatusCode::from_u16(args.code).unwrap();

    let mut headers = HeaderMap::new();
    for header in &args.headers {
        let val = header.iter().next().unwrap();
        headers.insert(val.0.clone(), val.1.clone());
    }

    // Manually insert a Date header here so that our log print will catch it later on as the
    // date is inserted _after_ logging otherwise.
    let time = Local::now();
    headers.insert("date", HeaderValue::from_str(&time.to_rfc2822()).unwrap());

    // Render body as Tera template.
    let mut tera = tera::Tera::default();
    tera.register_function("uuid", template_uuid);
    tera.register_function("lorem", template_lorem);
    let rendered_body = tera.render_str(&args.body, &tera::Context::new()).unwrap();
    (status_code, headers, rendered_body)
}

async fn print_request_response(
    req: Request<Body>,
    next: Next<Body>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let args = req.extensions().get::<Args>().unwrap().clone();
    let ConnectInfo(peer_info) = *req.extensions().get::<ConnectInfo<SocketAddr>>().unwrap();
    let method = req.method().to_string();
    let uri = req.uri().to_string();
    let http_version = format!("{:?}", req.version())
        .split('/')
        .nth(1)
        .unwrap_or("unknown")
        .to_string();
    let req_headers = req.headers().clone();

    let (parts, body) = req.into_parts();
    let bytes = buffer_and_print("request", body).await?;
    let bytes2 = bytes.clone();
    let req = Request::from_parts(parts, Body::from(bytes));

    let resp = next.run(req).await;

    let time = Local::now().format("%Y-%M-%d %H:%M:%S").to_string();

    let connect_line = format!(
        "{time} {peer_info} {method} {uri} {http}/{version}",
        time = time.yellow(),
        peer_info = peer_info.to_string().bold(),
        method = method.green(),
        uri = uri.cyan().underline(),
        http = "HTTP".blue(),
        version = http_version.blue(),
    );
    if args.verbose >= 1 {
        let method_path_version_line = format!(
            "{method} {uri} {http}/{version}",
            method = method.green(),
            uri = uri.cyan().underline(),
            http = "HTTP".blue(),
            version = http_version.blue(),
        );
        let mut incoming_headers_vec = vec![];
        for (hk, hv) in &req_headers {
            incoming_headers_vec.push(format!(
                "{deco} {key}: {value}",
                deco = "│".green().bold(),
                key = Inflector::to_train_case(hk.as_str()).cyan(),
                value = hv.to_str().unwrap_or("<unprintable>")
            ));
        }
        incoming_headers_vec.sort();
        if !incoming_headers_vec.is_empty() {
            incoming_headers_vec.insert(0, "".to_string());
        }
        let incoming_headers = incoming_headers_vec.join("\n");

        let body = String::from_utf8_lossy(&bytes2);
        let req_body_text = if body.is_empty() || args.verbose < 2 {
            "".to_string()
        } else {
            let body_formatted = if let Some(content_type) = req_headers.get(CONTENT_TYPE) {
                if content_type == "application/json" {
                    serde_json::from_str::<serde_json::Value>(&body)
                        .and_then(|loaded_json| serde_json::to_string_pretty(&loaded_json))
                        .and_then(|pretty_json| pretty_json.to_colored_json_auto())
                        .unwrap()
                } else {
                    body.to_string()
                }
            } else {
                body.to_string()
            };
            let body_formatted = body_formatted
                .lines()
                .map(|line| format!("{deco} {line}", deco = "│".green().bold(), line = line))
                .collect::<Vec<_>>()
                .join("\n");
            format!(
                "\n{deco} {body}\n{body_formatted}",
                deco = "│".green().bold(),
                body = "Body:".yellow(),
                body_formatted = body_formatted,
            )
        };

        let req_info = format!(
            "{deco} {method_path_version_line}{headers}{req_body_text}",
            deco = "│".green().bold(),
            method_path_version_line = method_path_version_line,
            headers = incoming_headers,
            req_body_text = req_body_text,
        );

        let status_line = format!(
            "{http}/{version} {status_code} {status_text}",
            http = "HTTP".blue(),
            version = http_version.blue(),
            status_code = resp.status().as_u16().to_string().blue(),
            status_text = resp.status().canonical_reason().unwrap_or("").cyan(),
        );

        let mut outgoing_headers_vec = vec![];
        for (hk, hv) in resp.headers() {
            outgoing_headers_vec.push(format!(
                "{deco} {key}: {value}",
                deco = "│".red().bold(),
                key = Inflector::to_train_case(hk.as_str()).cyan(),
                value = hv.to_str().unwrap_or("<unprintable>")
            ));
        }
        if !outgoing_headers_vec.is_empty() {
            outgoing_headers_vec.insert(0, "".to_string());
        }
        outgoing_headers_vec.sort();
        let outgoing_headers = outgoing_headers_vec.join("\n");

        let resp_body_text = if args.body.is_empty() || args.verbose < 2 {
            "".to_string()
        } else {
            let body_formatted = args
                .body
                .lines()
                .map(|line| format!("{deco} {line}", deco = "│".red().bold(), line = line))
                .collect::<Vec<_>>()
                .join("\n");
            format!(
                "\n{deco} {body}\n{body_formatted}",
                deco = "│".red().bold(),
                body = "Body:".yellow(),
                body_formatted = body_formatted,
            )
        };

        let resp_info = format!(
            "{deco} {status_line}{headers}{resp_body_text}",
            deco = "│".red().bold(),
            status_line = status_line,
            headers = outgoing_headers,
            resp_body_text = resp_body_text,
        );

        println!(
            "{connect_line}\n{req_banner}\n{req_info}\n{resp_banner}\n{resp_info}",
            req_banner = "┌─Incoming request".green().bold(),
            req_info = req_info,
            resp_banner = "┌─Outgoing response".red().bold(),
            resp_info = resp_info,
        );
    } else if !args.quiet {
        println!("{connect_line}",);
    }

    let (parts, body) = resp.into_parts();
    let bytes = buffer_and_print("response", body).await?;
    let resp = Response::from_parts(parts, Body::from(bytes));

    Ok(resp)
}

async fn buffer_and_print<B>(direction: &str, body: B) -> Result<Bytes, (StatusCode, String)>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
    let bytes = match hyper::body::to_bytes(body).await {
        Ok(bytes) => bytes,
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to read {} body: {}", direction, err),
            ));
        }
    };

    Ok(bytes)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::from_args();

    let app = Router::new()
        .fallback(dummy_response.into_service())
        .layer(middleware::from_fn(print_request_response))
        .layer(Extension(args.clone()));

    let addr = SocketAddr::from((args.interface, args.port));
    if !args.quiet {
        let protocol = {
            #[cfg(feature = "tls")]
            if args.tls_cert.is_some() {
                "https://"
            } else {
                "http://"
            }
            #[cfg(not(feature = "tls"))]
            "http://"
        };
        println!(
            "{}{} {} {}{}",
            "dummyhttp v".bold(),
            crate_version!().bold(),
            "listening on".dimmed(),
            protocol.bold(),
            addr.to_string().bold()
        );
    }

    // configure certificate and private key used by https
    #[cfg(feature = "tls")]
    if let (Some(tls_cert), Some(tls_key)) = (args.tls_cert, args.tls_key) {
        let tls_config = RustlsConfig::from_pem_file(&tls_cert, &tls_key)
            .await
            .context(format!(
                "Failed to load certificate file '{}' or key '{}'",
                tls_cert.to_string_lossy(),
                tls_key.to_string_lossy()
            ))?;
        axum_server::bind_rustls(addr, tls_config)
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .await?;
    } else {
        axum_server::bind(addr)
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .await?;
    }
    #[cfg(not(feature = "tls"))]
    axum_server::bind(addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;

    Ok(())
}

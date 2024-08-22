mod utils;

use axum::http::{self, Method, StatusCode};
use chrono::DateTime;
use reqwest::blocking::Client;
use rstest::rstest;
use utils::{DummyhttpProcess, Error};
use uuid::Uuid;

/// By default, we expect a 200 OK answer with a "dummyhttp" text body.
#[rstest(
    method,
    case::get(Method::GET),
    case::post(Method::POST),
    case::put(Method::PUT),
    case::delete(Method::DELETE),
    case::options(Method::OPTIONS),
    case::patch(Method::PATCH)
)]
fn serves_requests_with_no_options(method: http::Method) -> Result<(), Error> {
    let dh = DummyhttpProcess::new(Vec::<String>::new())?;

    let client = Client::new();
    let resp = client.request(method, &dh.url).send()?;

    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(resp.text()?, "dummyhttp");

    Ok(())
}

/// Setting a custom body will always answer with that body.
#[rstest(
    method,
    case::get(Method::GET),
    case::post(Method::POST),
    case::put(Method::PUT),
    case::delete(Method::DELETE),
    case::options(Method::OPTIONS),
    case::patch(Method::PATCH)
)]
fn returns_custom_body(method: Method) -> Result<(), Error> {
    let dh = DummyhttpProcess::new(vec!["-b", "hi test"])?;

    let client = Client::new();
    let resp = client.request(method, &dh.url).send()?;

    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(resp.text()?, "hi test");

    Ok(())
}

/// Setting a custom body with Tera templating will template fine.
#[rstest(
    method,
    case::get(Method::GET),
    case::post(Method::POST),
    case::put(Method::PUT),
    case::delete(Method::DELETE),
    case::options(Method::OPTIONS),
    case::patch(Method::PATCH)
)]
fn returns_templated_body(method: Method) -> Result<(), Error> {
    let dh = DummyhttpProcess::new(vec!["-b", "{{ uuid() }} {{ now() }} {{ lorem(words=5)}}"])?;

    let client = Client::new();
    let resp = client.request(method, &dh.url).send()?;

    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.text()?;
    let body_split = body.split_ascii_whitespace().collect::<Vec<_>>();
    assert!(Uuid::parse_str(body_split[0]).is_ok());
    assert!(DateTime::parse_from_rfc3339(body_split[1]).is_ok());
    assert_eq!(body_split.len(), 7);

    Ok(())
}

/// Setting a custom code will always answer with that code.
#[rstest(
    method,
    case::get(Method::GET),
    case::post(Method::POST),
    case::put(Method::PUT),
    case::delete(Method::DELETE),
    case::options(Method::OPTIONS),
    case::patch(Method::PATCH)
)]
fn returns_custom_code(method: Method) -> Result<(), Error> {
    let dh = DummyhttpProcess::new(vec!["-c", "201"])?;

    let client = Client::new();
    let resp = client.request(method, &dh.url).send()?;

    assert_eq!(resp.status(), StatusCode::CREATED);
    assert_eq!(resp.text()?, "dummyhttp");

    Ok(())
}

/// Setting a custom header will always return that header.
#[rstest(
    method,
    case::get(Method::GET),
    case::post(Method::POST),
    case::put(Method::PUT),
    case::delete(Method::DELETE),
    case::options(Method::OPTIONS),
    case::patch(Method::PATCH)
)]
fn returns_custom_headers(method: Method) -> Result<(), Error> {
    let dh = DummyhttpProcess::new(vec!["-H", "test:header"])?;

    let client = Client::new();
    let resp = client.request(method, &dh.url).send()?;

    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(resp.headers().get("test").unwrap(), "header");
    assert_eq!(resp.text()?, "dummyhttp");

    Ok(())
}

/// Setting a custom delay will delay the response making it at least that long.
#[rstest(
    method,
    case::get(Method::GET),
    case::post(Method::POST),
    case::put(Method::PUT),
    case::delete(Method::DELETE),
    case::options(Method::OPTIONS),
    case::patch(Method::PATCH),
)]
fn returns_custom_delay(method: Method) -> Result<(), Error> {
    let dh = DummyhttpProcess::new(vec!["-d", "1000"])?;

    let client = Client::new();
    let start = std::time::Instant::now();
    let resp = client.request(method, &dh.url).send()?;
    let elapsed = start.elapsed();

    assert_eq!(resp.status(), StatusCode::OK);
    assert!(elapsed >= std::time::Duration::from_millis(1000));
    assert_eq!(resp.text()?, "dummyhttp");

    Ok(())
}

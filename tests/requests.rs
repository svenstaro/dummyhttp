mod utils;

use http::{Method, StatusCode};
use reqwest::blocking::Client;
use rstest::rstest;
use utils::{DummyhttpProcess, Error};

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
    assert_eq!(resp.text()?, "dummyhttp\n");

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
    assert_eq!(resp.text()?, "hi test\n");

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
    assert_eq!(resp.text()?, "dummyhttp\n");

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
    let dh = DummyhttpProcess::new(vec!["-h", "test:header"])?;

    let client = Client::new();
    let resp = client.request(method, &dh.url).send()?;

    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(resp.headers().get("test").unwrap(), "header");
    assert_eq!(resp.text()?, "dummyhttp\n");

    Ok(())
}

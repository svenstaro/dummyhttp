mod utils;

use rstest::rstest_parametrize;
use http::{StatusCode, Method};
use utils::{DummyhttpProcess, Error};

/// By default, we expect a 200 OK answer with a "dummyhttp" text body.
#[rstest_parametrize(method,
    case::get(Method::GET),
    case::post(Method::POST),
    case::put(Method::PUT),
    case::delete(Method::DELETE),
    case::options(Method::OPTIONS),
    case::patch(Method::PATCH),
)]
fn serves_requests_with_no_options(method: http::Method) -> Result<(), Error> {
    let dh = DummyhttpProcess::new(Vec::<String>::new())?;

    let client = reqwest::Client::new();
    let mut resp = client.request(method, &dh.url).send()?;

    assert_eq!(resp.text()?, "dummyhttp\n");
    assert_eq!(resp.status(), StatusCode::OK);

    Ok(())
}

/// Setting a custom body will always answer with that body.
#[rstest_parametrize(method,
    case::get(Method::GET),
    case::post(Method::POST),
    case::put(Method::PUT),
    case::delete(Method::DELETE),
    case::options(Method::OPTIONS),
    case::patch(Method::PATCH),
)]
fn returns_custom_body(method: Method) -> Result<(), Error> {
    let dh = DummyhttpProcess::new(vec!["-b", "hi test"])?;

    let client = reqwest::Client::new();
    let mut resp = client.request(method, &dh.url).send()?;

    assert_eq!(resp.text()?, "hi test\n");
    assert_eq!(resp.status(), StatusCode::OK);

    Ok(())
}

/// Setting a custom code will always answer with that code.
#[rstest_parametrize(method,
    case::get(Method::GET),
    case::post(Method::POST),
    case::put(Method::PUT),
    case::delete(Method::DELETE),
    case::options(Method::OPTIONS),
    case::patch(Method::PATCH),
)]
fn returns_custom_code(method: Method) -> Result<(), Error> {
    let dh = DummyhttpProcess::new(vec!["-c", "201"])?;

    let client = reqwest::Client::new();
    let mut resp = client.request(method, &dh.url).send()?;

    assert_eq!(resp.text()?, "dummyhttp\n");
    assert_eq!(resp.status(), StatusCode::CREATED);

    Ok(())
}

/// Setting a custom header will always return that header.
#[rstest_parametrize(method,
    case::get(Method::GET),
    case::post(Method::POST),
    case::put(Method::PUT),
    case::delete(Method::DELETE),
    case::options(Method::OPTIONS),
    case::patch(Method::PATCH),
)]
fn returns_custom_headers(method: Method) -> Result<(), Error> {
    let dh = DummyhttpProcess::new(vec!["-h", "test:header"])?;

    let client = reqwest::Client::new();
    let mut resp = client.request(method, &dh.url).send()?;

    assert_eq!(resp.text()?, "dummyhttp\n");
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(resp.headers().get("test").unwrap(), "header");

    Ok(())
}

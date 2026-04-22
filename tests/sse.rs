mod utils;

use axum::http::StatusCode;
use reqwest::Client;
use utils::{DummyhttpProcess, Error};

/// SSE endpoint is not available without --sse flag.
#[test]
fn sse_endpoint_disabled_by_default() -> Result<(), Error> {
    let dh = DummyhttpProcess::new(Vec::<String>::new())?;

    let client = reqwest::blocking::Client::new();
    let resp = client.get(format!("{}/events", dh.url)).send()?;

    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(resp.text()?, "dummyhttp");

    Ok(())
}

/// SSE endpoint returns correct content type when enabled.
#[test]
fn sse_endpoint_enabled() -> Result<(), Error> {
    let dh = DummyhttpProcess::new(vec!["--sse", "--sse-count", "1"])?;

    let client = reqwest::blocking::Client::new();
    let resp = client.get(format!("{}/events", dh.url)).send()?;

    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers().get("content-type").unwrap(),
        "text/event-stream"
    );

    Ok(())
}

/// SSE messages are sent at the configured interval.
#[tokio::test]
async fn sse_messages_sent_at_interval() -> Result<(), Error> {
    let dh = DummyhttpProcess::new(vec!["--sse", "--sse-interval", "300", "--sse-count", "3"])?;

    let client = Client::new();
    let resp = client.get(format!("{}/events", dh.url)).send().await?;
    let body = resp.text().await?;

    // Verify that we received 3 messages
    let data_count = body.lines().filter(|l| l.starts_with("data: ")).count();
    assert_eq!(data_count, 3, "Expected 3 data messages");

    // Verify that the retry value is correct
    assert!(body.contains("retry: 300"), "Expected retry: 300");

    Ok(())
}

/// SSE respects the message count limit.
#[tokio::test]
async fn sse_respects_count_limit() -> Result<(), Error> {
    let dh = DummyhttpProcess::new(vec!["--sse", "--sse-count", "5"])?;

    let client = Client::new();
    let mut resp = client.get(format!("{}/events", dh.url)).send().await?;

    let mut count = 0;
    while let Some(chunk) = resp.chunk().await? {
        let chunk_str = String::from_utf8_lossy(&chunk);
        for line in chunk_str.lines() {
            if line.starts_with("data: ") {
                count += 1;
            }
        }
    }

    assert_eq!(count, 5);

    Ok(())
}

/// SSE includes custom event type when specified.
#[tokio::test]
async fn sse_custom_event_type() -> Result<(), Error> {
    let dh = DummyhttpProcess::new(vec!["--sse", "--sse-event", "myevent", "--sse-count", "1"])?;

    let client = Client::new();
    let mut resp = client.get(format!("{}/events", dh.url)).send().await?;

    let mut found_event = false;
    while let Some(chunk) = resp.chunk().await? {
        let chunk_str = String::from_utf8_lossy(&chunk);
        for line in chunk_str.lines() {
            if line == "event: myevent" {
                found_event = true;
                break;
            }
        }
        if found_event {
            break;
        }
    }

    assert!(found_event, "Expected event type 'myevent'");

    Ok(())
}

/// SSE messages include retry field.
#[tokio::test]
async fn sse_includes_retry_field() -> Result<(), Error> {
    let dh = DummyhttpProcess::new(vec!["--sse", "--sse-interval", "500", "--sse-count", "1"])?;

    let client = Client::new();
    let mut resp = client.get(format!("{}/events", dh.url)).send().await?;

    let mut found_retry = false;
    while let Some(chunk) = resp.chunk().await? {
        let chunk_str = String::from_utf8_lossy(&chunk);
        for line in chunk_str.lines() {
            if line.starts_with("retry: 500") {
                found_retry = true;
                break;
            }
        }
        if found_retry {
            break;
        }
    }

    assert!(found_retry, "Expected retry: 500 field");

    Ok(())
}

/// SSE body supports Tera templating.
#[tokio::test]
async fn sse_body_supports_templating() -> Result<(), Error> {
    let dh = DummyhttpProcess::new(vec![
        "--sse",
        "--sse-interval",
        "100",
        "--sse-count",
        "3",
        "-b",
        "{{ uuid() }}",
    ])?;

    let client = Client::new();
    let mut resp = client.get(format!("{}/events", dh.url)).send().await?;

    let mut uuid_count = 0;
    while let Some(chunk) = resp.chunk().await? {
        let chunk_str = String::from_utf8_lossy(&chunk);
        for line in chunk_str.lines() {
            if line.starts_with("data: ") {
                let data = line.strip_prefix("data: ").unwrap();
                assert!(uuid::Uuid::parse_str(data).is_ok());
                uuid_count += 1;
            }
        }
    }

    assert_eq!(uuid_count, 3);

    Ok(())
}

/// Default endpoint still works when SSE is enabled.
#[test]
fn default_endpoint_works_with_sse() -> Result<(), Error> {
    let dh = DummyhttpProcess::new(vec!["--sse", "-b", "custom body"])?;

    let client = reqwest::blocking::Client::new();
    let resp = client.get(&dh.url).send()?;

    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(resp.text()?, "custom body");

    Ok(())
}

/// SSE and default endpoint use same body template.
#[tokio::test]
async fn sse_uses_same_body_as_default() -> Result<(), Error> {
    let dh = DummyhttpProcess::new(vec!["--sse", "-b", "shared body", "--sse-count", "1"])?;

    let client = Client::new();

    let resp = client.get(&dh.url).send().await?;
    assert_eq!(resp.text().await?, "shared body");

    let mut sse_resp = client.get(format!("{}/events", dh.url)).send().await?;

    let mut found_data = false;
    while let Some(chunk) = sse_resp.chunk().await? {
        let chunk_str = String::from_utf8_lossy(&chunk);
        for line in chunk_str.lines() {
            if line == "data: shared body" {
                found_data = true;
                break;
            }
        }
        if found_data {
            break;
        }
    }
    assert!(found_data, "Expected SSE data to match default body");

    Ok(())
}

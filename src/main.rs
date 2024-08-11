use warp::Filter;
use std::convert::Infallible;
use warp::http::{ Response, StatusCode };
use reqwest::Client;
use reqwest::Url;
use warp::reply::with_status;
use warp::reply::WithStatus;
use tracing::{ info, error };
use tracing_subscriber::FmtSubscriber;
use colored::*;

fn convert_response(body: String) -> WithStatus<String> {
    with_status(body, StatusCode::OK)
}

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder().with_max_level(tracing::Level::INFO).finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let cors_proxy = warp::path("proxy").and(warp::query::query()).and_then(handle_request);

    warp::serve(cors_proxy).run(([127, 0, 0, 1], 3030)).await;
}

async fn handle_request(
    query: std::collections::HashMap<String, String>
) -> Result<impl warp::Reply, Infallible> {
    let target_url = query.get("url").cloned();
    let referer_url = query.get("referer").cloned().unwrap_or_default();
    let origin_url = query.get("origin").cloned().unwrap_or_default();
    let proxy_all = query.get("all").cloned().unwrap_or_default();

    info!(
        "Incoming request: url={} referer={} origin={}",
        target_url.clone().unwrap_or_default().cyan(),
        referer_url.clone().magenta(),
        origin_url.clone().green()
    );

    if target_url.is_none() {
        error!("Invalid URL");
        return Ok(warp::reply::with_status("Invalid URL".to_string(), StatusCode::BAD_REQUEST));
    }

    let target_url = target_url.unwrap();

    let client = Client::new();
    let response_result = client
        .get(&target_url)
        .header("Referer", referer_url.clone())
        .header("Origin", origin_url.clone())
        .send().await;

    if let Err(e) = response_result {
        error!("Request error: {}", e.to_string().red());
        return Ok(warp::reply::with_status(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR));
    }

    let response = response_result.unwrap();
    let status = response.status();
    let content_type = response
        .headers()
        .get("Content-Type")
        .map(|v| v.to_str().unwrap_or_default().to_string())
        .unwrap_or("application/vnd.apple.mpegurl".to_string());

    let mut body = response.text().await.unwrap_or_default();

    if target_url.contains(".m3u8") {
        let target_url_trimmed = Url::parse(&target_url)
            .ok()
            .and_then(|u| {
                let mut parts = u.path().split('/').collect::<Vec<_>>();
                parts.pop(); // Remove the last part (.m3u8)
                Some(parts.join("/"))
            })
            .unwrap_or_default();
        let encoded_url = urlencoding::encode(&referer_url);
        let encoded_origin = urlencoding::encode(&origin_url);

        body = body
            .split('\n')
            .map(|line| {
                if line.starts_with("#") || line.trim().is_empty() {
                    return line.to_string();
                } else if proxy_all == "yes" && line.starts_with("http") {
                    return format!("{}?url={}", query.get("url").unwrap(), line);
                }
                format!(
                    "?url={}{}{}{}",
                    urlencoding::encode(&target_url_trimmed),
                    line,
                    if !origin_url.is_empty() {
                        format!("&origin={}", encoded_origin)
                    } else {
                        String::new()
                    },
                    if !referer_url.is_empty() {
                        format!("&referer={}", encoded_url)
                    } else {
                        String::new()
                    }
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
    }

    let response_body = Response::builder()
        .status(warp::http::StatusCode::from_u16(status.as_u16()).unwrap())
        .header("Access-Control-Allow-Origin", "*")
        .header("Content-Type", content_type)
        .body(body)
        .unwrap();

    Ok(convert_response(response_body.into_body()))
}

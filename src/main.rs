use warp::Filter;
use warp::http::{ Response, StatusCode };
use reqwest::Client;
use reqwest::Url;
use tracing::{ info, error };
use tracing_subscriber::FmtSubscriber;
use colored::*;
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let subscriber = FmtSubscriber::builder().with_max_level(tracing::Level::INFO).finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let cors_proxy = warp::path("proxy").and(warp::query::query()).and_then(handle_request);

    let port = std::env::var("PORT").unwrap_or("3030".to_string()).parse().unwrap();

    use std::net::{ IpAddr, Ipv4Addr, SocketAddr };

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port);
    warp::serve(cors_proxy).run(addr).await;
}

async fn handle_request(
    query: std::collections::HashMap<String, String>
) -> anyhow::Result<impl warp::Reply, warp::Rejection> {
    let target_url = query.get("url").cloned();
    let referer_url = query.get("referer").cloned().unwrap_or_default();
    let origin_url = query.get("origin").cloned().unwrap_or_default();

    info!(
        "Incoming request: url={} referer={} origin={}",
        target_url.clone().unwrap_or_default().cyan(),
        referer_url.clone().magenta(),
        origin_url.clone().green()
    );

    if target_url.is_none() {
        error!("Invalid URL");
        return Err(warp::reject::custom(InvalidUrl));
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
        return Err(warp::reject::custom(ServerError));
    }

    let response = response_result.unwrap();
    let status = StatusCode::from_u16(response.status().as_u16()).unwrap();
    let content_type = response
        .headers()
        .get("Content-Type")
        .map(|v| v.to_str().unwrap_or_default().to_string())
        .unwrap_or("application/vnd.apple.mpegurl".to_string());

    let mut body = response.text().await.unwrap_or_default();

    if target_url.contains(".m3u8") {
        body = body
            .split('\n')
            .map(|line| {
                if line.starts_with("#") {
                    if line.starts_with("#EXT-X-KEY:") {
                        let regex = regex::Regex::new(r#"https?://[^\s"']+"#).unwrap();
                        regex
                            .replace_all(line, |caps: &regex::Captures| {
                                format!(
                                    "?url={}&referer={}",
                                    urlencoding::encode(&caps[0]),
                                    urlencoding::encode(&referer_url)
                                )
                            })
                            .to_string()
                    } else {
                        line.to_string()
                    }
                } else {
                    let uri = Url::parse(line).unwrap_or_else(|_| {
                        Url::parse(&format!("{}/{}", target_url, line)).unwrap()
                    });
                    format!(
                        "?url={}&referer={}",
                        urlencoding::encode(uri.as_str()),
                        urlencoding::encode(&referer_url)
                    )
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
    }

    let response_body = Response::builder()
        .status(status)
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Headers", "*")
        .header("Access-Control-Allow-Methods", "*")
        .header("Content-Type", content_type.clone())
        .body(body)
        .unwrap();

    Ok(response_body)
}

#[derive(Debug)]
struct InvalidUrl;

impl warp::reject::Reject for InvalidUrl {}

#[derive(Debug)]
struct ServerError;

impl warp::reject::Reject for ServerError {}

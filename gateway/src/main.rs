use std::collections::HashMap;
use std::convert::Infallible;
use std::time::Duration;
use ureq::{Agent};
use warp::{Filter, http::Method, path::FullPath, reply::Reply, http};
use warp::http::{HeaderMap, StatusCode};
use warp::hyper::body::Bytes;
use warp::reply::{Html, WithHeader, WithStatus};

const HOST: &str = "https://httpbin.org";

#[shuttle_runtime::main]
async fn warp() -> shuttle_warp::ShuttleWarp<(impl Reply, )> {
    let route = warp::any()
        .and(warp::method())
        .and(warp::filters::path::full())
        .and(warp::header::headers_cloned())
        .and(warp::query::query())
        .and(warp::body::bytes())
        .and_then(handle_request)
        .with(warp::cors::cors());

    let boxed = route.boxed();

    Ok(boxed.into())
}

async fn handle_request(
    method: Method,
    path: FullPath,
    headers: http::HeaderMap,
    query: HashMap<String, String>,
    body: warp::hyper::body::Bytes,
) -> Result<impl Reply, Infallible> {
    let uri = format!("{}{}", HOST, path.as_str());
    print!("{} {} -> ", method.as_str(), path.as_str());

    let response = forward_request(uri, method, headers, query, &body);

    Ok(response)
}

fn forward_request(uri: String, method: Method, headers: HeaderMap, query: HashMap<String, String>, body: &Bytes) -> WithStatus<WithHeader<Html<String>>> {
    let method_str = method.to_string();

    let agent: Agent = ureq::AgentBuilder::new()
        .timeout_read(Duration::from_secs(5))
        .timeout_write(Duration::from_secs(5))
        .build();

    let mut request = agent.request(method_str.as_str(), uri.as_str());

    // Set headers if provided
    if !headers.is_empty() {
        request = set_headers(request, headers);
    }

    // Set query parameters if provided
    if !query.is_empty() {
        request = set_query(request, query);
    }

    let result = if method == Method::GET && body.is_empty() {
        request.call()
    } else {
        request.send_bytes(&body)
    };

    let response = match result {
        Ok(res) => response_with_status(uri, res.status(), method, res),
        Err(ureq::Error::Status(code, res)) => response_with_status(uri, code, method, res),
        Err(_) => {
            let body = "Bad Gateway".to_string();
            let content = warp::reply::html(body);
            let reply = warp::reply::with_header(content, "Content-Type", "application/json");
            let response = warp::reply::with_status(reply, StatusCode::BAD_GATEWAY);

            print!("{} {} {}\n", StatusCode::BAD_GATEWAY.as_str(), method.as_str(), uri.as_str());

            response
        }
    };

    response
}

fn response_with_status(uri: String, code: u16, method: Method, res: ureq::Response) -> WithStatus<WithHeader<Html<String>>> {
    let status_code = StatusCode::try_from(code).unwrap();
    let body = res.into_string().unwrap().clone();
    let content = warp::reply::html(body);
    let reply = warp::reply::with_header(content, "Content-Type", "application/json");
    let response = warp::reply::with_status(reply, status_code);

    print!("{} {} {}\n", status_code.as_str(), method.as_str(), uri.as_str());

    response
}

fn set_headers(request: ureq::Request, mut headers: HeaderMap) -> ureq::Request {
    headers.remove("host");

    let mut modified_request = request;
    for (key, value) in headers.iter() {
        let key_str = key.as_str();
        let value_str = value.to_str().unwrap();
        modified_request = modified_request.set(key_str, value_str);
    }
    modified_request
}

fn set_query(request: ureq::Request, query: HashMap<String, String>) -> ureq::Request {
    let mut modified_request = request;
    for (key, value) in query.iter() {
        modified_request = modified_request.query(key, value);
    }
    modified_request
}

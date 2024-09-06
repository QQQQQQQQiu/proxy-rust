use hyper::{Client, Request, Body, Response, Method, StatusCode, client::{HttpConnector}};
use hyper::header::{HeaderValue};
use hyper_tls::HttpsConnector;
use serde_json::Value;
use crate::types::{XHRData, XHRResponseAll};
use crate::utils::{get_secret_str};
use serde_json::from_slice;
use std::collections::HashMap;

const API_PREFIX: &str = "/api/xhr";

lazy_static::lazy_static! {
    static ref CLIENT: Client<HttpsConnector<HttpConnector>> = {
        let https = HttpsConnector::new();
        Client::builder().build::<_, Body>(https)
    };
}

pub async fn handle_xhr<T>(req: Request<T>) -> Response<Body> where T: hyper::body::HttpBody + Send + 'static,{
    let options = match get_options(req).await {
        Ok(data) => data,
        Err(err) => {
            eprintln!("[handle_xhr] Error getting options: {}", err);
            return Response::builder().status(StatusCode::BAD_REQUEST).body(Body::from("Bad Request")).unwrap();
        }
    };
    eprintln!("Parsed JSON data: {:?}", options);


    let url = options.url.trim();
    let method = options.method.as_str();
    let headers = options.headers;
    let is_throw_headers = options.throw_headers;
    let body = if options.body.is_empty() {
        Body::empty()
    } else {
        Body::from(options.body)
    };

    // 构建请求
    println!("request url: {}", url);
    let mut request_builder = Request::builder().method(method).uri(url);
    // 添加请求头
    for (key, value) in headers {
        let header_value = match value {
            Value::String(s) => s,
            Value::Number(n) => n.to_string(),
            _ => continue,
        };
        request_builder = request_builder.header(key.as_str(), HeaderValue::from_str(&header_value).unwrap());
    }
    
    let request = request_builder.body(Body::from(body)).expect("Failed to build request body");
    println!("request_builder done");
    
    // 发送请求并获取响应
    let res = match send_request(request).await {
        Ok(response) => {
            println!("Request sent successfully");
            response
        },
        Err(e) => {
            println!("Error sending request");
            eprintln!("Error making XHR request: {}", e);
            Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::from("Internal Server Error")).unwrap()
        },
    };
    println!("request done");

    let mut response = Response::builder();
    if res.status().is_success() {
        response = response.status(StatusCode::OK);
    } else {
        response = response.status(StatusCode::INTERNAL_SERVER_ERROR);
    }
    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    return response.body(Body::from(body)).unwrap();
}

async fn send_request(request: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    println!("Sending request :::: {:?}", request);
    let response = CLIENT.request(request).await?;
    Ok(response)
}

pub async fn get_options<T>(req: Request<T>) -> Result<XHRData, String>  where T: hyper::body::HttpBody + Send + 'static,  {
    if req.method() == Method::GET {
        let uri = req.uri().to_string();
        eprintln!("full uri: {}", uri);
        let secret = get_secret_str();
        let is_pass_secret = handle_xhr_is_pass_secret(&req);
        let mut uri_str = if is_pass_secret {
            uri.trim_start_matches(&format!("{}/{}/", API_PREFIX, secret))
        } else {
            uri.trim_start_matches(&format!("{}/", API_PREFIX))
        }.to_string();
        uri_str = urlencoding::decode(&uri_str).unwrap().to_string();
        if uri_str.starts_with("http") {
            let json_str = format!(r#"{{ "url": "{}", "method": "GET", "headers": {{}}, "body": "", "throw_headers": false }}"#, uri_str);
            let data: XHRData = match from_slice(json_str.as_bytes()) {
                Ok(data) => data,
                Err(err) => {
                    eprintln!("Error parsing JSON: {}", err);
                    return Err("Parse Err".to_string());
                }
            };
            return Ok(data);
        }
        let data: XHRData = match from_slice(uri_str.as_bytes()) {
            Ok(data) => data,
            Err(err) => {
                eprintln!("Error parsing JSON: {}", err);
                return Err("Parse Err".to_string());
            }
        };
        return Ok(data);
    } 
    else if req.method() == Method::POST {
        let body_bytes = match hyper::body::to_bytes(req.into_body()).await {
            Ok(bytes) => bytes,
            Err(_) => return Err("Parse Err".to_string()),
        };
        let data: XHRData = serde_json::from_slice(&body_bytes)
            .map_err(|e| {
                eprintln!("Failed to parse JSON: {:?}", e);
                "Parse Err".to_string()
            })?;

        return Ok(data);
    }
    Err("Unsupported request method".to_string())
}

pub fn handle_xhr_is_match_route<T>(req: &hyper::Request<T>) -> bool {
    return req.uri().path().starts_with(API_PREFIX);
}

pub fn handle_xhr_is_pass_secret<T>(req: &Request<T>) -> bool {
    let secret = get_secret_str();
    let path = req.uri().path();
    let expected_path_prefix = format!("{}/{}/", API_PREFIX, secret);
    path.starts_with(&expected_path_prefix)
}
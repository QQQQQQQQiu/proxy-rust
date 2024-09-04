use hyper::{Client, Request, Body, Response, Method, StatusCode};
use hyper::header::{HeaderValue};
use hyper_tls::HttpsConnector;
use serde_json::Value;
use crate::types::{XHRData, XHRResponseAll};
use crate::utils::{get_secret_str};
use serde_json::from_slice;
use std::collections::HashMap;

const API_PREFIX: &str = "/api/xhr";

pub async fn handle_xhr<T>(req: Request<T>) -> Response<Body> where T: hyper::body::HttpBody + Send + 'static,{
    let options = match get_options(req).await {
        Ok(data) => data,
        Err(err) => {
            eprintln!("[handle_xhr] Error getting options: {}", err);
            return Response::builder().status(StatusCode::BAD_REQUEST).body(Body::from("Bad Request")).unwrap();
        }
    };
    // eprintln!("Parsed JSON data: {:?}", options);


    // 创建客户端
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
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
    // println!("request url: {}", url);
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
    
    let request = request_builder.body(Body::from(body)).unwrap();

    // 发送请求并获取响应
    let res = match client.request(request).await {
        Ok(response) => response,
        Err(e) => {
            // 处理请求错误
            eprintln!("Error making XHR request: {}", e);
            Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::from("Internal Server Error")).unwrap()
        },
    };

    // 是否把响应码、响应头、响应体一并放在body，结构为XHRResponseAll
    if is_throw_headers.unwrap_or(false) {
        let status_code = res.status().as_u16() as i32;
        let headers = res.headers().iter().map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string())).collect::<HashMap<String, String>>();
        let body_bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body = String::from_utf8_lossy(&body_bytes).to_string();
        let xhr_response = XHRResponseAll {
            status_code,
            headers,
            body,
        };
        let json_response = serde_json::to_string(&xhr_response).unwrap();
        return Response::builder().status(StatusCode::OK).header("Content-Type", "application/json").body(Body::from(json_response)).unwrap();
    }

    return res
}

pub async fn get_options<T>(req: Request<T>) -> Result<XHRData, String>  where T: hyper::body::HttpBody + Send + 'static,  {
    if req.method() == Method::GET {
        let uri = req.uri().to_string();
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
use serde_json::Value;
use crate::types::{XHRData, XHRResponseAll};
use crate::utils::{get_secret_str};
use serde_json::from_slice;
use std::collections::HashMap;
use tide::Request;
use tide::Response;
use tide::Body;
use http_types::Method;
use std::str::FromStr;
use surf:: {Client, Body as SurfBody};


const API_PREFIX: &str = "/api/xhr";

lazy_static::lazy_static! {
    static ref CLIENT: Client = {
        Client::new()
    };
}

pub async fn handle_xhr(mut req: Request<()>) -> Response {
    let options = match get_options::<()>(&mut req).await {
        Ok(data) => data,
        Err(err) => {
            eprintln!("[handle_xhr] Error getting options: {}", err);
            return Response::builder(500).body(Body::from("Bad Request")).build();
        }
    };
    eprintln!("Parsed JSON data: {:?}", options);
    
    let url = options.url.trim();
    let method = Method::from_str(&options.method).unwrap();
    let headers = options.headers;
    let is_throw_headers = options.throw_headers;
    let body = if options.body.is_empty() {
        SurfBody::empty()
    } else {
        SurfBody:: from_string(options.body)
    };
    println!("request url: {}", url);
    
    let mut resp = CLIENT.request(method, url).body(body).send().await.unwrap();

    let status = resp.status();
    println!("request done {}", status);
    // let body = Body::from(resp.body_bytes().await.unwrap());
    let body = resp.take_body();
    let mut tide_response = Response::new(status);
    tide_response.set_body(body);
    for (key, value) in resp.iter() {
        tide_response.insert_header(key.as_str(), value);
    }
    return tide_response;

    // 构建请求
    
    // let mut request_builder = Request::builder().method(method).uri(url);
    // // 添加请求头
    // for (key, value) in headers {
    //     let header_value = match value {
    //         Value::String(s) => s,
    //         Value::Number(n) => n.to_string(),
    //         _ => continue,
    //     };
    //     request_builder = request_builder.header(key.as_str(), HeaderValue::from_str(&header_value).unwrap());
    // }
    
    // let request = request_builder.body(Body::from(body)).expect("Failed to build request body");
    // println!("request_builder done");
    
    // // 发送请求并获取响应
    // let res = match send_request(request).await {
    //     Ok(response) => {
    //         println!("Request sent successfully");
    //         response
    //     },
    //     Err(e) => {
    //         println!("Error sending request");
    //         eprintln!("Error making XHR request: {}", e);
    //         Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::from("Internal Server Error")).unwrap()
    //     },
    // };
    // println!("request done");

    // let mut response = Response::builder();
    // if res.status().is_success() {
    //     response = response.status(StatusCode::OK);
    // } else {
    //     response = response.status(StatusCode::INTERNAL_SERVER_ERROR);
    // }
    // let body = Body::to_bytes(res.into_body()).await.unwrap();
    // return response.body(Body::from(body)).unwrap();
}

pub async fn get_options<T>(req: &mut tide::Request<()>) -> Result<XHRData, String> where T: Send + 'static, {
    if req.method() == Method::Get {
        let full_uri = req.url().to_string();
        let host = req.host().expect("REASON").to_string();
        let uri = full_uri.replace(&format!("http://{}", host), "");
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
    else if req.method() == Method::Post {
        let body_bytes = match req.body_bytes().await {
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

pub fn handle_xhr_is_match_route<T>(req: &Request<()>) -> bool {
    return req.url().path().starts_with(API_PREFIX);
}

pub fn handle_xhr_is_pass_secret<T>(req: &Request<T>) -> bool {
    let secret = get_secret_str();
    let path = req.url().path();
    let expected_path_prefix = format!("{}/{}/", API_PREFIX, secret);
    path.starts_with(&expected_path_prefix)
}
mod types;
mod utils;
mod module;
use utils::get_secret_str;
use std::env;
use module::{
    handle_doc,
    handle_doc_is_match_route,
    handle_xhr_is_match_route,
    handle_xhr_is_pass_secret,
    handle_xhr,
};
use tide::Request;
use tide::Response;
use tide;
use tide::prelude::*;


#[tokio::main]
async fn main() {
    // 从命令行获取-s参数，放入环境变量SECRET中
    let secret = parse_secret();
    println!("接口请求密码: {}", secret);
    env::set_var("SECRET", secret);

    let port = 801;
    println!("运行端口：{}", port);

    // 创建服务
    let mut app = tide::new();
    app.at("*").all(handle_request);
    let _ = app.listen(format!("0.0.0.0:{}", port)).await;
}


async fn handle_request(mut req: Request<()>) -> tide::Result {
    let secret_pass =  match () {
        _ if handle_doc_is_match_route(&req) => true,
        _ if handle_xhr_is_pass_secret(&req) => true,
        _ => false,
    };
    
    let is_valid = check_secret(&req);
    if !is_valid && !secret_pass {
        return Ok(Response::builder(401).body("401").build());
    }

    let mut resp: Response = match () {
        _ if handle_doc_is_match_route(&req) => handle_doc(),
        _ if handle_xhr_is_match_route::<()>(&req) => handle_xhr(req).await,
        _ => Response::builder(200).body("body").build(),
    };
    // 支持跨域
    for header in [
        ("Access-Control-Allow-Headers", "*"),
        ("Access-Control-Allow-Methods", "*"),
        ("Access-Control-Allow-Origin", "*"),
    ] {
        resp.insert_header(header.0, header.1);
    }
    println!("[main] route done");

    Ok(resp)
}

fn parse_secret() -> String {
    let args: Vec<String> = env::args().collect();
    let msg = "高危程序接口密码不能为空，运行程序加 -s 参数";
    match args.iter().position(|arg| arg == "-s") {
        Some(index) => {
            if index + 1 < args.len() {
                args[index + 1].clone() // 获取-s后面的值
            } else {
                panic!("{}", msg);
            }
        }
        None => panic!("{}", msg),
    }
}

fn check_secret(req: &Request<()>) -> bool {
    let secret = get_secret_str();
    if let Some(header_values) = req.header("s") {
        if let Some(header_value) = header_values.iter().next() {
            let header_value_str = header_value.to_string();
            return header_value_str == secret;
        }
    }
    false
}
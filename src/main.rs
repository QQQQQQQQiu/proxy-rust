mod types;
mod module;
mod utils;
use utils::get_secret_str;
use module::{
    handle_command,
    handle_command_is_match_route,
    handle_xhr,
    handle_xhr_is_match_route,
    handle_xhr_is_pass_secret,
    handle_doc,
    handle_doc_is_match_route,
};
use std::env;
use std::net::SocketAddr;
use std::convert::Infallible;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, header::HeaderValue};

#[tokio::main]
async fn main() {
    // 从命令行获取-s参数，放入环境变量SECRET中
    let secret = parse_secret();
    println!("接口请求密码: {}", secret);
    env::set_var("SECRET", secret);

    let port = 801;
    println!("运行端口：{}", port);

    // 创建服务
    let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle_request)) });
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let server = Server::builder(hyper::server::conn::AddrIncoming::bind(&addr).unwrap()).serve(make_svc);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, String> {

    let secret_pass =  match () {
        _ if handle_xhr_is_pass_secret(&req) => true,
        _ => false,
    };
    
    let is_valid = check_secret(&req);
    if !is_valid && !secret_pass {
        return Ok(Response::builder().status(hyper::StatusCode::UNAUTHORIZED).body(Body::from("401")).unwrap());
    }

    let mut resp = match () {
        _ if handle_command_is_match_route(&req) => handle_command(req).await,
        _ if handle_doc_is_match_route(&req) => handle_doc(),
        _ if handle_xhr_is_match_route(&req) => handle_xhr(req).await,
        _ => Response::builder().status(hyper::StatusCode::NOT_FOUND).body(Body::from("404")).expect("Failed to create NOT_FOUND response"),
    };
    
    
    // 支持跨域
    resp.headers_mut().insert("Access-Control-Allow-Headers", "*".parse().unwrap());
    resp.headers_mut().insert("Access-Control-Allow-Methods", "*".parse().unwrap());
    resp.headers_mut().insert("Access-Control-Allow-Origin", "*".parse().unwrap());

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

fn check_secret(req: &hyper::Request<hyper::Body>) -> bool {
    let secret = get_secret_str();
    let secret_header = match HeaderValue::from_str(&secret) {
        Ok(header_value) => header_value,
        Err(_) => return false,
    };
    // 检查请求头的 s 是否与环境变量的 SECRET 相同
    req.headers().get("s") == Some(&secret_header)
}

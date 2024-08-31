use crate::types::{CommandResponse, CommandData};
use tokio::process::Command;
use tokio::time::timeout;
use hyper::{Body, Request, Response, StatusCode};

pub async fn handle_command<T>(req: Request<T>) -> Response<Body> where T: hyper::body::HttpBody + 'static, {
    
    // 获取请求体
    let body_bytes = match hyper::body::to_bytes(req.into_body()).await {
        Ok(bytes) => bytes,
        Err(_) => return Response::builder().status(500).body(Body::from("Failed to read request body")).unwrap(),
    };

    // 尝试解析 JSON 数据
    let command_data_arr: Vec<CommandData> = serde_json::from_slice(&body_bytes)
    .and_then(|value| serde_json::from_value(value).map_err(|e| {
        eprintln!("[handle_command]Failed to convert JSON to CommandData: {:?}", e);
        e
    }))
    .unwrap_or_else(|e| {
        eprintln!("[handle_command]Failed to parse JSON : {:?}", e);
        Vec::new()
    });
    // eprintln!("Parsed JSON data: {:?}", command_data_arr);

    let mut results: Vec<CommandResponse> = Vec::new();

    for cmd_obj in command_data_arr {
        println!("Executing command: {}", cmd_obj.cmd);
        let child = match Command::new("sh")
        .arg("-c").arg(&cmd_obj.cmd)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn() {
            Ok(child) => child,
            Err(e) => {
                let response = CommandResponse {
                    id: cmd_obj.id,
                    output: String::from(""),
                    error: Some(format!("Error: failed to spawn command: {}", e)),
                };
                results.push(response);
                continue;
            }
        };

        let output = match timeout(std::time::Duration::from_secs(10), child.wait_with_output()).await {
            Ok(output) => output,
            Err(_) => {
                let response = CommandResponse {
                    id: cmd_obj.id,
                    output: String::from(""),
                    error: Some(String::from("Error: command timed out")),
                };
                results.push(response);
                continue;
            }
        };

        let output = match output {
            Ok(output) => output,
            Err(e) => {
                let response = CommandResponse {
                    id: cmd_obj.id,
                    output: String::from(""),
                    error: Some(format!("Error: failed to get command output: {}", e)),
                };
                results.push(response);
                continue;
            }
        };

        // 打印命令输出
        let command_output = String::from_utf8_lossy(&output.stdout).into_owned();
        let response = CommandResponse {
            id: cmd_obj.id,
            output: command_output,
            error: if output.status.success() {
                None
            } else {
                Some(format!("Error: {}", output.status))
            },
        };

        results.push(response);
    }

    let res_body = serde_json::to_string(&results).expect("Failed to serialize results");
    Response::builder().status(StatusCode::OK).body(Body::from(res_body))
    .unwrap_or_else(|e| {
        eprintln!("Failed to create response body: {}", e);
        Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::from("Internal server error")).expect("Failed to build error response")
    })
}

pub fn handle_command_is_match_route<T>(req: &hyper::Request<T>) -> bool {
    return req.uri().path().starts_with("/api/cmd");
}
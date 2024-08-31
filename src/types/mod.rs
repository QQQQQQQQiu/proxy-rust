use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use serde_json::Value;


#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CommandData {
    pub id: String,
    pub cmd: String,
}

#[derive(Deserialize, Serialize)]
pub struct CommandResponse {
    pub id: String,
    pub output: String,
    pub error: Option<String>,
}


#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct XHRData {
    pub url: String,
    pub method: String,
    pub headers: HashMap<String, Value>,
    pub body: String,
    #[serde(default)]
    #[serde(rename = "throwHeaders")]
    pub throw_headers: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct XHRResponseAll {
    pub status_code: i32,
    pub headers: HashMap<String, String>,
    pub body: String,
}
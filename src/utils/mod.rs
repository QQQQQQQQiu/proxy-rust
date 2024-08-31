use std::env;

pub fn get_secret_str() -> String {
    let result = env::var("SECRET");
    match result {
        Ok(secret) => secret,
        Err(_) => "".to_string(),
    }
}
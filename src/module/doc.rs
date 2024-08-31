use hyper::{Body, Response};
use std::env;

pub fn handle_doc() -> Response<Body>  {
  let data = include_str!(concat!(env!("OUT_DIR"), "/readme.md"));
  let resp = Response::builder()
      .header("Content-Type", "text/html; charset=utf-8")
      .body(Body::from(data.to_string()))
      .unwrap();
  return resp;
}

pub fn handle_doc_is_match_route<T>(req: &hyper::Request<T>) -> bool {
  return req.uri().path().starts_with("/doc");
}
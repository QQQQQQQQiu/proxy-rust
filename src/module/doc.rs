use std::env;
use tide::Request;
use tide::Response;
use tide::Body;

pub fn handle_doc() -> Response  {
  let data = include_str!(concat!(env!("OUT_DIR"), "/readme.md"));
  let resp = Response::builder(200)
      .header("Content-Type", "text/html; charset=utf-8")
      .body(Body::from(data.to_string()))
      .build();
  return resp;
}

pub fn handle_doc_is_match_route(req: &Request<()>) -> bool {
  return req.url().path().starts_with("/doc");
}
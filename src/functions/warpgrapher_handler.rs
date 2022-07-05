use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::convert::TryFrom;
use warpgrapher::engine::config::Configuration;
use warpgrapher::engine::context::RequestContext;
use warpgrapher::engine::database::gremlin::GremlinEndpoint;
use warpgrapher::engine::database::DatabaseEndpoint;
use warpgrapher::Engine;

use futures;
use serde_json::from_value;

use azure_functions::{
  bindings::{HttpRequest, HttpResponse},
  func,
};

static CONFIG: &'static str = "
version: 1
model: 
  - name: User
    props:
      - name: name
        type: String
  - name: Message
    props:
      - name: value
        type: String
    rels:
      - name: author
        nodes: [User]
        list: false
";

#[derive(Clone, Debug)]
pub struct Rctx {}

impl Rctx {}

impl RequestContext for Rctx {
  type DBEndpointType = GremlinEndpoint;

  fn new() -> Self {
    Rctx {}
  }
}

#[derive(Clone, Debug, Deserialize)]
struct GraphqlRequest {
  pub query: String,
  pub variables: Option<Value>,
}

#[func]
pub async fn handler(req: HttpRequest) -> HttpResponse {
  let metadata: HashMap<String, String> = HashMap::new(); //req.headers()

  let config = Configuration::try_from(CONFIG.to_string()).unwrap();
  let database_pool = GremlinEndpoint::from_env().unwrap().pool().await.unwrap();

  let engine = Engine::<Rctx>::new(config, database_pool)
    .with_version("1.0".to_string())
    .build()
    .expect("Could not create warpgrapher engine");

  let query = req.body().as_json::<serde_json::Value>().unwrap();
  let request = from_value::<GraphqlRequest>(query).unwrap();

  let result = engine
    .execute(request.query, request.variables, metadata)
    .await;

  match result {
    Ok(r) => r.into(),
    Err(_) => "error".into(),
  }
}

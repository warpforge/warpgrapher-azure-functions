use std::collections::HashMap;
use std::convert::TryFrom;
use warpgrapher::Engine;
use warpgrapher::engine::config::Configuration;
use warpgrapher::engine::database::cosmos::CosmosEndpoint;
use warpgrapher::engine::database::DatabaseEndpoint;
use warpgrapher::juniper::http::GraphQLRequest;

use serde_json::{from_value};
use futures;

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
  - name: Project
    props:
      - name: name
        type: String
    rels:
      - name: users
        nodes: [User]
        list: true
";

#[func]
pub async fn handler(req: HttpRequest) -> HttpResponse {
    let metadata: HashMap<String, String> = HashMap::new(); //req.headers()

    let config = Configuration::try_from(CONFIG.to_string()).unwrap();
    let database_pool = CosmosEndpoint::from_env().unwrap().pool().await.unwrap();

    let engine = Engine::<(), ()>::new(config, database_pool)
        .with_version("1.0".to_string())
        .build()
        .expect("Could not create warpgrapher engine");

    let query = req.body().as_json::<serde_json::Value>().unwrap();

    let result = engine.execute(&from_value::<GraphQLRequest>(query).unwrap(), &metadata);

    match result {
        Ok(r) => r.into(),
        Err(_) => "error".into()
    }
}

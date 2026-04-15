mod version;

use axum::{Json, Router};
use axum::extract::Query;
use serde::{Deserialize, Serialize};
use super::{AppState, Response};
use crate::k8s::crd;

#[derive(Deserialize, Debug)]
pub struct GetRequest {
    format: Option<GetFormat>,
}

#[derive(Deserialize, Debug)]
enum GetFormat {
    #[serde(rename = "yaml")]
    Yaml,
    #[serde(rename = "json")]
    Json,
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum PostResponse<'a> {
    Json {
        template: &'a crd::Fleet,
    },
    Yaml {
        template: String,
    },
}

async fn get(
    Query(GetRequest{ format }): Query<GetRequest>
) -> Response {
    let format = format.unwrap_or(GetFormat::Json);
    let template = crate::k8s::fleet_template();
    let res = match format {
        GetFormat::Json => PostResponse::Json { template, },
        GetFormat::Yaml => {
            let yaml = match serde_yaml::to_string(&template) {
                Ok(yaml) => yaml,
                Err(e) => return Response::error(
                    "FailedToSerialize",
                    &format!("Failed to serialize template to YAML: {e}")
                ),
            };
            PostResponse::Yaml { template: yaml }
        }
    };

    Response::success(res)
}

pub fn route(path: &str) -> Router<AppState> {
    let inner = Router::new()
        .route("/", axum::routing::get(get))
        .merge(version::route("/version"));

    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}

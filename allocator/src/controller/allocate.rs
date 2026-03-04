use axum::{extract::State, Json};
use std::collections::HashMap;
use std::sync::Arc;

use crate::{
    config::Config,
    controller::{
        error::{Error, Result},
        response::{AllocateRequest, AllocateResponse},
    },
    k8s::{allocation::create_allocation, K8sClient},
};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub k8s_client: K8sClient,
}

pub async fn allocate(
    State(state): State<AppState>,
    Json(req): Json<AllocateRequest>,
) -> Result<Json<AllocateResponse>> {
    // Validate bot_count
    if !state.config.is_bot_count_valid(req.bot_count) {
        return Err(Error::Validation(format!(
            "bot_count must be between {} and {}",
            state.config.min_bot_count, state.config.max_bot_count
        )));
    }

    // Validate client_version if provided
    if let Some(ref version) = req.client_version {
        if !state.config.is_version_allowed(version) {
            return Err(Error::Validation(format!(
                "client_version '{}' is not allowed",
                version
            )));
        }
    }

    // Build annotations from request
    let mut annotations = HashMap::new();
    annotations.insert("bot_count".to_string(), req.bot_count.to_string());

    if let Some(difficulty) = req.difficulty {
        annotations.insert("difficulty".to_string(), difficulty);
    }

    if let Some(env_params) = req.env_params {
        let env_params_json = serde_json::to_string(&env_params)
            .map_err(|e| Error::Internal(format!("Failed to serialize env_params: {}", e)))?;
        annotations.insert("env_params".to_string(), env_params_json);
    }

    if let Some(client_version) = req.client_version {
        annotations.insert("client_version".to_string(), client_version);
    }

    // Create allocation
    let response = create_allocation(
        state.k8s_client.client(),
        &state.config.namespace,
        &state.config.fleet_name,
        state.config.scheduling.as_str(),
        annotations,
    )
    .await?;

    Ok(Json(response))
}

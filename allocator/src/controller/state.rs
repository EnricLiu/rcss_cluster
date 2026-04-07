use std::sync::Arc;
use crate::args::Args;
use crate::k8s::K8sClient;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Args>,
    pub k8s: K8sClient,
}

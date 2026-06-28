use std::path::Path;
use std::sync::OnceLock;

use super::crd::GameServer;

static GS_TEMPLATE: OnceLock<GameServer> = OnceLock::new();
static GS_TEMPLATE_VERSION: OnceLock<String> = OnceLock::new();


/// Initialize the global GameServer template from a YAML file.
/// Must be called exactly once at startup before any `GameServerBuilder::build_into()` calls.
pub fn init_gs_template(path: &Path) -> Result<(), String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read GameServer template at '{}': {e}", path.display()))?;

    let gs: GameServer = serde_yaml::from_str(&content)
        .map_err(|e| format!("Failed to parse GameServer template YAML at '{}': {e}", path.display()))?;

    let version = gs.metadata.name.as_ref()
        .and_then(|name| parse_version_by_gs_name(name))
        .unwrap_or("unknown".to_string());

    GS_TEMPLATE.set(gs)
        .map_err(|_| "GameServer template has already been initialized".to_string())?;
    GS_TEMPLATE_VERSION.set(version)
        .map_err(|_| "GameServer template version has already been initialized".to_string())?;

    Ok(())
}

pub fn gs_template() -> &'static GameServer {
    GS_TEMPLATE.get()
        .expect("GameServer template not initialized. Call init_gs_template() first.")
}

pub fn gs_template_version() -> &'static str {
    GS_TEMPLATE_VERSION.get()
        .expect("GameServer template version not initialized. Call init_gs_template() first.").as_str()
}

fn parse_version_by_gs_name(name: &str) -> Option<String> {
    name.rsplitn(2, '-').next()
        .filter(|part| part.starts_with('v'))
        .map(|part| part.to_string())
}

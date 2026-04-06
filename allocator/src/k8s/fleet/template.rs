use std::path::Path;
use std::sync::OnceLock;

use super::crd::Fleet;

static FLEET_TEMPLATE: OnceLock<Fleet> = OnceLock::new();
static FLEET_TEMPLATE_VERSION: OnceLock<String> = OnceLock::new();


/// Initialize the global fleet template from a YAML file.
/// Must be called exactly once at startup before any `FleetBuilder::build_into()` calls.
pub fn init_fleet_template(path: &Path) -> Result<(), String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read fleet template at '{}': {e}", path.display()))?;

    let fleet: Fleet = serde_yaml::from_str(&content)
        .map_err(|e| format!("Failed to parse fleet template YAML at '{}': {e}", path.display()))?;

    let version = fleet.metadata.name.as_ref()
        .map(|name| parse_version_by_fleet_name(name)).flatten()
        .unwrap_or("unknown".to_string());

    FLEET_TEMPLATE.set(fleet)
        .map_err(|_| "Fleet template has already been initialized".to_string())?;
    FLEET_TEMPLATE_VERSION.set(version)
        .map_err(|_| "Fleet template version has already been initialized".to_string())?;

    Ok(())
}

pub fn fleet_template() -> &'static Fleet {
    FLEET_TEMPLATE.get()
        .expect("Fleet template not initialized. Call init_fleet_template() first.")
}

pub fn fleet_template_version() -> &'static str {
    FLEET_TEMPLATE_VERSION.get()
        .expect("Fleet template version not initialized. Call init_fleet_template() first.").as_str()
}

fn parse_version_by_fleet_name(name: &str) -> Option<String> {
    name.rsplitn(2, '-').next()
        .filter(|part| part.starts_with('v'))
        .map(|part| part.to_string())
}


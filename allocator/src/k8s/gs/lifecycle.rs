pub const MANAGED_BY_LABEL: &str = "rcss.dev/managed-by";
pub const MANAGED_BY_ALLOCATOR: &str = "allocator";
pub const ALLOCATION_MODE_LABEL: &str = "rcss.dev/allocation-mode";
pub const ALLOCATION_MODE_DIRECT_GS: &str = "direct-gs";
pub const TEMPLATE_VERSION_LABEL: &str = "rcss.dev/template-version";

pub const CREATED_AT_ANNOTATION: &str = "rcss.dev/created-at-unix";
pub const LAST_HEARTBEAT_ANNOTATION: &str = "rcss.dev/last-heartbeat-unix";

pub fn now_unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub fn managed_direct_selector() -> String {
    format!(
        "{MANAGED_BY_LABEL}={MANAGED_BY_ALLOCATOR},{ALLOCATION_MODE_LABEL}={ALLOCATION_MODE_DIRECT_GS}"
    )
}

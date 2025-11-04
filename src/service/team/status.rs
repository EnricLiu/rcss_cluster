#[derive(Debug)]
pub enum TeamStatus {
    Idle,
    Playing,
}
impl Default for TeamStatus {
    fn default() -> Self {
        TeamStatus::Idle
    }
}
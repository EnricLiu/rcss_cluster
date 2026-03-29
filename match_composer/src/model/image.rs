use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct ImageInfo {
    pub provider: String,
    pub model: String,
    pub path: Box<std::path::Path>,
}

impl ImageInfo {
    pub fn new(provider: String, model: String, path: Box<std::path::Path>) -> Self {
        ImageInfo { provider, model, path }
    }
    
    pub fn to_raw(&self) -> String {
        format!("{}/{}", self.provider, self.model)
    }
}

mod metadata;
mod labels;
mod annotations;
mod from_v1;
mod label_serdes;

#[cfg(feature = "agones")]
pub use agones::Sdk;

pub use metadata::MetaData;

pub use labels::Labels;
pub use annotations::Annotations;

mod metadata;
mod labels;
mod annotations;
mod model;
mod declaration;

pub use agones::Sdk;

pub use metadata::MetaData as AgonesMetaData;

pub use labels::Labels as AgonesMetaDataLabels;
pub use annotations::Annotations as AgonesMetaDataAnnotations;

use declaration::Declaration;
use model::Model;

pub mod crd;
mod builder;
mod template;
mod client_impl;

use super::{Error, Result, K8sClient};

pub use template::{
    init_gs_template,
    gs_template,
    gs_template_version,
};

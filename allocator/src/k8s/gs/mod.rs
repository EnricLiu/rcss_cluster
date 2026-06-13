pub mod crd;
pub mod lifecycle;
mod sweeper;
mod builder;
mod client_impl;
mod template;

use super::{Error, Result, K8sClient};

pub use sweeper::GsSweepConfig;
pub use template::{
    init_gs_template,
    gs_template,
    gs_template_version,
};

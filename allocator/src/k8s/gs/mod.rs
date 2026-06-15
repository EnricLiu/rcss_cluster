pub mod lifecycle;
mod sweeper;
mod builder;
mod client_impl;
mod template;

use super::{Error, Result, K8sClient, crd};

pub use sweeper::GsSweepConfig;
pub use template::{
    init_gs_template,
    gs_template,
    gs_template_version,
};

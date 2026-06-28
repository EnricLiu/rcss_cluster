mod builder;
mod template;
mod client_impl;


use super::{Error, Result, K8sClient, crd};

pub use template::{
    init_fleet_template,
    fleet_template,
    fleet_template_version,
};

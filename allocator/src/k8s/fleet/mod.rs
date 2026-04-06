mod crd;
mod builder;
mod template;
mod client_impl;


use super::{Error, Result, K8sClient};

pub use template::{
    init_fleet_template,
    fleet_template,
    fleet_template_version,
};

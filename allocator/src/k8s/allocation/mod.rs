pub mod crd;
mod builder;
mod client_impl;


use super::{Error, Result, K8sClient};

pub use builder::GsAllocation;
pub use client_impl::{AllocationError, AllocationResult};

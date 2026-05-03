use optify::builder::{OptionsProviderBuilder, OptionsRegistryBuilder};
use rustler::{NifResult, ResourceArc};
use std::sync::Mutex;

use crate::provider::ProviderResource;

pub struct ProviderBuilderResource(pub Mutex<OptionsProviderBuilder>);

#[rustler::resource_impl]
impl rustler::Resource for ProviderBuilderResource {}

#[rustler::nif]
pub fn provider_builder_new() -> ResourceArc<ProviderBuilderResource> {
    ResourceArc::new(ProviderBuilderResource(Mutex::new(
        OptionsProviderBuilder::new(),
    )))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn provider_builder_add_directory(
    builder: ResourceArc<ProviderBuilderResource>,
    directory: String,
) -> NifResult<ResourceArc<ProviderBuilderResource>> {
    let mut guard = builder
        .0
        .lock()
        .map_err(|e| rustler::Error::Term(Box::new(format!("Lock poisoned: {}", e))))?;
    guard
        .add_directory(&directory)
        .map_err(|e| rustler::Error::Term(Box::new(e)))?;
    drop(guard);
    Ok(builder)
}

#[rustler::nif]
pub fn provider_builder_build(
    builder: ResourceArc<ProviderBuilderResource>,
) -> NifResult<ResourceArc<ProviderResource>> {
    let mut guard = builder
        .0
        .lock()
        .map_err(|e| rustler::Error::Term(Box::new(format!("Lock poisoned: {}", e))))?;
    match guard.build() {
        Ok(provider) => Ok(ResourceArc::new(ProviderResource(provider))),
        Err(e) => Err(rustler::Error::Term(Box::new(e))),
    }
}

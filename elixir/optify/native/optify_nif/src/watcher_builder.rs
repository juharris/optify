use optify::builder::{OptionsRegistryBuilder, OptionsWatcherBuilder};
use rustler::{NifResult, ResourceArc};
use std::sync::Mutex;

use crate::watcher::WatcherResource;

pub struct WatcherBuilderResource(pub Mutex<OptionsWatcherBuilder>);

#[rustler::resource_impl]
impl rustler::Resource for WatcherBuilderResource {}

#[rustler::nif]
pub fn watcher_builder_new() -> ResourceArc<WatcherBuilderResource> {
    ResourceArc::new(WatcherBuilderResource(Mutex::new(
        OptionsWatcherBuilder::new(),
    )))
}

#[rustler::nif]
pub fn watcher_builder_add_directory(
    builder: ResourceArc<WatcherBuilderResource>,
    directory: String,
) -> NifResult<ResourceArc<WatcherBuilderResource>> {
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

#[rustler::nif(schedule = "DirtyCpu")]
pub fn watcher_builder_build(
    builder: ResourceArc<WatcherBuilderResource>,
) -> NifResult<ResourceArc<WatcherResource>> {
    let mut guard = builder
        .0
        .lock()
        .map_err(|e| rustler::Error::Term(Box::new(format!("Lock poisoned: {}", e))))?;
    match guard.build() {
        Ok(watcher) => Ok(ResourceArc::new(WatcherResource(Mutex::new(watcher)))),
        Err(e) => Err(rustler::Error::Term(Box::new(e))),
    }
}

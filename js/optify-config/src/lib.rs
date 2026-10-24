#![deny(clippy::all)]

mod analysis;
mod builder_options;
mod metadata;
mod policies;
mod preferences;
mod provider;
mod scope;
mod watcher;
mod watcher_options;

#[macro_use]
extern crate napi_derive;

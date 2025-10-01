#![deny(clippy::all)]

mod metadata;
mod preferences;
mod provider;
mod watcher;
mod watcher_options;

#[macro_use]
extern crate napi_derive;

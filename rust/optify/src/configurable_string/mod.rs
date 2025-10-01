pub mod configurable_string_impl;
pub mod locator;

// Re-export the main types for easier access
pub use configurable_string_impl::{
    ConfigurableString, LoadedFiles, ReplacementObject, ReplacementValue,
};

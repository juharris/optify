#![deny(clippy::all)]

use optify::builder::{OptionsProviderBuilder, OptionsRegistryBuilder};
use optify::provider::{OptionsProvider, OptionsRegistry};

#[macro_use]
extern crate napi_derive;

#[napi(js_name = "OptionsProvider")]
pub struct JsOptionsProvider {
  inner: OptionsProvider,
}

#[napi(js_name = "OptionsProviderBuilder")]
pub struct JsOptionsProviderBuilder {
  inner: OptionsProviderBuilder,
}

#[napi]
impl JsOptionsProviderBuilder {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: OptionsProviderBuilder::new(),
        }
    }

    #[napi]
    pub fn add_directory(&mut self, directory: String) -> napi::Result<()> {
        let path = std::path::Path::new(&directory);
        self.inner.add_directory(path).map_err(|e| napi::Error::from_reason(e.to_string()))?;
        Ok(())
    }

    #[napi]
    pub fn build(&mut self) -> napi::Result<JsOptionsProvider> {
        let provider = self.inner.build().map_err(|e| napi::Error::from_reason(e.to_string()))?;
        Ok(JsOptionsProvider { inner: provider })
    }
}

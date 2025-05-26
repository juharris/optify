#![deny(clippy::all)]

use optify::builder::{OptionsProviderBuilder, OptionsRegistryBuilder};
use optify::convert_to_str_slice;
use optify::provider::{OptionsProvider, OptionsRegistry};

#[macro_use]
extern crate napi_derive;

#[napi(js_name = "OptionsProvider")]
pub struct JsOptionsProvider {
  inner: Option<OptionsProvider>,
}

#[napi]
impl JsOptionsProvider {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self { inner: None }
  }

  #[napi]
  pub fn features(&self) -> Vec<String> {
    self.inner.as_ref().unwrap().get_features()
  }

  #[napi]
  pub fn get_options_json(&self, key: String, feature_names: Vec<String>) -> napi::Result<String> {
    let feature_names = convert_to_str_slice!(feature_names);
    self
      .inner
      .as_ref()
      .unwrap()
      .get_options(&key, &feature_names)
      .map(|json| json.to_string())
      .map_err(|e| napi::Error::from_reason(e.to_string()))
  }
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
    self
      .inner
      .add_directory(path)
      .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    Ok(())
  }

  #[napi]
  pub fn build(&mut self) -> napi::Result<JsOptionsProvider> {
    let provider = self
      .inner
      .build()
      .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    Ok(JsOptionsProvider {
      inner: Some(provider),
    })
  }
}

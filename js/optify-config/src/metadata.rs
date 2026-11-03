#![deny(clippy::all)]

use napi::bindgen_prelude::FromNapiValue;

/// Information about a group of options.
#[napi(js_name = "OptionsMetadata")]
pub struct JsOptionsMetadata {
  pub(crate) inner: optify::schema::metadata::OptionsMetadata,
}

impl FromNapiValue for JsOptionsMetadata {
  unsafe fn from_napi_value(
    env: napi::sys::napi_env,
    value: napi::sys::napi_value,
  ) -> napi::Result<Self> {
    let value = unsafe { serde_json::Value::from_napi_value(env, value) }?;
    let inner = serde_json::from_value(value)
      .map_err(|error| napi::Error::from_reason(format!("Invalid options metadata: {error}")))?;
    Ok(Self { inner })
  }
}

#[napi]
impl JsOptionsMetadata {
  #[napi(
    constructor,
    ts_args_type = "metadata?: { aliases?: string[]; dependents?: string[]; details?: unknown; name?: string; owners?: string; path?: string } | null"
  )]
  pub fn new(metadata: Option<serde_json::Value>) -> napi::Result<Self> {
    let inner = match metadata {
      Some(metadata) => serde_json::from_value(metadata)
        .map_err(|error| napi::Error::from_reason(format!("Invalid options metadata: {error}")))?,
      None => optify::schema::metadata::OptionsMetadata::new(None, None, None, None, None, None),
    };
    Ok(Self { inner })
  }

  #[napi]
  pub fn aliases(&self) -> Option<Vec<String>> {
    self.inner.aliases.clone()
  }

  /// The canonical names of features that import this one.
  #[napi]
  pub fn dependents(&self) -> Option<Vec<String>> {
    self.inner.dependents.clone()
  }

  #[napi]
  pub fn details(&self) -> Option<serde_json::Value> {
    self.inner.details.clone()
  }

  #[napi]
  pub fn name(&self) -> Option<String> {
    self.inner.name.clone()
  }

  #[napi]
  pub fn owners(&self) -> Option<String> {
    self.inner.owners.clone()
  }

  #[napi]
  pub fn path(&self) -> Option<String> {
    self.inner.path.clone()
  }

  #[napi]
  pub fn to_json(&self) -> napi::Result<serde_json::Value> {
    serde_json::to_value(&self.inner).map_err(|error| {
      napi::Error::from_reason(format!("Failed to serialize options metadata: {error}"))
    })
  }
}

pub fn to_js_options_metadata(
  metadata: optify::schema::metadata::OptionsMetadata,
) -> JsOptionsMetadata {
  JsOptionsMetadata { inner: metadata }
}

#![deny(clippy::all)]

/// Information about a group of options.
#[napi(js_name = "OptionsMetadata")]
pub struct JsOptionsMetadata {
  inner: Option<optify::schema::metadata::OptionsMetadata>,
}

#[napi]
impl JsOptionsMetadata {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self { inner: None }
  }

  #[napi]
  pub fn aliases(&self) -> Option<Vec<String>> {
    self.inner.as_ref().and_then(|m| m.aliases.clone())
  }

  /// The canonical names of features that import this one.
  #[napi]
  pub fn dependents(&self) -> Option<Vec<String>> {
    self.inner.as_ref().and_then(|m| m.dependents.clone())
  }

  #[napi]
  pub fn name(&self) -> Option<String> {
    self.inner.as_ref().and_then(|m| m.name.clone())
  }

  #[napi]
  pub fn owners(&self) -> Option<String> {
    self.inner.as_ref().and_then(|m| m.owners.clone())
  }

  #[napi]
  pub fn path(&self) -> Option<String> {
    self.inner.as_ref().and_then(|m| m.path.clone())
  }
}

pub fn to_js_options_metadata(
  metadata: optify::schema::metadata::OptionsMetadata,
) -> JsOptionsMetadata {
  JsOptionsMetadata {
    inner: Some(metadata),
  }
}

#![deny(clippy::all)]

/// Options for configuring the Builder.
#[napi(js_name = "BuilderOptions")]
pub struct JsBuilderOptions {
  pub(crate) inner: optify::builder::BuilderOptions,
}

#[napi]
impl JsBuilderOptions {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      inner: optify::builder::BuilderOptions::default(),
    }
  }

  #[napi]
  pub fn set_track_file_references_mode(
    &mut self,
    #[napi(ts_arg_type = "\"NONE\" | \"CONFIGURABLE_STRINGS\"")] mode: String,
  ) -> napi::Result<()> {
    self.inner.track_file_references = match mode.as_str() {
      "NONE" => optify::builder::TrackReferenceMode::None,
      "CONFIGURABLE_STRINGS" => optify::builder::TrackReferenceMode::ConfigurableStrings,
      _ => {
        return Err(napi::Error::from_reason(format!(
          "Invalid trackFileReferences mode: {}",
          mode
        )))
      }
    };
    Ok(())
  }
}

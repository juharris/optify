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

  /// Retains each feature's own options and imports so that `analyze` can explain where values came from.
  /// This uses more memory, so it is meant for local development and debugging tools.
  #[napi]
  pub fn set_analysis_enabled(&mut self, is_analysis_enabled: bool) {
    self.inner.is_analysis_enabled = is_analysis_enabled;
  }

  #[napi]
  pub fn set_track_file_references_mode(
    &mut self,
    #[napi(ts_arg_type = "\"NONE\" | \"CONFIGURABLE_STRINGS\" | \"KEY_NAME\"")] mode: String,
  ) -> napi::Result<()> {
    self.inner.track_file_references = match mode.as_str() {
      "NONE" => optify::builder::TrackReferenceMode::None,
      "CONFIGURABLE_STRINGS" => optify::builder::TrackReferenceMode::ConfigurableStrings,
      "KEY_NAME" => optify::builder::TrackReferenceMode::KeyName,
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

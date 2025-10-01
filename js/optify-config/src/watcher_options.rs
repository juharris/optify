#![deny(clippy::all)]

/// Options for configuring the OptionsWatcher.
#[napi(js_name = "WatcherOptions")]
pub struct JsWatcherOptions {
  pub(crate) inner: optify::provider::WatcherOptions,
}

#[napi]
impl JsWatcherOptions {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      inner: optify::provider::WatcherOptions::default(),
    }
  }

  /// Sets the duration to wait before triggering a rebuild after file changes (in milliseconds).
  #[napi]
  pub fn set_debounce_duration_ms(&mut self, debounce_duration_ms: u32) {
    self.inner.debounce_duration = std::time::Duration::from_millis(debounce_duration_ms as u64);
  }

  /// Gets the duration to wait before triggering a rebuild after file changes (in milliseconds).
  #[napi]
  pub fn get_debounce_duration_ms(&self) -> u32 {
    self.inner.debounce_duration.as_millis() as u32
  }
}

use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

pub fn init<R: Runtime, C: DeserializeOwned>(
  app: &AppHandle<R>,
  _api: PluginApi<R, C>,
) -> crate::Result<NetworkInterfaces<R>> {
  Ok(NetworkInterfaces(app.clone()))
}

/// No-op handle on every platform except Android, where process network
/// binding does not apply.
pub struct NetworkInterfaces<R: Runtime>(#[allow(dead_code)] AppHandle<R>);

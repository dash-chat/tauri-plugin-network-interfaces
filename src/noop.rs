use serde::de::DeserializeOwned;
use tauri::{
  plugin::{PermissionState, PluginApi},
  AppHandle, Runtime,
};

use crate::PermissionStatus;

pub fn init<R: Runtime, C: DeserializeOwned>(
  app: &AppHandle<R>,
  _api: PluginApi<R, C>,
) -> crate::Result<NetworkInterfaces<R>> {
  Ok(NetworkInterfaces(app.clone()))
}

/// No-op handle on every platform except Android, where neither the multicast
/// lock nor the local network permission applies.
pub struct NetworkInterfaces<R: Runtime>(#[allow(dead_code)] AppHandle<R>);

impl<R: Runtime> NetworkInterfaces<R> {
  pub fn check_permissions(&self) -> crate::Result<PermissionStatus> {
    Ok(PermissionStatus {
      local_network: PermissionState::Granted,
    })
  }

  pub fn request_permissions(&self) -> crate::Result<PermissionStatus> {
    self.check_permissions()
  }
}

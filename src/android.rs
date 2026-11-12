use serde::{de::DeserializeOwned, Serialize};
use tauri::{
  plugin::{PluginApi, PluginHandle},
  AppHandle, Runtime,
};

use crate::PermissionStatus;

pub fn init<R: Runtime, C: DeserializeOwned>(
  _app: &AppHandle<R>,
  api: PluginApi<R, C>,
) -> crate::Result<NetworkInterfaces<R>> {
  let handle =
    api.register_android_plugin("org.dashchat.networkinterfaces", "NetworkInterfacesPlugin")?;
  Ok(NetworkInterfaces(handle))
}

#[derive(Serialize)]
struct RequestArgs {
  permissions: Option<Vec<String>>,
}

/// Handle to the Android network-interfaces plugin. Constructing it (via
/// [`init`]) acquires the WiFi multicast lock.
pub struct NetworkInterfaces<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> NetworkInterfaces<R> {
  pub fn check_permissions(&self) -> crate::Result<PermissionStatus> {
    self
      .0
      .run_mobile_plugin("checkPermissions", ())
      .map_err(Into::into)
  }

  /// Asks for every permission the plugin declares; below API 37 the Android
  /// side resolves as granted without prompting.
  pub fn request_permissions(&self) -> crate::Result<PermissionStatus> {
    self
      .0
      .run_mobile_plugin("requestPermissions", RequestArgs { permissions: None })
      .map_err(Into::into)
  }
}

use serde::de::DeserializeOwned;
use tauri::{
  plugin::{PluginApi, PluginHandle},
  AppHandle, Runtime,
};

pub fn init<R: Runtime, C: DeserializeOwned>(
  _app: &AppHandle<R>,
  api: PluginApi<R, C>,
) -> crate::Result<NetworkInterfaces<R>> {
  let handle =
    api.register_android_plugin("org.dashchat.networkinterfaces", "NetworkInterfacesPlugin")?;
  Ok(NetworkInterfaces(handle))
}

/// Handle to the Android network-interfaces plugin. Constructing it (via
/// [`init`]) acquires the WiFi multicast lock; there are no further APIs.
pub struct NetworkInterfaces<R: Runtime>(#[allow(dead_code)] PluginHandle<R>);

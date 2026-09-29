use serde::de::DeserializeOwned;
use tauri::{
  plugin::{PermissionState, PluginApi, PluginHandle},
  AppHandle, Runtime,
};

use crate::wifi::{AddArgs, ForgetArgs, RequestJoinArgs, WifiAddedSsids, WifiCurrent};
use crate::PermissionStatus;

tauri::ios_plugin_binding!(init_plugin_network_interfaces);

pub fn init<R: Runtime, C: DeserializeOwned>(
  _app: &AppHandle<R>,
  api: PluginApi<R, C>,
) -> crate::Result<NetworkInterfaces<R>> {
  let handle = api.register_ios_plugin(init_plugin_network_interfaces)?;
  Ok(NetworkInterfaces(handle))
}

/// Handle to the iOS plugin, which only exists for the Wi-Fi commands; the
/// local network permission does not apply on iOS.
pub struct NetworkInterfaces<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> NetworkInterfaces<R> {
  pub fn check_permissions(&self) -> crate::Result<PermissionStatus> {
    Ok(PermissionStatus {
      local_network: PermissionState::Granted,
    })
  }

  pub fn request_permissions(&self) -> crate::Result<PermissionStatus> {
    self.check_permissions()
  }

  pub async fn wifi_current(&self) -> crate::Result<WifiCurrent> {
    self
      .0
      .run_mobile_plugin_async("wifiCurrent", ())
      .await
      .map_err(Into::into)
  }

  pub async fn wifi_added_ssids(&self) -> crate::Result<WifiAddedSsids> {
    self
      .0
      .run_mobile_plugin_async("wifiAddedSsids", ())
      .await
      .map_err(Into::into)
  }

  pub async fn wifi_add(&self, args: AddArgs) -> crate::Result<()> {
    self
      .0
      .run_mobile_plugin_async("wifiAdd", args)
      .await
      .map_err(Into::into)
  }

  pub async fn wifi_request_join(&self, args: RequestJoinArgs) -> crate::Result<()> {
    self
      .0
      .run_mobile_plugin_async("wifiRequestJoin", args)
      .await
      .map_err(Into::into)
  }

  pub async fn wifi_forget(&self, args: ForgetArgs) -> crate::Result<()> {
    self
      .0
      .run_mobile_plugin_async("wifiForget", args)
      .await
      .map_err(Into::into)
  }
}

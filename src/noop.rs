use serde::de::DeserializeOwned;
use tauri::{
  plugin::{PermissionState, PluginApi},
  AppHandle, Runtime,
};

#[cfg(feature = "wifi-control")]
use crate::wifi::{AddArgs, ForgetArgs, RequestJoinArgs, WifiAddedSsids, WifiCurrent};
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

  #[cfg(feature = "wifi-control")]
  pub async fn wifi_current(&self) -> crate::Result<WifiCurrent> {
    Err(crate::Error::WifiUnsupported)
  }

  #[cfg(feature = "wifi-control")]
  pub async fn wifi_added_ssids(&self) -> crate::Result<WifiAddedSsids> {
    Err(crate::Error::WifiUnsupported)
  }

  #[cfg(feature = "wifi-control")]
  pub async fn wifi_add(&self, _args: AddArgs) -> crate::Result<()> {
    Err(crate::Error::WifiUnsupported)
  }

  #[cfg(feature = "wifi-control")]
  pub async fn wifi_request_join(&self, _args: RequestJoinArgs) -> crate::Result<()> {
    Err(crate::Error::WifiUnsupported)
  }

  #[cfg(feature = "wifi-control")]
  pub async fn wifi_forget(&self, _args: ForgetArgs) -> crate::Result<()> {
    Err(crate::Error::WifiUnsupported)
  }
}

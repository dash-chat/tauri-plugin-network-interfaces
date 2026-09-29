use serde::{Deserialize, Serialize};
use tauri::{
    plugin::{Builder, PermissionState, TauriPlugin},
    AppHandle, Manager, Runtime,
};

#[cfg(target_os = "android")]
#[path = "android.rs"]
mod imp;
#[cfg(all(target_os = "ios", feature = "wifi-control"))]
#[path = "ios.rs"]
mod imp;
#[cfg(not(any(
  target_os = "android",
  all(target_os = "ios", feature = "wifi-control")
)))]
#[path = "noop.rs"]
mod imp;

mod error;
#[cfg(feature = "wifi-control")]
pub mod wifi;

pub use error::{Error, Result};
pub use imp::NetworkInterfaces;

/// State of the permission guarding access to the local network.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionStatus {
    pub local_network: PermissionState,
}

#[tauri::command]
async fn check_permissions<R: Runtime>(app: AppHandle<R>) -> Result<PermissionStatus> {
    app.state::<NetworkInterfaces<R>>().check_permissions()
}

#[tauri::command]
async fn request_permissions<R: Runtime>(app: AppHandle<R>) -> Result<PermissionStatus> {
    app.state::<NetworkInterfaces<R>>().request_permissions()
}

/// Initializes the plugin.
///
/// On Android it acquires a WiFi multicast lock so inbound mDNS reaches our
/// sockets (Android otherwise filters multicast not addressed to the device,
/// which silently breaks LAN-only peer discovery), and exposes the local
/// network permission that gates every LAN socket from API 37 on. On all other
/// platforms the lock is a no-op and the permission reads as granted.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    let builder = Builder::new("network-interfaces");
    #[cfg(feature = "wifi-control")]
    let builder = builder.invoke_handler(tauri::generate_handler![
        check_permissions,
        request_permissions,
        wifi::wifi_current,
        wifi::wifi_added_ssids,
        wifi::wifi_add,
        wifi::wifi_request_join,
        wifi::wifi_forget
    ]);
    #[cfg(not(feature = "wifi-control"))]
    let builder = builder.invoke_handler(tauri::generate_handler![
        check_permissions,
        request_permissions
    ]);
    builder
        .setup(|app, api| {
            let network_interfaces = imp::init(app, api)?;
            app.manage(network_interfaces);
            Ok(())
        })
        .build()
}

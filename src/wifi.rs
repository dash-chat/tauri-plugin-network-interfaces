//! Wi-Fi control for test harnesses, behind the `wifi-control` feature.
//!
//! The model is the one iOS imposes: an app owns the networks it configured
//! and nothing else. `wifi_add` registers one as the app's — the platform
//! then joins it when it sees fit, which on iOS is at once and on Android
//! only when nothing the user saved is in range — `wifi_request_join` (iOS
//! only) adds one, has the user asked, and returns once the device is on it
//! or the request failed, `wifi_forget` drops one
//! (the OS then falls back to whatever the user has saved),
//! `wifi_added_ssids` lists the app's own, and `wifi_current` reads the
//! interface. The SSID in `wifi_current` is only known where the OS lets the
//! app see it — on iOS that is the networks the app configured itself — so
//! the address is the reliable "on a network" signal.
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Runtime};

use crate::{NetworkInterfaces, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WifiCurrent {
  /// The SSID the device is on, or "" when it is on none or the OS hides it.
  pub ssid: String,
  /// The IPv4 address of the Wi-Fi interface, or "" while it has none.
  pub address: String,
  /// The prefix length of that address's subnet, 0 while there is none.
  pub prefix_length: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiAddedSsids {
  /// The SSIDs this app added, in range or not.
  pub ssids: Vec<String>,
}

#[derive(Serialize)]
pub struct AddArgs {
  pub ssid: String,
  pub passphrase: String,
}

pub type RequestJoinArgs = AddArgs;

#[derive(Serialize)]
pub struct ForgetArgs {
  pub ssid: String,
}

#[tauri::command]
pub async fn wifi_current<R: Runtime>(app: AppHandle<R>) -> Result<WifiCurrent> {
  app.state::<NetworkInterfaces<R>>().wifi_current().await
}

#[tauri::command]
pub async fn wifi_added_ssids<R: Runtime>(app: AppHandle<R>) -> Result<WifiAddedSsids> {
  app.state::<NetworkInterfaces<R>>().wifi_added_ssids().await
}

/// Save `ssid` (an empty `passphrase` means an open network) as one of this
/// app's networks; the platform joins it when it sees fit.
#[tauri::command]
pub async fn wifi_add<R: Runtime>(
  app: AppHandle<R>,
  ssid: String,
  passphrase: String,
) -> Result<()> {
  app
    .state::<NetworkInterfaces<R>>()
    .wifi_add(AddArgs { ssid, passphrase })
    .await
}

/// Add `ssid`, have the user asked to join it, and resolve once the device
/// is on it with an address, or reject with why not. iOS only: no other
/// platform lets an app so much as ask.
#[tauri::command]
pub async fn wifi_request_join<R: Runtime>(
  app: AppHandle<R>,
  ssid: String,
  passphrase: String,
) -> Result<()> {
  app
    .state::<NetworkInterfaces<R>>()
    .wifi_request_join(RequestJoinArgs { ssid, passphrase })
    .await
}

/// Drop `ssid` from this app's networks, leaving it if the device is on it.
#[tauri::command]
pub async fn wifi_forget<R: Runtime>(app: AppHandle<R>, ssid: String) -> Result<()> {
  app
    .state::<NetworkInterfaces<R>>()
    .wifi_forget(ForgetArgs { ssid })
    .await
}

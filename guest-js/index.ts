import { invoke } from '@tauri-apps/api/core'

export type PermissionState = 'granted' | 'denied' | 'prompt' | 'prompt-with-rationale'

/** State of the permission guarding access to the local network. */
export interface PermissionStatus {
  localNetwork: PermissionState
}

export function checkPermissions(): Promise<PermissionStatus> {
  return invoke('plugin:network-interfaces|check_permissions')
}

export function requestPermissions(): Promise<PermissionStatus> {
  return invoke('plugin:network-interfaces|request_permissions')
}

/**
 * Wi-Fi control, available when the plugin is built with its `wifi-control`
 * cargo feature. The model is the one iOS imposes: an app owns the networks
 * it configured and nothing else. `add` registers one as the app's,
 * `requestJoin` (iOS only) adds one, asks, and waits for the device to be on
 * it, `forget` drops
 * one (the OS then falls back to whatever the user has saved), `addedSsids`
 * lists the app's own, and `current` reads the interface.
 */
export interface WifiCurrent {
  /** The SSID the device is on, or "" when it is on none or the OS hides it
   *  (on iOS every network the app did not configure itself). */
  ssid: string
  /** The IPv4 address of the Wi-Fi interface, or "" while it has none. */
  address: string
  /** The prefix length of that address's subnet, 0 while there is none. */
  prefixLength: number
}

export interface WifiAddedSsids {
  /** The SSIDs this app added, in range or not. */
  ssids: string[]
}

export function wifiCurrent(): Promise<WifiCurrent> {
  return invoke('plugin:network-interfaces|wifi_current')
}

export function wifiAddedSsids(): Promise<WifiAddedSsids> {
  return invoke('plugin:network-interfaces|wifi_added_ssids')
}

/** Save `ssid` (an empty `passphrase` means an open network) as one of this
 *  app's networks. The platform joins it when it sees fit: iOS at once,
 *  behind its "join?" alert; Android only when nothing the user saved is in
 *  range, and after a one-time approval of the app's suggestions. */
export function wifiAdd(ssid: string, passphrase: string): Promise<void> {
  return invoke('plugin:network-interfaces|wifi_add', { ssid, passphrase })
}

/** Add `ssid` and ask to join it — iOS puts the question to the user — and
 *  resolve once the device is on it with an address, or reject with why not
 *  (the user declining, or 90 s without an association). iOS only; every
 *  other platform rejects. */
export function wifiRequestJoin(ssid: string, passphrase: string): Promise<void> {
  return invoke('plugin:network-interfaces|wifi_request_join', { ssid, passphrase })
}

/** Drop `ssid` from this app's networks, leaving it if the device is on it. */
export function wifiForget(ssid: string): Promise<void> {
  return invoke('plugin:network-interfaces|wifi_forget', { ssid })
}

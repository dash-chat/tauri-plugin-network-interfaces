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

# Tauri Plugin: Network Interfaces

A Tauri plugin that pins the whole process — every future socket and all DNS
lookups — to the network Android chose as the app's default, re-pinning whenever
that default changes.

On a multi-homed Android device (WiFi + cellular up at once, sometimes a third
IMS network with an empty DNS list) a Rust QUIC/p2p stack such as
[iroh](https://iroh.computer) binds its UDP sockets to `0.0.0.0`/`[::]` and lets
the OS routing table pick the source interface. The DNS resolver can then leak
onto a cellular interface whose DNS is broken, so the relay never resolves and
every peer connection times out — a reconnect storm that pegs the CPU. Binding
the process to Android's validated default network via
`ConnectivityManager.bindProcessToNetwork()` keeps DNS **and** sockets on one
coherent interface.

## Platform Support

| Platform | Process pinning / multicast lock | Wi-Fi control (`wifi-control` feature) |
|----------|-----------|-----------|
| Android  | Yes       | add / forget / list / current (network suggestions, API 29+); no requestJoin |
| iOS      | No (no-op) | add / requestJoin / forget / list / current (`NEHotspotConfiguration`) |
| Desktop  | No (no-op) | No (commands reject) |

iOS does not need this: a non-VPN iOS app is suspended in the background (no
long-lived node to storm), and there is no `bindProcessToNetwork` equivalent. If
iOS interface pinning is ever needed it is a per-socket `IP_BOUND_IF` +
`NWPathMonitor` job, which requires the networking stack to expose its socket —
out of scope here.

## Installation

Add the plugin to your `Cargo.toml`:

```toml
[dependencies]
tauri-plugin-network-interfaces = { git = "https://github.com/dash-chat/tauri-plugin-network-interfaces" }
```

## Usage

Register the plugin in your Tauri application:

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_network_interfaces::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

That's it — no commands, no JS API. The plugin starts the default-network
binding when it loads.

The required `android.permission.ACCESS_NETWORK_STATE` permission is declared in
the plugin's manifest and merged into the host app automatically.

### Timing

`bindProcessToNetwork` only affects sockets created **after** the bind. The
plugin binds on load (`Plugin.load`), which runs early in the Android activity
lifecycle. Register this plugin **before** the plugin/code that creates your
networking stack's sockets (e.g. the iroh endpoint) so those sockets inherit the
binding. Sockets opened before the bind are not retroactively re-routed.

## How It Works

1. On load, registers a `registerDefaultNetworkCallback` (API 24+).
2. On `onAvailable`, calls `bindProcessToNetwork(network)` — pinning all future
   sockets and all DNS resolution to that network.
3. On `onLost`, calls `bindProcessToNetwork(null)` to clear the binding; the
   next default network re-pins via `onAvailable`.

It deliberately does **not** require `NET_CAPABILITY_VALIDATED`, so it still
binds on LAN-only / offline networks (mDNS p2p, a local mailbox).

## Wi-Fi control (`wifi-control` feature)

For test harnesses that need to walk a phone across networks. Enable the cargo
feature and grant the `network-interfaces:wifi-control` permission set; the JS
bindings are `wifiCurrent`, `wifiAddedSsids`, `wifiAdd`, `wifiRequestJoin` and
`wifiForget` in `tauri-plugin-network-interfaces-api`.

The model is the one iOS imposes: the app owns the networks it configured and
nothing else. `wifiAdd(ssid, passphrase)` registers one as the app's and the
platform joins it when it sees fit; `wifiRequestJoin(ssid, passphrase)` (iOS
only) adds one, has the user asked, and resolves once the device is on it with
an address or rejects with why not;
`wifiForget(ssid)` drops one and the OS falls back to whatever the user has
saved; `wifiAddedSsids()` lists the app's own (in range or not); and
`wifiCurrent()` reads the interface: `{ ssid, address, prefixLength }`, where
`address` is the IPv4 of the Wi-Fi interface ("" while there is none),
`prefixLength` its subnet's (0 while there is none), and `ssid` is only known
where the OS lets the app see it.

- **iOS** needs the `com.apple.developer.networking.HotspotConfiguration` and
  `com.apple.developer.networking.wifi-info` entitlements on the host app (both
  self-serve; automatic signing adds them to the App ID). Every join shows a
  system "join this network?" alert. `ssid` is "" for networks the app did not
  configure unless the app also holds location permission. The radio cannot be
  turned off or on.
- **Android** uses `WifiNetworkSuggestion`, so there is no `wifiRequestJoin`: a
  suggestion is joined when the platform picks it as the best candidate, and a
  network the user saved outranks it while in range. The user has to approve
  the app's suggestions once (`adb shell cmd wifi network-suggestions-set-user-approved <package>
  yes` does it from a harness). `wifiAddedSsids` needs API 30. `ssid` is
  "" without location permission.


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

| Platform | Supported |
|----------|-----------|
| Android  | Yes       |
| iOS      | No (no-op) |
| Desktop  | No (no-op) |

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

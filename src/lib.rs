use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

#[cfg(target_os = "android")]
#[path = "android.rs"]
mod imp;
#[cfg(not(target_os = "android"))]
#[path = "noop.rs"]
mod imp;

mod error;

pub use error::{Error, Result};
pub use imp::NetworkInterfaces;

/// Initializes the plugin.
///
/// On Android this registers a default-network callback that pins the whole
/// process — every future socket and all DNS lookups — to the network Android
/// chose as the app's default, re-pinning whenever that default changes. On all
/// other platforms it is a no-op.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("network-interfaces")
        .setup(|app, api| {
            let network_interfaces = imp::init(app, api)?;
            app.manage(network_interfaces);
            Ok(())
        })
        .build()
}

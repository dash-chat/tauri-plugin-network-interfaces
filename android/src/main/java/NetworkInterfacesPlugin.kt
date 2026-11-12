package org.dashchat.networkinterfaces

import android.app.Activity
import android.os.Build
import android.webkit.WebView
import app.tauri.annotation.Command
import app.tauri.annotation.Permission
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin

/** `Manifest.permission.ACCESS_LOCAL_NETWORK` only exists from API 37, so the
 *  literal is what keeps this compiling against a lower compileSdk. */
private const val ACCESS_LOCAL_NETWORK = "android.permission.ACCESS_LOCAL_NETWORK"
private const val ALIAS_LOCAL_NETWORK = "localNetwork"

/** The release that starts gating local network access behind a runtime
 *  permission. Below it access comes with INTERNET and asking would prompt for
 *  a permission the platform does not enforce. */
private const val LOCAL_NETWORK_PERMISSION_SDK = 37

@TauriPlugin(
    permissions = [
        Permission(strings = [ACCESS_LOCAL_NETWORK], alias = ALIAS_LOCAL_NETWORK)
    ]
)
class NetworkInterfacesPlugin(private val activity: Activity) : Plugin(activity) {
    override fun load(webView: WebView) {
        super.load(webView)
        MulticastLockHolder.acquire(activity.applicationContext)
    }

    @Command
    override fun checkPermissions(invoke: Invoke) {
        if (enforcesLocalNetworkPermission()) super.checkPermissions(invoke)
        else invoke.resolve(implicitlyGranted())
    }

    @Command
    override fun requestPermissions(invoke: Invoke) {
        if (enforcesLocalNetworkPermission()) super.requestPermissions(invoke)
        else invoke.resolve(implicitlyGranted())
    }

    private fun enforcesLocalNetworkPermission() =
        Build.VERSION.SDK_INT >= LOCAL_NETWORK_PERMISSION_SDK

    private fun implicitlyGranted() =
        JSObject().apply { put(ALIAS_LOCAL_NETWORK, "granted") }
}

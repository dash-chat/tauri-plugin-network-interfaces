package org.dashchat.networkinterfaces

import android.app.Activity
import android.webkit.WebView
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Plugin

@TauriPlugin
class NetworkInterfacesPlugin(private val activity: Activity) : Plugin(activity) {
    override fun load(webView: WebView) {
        super.load(webView)
        NetworkBinder.start(activity.applicationContext)
    }
}

package org.dashchat.networkinterfaces

import android.content.Context
import android.net.ConnectivityManager
import android.net.NetworkCapabilities
import android.net.wifi.WifiInfo
import android.net.wifi.WifiManager
import android.net.wifi.WifiNetworkSuggestion
import android.os.Build
import java.net.Inet4Address

/** The SSID `WifiInfo` reports while the app may not see it (no location
 *  permission, or location off): treated as unknown. */
private const val UNKNOWN_SSID = "<unknown ssid>"

/**
 * Wi-Fi control through the network-suggestion API (API 29+), the closest
 * Android comes to iOS's app-owned networks: a suggestion is saved under this
 * app, listed and removed by it, and the platform joins it when it is a
 * candidate. A suggestion never forces the switch — a network the user saved
 * outranks it while in range — which is why there is no join here, only add.
 * The user has to approve this app's suggestions once
 * (`cmd wifi network-suggestions-set-user-approved` does it from adb).
 */
object WifiControl {
    data class Current(val ssid: String, val address: String, val prefixLength: Int)

    class Unsupported(message: String) : Exception(message)

    class Failed(message: String) : Exception(message)

    fun current(context: Context): Current {
        val cm = context.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager
        val wifiNetwork = cm.allNetworks.firstOrNull { network ->
            cm.getNetworkCapabilities(network)?.hasTransport(NetworkCapabilities.TRANSPORT_WIFI) == true
        } ?: return Current("", "", 0)
        val link = cm.getLinkProperties(wifiNetwork)?.linkAddresses
            ?.firstOrNull { it.address is Inet4Address }
        return Current(
            ssid(context, cm, wifiNetwork),
            link?.address?.hostAddress ?: "",
            link?.prefixLength ?: 0,
        )
    }

    private fun ssid(context: Context, cm: ConnectivityManager, network: android.net.Network): String {
        val info: WifiInfo? = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            cm.getNetworkCapabilities(network)?.transportInfo as? WifiInfo
        } else {
            @Suppress("DEPRECATION")
            wifiManager(context).connectionInfo
        }
        val raw = info?.ssid ?: return ""
        if (raw == UNKNOWN_SSID) return ""
        return raw.removeSurrounding("\"")
    }

    fun addedSsids(context: Context): List<String> {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.R) return emptyList()
        return wifiManager(context).networkSuggestions.mapNotNull { it.ssid }
    }

    fun add(context: Context, ssid: String, passphrase: String) {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.Q) {
            throw Unsupported("network suggestions need Android 10")
        }
        val suggestion = suggestion(ssid, passphrase)
        val status = wifiManager(context).addNetworkSuggestions(listOf(suggestion))
        if (status != WifiManager.STATUS_NETWORK_SUGGESTIONS_SUCCESS &&
            status != WifiManager.STATUS_NETWORK_SUGGESTIONS_ERROR_ADD_DUPLICATE
        ) {
            throw Failed("addNetworkSuggestions failed with status $status")
        }
    }

    fun forget(context: Context, ssid: String) {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.Q) {
            throw Unsupported("network suggestions need Android 10")
        }
        val wifi = wifiManager(context)
        val ours = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            wifi.networkSuggestions.filter { it.ssid == ssid }
        } else {
            listOf(suggestion(ssid, ""))
        }
        if (ours.isEmpty()) return
        val status = wifi.removeNetworkSuggestions(ours)
        if (status != WifiManager.STATUS_NETWORK_SUGGESTIONS_SUCCESS) {
            throw Failed("removeNetworkSuggestions failed with status $status")
        }
    }

    private fun suggestion(ssid: String, passphrase: String): WifiNetworkSuggestion {
        val builder = WifiNetworkSuggestion.Builder().setSsid(ssid)
        if (passphrase.isNotEmpty()) builder.setWpa2Passphrase(passphrase)
        return builder.build()
    }

    private fun wifiManager(context: Context) =
        context.applicationContext.getSystemService(Context.WIFI_SERVICE) as WifiManager
}

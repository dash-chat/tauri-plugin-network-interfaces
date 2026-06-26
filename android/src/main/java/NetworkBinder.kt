package org.dashchat.networkinterfaces

import android.content.Context
import android.net.ConnectivityManager
import android.net.Network
import android.os.Build
import android.util.Log

/**
 * Pins the whole process — all sockets and DNS lookups — to Android's current
 * default network, re-binding whenever that network changes.
 *
 * On multi-homed devices (WiFi + cellular up at once, sometimes a third IMS
 * network with no DNS servers) a QUIC/p2p stack's DNS resolver and UDP sockets
 * would otherwise leak onto a cellular interface whose DNS is broken, so the
 * relay never resolves and every peer connection times out — a reconnect storm
 * that burns the CPU. Binding the process to the network Android already chose
 * as the default keeps that traffic on a single coherent interface.
 *
 * We bind to whatever the OS picks as the app's default network and deliberately
 * do NOT require [android.net.NetworkCapabilities.NET_CAPABILITY_VALIDATED]:
 * LAN-only / offline networks (mDNS p2p, a local mailbox) never validate because
 * they have no public internet, and gating on validation would refuse to bind on
 * exactly those networks.
 */
object NetworkBinder {
    private const val TAG = "NetworkBinder"

    private var registered = false

    fun start(context: Context) {
        // registerDefaultNetworkCallback is API 24+; no-op on older devices.
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.N) return

        // load() can run again on Activity recreation; only register once per
        // process or we leak callbacks against the 100-per-UID limit.
        if (registered) return
        registered = true

        val cm = context.applicationContext
            .getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager
        cm.registerDefaultNetworkCallback(object : ConnectivityManager.NetworkCallback() {
            override fun onAvailable(network: Network) {
                val bound = cm.bindProcessToNetwork(network)
                Log.i(TAG, "Bound process to default network $network (success=$bound)")
            }

            override fun onLost(network: Network) {
                // Don't query cm.activeNetwork here — synchronous getters race
                // inside callbacks. Clear the binding so the process is no longer
                // pinned to the network that just left; the next default network
                // arrives via onAvailable and re-pins.
                cm.bindProcessToNetwork(null)
                Log.i(TAG, "Default network $network lost; cleared process binding")
            }
        })
    }
}

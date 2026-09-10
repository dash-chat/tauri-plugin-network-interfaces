package org.dashchat.networkinterfaces

import android.content.Context
import android.net.wifi.WifiManager
import android.util.Log

/**
 * Acquires and holds a [WifiManager.MulticastLock] for the process lifetime so
 * inbound multicast — specifically mDNS on 224.0.0.251 / ff02::fb — reaches the
 * app's sockets.
 *
 * To save power Android's WiFi stack filters out packets that aren't unicast-
 * addressed to the device, which silently drops the mDNS responses our userspace
 * p2p discovery (iroh / swarm-discovery / mdns-sd) depends on. On an internet-less
 * LAN mDNS is the only way two peers can find each other — there is no relay, DNS
 * discovery or cloud mailbox to fall back on — so without this lock discovery
 * never resolves the peer's address and messages can't be delivered. A pure-Rust
 * mDNS stack can't take the lock itself (it's a `WifiManager` Java API), so the
 * host must.
 *
 * Holding the lock has a small battery cost, so it is paired with our
 * continuously running p2p node: acquired once and held for the process
 * lifetime rather than toggled per discovery session. Requires the
 * `CHANGE_WIFI_MULTICAST_STATE` permission (declared in the plugin manifest,
 * merged into the host app).
 */
object MulticastLockHolder {
    private const val TAG = "MulticastLock"

    private var lock: WifiManager.MulticastLock? = null

    fun acquire(context: Context) {
        // load() can run again on Activity recreation; only acquire once per
        // process so we don't leak locks.
        if (lock != null) return

        val wifi = context.applicationContext
            .getSystemService(Context.WIFI_SERVICE) as? WifiManager
        if (wifi == null) {
            Log.w(TAG, "WifiManager unavailable; cannot acquire multicast lock")
            return
        }

        lock = wifi.createMulticastLock("dashchat-mdns").apply {
            setReferenceCounted(false)
            acquire()
            Log.i(TAG, "Acquired WiFi multicast lock (held=$isHeld)")
        }
    }
}

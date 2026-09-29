import Foundation
import NetworkExtension
import SwiftRs
import Tauri
import UIKit

class AddArgs: Decodable {
  let ssid: String
  let passphrase: String
}

/// How long a join gets to associate and obtain an address; comfortably
/// longer than a WPA2 association plus DHCP on a busy access point.
private let JOIN_TIMEOUT: TimeInterval = 90
private let JOIN_POLL: TimeInterval = 0.5

class ForgetArgs: Decodable {
  let ssid: String
}

struct WifiCurrent: Encodable {
  let ssid: String
  let address: String
  let prefixLength: Int
}

struct Ipv4Interface {
  let address: String
  let prefixLength: Int
}

struct WifiAddedSsids: Encodable {
  let ssids: [String]
}

private let WIFI_INTERFACE = "en0"

func numericHost(_ addr: UnsafeMutablePointer<sockaddr>) -> String? {
  var host = [CChar](repeating: 0, count: Int(NI_MAXHOST))
  let ok =
    getnameinfo(
      addr, socklen_t(addr.pointee.sa_len), &host, socklen_t(host.count), nil, 0, NI_NUMERICHOST)
    == 0
  return ok ? String(cString: host) : nil
}

func prefixLength(ofMask mask: String) -> Int {
  return mask.split(separator: ".").compactMap { UInt8($0) }.reduce(0) { $0 + $1.nonzeroBitCount }
}

/// The IPv4 address and prefix length of `interface`, or nil while it has
/// none.
func ipv4Interface(_ interface: String) -> Ipv4Interface? {
  var addrs: UnsafeMutablePointer<ifaddrs>?
  guard getifaddrs(&addrs) == 0, let first = addrs else { return nil }
  defer { freeifaddrs(addrs) }
  var result: Ipv4Interface? = nil
  for ptr in sequence(first: first, next: { $0.pointee.ifa_next }) {
    let ifa = ptr.pointee
    guard let addr = ifa.ifa_addr, addr.pointee.sa_family == UInt8(AF_INET),
      String(cString: ifa.ifa_name) == interface, let address = numericHost(addr)
    else { continue }
    let mask = ifa.ifa_netmask.flatMap(numericHost) ?? ""
    result = Ipv4Interface(address: address, prefixLength: prefixLength(ofMask: mask))
  }
  return result
}

func describe(_ error: Error) -> String {
  let e = error as NSError
  return "\(e.domain) \(e.code): \(e.localizedDescription)"
}

/// Wi-Fi control through `NEHotspotConfiguration`, which is what iOS lets an
/// app do: join a named network (behind a system "join?" alert), keep it as
/// one of the app's own, and drop it again. Needs the HotspotConfiguration
/// and wifi-info entitlements on the host app; `fetchCurrent` names only the
/// networks the app configured unless it also holds location permission.
class NetworkInterfacesPlugin: Plugin {
  @objc public func wifiCurrent(_ invoke: Invoke) throws {
    NEHotspotNetwork.fetchCurrent { network in
      let iface = ipv4Interface(WIFI_INTERFACE)
      invoke.resolve(
        WifiCurrent(
          ssid: network?.ssid ?? "", address: iface?.address ?? "",
          prefixLength: iface?.prefixLength ?? 0))
    }
  }

  @objc public func wifiAddedSsids(_ invoke: Invoke) throws {
    NEHotspotConfigurationManager.shared.getConfiguredSSIDs { ssids in
      invoke.resolve(WifiAddedSsids(ssids: ssids))
    }
  }

  private func apply(_ args: AddArgs, completion: @escaping (Error?) -> Void) {
    let config =
      args.passphrase.isEmpty
      ? NEHotspotConfiguration(ssid: args.ssid)
      : NEHotspotConfiguration(ssid: args.ssid, passphrase: args.passphrase, isWEP: false)
    // A joinOnce network is dropped as soon as the app leaves the foreground,
    // which would also make it impossible to forget deliberately.
    config.joinOnce = false
    NEHotspotConfigurationManager.shared.apply(config, completionHandler: completion)
  }

  /// Save the network as one of the app's. iOS joins it in the same breath
  /// (behind its "join?" alert), so this resolves once the configuration is
  /// accepted, not once the device is on it; see `wifiRequestJoin` for that.
  @objc public func wifiAdd(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(AddArgs.self)
    apply(args) { error in
      if let error = error {
        invoke.reject(describe(error))
      } else {
        invoke.resolve()
      }
    }
  }

  /// Ask to join: iOS puts its "join?" alert to the user, and this resolves
  /// once the device is on the network with an address, or rejects with why
  /// not — the user declining included.
  @objc public func wifiRequestJoin(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(AddArgs.self)
    apply(args) { error in
      if let error = error {
        invoke.reject(describe(error))
        return
      }
      self.waitForAssociation(
        with: args.ssid, until: Date().addingTimeInterval(JOIN_TIMEOUT), invoke)
    }
  }

  private func waitForAssociation(with ssid: String, until deadline: Date, _ invoke: Invoke) {
    NEHotspotNetwork.fetchCurrent { network in
      let address = ipv4Interface(WIFI_INTERFACE)?.address ?? ""
      if network?.ssid == ssid && !address.isEmpty {
        invoke.resolve()
        return
      }
      if Date() > deadline {
        invoke.reject(
          "still not on \"\(ssid)\" with an address \(Int(JOIN_TIMEOUT))s after joining "
            + "(on \"\(network?.ssid ?? "")\", address \"\(address)\")")
        return
      }
      DispatchQueue.global().asyncAfter(deadline: .now() + JOIN_POLL) {
        self.waitForAssociation(with: ssid, until: deadline, invoke)
      }
    }
  }

  @objc public func wifiForget(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(ForgetArgs.self)
    NEHotspotConfigurationManager.shared.removeConfiguration(forSSID: args.ssid)
    invoke.resolve()
  }
}

@_cdecl("init_plugin_network_interfaces")
func initPlugin() -> Plugin {
  return NetworkInterfacesPlugin()
}

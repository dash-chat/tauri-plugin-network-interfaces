// swift-tools-version:5.3
import PackageDescription

let package = Package(
    name: "tauri-plugin-network-interfaces",
    platforms: [.iOS(.v13)],
    products: [
        .library(
            name: "tauri-plugin-network-interfaces",
            type: .static,
            targets: ["tauri-plugin-network-interfaces"])
    ],
    dependencies: [
        .package(name: "Tauri", path: "../.tauri/tauri-api")
    ],
    targets: [
        .target(
            name: "tauri-plugin-network-interfaces",
            dependencies: [.byName(name: "Tauri")],
            path: "Sources")
    ]
)

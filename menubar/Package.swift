// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "MenuBar",
    targets: [
        .target(
            name: "MenuBarCore",
            path: "Sources/MenuBarCore"
        ),
        .executableTarget(
            name: "MenuBar",
            dependencies: ["MenuBarCore"],
            path: "Sources/MenuBar"
        ),
        .testTarget(
            name: "MenuBarTests",
            dependencies: ["MenuBarCore"],
            path: "Tests/MenuBarTests"
        ),
    ]
)

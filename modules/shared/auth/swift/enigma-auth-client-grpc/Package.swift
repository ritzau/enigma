// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "auth-grpc-client",
    products: [
        .library(
            name: "EnigmaAuth",
            targets: ["EnigmaAuth"]),
    ],
    dependencies: [
        .package(url: "https://github.com/grpc/grpc-swift", .upToNextMajor(from: "1.24.2")),
    ],
    targets: [
        .target(
            name: "EnigmaAuth",
            dependencies: [
                .product(name: "GRPC", package: "grpc-swift"),
            ]
        ),
        .testTarget(
            name: "auth-grpcs-client-tests",
            dependencies: ["EnigmaAuth"]
        ),
    ]
)

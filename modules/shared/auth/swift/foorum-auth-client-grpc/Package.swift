// swift-tools-version: 6.0
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "grpc-client",
    products: [
        // Products define the executables and libraries a package produces, making them visible to other packages.
        .library(
            name: "FoorumGrpcClient",
            targets: ["FoorumGrpcClient"]),
    ],
    dependencies: [
        .package(url: "https://github.com/grpc/grpc-swift", .upToNextMajor(from: "1.24.2")),
    ],
    targets: [
        .target(
            name: "FoorumGrpcClient",
            dependencies: [
                .product(name: "GRPC", package: "grpc-swift"), // Add the GRPC dependency here
            ]
        ),
        .testTarget(
            name: "grpcs-client-tests",
            dependencies: ["FoorumGrpcClient"]
        ),
    ]
)

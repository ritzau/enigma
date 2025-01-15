// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "auth-grpc-client",
    products: [
        .library(
            name: "EnigmaAuthGrpcClient",
            targets: ["EnigmaAuthGrpcClient"]),
    ],
    dependencies: [
        .package(url: "https://github.com/grpc/grpc-swift", .upToNextMajor(from: "1.24.2")),
    ],
    targets: [
        .target(
            name: "EnigmaAuthGrpcClient",
            dependencies: [
                .product(name: "GRPC", package: "grpc-swift"),
            ]
        ),
        .testTarget(
            name: "auth-grpcs-client-tests",
            dependencies: ["EnigmaAuthGrpcClient"]
        ),
    ]
)

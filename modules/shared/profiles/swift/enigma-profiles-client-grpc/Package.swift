// swift-tools-version: 6.0

import PackageDescription

let package = Package(
  name: "profiles-grpc-client",
  products: [
    .library(
      name: "EnigmaProfiles",
      targets: ["EnigmaProfiles"])
  ],
  dependencies: [
    .package(url: "https://github.com/grpc/grpc-swift", .upToNextMajor(from: "1.24.2")),
    .package(path: "../../../auth/swift/enigma-auth-client-grpc"),
  ],
  targets: [
    .target(
      name: "EnigmaProfiles",
      dependencies: [
        .product(name: "GRPC", package: "grpc-swift"),
        .product(name: "EnigmaAuth", package: "enigma-auth-client-grpc"),
      ]
    ),
    .testTarget(
      name: "profiles-grpcs-client-tests",
      dependencies: ["EnigmaProfiles"]
    ),
  ]
)

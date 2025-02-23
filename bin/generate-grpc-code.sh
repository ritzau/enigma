#!/bin/zsh

set -xeuo pipefail

API_DIR=modules/shared/api/proto

AUTH_OUT=modules/shared/auth/swift/enigma-auth-client-grpc/Sources/Generated
PROFILES_OUT=modules/shared/profiles/swift/enigma-profiles-client-grpc/Sources/Generated

function generate_swift() {
  local SWIFT_OUT=$1
  local API_DIR=$2
  local PROTO_FILE=$3
  protoc --grpc-swift_out="$SWIFT_OUT" --proto_path="$API_DIR" "$PROTO_FILE"
  protoc --swift_out="$SWIFT_OUT" --proto_path="$API_DIR" "$PROTO_FILE"
}

generate_swift "$AUTH_OUT" "$API_DIR" auth.proto
generate_swift "$PROFILES_OUT" "$API_DIR" profiles.proto

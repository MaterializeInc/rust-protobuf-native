#!/usr/bin/env bash

set -euo pipefail

cd "$(dirname "$0")"

if [[ $# -ne 1 ]]; then
    echo "fatal: usage: $0 VERSION" >&2
    exit 1
fi

version=$1

set -x
curl -fsSL "https://github.com/protocolbuffers/protobuf/releases/download/v$version/protobuf-cpp-$version.tar.gz" > protobuf.tar.gz

rm -rf protobuf
mkdir -p protobuf
tar --strip-components=1 -C protobuf -xf protobuf.tar.gz
rm protobuf.tar.gz

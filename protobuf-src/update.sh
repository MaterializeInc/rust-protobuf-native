#!/usr/bin/env bash

# Copyright Materialize, Inc. All rights reserved.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License in the LICENSE file at the
# root of this repository, or online at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

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

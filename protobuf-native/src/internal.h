// Copyright Materialize, Inc. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE file at the
// root of this repository, or online at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#pragma once

#include "absl/strings/string_view.h"
#include "rust/cxx.h"

namespace protobuf_native {
namespace internal {

typedef int CInt;
typedef void CVoid;

static_assert(sizeof(absl::string_view) == 2 * sizeof(void *), "");
static_assert(alignof(absl::string_view) == alignof(void *), "");

inline absl::string_view string_view_from_bytes(rust::Slice<const uint8_t> s) {
    const char *data = reinterpret_cast<const char *>(s.data());
    return {data, s.size()};
}

}  // namespace internal
}  // namespace protobuf_native

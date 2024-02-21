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

use std::env;

fn main() {
    cxx_build::bridges([
        "src/compiler.rs",
        "src/internal.rs",
        "src/io.rs",
        "src/lib.rs",
    ])
    .flag("-std=c++14")
    .files(["src/compiler.cc", "src/io.cc", "src/lib.cc"])
    .warnings_into_errors(cfg!(deny_warnings))
    .compile("protobuf_native");

    // NOTE(benesch): once the bindings in protobuf-sys are more complete,
    // we'll switch to depending on protobuf-sys instead of protobuf-src,
    // and let protobuf-sys drive the linking.
    println!(
        "cargo:rustc-link-search=native={}/lib",
        env::var("DEP_PROTOBUF_SRC_ROOT").unwrap()
    );
    println!(
        "cargo:rustc-link-search=native={}/lib64",
        env::var("DEP_PROTOBUF_SRC_ROOT").unwrap()
    );
    println!("cargo:rustc-link-lib=static=protobuf");
}

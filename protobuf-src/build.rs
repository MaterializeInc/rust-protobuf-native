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

use std::error::Error;

fn wrapper() -> Option<String> {
    if let Ok(wrapper) = std::env::var("PROTOBUF_SRC_WRAPPER") {
        return Some(wrapper);
    }
    if let Ok(rustc_wrapper) = std::env::var("RUSTC_WRAPPER") {
        // only pass through wrappers that we know are able to handle both
        // rust and c/c++ code
        if rustc_wrapper == "sccache" || rustc_wrapper.ends_with("/sccache") {
            return Some(rustc_wrapper);
        }
    }
    None
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut config = cmake::Config::new("protobuf");
    config
        .define("ABSL_PROPAGATE_CXX_STD", "ON")
        .define("protobuf_BUILD_TESTS", "OFF")
        .define("protobuf_DEBUG_POSTFIX", "")
        .define("CMAKE_CXX_STANDARD", "14")
        // CMAKE_INSTALL_LIBDIR is inferred as "lib64" on some platforms, but we
        // want a stable location that we can add to the linker search path.
        // Since we're not actually installing to /usr or /usr/local, there's no
        // harm to always using "lib" here.
        .define("CMAKE_INSTALL_LIBDIR", "lib");
    if let Some(wrapper) = wrapper() {
        config
            .define("CMAKE_C_COMPILER_LAUNCHER", &wrapper)
            .define("CMAKE_CXX_COMPILER_LAUNCHER", &wrapper);
    }
    let install_dir = config.build();

    println!("cargo:rustc-env=INSTALL_DIR={}", install_dir.display());
    println!("cargo:CXXBRIDGE_DIR0={}/include", install_dir.display());
    Ok(())
}

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
use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let include_paths = [
        PathBuf::from(env::var("DEP_PROTOBUF_SRC_ROOT").unwrap()).join("include"),
        PathBuf::from("src"),
    ];
    autocxx_build::Builder::new("src/lib.rs", &include_paths)
        .build()?
        .flag_if_supported("-std=c++14")
        .compile("protobuf-sys");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!(
        "cargo:rustc-link-search=native={}/lib",
        env::var("DEP_PROTOBUF_SRC_ROOT").unwrap()
    );
    println!(
        "cargo:rustc-link-search=native={}/lib64",
        env::var("DEP_PROTOBUF_SRC_ROOT").unwrap()
    );
    println!("cargo:rustc-link-lib=static=protobuf");

    Ok(())
}

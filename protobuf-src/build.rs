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
use std::fs;
use std::error::Error;
use std::path::PathBuf;
use git2::build::RepoBuilder;
use git2::FetchOptions;

// We check out the appropriate revision of the protobuf source code from its git repository,
// instead of bundling the source code. It's also necessary to check out the abseil-cpp source code
// separately, as this is not bundled with protobuf.
fn main() -> Result<(), Box<dyn Error>> {
    let build_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let protobuf_dir = build_dir.join("protobuf");
    let mut fo = FetchOptions::new();
    fo.depth(1);
    fo.download_tags(git2::AutotagOption::All);
    if !protobuf_dir.exists() {
        let r = RepoBuilder::new()
            .fetch_options(fo)
            .clone("https://github.com/protocolbuffers/protobuf", &protobuf_dir)
            .expect("cloning protobuf repository");
        let (object, reference) = r.revparse_ext("v27.2")
            .expect("finding release tag");
        r.checkout_tree(&object, None)
            .expect("checking out release");
        r.set_head(reference.unwrap().name().unwrap())
            .expect("setting HEAD to release tag");
    }
    let abseil_dir = build_dir.join("protobuf").join("third_party").join("abseil-cpp");
    let mut fo = FetchOptions::new();
    fo.depth(1);
    if abseil_dir.exists() &&
        fs::read_dir(&abseil_dir).unwrap().count() == 0
    {
        RepoBuilder::new()
            .fetch_options(fo)
            .clone("https://github.com/abseil/abseil-cpp.git", &abseil_dir)
            .expect("cloning abseil-cpp repository");
    }

    let install_dir = cmake::Config::new(protobuf_dir)
        .define("ABSL_PROPAGATE_CXX_STD", "ON")
        .define("protobuf_BUILD_TESTS", "OFF")
        .define("protobuf_DEBUG_POSTFIX", "")
        .define("CMAKE_CXX_STANDARD", "14")
        // CMAKE_INSTALL_LIBDIR is inferred as "lib64" on some platforms, but we
        // want a stable location that we can add to the linker search path.
        // Since we're not actually installing to /usr or /usr/local, there's no
        // harm to always using "lib" here.
        .define("CMAKE_INSTALL_LIBDIR", "lib")
        .build();

    println!("cargo:rustc-env=INSTALL_DIR={}", install_dir.display());
    println!("cargo:CXXBRIDGE_DIR0={}/include", install_dir.display());
    Ok(())
}

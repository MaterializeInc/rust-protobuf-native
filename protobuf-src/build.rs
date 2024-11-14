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

fn main() -> Result<(), Box<dyn Error>> {
    let mut build_config = cmake::Config::new("protobuf");
    build_config
        .define("ABSL_PROPAGATE_CXX_STD", "ON")
        .define("protobuf_BUILD_TESTS", "OFF")
        .define("protobuf_DEBUG_POSTFIX", "")
        .define("CMAKE_CXX_STANDARD", "14")
        // CMAKE_INSTALL_LIBDIR is inferred as "lib64" on some platforms, but we
        // want a stable location that we can add to the linker search path.
        // Since we're not actually installing to /usr or /usr/local, there's no
        // harm to always using "lib" here.
        .define("CMAKE_INSTALL_LIBDIR", "lib");

    #[cfg(feature = "parallel")]
    {
        // The `cmake` crate does not enable parallelism for Makefile backed builds, instead it
        // tries to parallelize the build jobs with Cargo's jobserver. This doesn't seem to work
        // very well and manually specifying the `-j` flag based on the value from `NUM_JOBS` more
        // effectively parallelizes the build. Also, using `NUM_JOBS` is recommended by the Cargo
        // documentation for this purpose.
        //
        // Cargo Docs: <https://doc.rust-lang.org/cargo/reference/environment-variables.html>
        let maybe_num_jobs: Result<_, Box<dyn Error>> = std::env::var("NUM_JOBS")
            .map_err(|err| Box::new(err).into())
            .and_then(|val| val.parse::<usize>().map_err(|err| Box::new(err).into()));
        if let Ok(num_jobs) = maybe_num_jobs {
            build_config.build_arg(format!("-j{num_jobs}"));
        }
    }

    let install_dir = build_config.build();

    println!("cargo:rustc-env=INSTALL_DIR={}", install_dir.display());
    println!("cargo:CXXBRIDGE_DIR0={}/include", install_dir.display());
    Ok(())
}

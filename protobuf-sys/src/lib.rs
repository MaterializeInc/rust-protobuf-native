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

//! [<img src="https://materialize.com/wp-content/uploads/2020/01/materialize_logo_primary.png" width=180 align=right>](https://materialize.com)
//! Low-level bindings to `libprotobuf`, the C++ implementation of [Protocol
//! Buffers], Google's data interchange format.
//!
//! # Maintainership
//!
//! This crate is maintained by [Materialize]. Contributions are encouraged:
//!
//! * [View source code](https://github.com/MaterializeInc/rust-protobuf-native/tree/master/src/protobuf-sys)
//! * [Report an issue](https://github.com/MaterializeInc/rust-protobuf-native/issues/new/choose)
//! * [Submit a pull request](https://github.com/MaterializeInc/rust-protobuf-native/compare)
//!
//! # Details
//!
//! Documentation for these types can be found in the official
//! [C++ API reference][cxx-api].
//!
//! These bindings are automatically generated by [autocxx]. Many types and
//! methods are missing due to missing features in autocxx. As autocxx improves,
//! so will these bindings. If you discover new types that autocxx is capable
//! of generating bindings for, please submit an issue!
//!
//! At present, autocxx is invoked automatically in the crate's build script.
//! This creates a dependency on libclang at build time via the [clang-sys]
//! crate. Once the bindings stabilize, we plan to manually commit the generated
//! bindings to the repository to avoid this dependency.
//!
//! Depending on your use case, the handwritten bindings in [protobuf-native]
//! may be more suitable.
//!
//! [autocxx]: https://github.com/google/autocxx
//! [clang-sys]: https://github.com/KyleMayes/clang-sys
//! [cxx-api]: https://developers.google.com/protocol-buffers/docs/reference/cpp
//! [protobuf-native]: https://docs.rs/protobuf-native
//! [Materialize]: https://materialize.com
//! [Protocol Buffers]: https://github.com/google/protobuf

autocxx::include_cpp! {
    #include "google/protobuf/descriptor_database.h"
    #include "google/protobuf/compiler/importer.h"
    #include "google/protobuf/io/coded_stream.h"
    #include "google/protobuf/json/json.h"
    #include "google/protobuf/util/time_util.h"

    generate!("google::protobuf::DescriptorDatabase")
    generate!("google::protobuf::compiler::SourceTree")
    generate!("google::protobuf::compiler::Importer")
    generate!("google::protobuf::compiler::DiskSourceTree")
    generate!("google::protobuf::io::CodedInputStream")
    generate!("google::protobuf::io::ZeroCopyInputStream")
    generate!("google::protobuf::io::CodedOutputStream")
    generate!("google::protobuf::io::ZeroCopyOutputStream")
    generate_pod!("google::protobuf::json::ParseOptions")
    generate_pod!("google::protobuf::json::PrintOptions")
    generate_pod!("google::protobuf::util::TimeUtil")
    safety!(unsafe)
}

pub use ffi::*;

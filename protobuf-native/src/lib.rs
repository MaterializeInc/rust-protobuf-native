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
//!
//! High-level, safe bindings to `libprotobuf`, the C++ implementation of
//! [Protocol Buffers], Google's data interchange format.
//!
//! # Maintainership
//!
//! This crate is maintained by [Materialize]. Contributions are encouraged:
//!
//! * [View source code](https://github.com/MaterializeInc/rust-protobuf-native/tree/master/src/protobuf-native)
//! * [Report an issue](https://github.com/MaterializeInc/rust-protobuf-native/issues/new/choose)
//! * [Submit a pull request](https://github.com/MaterializeInc/rust-protobuf-native/compare)
//!
//! # Details
//!
//! This crate contains handwritten bindings to libprotobuf facilitated by
//! [cxx]. The API that is exposed is extremely specific to the few users of
//! this library and is subject to frequent change.
//!
//! Depending on your use case, the auto-generated bindings in [protobuf-sys]
//! may be more suitable.
//!
//! [cxx]: https://github.com/dtolnay/cxx
//! [protobuf-sys]: https://docs.rs/protobuf-sys
//! [Materialize]: https://materialize.com
//! [Protocol Buffers]: https://github.com/google/protobuf

pub mod compiler;
pub mod descriptor_database;
pub mod descriptor_pb;
pub mod error;
pub mod io;

mod internal;

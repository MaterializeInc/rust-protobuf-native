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

//! This module contains the `ZeroCopyInputStream` and `ZeroCopyOutputStream`
//! interfaces, which represent abstract I/O streams to and from which
//! protocol buffers can be read and written.
//!
//! These interfaces are different from classic I/O streams in that they
//! try to minimize the amount of data copying that needs to be done.
//! To accomplish this, responsibility for allocating buffers is moved to
//! the stream object, rather than being the responsibility of the caller.
//! So, the stream can return a buffer which actually points directly into
//! the final data structure where the bytes are to be stored, and the caller
//! can interact directly with that buffer, eliminating an intermediate copy
//! operation.

use cxx::UniquePtr;

#[cxx::bridge(namespace = "protobuf_native::io")]
pub(crate) mod ffi {
    unsafe extern "C++" {
        include!("protobuf-native/src/io/zero_copy_stream.h");

        #[namespace = "google::protobuf::io"]
        type ZeroCopyInputStream;

        unsafe fn MakeUniqueZeroCopyInputStream(
            ptr: *mut ZeroCopyInputStream,
        ) -> UniquePtr<ZeroCopyInputStream>;
    }
}

/// Abstract interface similar to an input stream but designed to minimize
/// copying.
pub struct ZeroCopyInputStream(UniquePtr<ffi::ZeroCopyInputStream>);

impl ZeroCopyInputStream {
    /// Creates a `ZeroCopyInputStream` from a pointer to an
    /// [`ffi::ZeroCopyInputStream`].
    ///
    /// If the pointer is null, returns `None`.
    ///
    /// # Safety
    ///
    /// You must have ownership of the provided `ZeroCopyInputStream`.
    pub(crate) unsafe fn from_ptr(
        ptr: *mut ffi::ZeroCopyInputStream,
    ) -> Option<ZeroCopyInputStream> {
        if !ptr.is_null() {
            Some(ZeroCopyInputStream(ffi::MakeUniqueZeroCopyInputStream(ptr)))
        } else {
            None
        }
    }
}

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

use cxx::UniquePtr;

#[cxx::bridge(namespace = "protobuf_native")]
pub(crate) mod ffi {
    unsafe extern "C++" {
        include!("protobuf-native/src/descriptor_pb.h");

        #[namespace = "google::protobuf"]
        type FileDescriptorProto;

        fn NewFileDescriptorProto() -> UniquePtr<FileDescriptorProto>;
    }
}

/// Describes a complete .proto file.
pub struct FileDescriptorProto {
    pub(crate) ffi: UniquePtr<ffi::FileDescriptorProto>,
}

impl FileDescriptorProto {
    /// Creates a new file descriptor proto.
    pub fn new() -> FileDescriptorProto {
        FileDescriptorProto {
            ffi: ffi::NewFileDescriptorProto(),
        }
    }
}

use std::fmt;
impl fmt::Debug for FileDescriptorProto {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a real file desc")
    }
}

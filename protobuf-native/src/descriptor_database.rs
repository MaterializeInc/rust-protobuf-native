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

use std::path::Path;

use crate::descriptor_pb::FileDescriptorProto;
use crate::error::FileNotLoadableError;

/// Abstract interface for a database of descriptors.
///
/// This is useful if you want to create a DescriptorPool which loads
/// descriptors on-demand from some sort of large database.  If the database
/// is large, it may be inefficient to enumerate every .proto file inside it
/// calling DescriptorPool::BuildFile() for each one.  Instead, a DescriptorPool
/// can be created which wraps a DescriptorDatabase and only builds particular
/// descriptors when they are needed.
pub trait DescriptorDatabase {
    /// Finds a file by file name.
    fn find_file_by_name(
        &mut self,
        filename: &Path,
    ) -> Result<FileDescriptorProto, FileNotLoadableError>;
}

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

//! This module is the public interface to the .proto file parser.

use std::path::Path;

use cxx::{let_cxx_string, UniquePtr};

use crate::descriptor_database::DescriptorDatabase;
use crate::descriptor_pb::FileDescriptorProto;
use crate::error::{FileNotFoundError, FileNotLoadableError};
use crate::internal::cxx::{Inherit, UniquePtrExt};
use crate::internal::path::ProtobufPath;
use crate::io::zero_copy_stream::ZeroCopyInputStream;

#[cxx::bridge(namespace = "protobuf_native::compiler")]
pub(crate) mod ffi {
    unsafe extern "C++" {
        include!("protobuf-native/src/compiler/importer.h");

        #[namespace = "google::protobuf"]
        type FileDescriptorProto = crate::descriptor_pb::ffi::FileDescriptorProto;

        #[namespace = "google::protobuf::io"]
        type ZeroCopyInputStream = crate::io::zero_copy_stream::ffi::ZeroCopyInputStream;

        #[namespace = "google::protobuf::compiler"]
        type SourceTree;

        fn Open(self: Pin<&mut SourceTree>, filename: &CxxString) -> *mut ZeroCopyInputStream;

        #[namespace = "google::protobuf::compiler"]
        type SourceTreeDescriptorDatabase;

        unsafe fn NewSourceTreeDescriptorDatabase(
            source_tree: *mut SourceTree,
        ) -> UniquePtr<SourceTreeDescriptorDatabase>;

        unsafe fn FindFileByName(
            self: Pin<&mut SourceTreeDescriptorDatabase>,
            filename: &CxxString,
            output: *mut FileDescriptorProto,
        ) -> bool;

        #[namespace = "google::protobuf::compiler"]
        type DiskSourceTree;

        fn NewDiskSourceTree() -> UniquePtr<DiskSourceTree>;
        fn MapPath(self: Pin<&mut DiskSourceTree>, virtual_path: &CxxString, disk_path: &CxxString);
    }

    impl UniquePtr<SourceTree> {}
}

// SAFETY: `DiskSourceTree` inherits from `SourceTree` in C++.
unsafe impl Inherit<ffi::SourceTree> for ffi::DiskSourceTree {}

/// An implementation of `DescriptorDatabase` which loads files from a
/// `SourceTree` and parses them.
///
/// Note: This class does not implement `FindFileContainingSymbol` or
/// `FindFileContainingExtension`; these will always return false.
pub struct SourceTreeDescriptorDatabase<'a> {
    inner: UniquePtr<ffi::SourceTreeDescriptorDatabase>,
    _source_tree: &'a mut dyn SourceTree,
}

impl<'a> SourceTreeDescriptorDatabase<'a> {
    pub fn new(source_tree: &'a mut dyn SourceTree) -> SourceTreeDescriptorDatabase<'a> {
        SourceTreeDescriptorDatabase {
            inner: unsafe {
                ffi::NewSourceTreeDescriptorDatabase(source_tree.upcast_mut().as_mut_ptr())
            },
            _source_tree: source_tree,
        }
    }
}

impl<'a> DescriptorDatabase for SourceTreeDescriptorDatabase<'a> {
    fn find_file_by_name(
        &mut self,
        filename: &Path,
    ) -> Result<FileDescriptorProto, FileNotLoadableError> {
        let fd = FileDescriptorProto::new();
        let_cxx_string!(filename = ProtobufPath::from(filename));
        if unsafe {
            self.inner
                .pin_mut()
                .FindFileByName(&filename, fd.ffi.as_mut_ptr())
        } {
            Ok(fd)
        } else {
            Err(FileNotLoadableError)
        }
    }
}

/// Abstract interface which represents a directory tree containing .proto
/// files.
///
/// Used by the default implementation of `Importer` to resolve import
/// statements. Most users will probably want to use the `DiskSourceTree`
/// implementation.
///
/// This trait is sealed and cannot be implemented outside of this crate.
pub trait SourceTree: private::SourceTree {
    /// Open the given file and return a stream that reads it, or NULL if not
    /// found.  The caller takes ownership of the returned object.  The filename
    /// must be a path relative to the root of the source tree and must not
    /// contain "." or ".." components.
    fn open(&mut self, filename: &Path) -> Result<ZeroCopyInputStream, FileNotFoundError> {
        let_cxx_string!(filename = ProtobufPath::from(filename));
        let ptr = self.upcast_mut().pin_mut().Open(&filename);
        // SAFETY: `Open` is documented to return a stream over which we have
        // ownership.
        unsafe { ZeroCopyInputStream::from_ptr(ptr) }.ok_or(FileNotFoundError)
    }
}

mod private {
    use cxx::UniquePtr;

    use super::ffi;

    pub trait SourceTree {
        /// Retrieve a pointer to the underlying `ffi::SourceTree`.
        fn upcast_mut(&mut self) -> &mut UniquePtr<ffi::SourceTree>;
    }
}

/// An implementation of `SourceTree` which loads files from locations on disk.
///
/// Multiple mappings can be set up to map locations in the `DiskSourceTree` to
/// locations in the physical filesystem.
pub struct DiskSourceTree(UniquePtr<ffi::DiskSourceTree>);

impl DiskSourceTree {
    /// Creates a new disk source tree.
    pub fn new() -> DiskSourceTree {
        DiskSourceTree(ffi::NewDiskSourceTree())
    }

    /// Maps a path on disk to a location in the source tree.
    ///
    /// The path may be either a file or a directory. If it is a directory, the
    /// entire tree under it will be mapped to the given virtual location. To
    /// map a directory to the root of the source tree, pass an empty string for
    /// `virtual_path`.
    ///
    /// If multiple mapped paths apply when opening a file, they will be
    /// searched in order. For example, if you do:
    ///
    /// ```text
    /// map_path("bar", "foo/bar");
    /// map_path("", "baz");
    /// ```
    ///
    /// and then you do:
    ///
    /// ```text
    /// open("bar/qux");
    /// ```
    ///
    /// the `DiskSourceTree` will first try to open foo/bar/qux, then
    /// baz/bar/qux, returning the first one that opens successfully.
    ///
    /// `disk_path` may be an absolute path or relative to the current directory,
    /// just like a path you'd pass to [`std::fs::File::open`].
    pub fn map_path(&mut self, virtual_path: &Path, disk_path: &Path) {
        let_cxx_string!(virtual_path = ProtobufPath::from(virtual_path));
        let_cxx_string!(disk_path = ProtobufPath::from(disk_path));
        self.0.pin_mut().MapPath(&virtual_path, &disk_path)
    }
}

impl SourceTree for DiskSourceTree {}

impl private::SourceTree for DiskSourceTree {
    fn upcast_mut(&mut self) -> &mut UniquePtr<ffi::SourceTree> {
        self.0.upcast_mut()
    }
}

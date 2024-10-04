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

//! Implementation of the Protocol Buffer compiler.
//!
//! This module contains code for parsing .proto files and generating code based
//! on them. It is particularly useful when you need to deal with arbitrary
//! Protobuf messages at runtime.

use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::marker::PhantomData;
use std::marker::PhantomPinned;
use std::mem;
use std::path::Path;
use std::pin::Pin;

use cxx::let_cxx_string;

use crate::internal::{unsafe_ffi_conversions, CInt, ProtobufPath};
use crate::io::DynZeroCopyInputStream;
use crate::{DescriptorDatabase, FileDescriptorProto, FileDescriptorSet, OperationFailedError};

#[cxx::bridge(namespace = "protobuf_native::compiler")]
pub(crate) mod ffi {
    #[derive(Debug)]
    struct FileLoadError {
        filename: String,
        line: i64,
        column: i64,
        message: String,
        warning: bool,
    }

    unsafe extern "C++" {
        include!("protobuf-native/src/compiler.h");
        include!("protobuf-native/src/internal.h");

        #[namespace = "protobuf_native::internal"]
        type CInt = crate::internal::CInt;

        #[namespace = "absl"]
        type string_view<'a> = crate::internal::StringView<'a>;

        #[namespace = "google::protobuf"]
        type FileDescriptorProto = crate::ffi::FileDescriptorProto;

        #[namespace = "google::protobuf::io"]
        type ZeroCopyInputStream = crate::io::ffi::ZeroCopyInputStream;

        type SimpleErrorCollector;
        fn NewSimpleErrorCollector() -> *mut SimpleErrorCollector;
        unsafe fn DeleteSimpleErrorCollector(collector: *mut SimpleErrorCollector);
        fn Errors(self: Pin<&mut SimpleErrorCollector>) -> Pin<&mut CxxVector<FileLoadError>>;

        #[namespace = "google::protobuf::compiler"]
        type MultiFileErrorCollector;
        fn RecordError(
            self: Pin<&mut MultiFileErrorCollector>,
            filename: string_view,
            line: CInt,
            column: CInt,
            message: string_view,
        );
        fn RecordWarning(
            self: Pin<&mut MultiFileErrorCollector>,
            filename: string_view,
            line: CInt,
            column: CInt,
            message: string_view,
        );

        #[namespace = "google::protobuf::compiler"]
        type SourceTree;
        fn Open(self: Pin<&mut SourceTree>, filename: string_view) -> *mut ZeroCopyInputStream;
        fn SourceTreeGetLastErrorMessage(source_tree: Pin<&mut SourceTree>) -> String;

        #[namespace = "google::protobuf::compiler"]
        type SourceTreeDescriptorDatabase;
        unsafe fn NewSourceTreeDescriptorDatabase(
            source_tree: *mut SourceTree,
        ) -> *mut SourceTreeDescriptorDatabase;
        unsafe fn DeleteSourceTreeDescriptorDatabase(
            source_tree: *mut SourceTreeDescriptorDatabase,
        );
        unsafe fn FindFileByName(
            self: Pin<&mut SourceTreeDescriptorDatabase>,
            filename: &CxxString,
            output: *mut FileDescriptorProto,
        ) -> bool;
        unsafe fn RecordErrorsTo(
            self: Pin<&mut SourceTreeDescriptorDatabase>,
            error_collector: *mut MultiFileErrorCollector,
        );

        type VirtualSourceTree;
        fn NewVirtualSourceTree() -> *mut VirtualSourceTree;
        unsafe fn DeleteVirtualSourceTree(tree: *mut VirtualSourceTree);
        fn AddFile(self: Pin<&mut VirtualSourceTree>, filename: string_view, contents: Vec<u8>);

        #[namespace = "google::protobuf::compiler"]
        type DiskSourceTree;
        fn NewDiskSourceTree() -> *mut DiskSourceTree;
        unsafe fn DeleteDiskSourceTree(tree: *mut DiskSourceTree);
        fn MapPath(
            self: Pin<&mut DiskSourceTree>,
            virtual_path: string_view,
            disk_path: string_view,
        );
    }
}

/// If the importer encounters problems while trying to import the proto files,
/// it reports them to a `MultiFileErrorCollector`.
pub trait MultiFileErrorCollector: multi_file_error_collector::Sealed {
    /// Adds an error message to the error collector at the specified position.
    ///
    /// Line and column numbers are zero-based. A line number of -1 indicates
    /// an error with the entire file (e.g., "not found").
    fn add_error(self: Pin<&mut Self>, filename: &str, line: i32, column: i32, message: &str) {
        self.upcast_mut().RecordError(
            filename.into(),
            CInt::expect_from(line),
            CInt::expect_from(column),
            message.into(),
        )
    }

    /// Adds a warning to the error collector at the specified position.
    ///
    /// See the documentation for [`add_error`] for details on the meaning of
    /// the `line` and `column` parameters.
    ///
    /// [`add_error`]: MultiFileErrorCollector::add_error
    fn add_warning(self: Pin<&mut Self>, filename: &str, line: i32, column: i32, message: &str) {
        self.upcast_mut().RecordWarning(
            filename.into(),
            CInt::expect_from(line),
            CInt::expect_from(column),
            message.into(),
        )
    }
}

mod multi_file_error_collector {
    use std::pin::Pin;

    use super::ffi;

    pub trait Sealed {
        fn upcast(&self) -> &ffi::MultiFileErrorCollector;
        fn upcast_mut(self: Pin<&mut Self>) -> Pin<&mut ffi::MultiFileErrorCollector>;
        unsafe fn upcast_mut_ptr(self: Pin<&mut Self>) -> *mut ffi::MultiFileErrorCollector {
            self.upcast_mut().get_unchecked_mut() as *mut _
        }
    }
}

/// A simple implementation of [`MultiFileErrorCollector`] that records errors
/// in memory for later retrieval.
pub struct SimpleErrorCollector {
    _opaque: PhantomPinned,
}

impl Drop for SimpleErrorCollector {
    fn drop(&mut self) {
        unsafe { ffi::DeleteSimpleErrorCollector(self.as_ffi_mut_ptr_unpinned()) }
    }
}

impl SimpleErrorCollector {
    /// Creates a new simple error collector.
    pub fn new() -> Pin<Box<SimpleErrorCollector>> {
        let collector = ffi::NewSimpleErrorCollector();
        unsafe { Self::from_ffi_owned(collector) }
    }

    unsafe_ffi_conversions!(ffi::SimpleErrorCollector);
}

impl<'a> Iterator for Pin<&'a mut SimpleErrorCollector> {
    type Item = FileLoadError;

    fn next(&mut self) -> Option<FileLoadError> {
        self.as_mut().as_ffi_mut().Errors().pop().map(Into::into)
    }
}

impl MultiFileErrorCollector for SimpleErrorCollector {}

impl multi_file_error_collector::Sealed for SimpleErrorCollector {
    fn upcast(&self) -> &ffi::MultiFileErrorCollector {
        unsafe { mem::transmute(self) }
    }

    fn upcast_mut(self: Pin<&mut Self>) -> Pin<&mut ffi::MultiFileErrorCollector> {
        unsafe { mem::transmute(self) }
    }
}

/// An implementation of `DescriptorDatabase` which loads files from a
/// `SourceTree` and parses them.
///
/// Note: This class does not implement `FindFileContainingSymbol` or
/// `FindFileContainingExtension`; these will always return false.
pub struct SourceTreeDescriptorDatabase<'a> {
    _opaque: PhantomPinned,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> Drop for SourceTreeDescriptorDatabase<'a> {
    fn drop(&mut self) {
        unsafe { ffi::DeleteSourceTreeDescriptorDatabase(self.as_ffi_mut_ptr_unpinned()) }
    }
}

impl<'a> SourceTreeDescriptorDatabase<'a> {
    /// Constructs a new descriptor database for the provided source tree.
    pub fn new(
        source_tree: Pin<&'a mut dyn SourceTree>,
    ) -> Pin<Box<SourceTreeDescriptorDatabase<'a>>> {
        let db = unsafe { ffi::NewSourceTreeDescriptorDatabase(source_tree.upcast_mut_ptr()) };
        unsafe { Self::from_ffi_owned(db) }
    }

    /// Instructs the source tree descriptor database to report any parse errors
    /// to the given [`MultiFileErrorCollector`].
    ///
    /// This should b ecalled before parsing.
    pub fn record_errors_to(
        self: Pin<&mut Self>,
        error_collector: Pin<&'a mut dyn MultiFileErrorCollector>,
    ) {
        unsafe {
            self.as_ffi_mut()
                .RecordErrorsTo(error_collector.upcast_mut_ptr())
        }
    }

    /// Builds a file descriptor set containing all file descriptor protos
    /// reachable from the specified roots.
    pub fn build_file_descriptor_set<P>(
        mut self: Pin<&mut Self>,
        roots: &[P],
    ) -> Result<Pin<Box<FileDescriptorSet>>, OperationFailedError>
    where
        P: AsRef<Path>,
    {
        let mut out = FileDescriptorSet::new();
        let mut seen = HashSet::new();
        let mut stack = vec![];
        for root in roots {
            let root = root.as_ref();
            stack.push(self.as_mut().find_file_by_name(root)?);
            seen.insert(ProtobufPath::from(root).as_bytes().to_vec());
        }
        while let Some(file) = stack.pop() {
            out.as_mut().add_file().copy_from(&file);
            for i in 0..file.dependency_size() {
                let dep_path = ProtobufPath::from(file.dependency(i));
                if !seen.contains(dep_path.as_bytes()) {
                    let dep = self
                        .as_mut()
                        .find_file_by_name(dep_path.as_path().as_ref())?;
                    stack.push(dep);
                    seen.insert(dep_path.as_bytes().to_vec());
                }
            }
        }
        Ok(out)
    }

    unsafe_ffi_conversions!(ffi::SourceTreeDescriptorDatabase);
}

impl<'a> DescriptorDatabase for SourceTreeDescriptorDatabase<'a> {
    fn find_file_by_name(
        self: Pin<&mut Self>,
        filename: &Path,
    ) -> Result<Pin<Box<FileDescriptorProto>>, OperationFailedError> {
        let mut fd = FileDescriptorProto::new();
        let_cxx_string!(filename = ProtobufPath::from(filename).as_bytes());
        if unsafe {
            self.as_ffi_mut()
                .FindFileByName(&filename, fd.as_mut().as_ffi_mut_ptr())
        } {
            Ok(fd)
        } else {
            Err(OperationFailedError)
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
pub trait SourceTree: source_tree::Sealed {
    /// Opens the given file and return a stream that reads it.
    ///
    /// The filename must be a path relative to the root of the source tree and
    /// must not contain "." or ".." components.
    fn open<'a>(
        self: Pin<&'a mut Self>,
        filename: &Path,
    ) -> Result<Pin<Box<DynZeroCopyInputStream<'a>>>, FileOpenError> {
        let filename = ProtobufPath::from(filename);
        let mut source_tree = self.upcast_mut();
        let stream = source_tree.as_mut().Open(filename.into());
        if stream.is_null() {
            Err(FileOpenError(ffi::SourceTreeGetLastErrorMessage(
                source_tree,
            )))
        } else {
            Ok(unsafe { DynZeroCopyInputStream::from_ffi_owned(stream) })
        }
    }
}

mod source_tree {
    use std::pin::Pin;

    use super::ffi;

    pub trait Sealed {
        fn upcast(&self) -> &ffi::SourceTree;
        fn upcast_mut(self: Pin<&mut Self>) -> Pin<&mut ffi::SourceTree>;
        unsafe fn upcast_mut_ptr(self: Pin<&mut Self>) -> *mut ffi::SourceTree {
            self.upcast_mut().get_unchecked_mut() as *mut _
        }
    }
}

/// An implementation of `SourceTree` which stores files in memory.
pub struct VirtualSourceTree {
    _opaque: PhantomPinned,
}

impl Drop for VirtualSourceTree {
    fn drop(&mut self) {
        unsafe { ffi::DeleteVirtualSourceTree(self.as_ffi_mut_ptr_unpinned()) }
    }
}

impl VirtualSourceTree {
    /// Creates a new virtual source tree.
    pub fn new() -> Pin<Box<VirtualSourceTree>> {
        let tree = ffi::NewVirtualSourceTree();
        unsafe { Self::from_ffi_owned(tree) }
    }

    /// Adds a file to the source tree with the specified name and contents.
    pub fn add_file(self: Pin<&mut Self>, filename: &Path, contents: Vec<u8>) {
        let filename = ProtobufPath::from(filename);
        self.as_ffi_mut().AddFile(filename.into(), contents)
    }

    unsafe_ffi_conversions!(ffi::VirtualSourceTree);
}

impl SourceTree for VirtualSourceTree {}

impl source_tree::Sealed for VirtualSourceTree {
    fn upcast(&self) -> &ffi::SourceTree {
        unsafe { mem::transmute(self) }
    }

    fn upcast_mut(self: Pin<&mut Self>) -> Pin<&mut ffi::SourceTree> {
        unsafe { mem::transmute(self) }
    }
}

/// An implementation of `SourceTree` which loads files from locations on disk.
///
/// Multiple mappings can be set up to map locations in the `DiskSourceTree` to
/// locations in the physical filesystem.
pub struct DiskSourceTree {
    _opaque: PhantomPinned,
}

impl Drop for DiskSourceTree {
    fn drop(&mut self) {
        unsafe { ffi::DeleteDiskSourceTree(self.as_ffi_mut_ptr_unpinned()) }
    }
}

impl DiskSourceTree {
    /// Creates a new disk source tree.
    pub fn new() -> Pin<Box<DiskSourceTree>> {
        let tree = ffi::NewDiskSourceTree();
        unsafe { Self::from_ffi_owned(tree) }
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
    /// ```
    /// use std::path::Path;
    /// use protobuf_native::compiler::DiskSourceTree;
    ///
    /// let mut source_tree = DiskSourceTree::new();
    /// source_tree.as_mut().map_path(Path::new("bar"), Path::new("foo/bar"));
    /// source_tree.as_mut().map_path(Path::new(""), Path::new("baz"));
    /// ```
    ///
    /// and then you do:
    ///
    /// ```
    /// # use std::path::Path;
    /// # use std::pin::Pin;
    /// # use protobuf_native::compiler::{SourceTree, DiskSourceTree};
    /// # fn f(mut source_tree: Pin<&mut DiskSourceTree>) {
    /// source_tree.open(Path::new("bar/qux"));
    /// # }
    /// ```
    ///
    /// the `DiskSourceTree` will first try to open foo/bar/qux, then
    /// baz/bar/qux, returning the first one that opens successfully.
    ///
    /// `disk_path` may be an absolute path or relative to the current directory,
    /// just like a path you'd pass to [`File::open`].
    ///
    /// [`File::open`]: std::fs::File::open
    pub fn map_path(self: Pin<&mut Self>, virtual_path: &Path, disk_path: &Path) {
        let virtual_path = ProtobufPath::from(virtual_path);
        let disk_path = ProtobufPath::from(disk_path);
        self.as_ffi_mut()
            .MapPath(virtual_path.into(), disk_path.into())
    }

    unsafe_ffi_conversions!(ffi::DiskSourceTree);
}

impl SourceTree for DiskSourceTree {}

impl source_tree::Sealed for DiskSourceTree {
    fn upcast(&self) -> &ffi::SourceTree {
        unsafe { mem::transmute(self) }
    }

    fn upcast_mut(self: Pin<&mut Self>) -> Pin<&mut ffi::SourceTree> {
        unsafe { mem::transmute(self) }
    }
}

/// An error occurred while opening a file.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct FileOpenError(String);

impl fmt::Display for FileOpenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // The underlying error is descriptive enough in all cases to not
        // warrant any additional context.
        f.write_str(&self.0)
    }
}

impl Error for FileOpenError {}

/// Describes the severity of a [`FileLoadError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
    /// A true error.
    Error,
    /// An informational warning.
    Warning,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Severity::Warning => f.write_str("warning"),
            Severity::Error => f.write_str("error"),
        }
    }
}

/// Describes the location at which a [`FileLoadError`] occurred.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Location {
    /// The 1-based line number.
    pub line: i64,
    /// The 1-based column number.
    pub column: i64,
}

/// An error occured while loading a file.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]

pub struct FileLoadError {
    /// The name of the file which failed to load.
    pub filename: String,
    /// A message describing the cause of the error.
    pub message: String,
    /// The severity of the error.
    pub severity: Severity,
    /// The specific location at which the error occurred, if applicable.
    pub location: Option<Location>,
}

impl From<ffi::FileLoadError> for FileLoadError {
    fn from(ffi: ffi::FileLoadError) -> FileLoadError {
        let location = (ffi.line >= 0).then(|| Location {
            line: ffi.line + 1,
            column: ffi.column + 1,
        });
        FileLoadError {
            filename: ffi.filename,
            message: ffi.message,
            severity: if ffi.warning {
                Severity::Warning
            } else {
                Severity::Error
            },
            location,
        }
    }
}

impl fmt::Display for FileLoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:", self.filename)?;
        if let Some(location) = &self.location {
            write!(f, "{}:{}:", location.line, location.column)?;
        }
        write!(f, " {}: {}", self.severity, self.message)
    }
}

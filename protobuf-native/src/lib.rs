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
//! # Overview
//!
//! This crate contains handwritten bindings to libprotobuf facilitated by
//! [cxx]. The API that is exposed is extremely specific to the few users of
//! this library and is subject to frequent change.
//!
//! Depending on your use case, the auto-generated bindings in [protobuf-sys]
//! may be more suitable.
//!
//! # API details
//!
//! This section, as well as the documentation on individual types, is
//! copied directly from the official [C++ API reference][cxx-api], with a few
//! modifications made as necessary.
//!
//! [cxx]: https://github.com/dtolnay/cxx
//! [cxx-api]: https://developers.google.com/protocol-buffers/docs/reference/cpp
//! [protobuf-sys]: https://docs.rs/protobuf-sys
//! [Materialize]: https://materialize.com
//! [Protocol Buffers]: https://github.com/google/protobuf

use std::error::Error;
use std::fmt;
use std::io::Write;
use std::marker::PhantomPinned;
use std::mem;
use std::path::Path;
use std::pin::Pin;

use crate::internal::{unsafe_ffi_conversions, BoolExt, CInt};
use crate::io::{CodedInputStream, CodedOutputStream, WriterStream, ZeroCopyOutputStream};

pub mod compiler;
pub mod io;

mod internal;

#[cxx::bridge(namespace = "protobuf_native")]
pub(crate) mod ffi {
    unsafe extern "C++" {
        include!("protobuf-native/src/internal.h");
        include!("protobuf-native/src/lib.h");

        #[namespace = "protobuf_native::internal"]
        type CInt = crate::internal::CInt;

        #[namespace = "google::protobuf::io"]
        type ZeroCopyOutputStream = crate::io::ffi::ZeroCopyOutputStream;

        #[namespace = "google::protobuf::io"]
        type CodedInputStream = crate::io::ffi::CodedInputStream;

        #[namespace = "google::protobuf::io"]
        type CodedOutputStream = crate::io::ffi::CodedOutputStream;

        #[namespace = "google::protobuf"]
        type MessageLite;

        fn NewMessageLite(message: &MessageLite) -> *mut MessageLite;
        unsafe fn DeleteMessageLite(message: *mut MessageLite);
        fn Clear(self: Pin<&mut MessageLite>);
        fn IsInitialized(self: &MessageLite) -> bool;
        unsafe fn MergeFromCodedStream(
            self: Pin<&mut MessageLite>,
            input: *mut CodedInputStream,
        ) -> bool;
        unsafe fn SerializeToCodedStream(
            self: &MessageLite,
            output: *mut CodedOutputStream,
        ) -> bool;
        unsafe fn SerializeToZeroCopyStream(
            self: &MessageLite,
            output: *mut ZeroCopyOutputStream,
        ) -> bool;
        fn ByteSizeLong(self: &MessageLite) -> usize;

        #[namespace = "google::protobuf"]
        type Message;

        #[namespace = "google::protobuf"]
        type FileDescriptor;

        unsafe fn DeleteFileDescriptor(proto: *mut FileDescriptor);

        #[namespace = "google::protobuf"]
        type DescriptorPool;

        fn NewDescriptorPool() -> *mut DescriptorPool;
        unsafe fn DeleteDescriptorPool(proto: *mut DescriptorPool);
        fn BuildFile(
            self: Pin<&mut DescriptorPool>,
            proto: &FileDescriptorProto,
        ) -> *const FileDescriptor;

        #[namespace = "google::protobuf"]
        type FileDescriptorSet;

        fn NewFileDescriptorSet() -> *mut FileDescriptorSet;
        unsafe fn DeleteFileDescriptorSet(set: *mut FileDescriptorSet);
        fn file_size(self: &FileDescriptorSet) -> CInt;
        fn clear_file(self: Pin<&mut FileDescriptorSet>);
        fn file(self: &FileDescriptorSet, i: CInt) -> &FileDescriptorProto;
        fn mutable_file(self: Pin<&mut FileDescriptorSet>, i: CInt) -> *mut FileDescriptorProto;
        fn add_file(self: Pin<&mut FileDescriptorSet>) -> *mut FileDescriptorProto;

        #[namespace = "google::protobuf"]
        type FileDescriptorProto;

        fn NewFileDescriptorProto() -> *mut FileDescriptorProto;
        unsafe fn DeleteFileDescriptorProto(proto: *mut FileDescriptorProto);
        fn CopyFrom(self: Pin<&mut FileDescriptorProto>, from: &FileDescriptorProto);
        fn MergeFrom(self: Pin<&mut FileDescriptorProto>, from: &FileDescriptorProto);
        fn dependency_size(self: &FileDescriptorProto) -> CInt;
        fn dependency(self: &FileDescriptorProto, i: CInt) -> &CxxString;
        fn message_type_size(self: &FileDescriptorProto) -> CInt;
        fn message_type(self: &FileDescriptorProto, i: CInt) -> &DescriptorProto;

        #[namespace = "google::protobuf"]
        type DescriptorProto;
        unsafe fn DeleteDescriptorProto(proto: *mut DescriptorProto);
        fn name(self: &DescriptorProto) -> &CxxString;
    }

    impl UniquePtr<MessageLite> {}
    impl UniquePtr<Message> {}
}

mod private {
    use std::pin::Pin;

    use super::ffi;

    pub trait MessageLite {
        fn upcast(&self) -> &ffi::MessageLite;
        fn upcast_mut(self: Pin<&mut Self>) -> Pin<&mut ffi::MessageLite>;
    }

    pub trait Message {}
}

/// Abstract interface for a database of descriptors.
///
/// This is useful if you want to create a [`DescriptorPool`] which loads
/// descriptors on-demand from some sort of large database.  If the database is
/// large, it may be inefficient to enumerate every .proto file inside it
/// calling [`DescriptorPool::build_file`] for each one.  Instead, a
/// `DescriptorPool` can be created which wraps a `DescriptorDatabase` and only
/// builds particular descriptors when they are needed.
pub trait DescriptorDatabase {
    /// Finds a file by file name.
    fn find_file_by_name(
        self: Pin<&mut Self>,
        filename: &Path,
    ) -> Result<Pin<Box<FileDescriptorProto>>, OperationFailedError>;
}

/// Describes a whole .proto file.
///
/// To get the `FileDescriptor` for a compiled-in file, get the descriptor for
/// something defined in that file and call `descriptor.file()`. Use
/// `DescriptorPool` to construct your own descriptors.
pub struct FileDescriptor {
    _opaque: PhantomPinned,
}

impl FileDescriptor {
    unsafe_ffi_conversions!(ffi::FileDescriptor);
}

impl Drop for FileDescriptor {
    fn drop(&mut self) {
        unsafe { ffi::DeleteFileDescriptor(self.as_ffi_mut_ptr_unpinned()) }
    }
}

/// Used to construct descriptors.
///
/// Normally you won't want to build your own descriptors. Message classes
/// constructed by the protocol compiler will provide them for you. However, if
/// you are implementing [`Message`] on your own, or if you are writing a
/// program which can operate on totally arbitrary types and needs to load them
/// from some sort of database, you might need to.
///
/// Since [`Descriptor`]s are composed of a whole lot of cross-linked bits of
/// data that would be a pain to put together manually, the [`DescriptorPool`]
/// class is provided to make the process easier. It can take a
/// [`FileDescriptorProto`] (defined in descriptor.proto), validate it, and
/// convert it to a set of nicely cross-linked `Descriptor`s.
///
/// [`DescriptorPool`] also helps with memory management. Descriptors are
/// composed of many objects containing static data and pointers to each other.
/// In all likelihood, when it comes time to delete this data, you'll want to
/// delete it all at once.  In fact, it is not uncommon to have a whole pool of
/// descriptors all cross-linked with each other which you wish to delete all at
/// once. This class represents such a pool, and handles the memory management
/// for you.
///
/// You can also search for descriptors within a `DescriptorPool` by name, and
/// extensions by number.
pub struct DescriptorPool {
    _opaque: PhantomPinned,
}

impl Drop for DescriptorPool {
    fn drop(&mut self) {
        unsafe { ffi::DeleteDescriptorPool(self.as_ffi_mut_ptr_unpinned()) }
    }
}

impl DescriptorPool {
    pub fn new() -> Pin<Box<DescriptorPool>> {
        let pool = ffi::NewDescriptorPool();
        unsafe { Self::from_ffi_owned(pool) }
    }

    /// Converts the `FileDescriptorProto` to real descriptors and places them
    /// in this descriptor pool.
    ///
    /// All dependencies of the file must already be in the pool. Returns the
    /// resulting [`FileDescriptor`], or `None` if there were problems with the
    /// input (e.g. the message was invalid, or dependencies were missing).
    /// Details about the errors are written to the error log.
    pub fn build_file(self: Pin<&mut Self>, proto: &FileDescriptorProto) -> &FileDescriptor {
        let file = self.as_ffi_mut().BuildFile(proto.as_ffi());
        unsafe { FileDescriptor::from_ffi_ptr(file) }
    }

    unsafe_ffi_conversions!(ffi::DescriptorPool);
}

/// Describes a type of protocol message, or a particular group within a
/// message.
///
/// To obtain the `Descriptor` for a given message object, call
/// [`Message::get_descriptor`]. Generated message classes also have a static
/// method called `descriptor` which returns the type's descriptor. Use
/// [`DescriptorPool`] to construct your own descriptors.
pub struct Descriptor {}

/// Interface to light weight protocol messages.
///
/// This interface is implemented by all protocol message objects.  Non-lite
/// messages additionally implement the [`Message`] interface, which is a
/// subclass of `MessageLite`. Use `MessageLite` instead when you only need the
/// subset of features which it supports -- namely, nothing that uses
/// descriptors or reflection. You can instruct the protocol compiler to
/// generate classes which implement only `MessageLite`, not the full `Message`
/// interface, by adding the following line to the .proto file:
///
/// ```proto
/// option optimize_for = LITE_RUNTIME;
/// ```
///
/// This is particularly useful on resource-constrained systems where the full
/// protocol buffers runtime library is too big.
///
/// Note that on non-constrained systems (e.g. servers) when you need to link in
/// lots of protocol definitions, a better way to reduce total code footprint is
/// to use `optimize_for = CODE_SIZE`. This will make the generated code smaller
/// while still supporting all the same features (at the expense of speed).
/// `optimize_for = LITE_RUNTIME` is best when you only have a small number of
/// message types linked into your binary, in which case the size of the
/// protocol buffers runtime itself is the biggest problem.
///
/// Users must not derive from this class. Only the protocol compiler and the
/// internal library are allowed to create subclasses.
pub trait MessageLite: private::MessageLite {
    /// Constructs a new instance of the same type.
    fn new(&self) -> Pin<Box<dyn MessageLite>> {
        unsafe { DynMessageLite::from_ffi_owned(ffi::NewMessageLite(self.upcast())) }
    }

    /// Clears all fields of the message and set them to their default values.
    ///
    /// This method avoids freeing memory, assuming that any memory allocated to
    /// hold parts of the message will be needed again to hold the next message.
    /// If you actually want to free the memory used by a `MessageLite`, you
    /// must drop it.
    fn clear(self: Pin<&mut Self>) {
        self.upcast_mut().Clear()
    }

    /// Quickly checks if all required fields have been set.
    fn is_initialized(&self) -> bool {
        self.upcast().IsInitialized()
    }

    /// Reads a protocol buffer from the stream and merges it into this message.
    ///
    /// Singular fields read from the what is already in the message and
    /// repeated fields are appended to those already present.
    ///
    /// It is the responsibility of the caller to call input->LastTagWas() (for
    /// groups) or input->ConsumedEntireMessage() (for non-groups) after this
    /// returns to verify that the message's end was delimited correctly.
    fn merge_from_coded_stream(
        self: Pin<&mut Self>,
        input: Pin<&mut CodedInputStream>,
    ) -> Result<(), OperationFailedError> {
        unsafe {
            self.upcast_mut()
                .MergeFromCodedStream(input.as_ffi_mut_ptr())
                .as_result()
        }
    }

    /// Writes a protocol buffer of this message to the given output.
    ///
    /// All required fields must be set.
    fn serialize_to_coded_stream(
        &self,
        output: Pin<&mut CodedOutputStream>,
    ) -> Result<(), OperationFailedError> {
        unsafe {
            self.upcast()
                .SerializeToCodedStream(output.as_ffi_mut_ptr())
                .as_result()
        }
    }

    /// Writes the message to the given zero-copy output stream.
    ///
    /// All required fields must be set.
    fn serialize_to_zero_copy_stream(
        &self,
        output: Pin<&mut dyn ZeroCopyOutputStream>,
    ) -> Result<(), OperationFailedError> {
        unsafe {
            self.upcast()
                .SerializeToZeroCopyStream(output.upcast_mut_ptr())
                .as_result()
        }
    }

    /// Writes the message to the given [`Write`] implementor.
    ///
    /// All required fields must be set.
    fn serialize_to_writer(&self, output: &mut dyn Write) -> Result<(), OperationFailedError> {
        self.serialize_to_zero_copy_stream(WriterStream::new(output).as_mut())
    }

    /// Serializes the message to a byte vector.
    ///
    /// All required fields must be set.
    fn serialize(&self) -> Result<Vec<u8>, OperationFailedError> {
        let mut output = vec![];
        self.serialize_to_writer(&mut output)?;
        Ok(output)
    }

    /// Computes the serialized size of the message.
    ///
    /// This recursively calls `byte_size` on all embedded messages. The
    /// computation is generally linear in the number of the fields defined for
    /// the proto.
    fn byte_size(&self) -> usize {
        self.upcast().ByteSizeLong()
    }
}

struct DynMessageLite {
    _opaque: PhantomPinned,
}

impl Drop for DynMessageLite {
    fn drop(&mut self) {
        unsafe { ffi::DeleteMessageLite(self.as_ffi_mut_ptr_unpinned()) }
    }
}

impl DynMessageLite {
    unsafe_ffi_conversions!(ffi::MessageLite);
}

impl MessageLite for DynMessageLite {}

impl private::MessageLite for DynMessageLite {
    fn upcast(&self) -> &ffi::MessageLite {
        unsafe { mem::transmute(self) }
    }

    fn upcast_mut(self: Pin<&mut Self>) -> Pin<&mut ffi::MessageLite> {
        unsafe { mem::transmute(self) }
    }
}

/// Abstract interface for protocol messages.
///
/// See also `MessageLite`, which contains most every-day operations.  `Message`
/// adds descriptors and reflection on top of that.
///
/// The methods of this class that have default implementations have default
/// implementations based on reflection. Message classes which are optimized for
/// speed will want to override these with faster implementations, but classes
/// optimized for code size may be happy with keeping them.  See the
/// optimize_for option in descriptor.proto.
///
/// Users must not derive from this class. Only the protocol compiler and the
/// internal library are allowed to create subclasses.
pub trait Message: private::Message + MessageLite {}

/// The protocol compiler can output a file descriptor set containing the .proto
/// files it parses.
pub struct FileDescriptorSet {
    _opaque: PhantomPinned,
}

impl Drop for FileDescriptorSet {
    fn drop(&mut self) {
        unsafe { ffi::DeleteFileDescriptorSet(self.as_ffi_mut_ptr_unpinned()) }
    }
}

impl FileDescriptorSet {
    /// Creates a a new file descriptor set.
    fn new() -> Pin<Box<FileDescriptorSet>> {
        let set = ffi::NewFileDescriptorSet();
        unsafe { Self::from_ffi_owned(set) }
    }

    /// Returns the number of file descriptors in the file descriptor set.
    pub fn file_size(&self) -> usize {
        self.as_ffi().file_size().expect_usize()
    }

    /// Clears the file descriptors.
    pub fn clear_file(self: Pin<&mut Self>) {
        self.as_ffi_mut().clear_file()
    }

    /// Returns a reference the `i`th file descriptor.
    pub fn file(&self, i: usize) -> &FileDescriptorProto {
        let file = self.as_ffi().file(CInt::expect_from(i));
        FileDescriptorProto::from_ffi_ref(file)
    }

    /// Returns a mutable reference to the `i`th file descriptor.
    pub fn file_mut(self: Pin<&mut Self>, i: usize) -> Pin<&mut FileDescriptorProto> {
        let file = self.as_ffi_mut().mutable_file(CInt::expect_from(i));
        unsafe { FileDescriptorProto::from_ffi_mut(file) }
    }

    /// Adds a new empty file descriptor and returns a mutable reference to it.
    pub fn add_file(self: Pin<&mut Self>) -> Pin<&mut FileDescriptorProto> {
        let file = self.as_ffi_mut().add_file();
        unsafe { FileDescriptorProto::from_ffi_mut(file) }
    }

    unsafe_ffi_conversions!(ffi::FileDescriptorSet);
}

impl MessageLite for FileDescriptorSet {}

impl private::MessageLite for FileDescriptorSet {
    fn upcast(&self) -> &ffi::MessageLite {
        unsafe { mem::transmute(self) }
    }

    fn upcast_mut(self: Pin<&mut Self>) -> Pin<&mut ffi::MessageLite> {
        unsafe { mem::transmute(self) }
    }
}

impl Message for FileDescriptorSet {}
impl private::Message for FileDescriptorSet {}

/// Describes a complete .proto file.
pub struct FileDescriptorProto {
    _opaque: PhantomPinned,
}

impl FileDescriptorProto {
    /// Creates a a new file descriptor proto.
    fn new() -> Pin<Box<FileDescriptorProto>> {
        let proto = ffi::NewFileDescriptorProto();
        unsafe { Self::from_ffi_owned(proto) }
    }

    /// Make this file descriptor proto into a copy of the given file descriptor
    /// proto.
    pub fn copy_from(self: Pin<&mut Self>, from: &FileDescriptorProto) {
        self.as_ffi_mut().CopyFrom(from.as_ffi())
    }

    /// Merge the fields of the file descriptor proto into this file descriptor
    /// proto.
    pub fn merge_from(self: Pin<&mut Self>, from: &FileDescriptorProto) {
        self.as_ffi_mut().MergeFrom(from.as_ffi())
    }

    /// Returns the number of entries in the `dependency` field.
    pub fn dependency_size(&self) -> usize {
        self.as_ffi().dependency_size().expect_usize()
    }

    /// Returns the `i`th entry in the `dependency` field.
    pub fn dependency(&self, i: usize) -> &[u8] {
        if i >= self.dependency_size() {
            panic!(
                "index out of bounds: the length is {} but the index is {}",
                self.dependency_size(),
                i
            );
        }
        self.as_ffi().dependency(CInt::expect_from(i)).as_bytes()
    }

    /// Returns the number of entries in the `message_type` field.
    pub fn message_type_size(&self) -> usize {
        self.as_ffi().message_type_size().expect_usize()
    }

    /// Returns the `i`th entry in the `message_type` field.
    pub fn message_type(&self, i: usize) -> &DescriptorProto {
        if i >= self.message_type_size() {
            panic!(
                "index out of bounds: the length is {} but the index is {}",
                self.message_type_size(),
                i
            );
        }
        DescriptorProto::from_ffi_ref(self.as_ffi().message_type(CInt::expect_from(i)))
    }

    unsafe_ffi_conversions!(ffi::FileDescriptorProto);
}

impl Drop for FileDescriptorProto {
    fn drop(&mut self) {
        unsafe { ffi::DeleteFileDescriptorProto(self.as_ffi_mut_ptr_unpinned()) }
    }
}

impl MessageLite for FileDescriptorProto {}

impl private::MessageLite for FileDescriptorProto {
    fn upcast(&self) -> &ffi::MessageLite {
        unsafe { mem::transmute(self) }
    }

    fn upcast_mut(self: Pin<&mut Self>) -> Pin<&mut ffi::MessageLite> {
        unsafe { mem::transmute(self) }
    }
}

impl Message for FileDescriptorProto {}
impl private::Message for FileDescriptorProto {}

/// Describes a message type.
pub struct DescriptorProto {
    _opaque: PhantomPinned,
}

impl Drop for DescriptorProto {
    fn drop(&mut self) {
        unsafe { ffi::DeleteDescriptorProto(self.as_ffi_mut_ptr_unpinned()) }
    }
}

impl DescriptorProto {
    /// Returns the name of tis message.
    pub fn name(&self) -> &[u8] {
        self.as_ffi().name().as_bytes()
    }

    unsafe_ffi_conversions!(ffi::DescriptorProto);
}

impl MessageLite for DescriptorProto {}

impl private::MessageLite for DescriptorProto {
    fn upcast(&self) -> &ffi::MessageLite {
        unsafe { mem::transmute(self) }
    }

    fn upcast_mut(self: Pin<&mut Self>) -> Pin<&mut ffi::MessageLite> {
        unsafe { mem::transmute(self) }
    }
}

impl Message for DescriptorProto {}
impl private::Message for DescriptorProto {}

/// An operation failed.
///
/// This error does not contain details about why the operation failed or what
/// the operation was. Unfortunately this is a limitation of the underlying
/// `libprotobuf` APIs.
///
/// In some cases, you may be able to find an alternative API that returns a
/// more descriptive error type (e.g., the APIs that return
/// [`compiler::FileLoadError`]), but in most cases the underlying library
/// simply provides no additional details about what went wrong.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct OperationFailedError;

impl fmt::Display for OperationFailedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("operation failed")
    }
}

impl Error for OperationFailedError {}

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

//! Auxiliary classes used for I/O.
//!
//! The Protocol Buffer library uses the classes in this package to deal with
//! I/O and encoding/decoding raw bytes. Most users will not need to deal with
//! this package. However, users who want to adapt the system to work with their
//! own I/O abstractions – e.g., to allow Protocol Buffers to be read from a
//! different kind of input stream without the need for a temporary buffer –
//! should take a closer look.
//!
//! # Zero-copy streams
//!
//! The [`ZeroCopyInputStream`] and [`ZeroCopyOutputStream`] interfaces
//! represent abstract I/O streams to and from which protocol buffers can be
//! read and written.
//!
//! These interfaces are different from classic I/O streams in that they try to
//! minimize the amount of data copying that needs to be done. To accomplish
//! this, responsibility for allocating buffers is moved to the stream object,
//! rather than being the responsibility of the caller. So, the stream can
//! return a buffer which actually points directly into the final data structure
//! where the bytes are to be stored, and the caller can interact directly with
//! that buffer, eliminating an intermediate copy operation.
//!
//! As an example, consider the common case in which you are reading bytes from
//! an array that is already in memory (or perhaps an `mmap`ed file).
//!
//! With classic I/O streams, you would do something like:
//!
//! ```
//! # use std::io::Read;
//! # use protobuf_native::io::ZeroCopyInputStream;
//! # const BUFFER_SIZE: usize = 1024;
//! # fn f(input: &mut dyn Read) {
//! let mut buffer = [0; BUFFER_SIZE];
//! input.read(&mut buffer);
//! // Do something with `buffer`.
//! # }
//! ```
//!
//! Then the stream basically just calls `memcpy` to copy the data from the
//! array into your buffer. With a `ZeroCopyInputStream`, you would do this
//! instead:
//!
//! ```
//! # use protobuf_native::io::ZeroCopyInputStream;
//! # fn f(input: &mut dyn ZeroCopyInputStream) {
//! let buffer = input.next();
//! // Do something with `buffer`.
//! # }
//! ```
//! Here, no copy is performed. The input stream returns a slice directly into
//! the backing array, and the caller ends up reading directly from it.
//
//! If you want to be able to read the old-fashioned way, you can create a
//! [`CodedInputStream`] or [`CodedOutputStream`] wrapping these objects and use
//! their [`Read`]/[`Write`] implementations. These will, of course, add a copy
//! step, but the coded streams will handle buffering so at least it will be
//! reasonably efficient.
//
//! # Coded streams
//!
//! The [`CodedInputStream`] and [`CodedOutputStream`] classes, which wrap a
//! [`ZeroCopyInputStream`] or [`ZeroCopyOutputStream`], respectively, and allow
//! you to read or write individual pieces of data in various formats. In
//! particular, these implement the varint encoding for integers, a simple
//! variable-length encoding in which smaller numbers take fewer bytes.
//!
//! Typically these classes will only be used internally by the protocol buffer
//! library in order to encode and decode protocol buffers. Clients of the
//! library only need to know about this class if they wish to write custom
//! message parsing or serialization procedures.
//!
//! For those who are interested, varint encoding is defined as follows:
//!
//! The encoding operates on unsigned integers of up to 64 bits in length. Each
//! byte of the encoded value has the format:
//!
//! * bits 0-6: Seven bits of the number being encoded.
//!
//! * bit 7: Zero if this is the last byte in the encoding (in which case all
//!   remaining bits of the number are zero) or 1 if more bytes follow. The
//!   first byte contains the least-significant 7 bits of the number, the second
//!   byte (if present) contains the next-least-significant 7 bits, and so on.
//!   So, the binary number 1011000101011 would be encoded in two bytes as
//!   "10101011 00101100".
//!
//! In theory, varint could be used to encode integers of any length. However,
//! for practicality we set a limit at 64 bits. The maximum encoded length of a
//! number is thus 10 bytes.

use std::io::{self, Read, Write};
use std::marker::{PhantomData, PhantomPinned};
use std::mem::{self, MaybeUninit};
use std::pin::Pin;
use std::slice;

use crate::internal::{unsafe_ffi_conversions, BoolExt, CInt, CVoid, ReadAdaptor, WriteAdaptor};
use crate::OperationFailedError;

#[cxx::bridge(namespace = "protobuf_native::io")]
pub(crate) mod ffi {
    extern "Rust" {
        type ReadAdaptor<'a>;
        fn read(self: &mut ReadAdaptor<'_>, buf: &mut [u8]) -> isize;

        type WriteAdaptor<'a>;
        fn write(self: &mut WriteAdaptor<'_>, buf: &[u8]) -> bool;
    }
    unsafe extern "C++" {
        include!("protobuf-native/src/internal.h");
        include!("protobuf-native/src/io.h");

        #[namespace = "protobuf_native::internal"]
        type CVoid = crate::internal::CVoid;

        #[namespace = "protobuf_native::internal"]
        type CInt = crate::internal::CInt;

        #[namespace = "google::protobuf::io"]
        type ZeroCopyInputStream;
        unsafe fn DeleteZeroCopyInputStream(stream: *mut ZeroCopyInputStream);
        unsafe fn Next(
            self: Pin<&mut ZeroCopyInputStream>,
            data: *mut *const CVoid,
            size: *mut CInt,
        ) -> bool;
        fn BackUp(self: Pin<&mut ZeroCopyInputStream>, count: CInt);
        fn Skip(self: Pin<&mut ZeroCopyInputStream>, count: CInt) -> bool;
        fn ByteCount(self: &ZeroCopyInputStream) -> i64;

        type ReaderStream;
        fn NewReaderStream(adaptor: Box<ReadAdaptor<'_>>) -> *mut ReaderStream;
        unsafe fn DeleteReaderStream(stream: *mut ReaderStream);

        #[namespace = "google::protobuf::io"]
        type ArrayInputStream;
        unsafe fn NewArrayInputStream(data: *const u8, size: CInt) -> *mut ArrayInputStream;
        unsafe fn DeleteArrayInputStream(stream: *mut ArrayInputStream);

        #[namespace = "google::protobuf::io"]
        type ZeroCopyOutputStream;
        unsafe fn Next(
            self: Pin<&mut ZeroCopyOutputStream>,
            data: *mut *mut CVoid,
            size: *mut CInt,
        ) -> bool;
        fn BackUp(self: Pin<&mut ZeroCopyOutputStream>, count: CInt);
        fn ByteCount(self: &ZeroCopyOutputStream) -> i64;

        type WriterStream;
        fn NewWriterStream(adaptor: Box<WriteAdaptor<'_>>) -> *mut WriterStream;
        unsafe fn DeleteWriterStream(stream: *mut WriterStream);

        #[namespace = "google::protobuf::io"]
        type ArrayOutputStream;
        unsafe fn NewArrayOutputStream(data: *mut u8, size: CInt) -> *mut ArrayOutputStream;
        unsafe fn DeleteArrayOutputStream(stream: *mut ArrayOutputStream);

        type VecOutputStream;
        fn NewVecOutputStream(target: &mut Vec<u8>) -> *mut VecOutputStream;
        unsafe fn DeleteVecOutputStream(stream: *mut VecOutputStream);

        #[namespace = "google::protobuf::io"]
        type CodedInputStream;
        unsafe fn NewCodedInputStream(ptr: *mut ZeroCopyInputStream) -> *mut CodedInputStream;
        unsafe fn DeleteCodedInputStream(stream: *mut CodedInputStream);
        fn IsFlat(self: &CodedInputStream) -> bool;
        unsafe fn ReadRaw(self: Pin<&mut CodedInputStream>, buffer: *mut CVoid, size: CInt)
            -> bool;
        unsafe fn ReadVarint32(self: Pin<&mut CodedInputStream>, value: *mut u32) -> bool;
        unsafe fn ReadVarint64(self: Pin<&mut CodedInputStream>, value: *mut u64) -> bool;
        fn ReadTag(self: Pin<&mut CodedInputStream>) -> u32;
        fn ReadTagNoLastTag(self: Pin<&mut CodedInputStream>) -> u32;
        fn LastTagWas(self: Pin<&mut CodedInputStream>, expected: u32) -> bool;
        fn ConsumedEntireMessage(self: Pin<&mut CodedInputStream>) -> bool;
        fn CurrentPosition(self: &CodedInputStream) -> CInt;

        #[namespace = "google::protobuf::io"]
        type CodedOutputStream;
        unsafe fn DeleteCodedOutputStream(stream: *mut CodedOutputStream);
    }

    impl UniquePtr<ZeroCopyOutputStream> {}
    impl UniquePtr<CodedOutputStream> {}
}

/// Abstract interface similar to an input stream but designed to minimize
/// copying.
///
/// # Examples
///
/// Read in a file and print its contents to stdout:
///
/// ```no_run
/// use std::fs::File;
/// use std::io::{self, Write};
/// use protobuf_native::io::{ReaderStream, ZeroCopyInputStream};
///
/// let mut f = File::open("myfile")?;
/// let mut input = ReaderStream::new(&mut f);
/// while let Ok(buf) = input.next() {
///     io::stdout().write_all(buf)?;
/// }
/// # Ok::<_, io::Error>(())
/// ```
pub trait ZeroCopyInputStream: zero_copy_input_stream::Sealed {
    /// Obtains a chunk of data from the stream.
    ///
    /// If the function returns an error, either there is no more data to return
    /// or an I/O error occurred. All errors are permanent.
    ///
    /// It is legal for the returned buffer to have zero size, as long as
    /// repeatedly calling `next` eventually yields a buffer with non-zero size.
    fn next(self: Pin<&mut Self>) -> Result<&[u8], OperationFailedError> {
        let mut data = MaybeUninit::uninit();
        let mut size = MaybeUninit::uninit();
        unsafe {
            // SAFETY: `data` and `size` are non-null, as required.
            self.upcast_mut()
                .Next(data.as_mut_ptr(), size.as_mut_ptr())
                .as_result()?;
            // SAFETY: `Next` has succeeded and so has promised to provide us
            // with a valid buffer.
            let data = data.assume_init() as *const u8;
            let size = size.assume_init().to_usize()?;
            Ok(slice::from_raw_parts(data, size))
        }
    }

    /// Backs up a number of bytes, so that the next call to [`next`] returns
    /// data again that was already returned by the last call to `next`.
    ///
    /// This is useful when writing procedures that are only supposed to read up
    /// to a certain point in the input, then return. If `next` returns a buffer
    /// that goes beyond what you wanted to read, you can use `back_up` to
    /// return to the point where you intended to finish.
    ///
    /// The last method called must have been `next`. The `count` parameter
    /// must be less than or equal to the size of the last buffer returned
    /// by `next`.
    ///
    /// [`next`]: ZeroCopyInputStream::next
    fn back_up(self: Pin<&mut Self>, count: usize) {
        // `count` is required to be less than the size of the buffer returned
        // by the last call to `next`. Since `count` originated as a C int, if
        // it's valid it must be representible as a C int. No point doing
        // something more graceful than panicking since `BackUp` will often
        // crash the process on too-large input.
        let count = CInt::try_from(count).expect("count did not fit in a C int");
        self.upcast_mut().BackUp(count)
    }

    /// Skips `count` bytes.
    ///
    /// Returns an error if the end of stream is reached or an I/O error
    /// occurred. In the end-of-stream case, the stream is advanced to its end,
    /// so [`byte_count`] will return the total size of the stream.
    ///
    /// [`byte_count`]: ZeroCopyInputStream::byte_count
    fn skip(self: Pin<&mut Self>, count: usize) -> Result<(), OperationFailedError> {
        let count = CInt::try_from(count).map_err(|_| OperationFailedError)?;
        self.upcast_mut().Skip(count).as_result()
    }

    /// Returns the total number of bytes read since this stream was created.
    fn byte_count(&self) -> i64 {
        self.upcast().ByteCount()
    }
}

mod zero_copy_input_stream {
    use std::pin::Pin;

    use crate::io::ffi;

    pub trait Sealed {
        fn upcast(&self) -> &ffi::ZeroCopyInputStream;
        fn upcast_mut(self: Pin<&mut Self>) -> Pin<&mut ffi::ZeroCopyInputStream>;
        unsafe fn upcast_mut_ptr(self: Pin<&mut Self>) -> *mut ffi::ZeroCopyInputStream {
            self.upcast_mut().get_unchecked_mut() as *mut _
        }
    }
}

/// Converts an [`Read`] implementor to a [`ZeroCopyInputStream`].
pub struct ReaderStream<'a> {
    _opaque: PhantomPinned,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> Drop for ReaderStream<'a> {
    fn drop(&mut self) {
        unsafe { ffi::DeleteReaderStream(self.as_ffi_mut_ptr_unpinned()) }
    }
}

impl<'a> ReaderStream<'a> {
    /// Creates a reader stream from the specified [`Read`] implementor.
    pub fn new(reader: &'a mut dyn Read) -> Pin<Box<ReaderStream<'a>>> {
        let stream = ffi::NewReaderStream(Box::new(ReadAdaptor(reader)));
        unsafe { Self::from_ffi_owned(stream) }
    }

    unsafe_ffi_conversions!(ffi::ReaderStream);
}

impl<'a> ZeroCopyInputStream for ReaderStream<'a> {}

impl<'a> zero_copy_input_stream::Sealed for ReaderStream<'a> {
    fn upcast(&self) -> &ffi::ZeroCopyInputStream {
        unsafe { mem::transmute(self) }
    }

    fn upcast_mut(self: Pin<&mut Self>) -> Pin<&mut ffi::ZeroCopyInputStream> {
        unsafe { mem::transmute(self) }
    }
}

/// A [`ZeroCopyInputStream`] specialized for reading from byte slices.
///
/// Using this type is more efficient than using a [`ReaderStream`] when the
/// underlying reader is a type that exposes a simple byte slice.
pub struct SliceInputStream<'a> {
    _opaque: PhantomPinned,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> Drop for SliceInputStream<'a> {
    fn drop(&mut self) {
        unsafe { ffi::DeleteArrayInputStream(self.as_ffi_mut_ptr_unpinned()) }
    }
}

impl<'a> SliceInputStream<'a> {
    /// Creates a new `SliceInputStream` from the provided byte slice.
    pub fn new(slice: &[u8]) -> Pin<Box<SliceInputStream<'a>>> {
        let size = CInt::expect_from(slice.len());
        let stream = unsafe { ffi::NewArrayInputStream(slice.as_ptr(), size) };
        unsafe { Self::from_ffi_owned(stream) }
    }

    unsafe_ffi_conversions!(ffi::ArrayInputStream);
}

impl<'a> ZeroCopyInputStream for SliceInputStream<'a> {}

impl<'a> zero_copy_input_stream::Sealed for SliceInputStream<'a> {
    fn upcast(&self) -> &ffi::ZeroCopyInputStream {
        unsafe { mem::transmute(self) }
    }

    fn upcast_mut(self: Pin<&mut Self>) -> Pin<&mut ffi::ZeroCopyInputStream> {
        unsafe { mem::transmute(self) }
    }
}

/// An arbitrary stream that implements [`ZeroCopyInputStream`].
///
/// This is like `Box<dyn ZeroCopyInputStream>` but it avoids additional virtual
/// method calls on the Rust side of the FFI boundary.
pub struct DynZeroCopyInputStream<'a> {
    _opaque: PhantomPinned,
    lifetime_: PhantomData<&'a ()>,
}

impl<'a> Drop for DynZeroCopyInputStream<'a> {
    fn drop(&mut self) {
        unsafe { ffi::DeleteZeroCopyInputStream(self.as_ffi_mut_ptr_unpinned()) }
    }
}

impl<'a> DynZeroCopyInputStream<'a> {
    unsafe_ffi_conversions!(ffi::ZeroCopyInputStream);
}

impl ZeroCopyInputStream for DynZeroCopyInputStream<'_> {}

impl zero_copy_input_stream::Sealed for DynZeroCopyInputStream<'_> {
    fn upcast(&self) -> &ffi::ZeroCopyInputStream {
        unsafe { mem::transmute(self) }
    }

    fn upcast_mut(self: Pin<&mut Self>) -> Pin<&mut ffi::ZeroCopyInputStream> {
        unsafe { mem::transmute(self) }
    }
}

/// Abstract interface similar to an output stream but designed to minimize
/// copying.
///
/// # Examples
///
/// Copy the contents of infile to outfile, using plain [`Read`] for infile
/// but a `ZeroCopyOutputStream` for outfile:
///
/// ```ignore
/// use std::fs::File;
/// use std::io::{self, Read, Write};
/// use protobuf_native::io::{WriterStream, ZeroCopyOutputStream};
///
/// let mut infile = File::open("infile")?;
/// let mut outfile = File::create("outfile")?;
/// let mut output = WriterStream::new(&mut outfile);
///
/// while let Ok(buf) = output.next() {
///     // Reading into uninitialized memory requires the unstable `ReadBuf` API.
///     // See: https://rust-lang.github.io/rfcs/2930-read-buf.html
///     let buf = ReadBuf::uninit(buf);
///     infile.read_buf(buf)?;
///     output.back_up(buf.remaining());
///     if buf.filled().is_empty() {
///         break;
///     }
/// }
///
/// # Ok::<_, io::Error>(())
/// ```
pub trait ZeroCopyOutputStream: zero_copy_output_stream::Sealed {
    /// Obtains a buffer into which data can be written.
    ///
    /// Any data written into this buffer will eventually (maybe instantly,
    /// maybe later on) be written to the output.
    ///
    /// # Safety
    ///
    /// If this function returns `Ok`, you **must** initialize the returned byte
    /// slice before you either call `next` again or drop the slice. You can
    /// choose to initialize only a portion of the byte slice by calling
    /// [`back_up`].
    ///
    /// This is a very unusual invariant to maintain in Rust.
    ///
    /// [`back_up`]: ZeroCopyOutputStream::back_up
    unsafe fn next(self: Pin<&mut Self>) -> Result<&mut [MaybeUninit<u8>], OperationFailedError> {
        let mut data = MaybeUninit::uninit();
        let mut size = MaybeUninit::uninit();
        self.upcast_mut()
            .Next(data.as_mut_ptr(), size.as_mut_ptr())
            .as_result()?;
        let data = data.assume_init() as *mut MaybeUninit<u8>;
        let size = size.assume_init().to_usize()?;
        Ok(slice::from_raw_parts_mut(data, size))
    }

    /// Backs up a number of bytes, so that the end of the last buffer returned
    /// by [`next`] is not actually written.
    ///
    /// This is needed when you finish writing all the data you want to write,
    /// but the last buffer was bigger than you needed. You don't want to write
    /// a bunch of garbage after the end of your data, so you use `back_up` to
    /// back up.
    ///
    /// [`next`]: ZeroCopyOutputStream::next
    fn back_up(self: Pin<&mut Self>, count: usize) {
        // See comment in `ZeroCopyInputStream::back_up` for why we tolerate
        // panics here.
        let count = CInt::try_from(count).expect("count did not fit in a C int");
        self.upcast_mut().BackUp(count)
    }

    /// Returns the total number of bytes written since this object was created.
    fn byte_count(&self) -> i64 {
        self.upcast().ByteCount()
    }
}

mod zero_copy_output_stream {
    use std::pin::Pin;

    use crate::io::ffi;

    pub trait Sealed {
        fn upcast(&self) -> &ffi::ZeroCopyOutputStream;
        fn upcast_mut(self: Pin<&mut Self>) -> Pin<&mut ffi::ZeroCopyOutputStream>;
        unsafe fn upcast_mut_ptr(self: Pin<&mut Self>) -> *mut ffi::ZeroCopyOutputStream {
            self.upcast_mut().get_unchecked_mut() as *mut _
        }
    }
}

/// Converts an [`Write`] implementor to a [`ZeroCopyOutputStream`].
pub struct WriterStream<'a> {
    _opaque: PhantomPinned,
    _lifetime: PhantomData<&'a mut ()>,
}

impl<'a> WriterStream<'a> {
    /// Creates a writer stream from the specified [`Write`] implementor.
    pub fn new(writer: &'a mut dyn Write) -> Pin<Box<WriterStream<'a>>> {
        let stream = ffi::NewWriterStream(Box::new(WriteAdaptor(writer)));
        unsafe { Self::from_ffi_owned(stream) }
    }

    unsafe_ffi_conversions!(ffi::WriterStream);
}

impl<'a> Drop for WriterStream<'a> {
    fn drop(&mut self) {
        unsafe { ffi::DeleteWriterStream(self.as_ffi_mut_ptr_unpinned()) }
    }
}

impl<'a> ZeroCopyOutputStream for WriterStream<'a> {}

impl<'a> zero_copy_output_stream::Sealed for WriterStream<'a> {
    fn upcast(&self) -> &ffi::ZeroCopyOutputStream {
        unsafe { mem::transmute(self) }
    }

    fn upcast_mut(self: Pin<&mut Self>) -> Pin<&mut ffi::ZeroCopyOutputStream> {
        unsafe { mem::transmute(self) }
    }
}

/// A [`ZeroCopyOutputStream`] specialized for writing to byte slices.
///
/// Using this type is more efficient than using a [`WriterStream`] when the
/// underlying writer is a type that exposes a simple mutable byte slice.
pub struct SliceOutputStream<'a> {
    _opaque: PhantomPinned,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> SliceOutputStream<'a> {
    /// Creates a new `SliceOutputStream` from the provided byte slice.
    pub fn new(slice: &mut [u8]) -> Pin<Box<SliceOutputStream<'a>>> {
        let size = CInt::expect_from(slice.len());
        let stream = unsafe { ffi::NewArrayOutputStream(slice.as_mut_ptr(), size) };
        unsafe { Self::from_ffi_owned(stream) }
    }

    unsafe_ffi_conversions!(ffi::ArrayOutputStream);
}

impl<'a> Drop for SliceOutputStream<'a> {
    fn drop(&mut self) {
        unsafe { ffi::DeleteArrayOutputStream(self.as_ffi_mut_ptr_unpinned()) }
    }
}

impl<'a> ZeroCopyOutputStream for SliceOutputStream<'a> {}

impl<'a> zero_copy_output_stream::Sealed for SliceOutputStream<'a> {
    fn upcast(&self) -> &ffi::ZeroCopyOutputStream {
        unsafe { mem::transmute(self) }
    }

    fn upcast_mut(self: Pin<&mut Self>) -> Pin<&mut ffi::ZeroCopyOutputStream> {
        unsafe { mem::transmute(self) }
    }
}

/// A [`ZeroCopyOutputStream`] specialized for writing to byte vectors.
///
/// Using this type is more efficient than using a [`WriterStream`] when the
/// underlying writer is a byte vector.
pub struct VecOutputStream<'a> {
    _opaque: PhantomPinned,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> VecOutputStream<'a> {
    /// Creates a new `VecOutputStream` from the provided byte vector.
    pub fn new(vec: &mut Vec<u8>) -> Pin<Box<VecOutputStream<'a>>> {
        let stream = ffi::NewVecOutputStream(vec);
        unsafe { Self::from_ffi_owned(stream) }
    }

    unsafe_ffi_conversions!(ffi::VecOutputStream);
}

impl<'a> Drop for VecOutputStream<'a> {
    fn drop(&mut self) {
        unsafe { ffi::DeleteVecOutputStream(self.as_ffi_mut_ptr_unpinned()) }
    }
}

impl<'a> ZeroCopyOutputStream for VecOutputStream<'a> {}

impl<'a> zero_copy_output_stream::Sealed for VecOutputStream<'a> {
    fn upcast(&self) -> &ffi::ZeroCopyOutputStream {
        unsafe { mem::transmute(self) }
    }

    fn upcast_mut(self: Pin<&mut Self>) -> Pin<&mut ffi::ZeroCopyOutputStream> {
        unsafe { mem::transmute(self) }
    }
}

/// Type which reads and decodes binary data which is composed of varint-
/// encoded integers and fixed-width pieces.
///
/// Wraps a [`ZeroCopyInputStream`]. Most users will not need to deal with
/// `CodedInputStream`.
///
/// Most methods of `CodedInputStream` that return a `Result` return an error if
/// an underlying I/O error occurs or if the data is malformed. Once such a
/// failure occurs, the `CodedInputStream` is broken and is no longer useful.
pub struct CodedInputStream<'a> {
    _opaque: PhantomPinned,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> Drop for CodedInputStream<'a> {
    fn drop(&mut self) {
        unsafe { ffi::DeleteCodedInputStream(self.as_ffi_mut_ptr_unpinned()) }
    }
}

impl<'a> CodedInputStream<'a> {
    /// Creates a `CodedInputStream` that reads from the given
    /// [`ZeroCopyInputStream`].
    pub fn new(input: Pin<&'a mut dyn ZeroCopyInputStream>) -> Pin<Box<CodedInputStream<'a>>> {
        let stream = unsafe { ffi::NewCodedInputStream(input.upcast_mut_ptr()) };
        unsafe { Self::from_ffi_owned(stream) }
    }

    /// Reports whether this coded input stream reads from a flat array instead
    /// of a [`ZeroCopyInputStream`].
    pub fn is_flat(&self) -> bool {
        self.as_ffi().IsFlat()
    }

    /// Reads an unsigned integer with varint encoding, truncating to 32 bits.
    ///
    /// Reading a 32-bit value is equivalent to reading a 64-bit one and casting
    /// it to `u32`, but may be more efficient.
    pub fn read_varint32(self: Pin<&mut Self>) -> Result<u32, OperationFailedError> {
        let mut value = MaybeUninit::uninit();
        // SAFETY: `ReadVarint32` promises to initialize `value` if it returns
        // true.
        unsafe {
            match self.as_ffi_mut().ReadVarint32(value.as_mut_ptr()) {
                true => Ok(value.assume_init()),
                false => Err(OperationFailedError),
            }
        }
    }

    /// Reads an unsigned 64-bit integer with varint encoding.
    pub fn read_varint64(self: Pin<&mut Self>) -> Result<u64, OperationFailedError> {
        let mut value = MaybeUninit::uninit();
        // SAFETY: `ReadVarint32` promises to initialize `value` if it returns
        // true.
        unsafe {
            match self.as_ffi_mut().ReadVarint64(value.as_mut_ptr()) {
                true => Ok(value.assume_init()),
                false => Err(OperationFailedError),
            }
        }
    }

    /// Reads a tag.
    ///
    /// This calls [`read_varint32`] and returns the result. Also updates the
    /// last tag value, which can be checked with [`last_tag_was`].
    ///
    /// [`read_varint32`]: CodedInputStream::read_varint32
    /// [`last_tag_was`]: CodedInputStream::last_tag_was
    pub fn read_tag(self: Pin<&mut Self>) -> Result<u32, OperationFailedError> {
        match self.as_ffi_mut().ReadTag() {
            0 => Err(OperationFailedError), // 0 is error sentinel
            tag => Ok(tag),
        }
    }

    /// Like [`read_tag`], but does not update the last tag
    /// value.
    ///
    /// [`read_tag`]: `CodedInputStream::read_tag`
    pub fn read_tag_no_last_tag(self: Pin<&mut Self>) -> Result<u32, OperationFailedError> {
        match self.as_ffi_mut().ReadTag() {
            0 => Err(OperationFailedError), // 0 is error sentinel
            tag => Ok(tag),
        }
    }

    /// Reports whether the last call to [`read_tag`] or
    /// [`read_tag_with_cutoff`] returned the given value.
    ///
    /// [`read_tag_no_last_tag`] and [`read_tag_with_cutoff_no_last_tag`] do not
    /// preserve the last returned value.
    ///
    /// This is needed because parsers for some types of embedded messages (with
    /// field type `TYPE_GROUP`) don't actually know that they've reached the
    /// end of a message until they see an `ENDGROUP` tag, which was actually
    /// part of the enclosing message. The enclosing message would like to check
    /// that tag to make sure it had the right number, so it calls
    /// `last_tag_was` on return from the embedded parser to check.
    ///
    /// [`read_tag`]: CodedInputStream::read_tag
    /// [`read_tag_with_cutoff`]: CodedInputStream::read_tag_with_cutoff
    /// [`read_tag_no_last_tag`]: CodedInputStream::read_tag_no_last_tag
    /// [`read_tag_with_cutoff_no_last_tag`]: CodedInputStream::read_tag_with_cutoff_no_last_tag
    pub fn last_tag_was(self: Pin<&mut Self>, expected: u32) -> bool {
        self.as_ffi_mut().LastTagWas(expected)
    }

    /// When parsing a message (but NOT a group), this method must be called
    /// immediately after [`MessageLite::merge_from_coded_stream`] returns (if
    /// it returns true) to further verify that the message ended in a
    /// legitimate way.
    ///
    /// For example, this verifies that parsing did not end on an end-group tag.
    /// It also checks for some cases where, due to optimizations,
    /// `merge_from_coded_stream` can incorrectly return true.
    ///
    /// [`MessageLite::merge_from_coded_stream`]: crate::MessageLite::merge_from_coded_stream
    pub fn consumed_entire_message(self: Pin<&mut Self>) -> bool {
        self.as_ffi_mut().ConsumedEntireMessage()
    }

    /// Returns the stream's current position relative to the beginning of the
    /// input.
    pub fn current_position(&self) -> usize {
        self.as_ffi()
            .CurrentPosition()
            .to_usize()
            .expect("stream position not representable as usize")
    }

    unsafe_ffi_conversions!(ffi::CodedInputStream);
}

impl<'a> Read for Pin<&mut CodedInputStream<'a>> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        let start = self.current_position();
        let data = buf.as_mut_ptr() as *mut CVoid;
        let size = CInt::try_from(buf.len()).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "buffer exceeds size of a C int",
            )
        })?;
        unsafe { self.as_mut().as_ffi_mut().ReadRaw(data, size) };
        let end = self.current_position();
        Ok(end - start)
    }
}

/// Type which encodes and writes binary data which is composed of varint-
/// encoded integers and fixed-width pieces.
///
/// Wraps a [`ZeroCopyOutputStream`]. Most users will not need to deal with
/// `CodedOutputStream`.
///
/// Most methods of `CodedOutputStream` which return a `bool` return false if an
/// underlying I/O error occurs. Once such a failure occurs, the
/// CodedOutputStream is broken and is no longer useful. The `write_*` methods
/// do not return the stream status, but will invalidate the stream if an error
/// occurs. The client can probe `had_error` to determine the status.
pub struct CodedOutputStream<'a> {
    _opaque: PhantomPinned,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> CodedOutputStream<'a> {
    unsafe_ffi_conversions!(ffi::CodedOutputStream);
}

impl<'a> Drop for CodedOutputStream<'a> {
    fn drop(&mut self) {
        unsafe { ffi::DeleteCodedOutputStream(self.as_ffi_mut_ptr_unpinned()) }
    }
}

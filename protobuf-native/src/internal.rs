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

#[cfg(unix)]
use std::ffi::OsStr;
use std::fmt;
use std::io::{Read, Write};
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::os::raw::{c_char, c_int, c_void};
#[cfg(unix)]
use std::os::unix::prelude::OsStrExt;
use std::path::Path;

use cxx::kind::Trivial;
use cxx::{type_id, ExternType};

use crate::OperationFailedError;

// Pollyfill C++ APIs that aren't yet in cxx.
// See: https://github.com/dtolnay/cxx/pull/984
// See: https://github.com/dtolnay/cxx/pull/990

#[cxx::bridge]
mod ffi {
    extern "Rust" {
        unsafe fn vec_u8_set_len(v: &mut Vec<u8>, new_len: usize);
    }

    unsafe extern "C++" {
        include!("protobuf-native/src/internal.h");

        #[namespace = "absl"]
        #[cxx_name = "string_view"]
        type StringView<'a> = crate::internal::StringView<'a>;

        #[namespace = "protobuf_native::internal"]
        fn string_view_from_bytes(bytes: &[u8]) -> StringView;
    }
}

unsafe fn vec_u8_set_len(v: &mut Vec<u8>, new_len: usize) {
    v.set_len(new_len)
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct StringView<'a> {
    repr: MaybeUninit<[*const c_void; 2]>,
    borrow: PhantomData<&'a [c_char]>,
}

impl<'a> From<&'a str> for StringView<'a> {
    fn from(s: &'a str) -> StringView<'a> {
        ffi::string_view_from_bytes(s.as_bytes())
    }
}

impl<'a> From<ProtobufPath<'a>> for StringView<'a> {
    fn from(path: ProtobufPath<'a>) -> StringView<'a> {
        ffi::string_view_from_bytes(path.as_bytes())
    }
}

unsafe impl<'a> ExternType for StringView<'a> {
    type Id = type_id!("absl::string_view");
    type Kind = Trivial;
}

// Variable-width integer types.
// See: https://github.com/google/autocxx/issues/422#issuecomment-826987408

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CInt(pub c_int);

impl CInt {
    pub fn to_usize(self) -> Result<usize, OperationFailedError> {
        usize::try_from(self.0).map_err(|_| OperationFailedError)
    }

    pub fn expect_usize(self) -> usize {
        match self.to_usize() {
            Ok(n) => n,
            Err(_) => panic!("C int is not representible as a Rust usize: {}", self.0),
        }
    }

    pub fn try_from<T>(value: T) -> Result<CInt, T::Error>
    where
        T: TryInto<c_int>,
    {
        value.try_into().map(CInt)
    }

    pub fn expect_from<T>(value: T) -> CInt
    where
        T: TryInto<c_int> + Copy + fmt::Display,
    {
        match CInt::try_from(value) {
            Ok(n) => n,
            Err(_) => panic!("value is not representable as a C int: {}", value),
        }
    }
}

unsafe impl ExternType for CInt {
    type Id = type_id!("protobuf_native::internal::CInt");
    type Kind = Trivial;
}

#[derive(Debug)]
pub struct CVoid(pub c_void);

unsafe impl ExternType for CVoid {
    type Id = type_id!("protobuf_native::internal::CVoid");
    type Kind = Trivial;
}

// `Read` and `Write` adaptors for C++.

pub struct ReadAdaptor<'a>(pub &'a mut dyn Read);

impl ReadAdaptor<'_> {
    pub fn read(&mut self, buf: &mut [u8]) -> isize {
        match self.0.read(buf) {
            Ok(n) => n.try_into().expect("read bytes do not fit into isize"),
            Err(_) => -1,
        }
    }
}

pub struct WriteAdaptor<'a>(pub &'a mut dyn Write);

impl WriteAdaptor<'_> {
    pub fn write(&mut self, buf: &[u8]) -> bool {
        self.0.write_all(buf).as_status()
    }
}

/// Extensions to [`Result`].
pub trait ResultExt {
    /// Converts this result into a status boolean.
    ///
    /// If the result is `Ok`, returns true. If the result is `Err`, returns
    /// false.
    fn as_status(&self) -> bool;
}

impl<T, E> ResultExt for Result<T, E> {
    fn as_status(&self) -> bool {
        match self {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

/// Extensions to [`bool`].
pub trait BoolExt {
    /// Converts this status boolean into a result.
    ///
    /// If the status boolean is true, returns `Ok`. If the status boolean is
    /// false, returns `Err`.
    fn as_result(self) -> Result<(), OperationFailedError>;
}

impl BoolExt for bool {
    fn as_result(self) -> Result<(), OperationFailedError> {
        match self {
            true => Ok(()),
            false => Err(OperationFailedError),
        }
    }
}

/// An adapter for passing paths to `libprotobuf`.
///
/// On Unix, the bytes in a path can be passed directly.
///
/// On Windows, the situation is complicated. Protobuf assumes paths are UTF-8
/// and converts them to wide-character strings before passing them to the
/// underlying Windows wide-char APIs. But paths in Rust might not valid UTF-8.
/// There's not much we can do to handle invalid UTF-8 correctly; we just throw
/// `to_string_lossy` at the problem and hope `libprotobuf` sorts it out.
///
/// The point is to make this correct and performant on Unix in all cases, and
/// correct in Windows as long as the path is valid UTF-8.
#[cfg(unix)]
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ProtobufPath<'a>(&'a Path);

#[cfg(windows)]
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ProtobufPath<'a> {
    inner: Vec<u8>,
    _phantom: PhantomData<'a>,
}

#[cfg(unix)]
impl<'a> ProtobufPath<'a> {
    pub fn as_path(&self) -> impl AsRef<Path> + 'a {
        self.0
    }
}

#[cfg(unix)]
impl<'a> From<&'a [u8]> for ProtobufPath<'a> {
    fn from(p: &'a [u8]) -> ProtobufPath<'a> {
        ProtobufPath(Path::new(OsStr::from_bytes(p)))
    }
}

#[cfg(unix)]
impl<'a> From<&'a Path> for ProtobufPath<'a> {
    fn from(p: &'a Path) -> ProtobufPath<'a> {
        ProtobufPath(p)
    }
}

#[cfg(unix)]
impl<'a> ProtobufPath<'a> {
    pub fn as_bytes(&self) -> &'a [u8] {
        self.0.as_os_str().as_bytes()
    }
}

#[cfg(windows)]
impl<'a> ProtobufPath<'a> {
    pub fn as_path(&self) -> impl AsRef<Path> {
        PathBuf::from(String::from_utf8_lossy(self.inner))
    }
}

#[cfg(windows)]
impl<'a> From<&'a [u8]> for ProtobufPath<'static> {
    fn from(p: &'a [u8]) -> ProtobufPath<'static> {
        ProtobufPath {
            inner: p.to_vec(),
            _phantom: PhantomData,
        }
    }
}

#[cfg(windows)]
impl<'a> From<Path> for ProtobufPath<'a> {
    fn from(p: Path) -> ProtobufPath<'a> {
        ProtobufPath {
            inner: p.to_string_lossy().into_owned().into_bytes(),
            _phantom: PhantomData,
        }
    }
}

#[cfg(windows)]
impl<'a> ProtobufPath<'a> {
    pub fn as_bytes(&self) -> &'a [u8] {
        &self.inner
    }
}

macro_rules! unsafe_ffi_conversions {
    ($ty:ty) => {
        #[allow(dead_code)]
        pub(crate) unsafe fn from_ffi_owned(from: *mut $ty) -> Pin<Box<Self>> {
            std::mem::transmute(from)
        }

        #[allow(dead_code)]
        pub(crate) unsafe fn from_ffi_ptr<'_a>(from: *const $ty) -> &'_a Self {
            std::mem::transmute(from)
        }

        #[allow(dead_code)]
        pub(crate) fn from_ffi_ref(from: &$ty) -> &Self {
            unsafe { std::mem::transmute(from) }
        }

        #[allow(dead_code)]
        pub(crate) unsafe fn from_ffi_mut<'_a>(from: *mut $ty) -> Pin<&'_a mut Self> {
            std::mem::transmute(from)
        }

        #[allow(dead_code)]
        pub(crate) fn as_ffi(&self) -> &$ty {
            unsafe { std::mem::transmute(self) }
        }

        #[allow(dead_code)]
        pub(crate) fn as_ffi_mut(self: Pin<&mut Self>) -> Pin<&mut $ty> {
            unsafe { std::mem::transmute(self) }
        }

        #[allow(dead_code)]
        pub(crate) fn as_ffi_mut_ptr(self: Pin<&mut Self>) -> *mut $ty {
            unsafe { std::mem::transmute(self) }
        }

        #[allow(dead_code)]
        pub(crate) unsafe fn as_ffi_mut_ptr_unpinned(&mut self) -> *mut $ty {
            std::mem::transmute(self)
        }
    };
}

pub(crate) use unsafe_ffi_conversions;

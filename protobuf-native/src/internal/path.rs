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

//! Support for passing paths to `libprotobuf`.
//!
//! On Unix, the bytes in a path can be passed directly.
//!
//! On Windows, the situation is complicated. Protobuf assumes paths are UTF-8
//! and converts them to wide-character strings before passing them to the
//! underlying Windows wide-char APIs. But paths in Rust might not valid UTF-8.
//! There's not much we can do to handle invalid UTF-8 correctly; we just throw
//! `to_string_lossy` at the problem and hope `libprotobuf` sorts it out.
//!
//! The point is to make this correct and performant on Unix in all cases, and
//! correct in Windows as long as the path is valid UTF-8.

#[cfg(windows)]
use std::marker::PhantomData;
#[cfg(unix)]
use std::os::unix::prelude::OsStrExt;
use std::path::Path;

#[cfg(unix)]
pub struct ProtobufPath<'a>(&'a Path);

#[cfg(windows)]
pub struct ProtobufPath<'a> {
    inner: String,
    _phantom: PhantomData<'a>,
}

#[cfg(unix)]
impl<'a> From<&'a Path> for ProtobufPath<'a> {
    fn from(p: &'a Path) -> ProtobufPath<'a> {
        ProtobufPath(p)
    }
}

#[cfg(unix)]
impl<'a> AsRef<[u8]> for ProtobufPath<'a> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_os_str().as_bytes()
    }
}

#[cfg(windows)]
impl<'a> From<Path> for ProtobufPath<'a> {
    fn from(p: Path) -> ProtobufPath<'a> {
        ProtobufPath {
            inner: p.to_string_lossy().into_owned(),
            _phantom: PhantomData,
        }
    }
}

#[cfg(windows)]
impl<'a> AsRef<[u8]> for ProtobufPath<'a> {
    fn as_ref(&self) -> &[u8] {
        self.inner.as_bytes()
    }
}

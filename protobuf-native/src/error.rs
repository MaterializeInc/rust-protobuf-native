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

//! Error types.

use std::error::Error;
use std::fmt;

#[cxx::bridge(namespace = "protobuf_native")]
pub(crate) mod ffi {

}

/// An error occurred while opening a file.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct FileOpenError(String);

impl FileOpenError {
    pub(crate) fn new(message: String) -> FileOpenError {
        FileOpenError(message)
    }
}

impl fmt::Display for FileOpenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for FileOpenError {}

/// A file was not loadable.
///
/// This error does not contain details about why the file was not loadable.
/// For details, seek an API that returns a [`FileLoadError`] instead.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct FileNotLoadableError;

impl fmt::Display for FileNotLoadableError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("file not loadable")
    }
}

impl Error for FileNotLoadableError {}

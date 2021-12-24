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

use std::error::Error;
use std::fmt;

/// A file was not found.
#[derive(Debug, Clone, Copy)]
pub struct FileNotFoundError;

impl fmt::Display for FileNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("file not found")
    }
}

impl Error for FileNotFoundError {}

/// A file was not loadable.
#[derive(Debug, Clone, Copy)]
pub struct FileNotLoadableError;

impl fmt::Display for FileNotLoadableError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("file not loadable")
    }
}

impl Error for FileNotLoadableError {}

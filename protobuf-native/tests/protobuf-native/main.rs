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
use std::path::Path;

use pretty_assertions::assert_eq;

use protobuf_native::compiler::{
    DiskSourceTree, FileLoadError, Location, Severity, SimpleErrorCollector, SourceTree,
    SourceTreeDescriptorDatabase, VirtualSourceTree,
};
use protobuf_native::{DescriptorDatabase, MessageLite, OperationFailedError};

mod io;
mod util;

/// Test that opening a nonexistent file fails with an appropriate error
/// message.
#[test]
fn test_open_nonexistent() {
    let mut source_tree = DiskSourceTree::new();
    let res = source_tree.as_mut().open(Path::new("noexist"));
    assert_eq!(util::unwrap_err(res).to_string(), "File not found.");
}

/// Test that opening a path with disallowed path characters fails with a
/// descriptive error message.
///
/// We don't particularly care about these disallowed path characters. Rather,
/// this test exists to ensure that the machinery that calls
/// `SourceTree::GetLastError` for descriptive error messages is wired up
/// properly, rather than always returning something bland like "file not
/// found."
#[test]
fn test_open_disallowed_path() {
    let mut source_tree = DiskSourceTree::new();
    let res = source_tree
        .as_mut()
        .open(Path::new("../protobuf/rejects/this"));
    assert_eq!(
        util::unwrap_err(res).to_string(),
        r#"Backslashes, consecutive slashes, ".", or ".." are not allowed in the virtual path"#
    );
}

/// Test the `fmt::Display` implementation of `FileLoadError`.
#[test]
fn test_file_load_error_display() {
    struct TestCase {
        error: FileLoadError,
        expected: &'static str,
    }

    for test_case in [
        TestCase {
            error: FileLoadError {
                filename: "test.proto".into(),
                location: Some(Location { line: 1, column: 1 }),
                severity: Severity::Error,
                message: "some error".into(),
            },
            expected: "test.proto:1:1: error: some error",
        },
        TestCase {
            error: FileLoadError {
                filename: "test.proto".into(),
                location: Some(Location { line: 1, column: 1 }),
                severity: Severity::Warning,
                message: "some warning".into(),
            },
            expected: "test.proto:1:1: warning: some warning",
        },
        TestCase {
            error: FileLoadError {
                filename: "test.proto".into(),
                location: None,
                severity: Severity::Error,
                message: "floating error".into(),
            },
            expected: "test.proto: error: floating error",
        },
    ] {
        assert_eq!(test_case.error.to_string(), test_case.expected);
    }
}

// Test that loading a file with parser errors produces descriptive error
// messages with the appropriate locations.
#[test]
fn test_load_parser_errors() {
    let mut source_tree = VirtualSourceTree::new();
    source_tree.as_mut().add_file(
        Path::new("test.proto"),
        br#"
syntax = "proto2";

message M {
    f = 1;
"#
        .to_vec(),
    );
    let mut error_collector = SimpleErrorCollector::new();
    let mut db = SourceTreeDescriptorDatabase::new(source_tree.as_mut());
    db.as_mut().record_errors_to(error_collector.as_mut());
    let res = db.as_mut().find_file_by_name(Path::new("test.proto"));
    assert_eq!(util::unwrap_err(res), OperationFailedError);
    drop(db);
    let errors: Vec<_> = error_collector.as_mut().collect();
    assert_eq!(
        errors,
        &[
            FileLoadError {
                filename: "test.proto".into(),
                message: "Reached end of input in message definition (missing '}').".into(),
                severity: Severity::Error,
                location: Some(Location { line: 6, column: 1 }),
            },
            FileLoadError {
                filename: "test.proto".into(),
                message: "Expected field name.".into(),
                severity: Severity::Error,
                location: Some(Location { line: 5, column: 7 }),
            },
            FileLoadError {
                filename: "test.proto".into(),
                message: r#"Expected "required", "optional", or "repeated"."#.into(),
                severity: Severity::Error,
                location: Some(Location { line: 5, column: 5 }),
            },
        ],
    )
}

// Test that loading a file that triggers parser warnings propagates those
// warnings with the appropriate locations.
#[test]
fn test_load_warning() {
    let mut source_tree = VirtualSourceTree::new();
    source_tree.as_mut().add_file(
        Path::new("test.proto"),
        br#"
syntax = "proto2";

message bad_to_the_bone {}
"#
        .to_vec(),
    );
    let mut error_collector = SimpleErrorCollector::new();
    let mut db = SourceTreeDescriptorDatabase::new(source_tree.as_mut());
    db.as_mut().record_errors_to(error_collector.as_mut());
    let res = db.as_mut().find_file_by_name(Path::new("test.proto"));
    assert!(res.is_ok());
    drop(db);
    let errors: Vec<_> = error_collector.as_mut().collect();
    assert_eq!(
        errors,
        &[FileLoadError {
            filename: "test.proto".into(),
            message: "Message name should be in UpperCamelCase. Found: bad_to_the_bone. \
                      See https://developers.google.com/protocol-buffers/docs/style"
                .into(),
            severity: Severity::Warning,
            location: Some(Location {
                line: 4,
                column: 25
            }),
        },],
    )
}

#[test]
fn test_file_descriptor_set() -> Result<(), Box<dyn Error>> {
    let mut source_tree = VirtualSourceTree::new();
    source_tree.as_mut().add_file(
        Path::new("imported.proto"),
        br#"
syntax = "proto3";

message ImportMe {
    int f = 1;
}
"#
        .to_vec(),
    );
    source_tree.as_mut().add_file(
        Path::new("root.proto"),
        br#"
syntax = "proto3";

import "imported.proto";

message Test {
    ImportMe im = 1;
}
"#
        .to_vec(),
    );
    let mut db = SourceTreeDescriptorDatabase::new(source_tree.as_mut());
    let fds = db
        .as_mut()
        .build_file_descriptor_set(&[Path::new("root.proto")])?;
    assert_eq!(fds.file_size(), 2);
    assert_eq!(fds.file(0).message_type_size(), 1);
    assert_eq!(fds.file(0).message_type(0).name(), b"Test");
    let mut out = vec![];
    fds.serialize_to_writer(&mut out)?;
    assert!(out.len() > 0);
    Ok(())
}

#[test]
fn test_well_known_types() -> Result<(), Box<dyn Error>> {
    let tempdir = tempfile::tempdir()?;
    std::fs::write(
        tempdir.path().join("test.proto"),
        br#"
syntax = "proto3";

import "google/protobuf/timestamp.proto";
import "google/protobuf/any.proto";
import "google/protobuf/duration.proto";

message Event {
    google.protobuf.Timestamp created_at = 1;
    google.protobuf.Duration ttl = 2;
    google.protobuf.Any payload = 3;
}
"#,
    )?;

    let mut source_tree = DiskSourceTree::new();
    source_tree.as_mut().map_well_known_types();
    source_tree.as_mut().map_path(Path::new(""), tempdir.path());

    let mut error_collector = SimpleErrorCollector::new();
    let mut db = SourceTreeDescriptorDatabase::new(source_tree.as_mut());
    db.as_mut().record_errors_to(error_collector.as_mut());

    let fds = db
        .as_mut()
        .build_file_descriptor_set(&[Path::new("test.proto")])?;

    assert_eq!(fds.file_size(), 4);

    let mut found_event_message = false;
    for i in 0..fds.file_size() {
        let file = fds.file(i);
        if file.message_type_size() > 0 && file.message_type(0).name() == b"Event" {
            found_event_message = true;
            break;
        }
    }
    assert!(found_event_message);

    drop(db);
    let errors: Vec<_> = error_collector.as_mut().collect();
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);

    Ok(())
}

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

use protobuf_native::compiler::importer::DiskSourceTree;
use protobuf_native::compiler::importer::SourceTreeDescriptorDatabase;
use protobuf_native::descriptor_database::DescriptorDatabase;

#[test]
fn test_open() {
    let mut source_tree = DiskSourceTree::new();
    source_tree.map_path(Path::new(""), Path::new("testdata"));
    let mut database = SourceTreeDescriptorDatabase::new(&mut source_tree);
    database.find_file_by_name(Path::new("test.proto")).unwrap();
}

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

use autocxx::moveit::Emplace;

use protobuf_sys::google::protobuf::util::TimeUtil;

// Currently segfaults.
#[test]
fn test_linkage() {
    // Simple test that calls a function to verify that linking has occurred.
    let time = Box::emplace(TimeUtil::SecondsToTimestamp(42));
    let s = TimeUtil::ToString(&time);
    assert_eq!(s.to_str().unwrap(), "1970-01-01T00:00:42Z");
}

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

#include "protobuf-native/src/compiler.h"

#include "protobuf-native/src/compiler.rs.h"
#include "protobuf-native/src/internal.rs.h"

namespace protobuf_native {
namespace compiler {

using namespace google::protobuf::compiler;

void SimpleErrorCollector::AddError(const std::string& filename, int line, int column,
                                    const std::string& message) {
    AddErrorOrWarning(filename, line, column, message, false);
}

void SimpleErrorCollector::AddWarning(const std::string& filename, int line, int column,
                                      const std::string& message) {
    AddErrorOrWarning(filename, line, column, message, true);
}

void SimpleErrorCollector::AddErrorOrWarning(const std::string& filename, int line, int column,
                                             const std::string& message, bool warning) {
    errors_.push_back(FileLoadError{.filename = filename,
                                    .line = line,
                                    .column = column,
                                    .message = message,
                                    .warning = warning});
}

std::vector<FileLoadError>& SimpleErrorCollector::Errors() { return errors_; }

SimpleErrorCollector* NewSimpleErrorCollector() { return new SimpleErrorCollector(); }

void DeleteSimpleErrorCollector(SimpleErrorCollector* collector) { delete collector; }

rust::String SourceTreeGetLastErrorMessage(SourceTree& source_tree) {
    return rust::String::lossy(source_tree.GetLastErrorMessage());
}

VirtualSourceTree* NewVirtualSourceTree() { return new VirtualSourceTree(); }

void DeleteVirtualSourceTree(VirtualSourceTree* tree) { delete tree; }

void VirtualSourceTree::AddFile(const std::string& name, rust::Vec<rust::u8> contents) {
    files_[name] = contents;
}

io::ZeroCopyInputStream* VirtualSourceTree::Open(const std::string& filename) {
    auto entry = files_.find(filename);
    if (entry == files_.end()) {
        return nullptr;
    }
    auto& file = entry->second;
    return new io::ArrayInputStream(file.data(), file.size());
}

std::string VirtualSourceTree::GetLastErrorMessage() { return "File not found."; }

DiskSourceTree* NewDiskSourceTree() { return new DiskSourceTree(); }
void DeleteDiskSourceTree(DiskSourceTree* tree) { delete tree; }

SourceTreeDescriptorDatabase* NewSourceTreeDescriptorDatabase(SourceTree* source_tree) {
    return new SourceTreeDescriptorDatabase(source_tree);
}

void DeleteSourceTreeDescriptorDatabase(SourceTreeDescriptorDatabase* source_tree) {
    delete source_tree;
}

}  // namespace compiler
}  // namespace protobuf_native

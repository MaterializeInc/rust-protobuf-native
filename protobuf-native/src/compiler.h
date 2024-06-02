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

#pragma once

#include "google/protobuf/compiler/importer.h"

#include "rust/cxx.h"

namespace protobuf_native {
namespace compiler {

using namespace google::protobuf;
using namespace google::protobuf::compiler;

struct FileLoadError;

class SimpleErrorCollector : public MultiFileErrorCollector {
   public:
    void RecordError(absl::string_view filename, int line, int column,
                     absl::string_view message) override;
    void RecordWarning(absl::string_view filename, int line, int column,
                       absl::string_view message) override;
    std::vector<FileLoadError>& Errors();

   private:
    void RecordErrorOrWarning(absl::string_view filename, int line, int column,
                              absl::string_view message, bool warning);
    std::vector<FileLoadError> errors_;
};

SimpleErrorCollector* NewSimpleErrorCollector();
void DeleteSimpleErrorCollector(SimpleErrorCollector*);

rust::String SourceTreeGetLastErrorMessage(SourceTree&);

class VirtualSourceTree : public SourceTree {
   public:
    void AddFile(absl::string_view name, rust::Vec<rust::u8> contents);
    io::ZeroCopyInputStream* Open(absl::string_view filename);
    std::string GetLastErrorMessage();

   private:
    absl::flat_hash_map<std::string, rust::Vec<rust::u8>> files_;
};

VirtualSourceTree* NewVirtualSourceTree();

void DeleteVirtualSourceTree(VirtualSourceTree*);

DiskSourceTree* NewDiskSourceTree();

void DeleteDiskSourceTree(DiskSourceTree*);

SourceTreeDescriptorDatabase* NewSourceTreeDescriptorDatabase(SourceTree* source_tree);

void DeleteSourceTreeDescriptorDatabase(SourceTreeDescriptorDatabase* source_tree);

}  // namespace compiler
}  // namespace protobuf_native

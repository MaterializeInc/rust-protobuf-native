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

#include <google/protobuf/compiler/importer.h>

#include "rust/cxx.h"

namespace protobuf_native {
namespace compiler {

using namespace google::protobuf;
using namespace google::protobuf::compiler;

struct FileLoadError;

class SimpleErrorCollector : public MultiFileErrorCollector {
   public:
    void AddError(const std::string& filename, int line, int column,
                  const std::string& message) override;
    void AddWarning(const std::string& filename, int line, int column,
                    const std::string& message) override;
    std::vector<FileLoadError>& Errors();

   private:
    void AddErrorOrWarning(const std::string& filename, int line, int column,
                           const std::string& message, bool warning);
    std::vector<FileLoadError> errors_;
};

SimpleErrorCollector* NewSimpleErrorCollector();
void DeleteSimpleErrorCollector(SimpleErrorCollector*);

rust::String SourceTreeGetLastErrorMessage(SourceTree&);

class VirtualSourceTree : public SourceTree {
   public:
    void AddFile(const std::string& name, rust::Vec<rust::u8> contents);
    io::ZeroCopyInputStream* Open(const std::string& filename);
    std::string GetLastErrorMessage();

   private:
    std::unordered_map<std::string, rust::Vec<rust::u8>> files_;
};

VirtualSourceTree* NewVirtualSourceTree();

void DeleteVirtualSourceTree(VirtualSourceTree*);

DiskSourceTree* NewDiskSourceTree();

void DeleteDiskSourceTree(DiskSourceTree*);

SourceTreeDescriptorDatabase* NewSourceTreeDescriptorDatabase(SourceTree* source_tree);

void DeleteSourceTreeDescriptorDatabase(SourceTreeDescriptorDatabase* source_tree);

}  // namespace compiler
}  // namespace protobuf_native

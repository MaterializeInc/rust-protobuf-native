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

#include <memory>

#include "google/protobuf/descriptor.h"
#include "google/protobuf/descriptor.pb.h"

using namespace google::protobuf;

namespace protobuf_native {

MessageLite* NewMessageLite(const MessageLite& message);
void DeleteMessageLite(MessageLite*);

DescriptorPool* NewDescriptorPool();
void DeleteDescriptorPool(DescriptorPool*);

FileDescriptorSet* NewFileDescriptorSet();
void DeleteFileDescriptorSet(FileDescriptorSet* set);

FileDescriptorProto* NewFileDescriptorProto();
void DeleteFileDescriptorProto(FileDescriptorProto*);

DescriptorProto* NewDescriptorProto();
void DeleteDescriptorProto(DescriptorProto* proto);

void DeleteFileDescriptor(FileDescriptor*);

}  // namespace protobuf_native

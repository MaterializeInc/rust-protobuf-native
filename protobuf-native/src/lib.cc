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

#include "protobuf-native/src/lib.h"

using namespace google::protobuf;

namespace protobuf_native {

// Disable libprotobuf's logging to stderr. Libraries should not log to
// stderr.
static LogHandler* default_log_handler = SetLogHandler(nullptr);

MessageLite* NewMessageLite(const MessageLite& message) { return message.New(); }

void DeleteMessageLite(MessageLite* message) { delete message; }

DescriptorPool* NewDescriptorPool() { return new DescriptorPool(); }

void DeleteDescriptorPool(DescriptorPool* pool) { delete pool; }

FileDescriptorSet* NewFileDescriptorSet() { return new FileDescriptorSet(); }

void DeleteFileDescriptorSet(FileDescriptorSet* set) { delete set; }

FileDescriptorProto* NewFileDescriptorProto() { return new FileDescriptorProto(); }

void DeleteFileDescriptorProto(FileDescriptorProto* proto) { delete proto; }

DescriptorProto* NewDescriptorProto() { return new DescriptorProto(); }

void DeleteDescriptorProto(DescriptorProto* proto) { delete proto; }

void DeleteFileDescriptor(FileDescriptor* descriptor) { delete descriptor; }

}  // namespace protobuf_native

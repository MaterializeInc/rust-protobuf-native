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

#include <google/protobuf/io/coded_stream.h>
#include <google/protobuf/io/zero_copy_stream.h>
#include <google/protobuf/io/zero_copy_stream_impl.h>
#include <google/protobuf/io/zero_copy_stream_impl_lite.h>

#include <memory>

#include "rust/cxx.h"

namespace protobuf_native {
namespace io {

using namespace google::protobuf::io;

struct ReadAdaptor;
struct WriteAdaptor;

void DeleteZeroCopyInputStream(ZeroCopyInputStream*);

class ReaderStream : public CopyingInputStreamAdaptor {
   public:
    ReaderStream(rust::Box<ReadAdaptor> adaptor);

   private:
    class CopyingReaderStream : public CopyingInputStream {
       public:
        CopyingReaderStream(rust::Box<ReadAdaptor> adaptor);

        int Read(void* buffer, int size) override;

       private:
        rust::Box<ReadAdaptor> adaptor_;
    };
};

ReaderStream* NewReaderStream(rust::Box<ReadAdaptor> adaptor);
void DeleteReaderStream(ReaderStream*);

ArrayInputStream* NewArrayInputStream(const uint8_t* data, int size);
void DeleteArrayInputStream(ArrayInputStream*);

void DeleteZeroCopyOutputStream(ZeroCopyOutputStream*);

class WriterStream : public CopyingOutputStreamAdaptor {
   public:
    WriterStream(rust::Box<WriteAdaptor> adaptor);

   private:
    class CopyingWriterStream : public CopyingOutputStream {
       public:
        CopyingWriterStream(rust::Box<WriteAdaptor> adaptor);

        bool Write(const void* buffer, int size) override;

       private:
        rust::Box<WriteAdaptor> adaptor_;
    };
};

WriterStream* NewWriterStream(rust::Box<WriteAdaptor> adaptor);
void DeleteWriterStream(WriterStream*);

ArrayOutputStream* NewArrayOutputStream(uint8_t* data, int size);
void DeleteArrayOutputStream(ArrayOutputStream*);

class VecOutputStream : public ZeroCopyOutputStream {
   public:
    VecOutputStream(rust::Vec<uint8_t>& target);
    ~VecOutputStream();

    bool Next(void** data, int* size) override;
    void BackUp(int count) override;
    int64_t ByteCount() const override;

   private:
    const size_t kMinimumSize = 16;

    rust::Vec<uint8_t>& target_;
    size_t start_position_;
    size_t position_;
};

VecOutputStream* NewVecOutputStream(rust::Vec<uint8_t>& target);
void DeleteVecOutputStream(VecOutputStream*);

CodedInputStream* NewCodedInputStream(ZeroCopyInputStream* input);
void DeleteCodedInputStream(CodedInputStream*);

void DeleteCodedOutputStream(CodedOutputStream*);

}  // namespace io
}  // namespace protobuf_native

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

#include "protobuf-native/src/io.h"

#include "protobuf-native/src/internal.rs.h"
#include "protobuf-native/src/io.rs.h"

namespace protobuf_native {
namespace io {

using namespace google::protobuf::io;

void DeleteZeroCopyInputStream(ZeroCopyInputStream* stream) { delete stream; }

ReaderStream::ReaderStream(rust::Box<ReadAdaptor> adaptor)
    : CopyingInputStreamAdaptor(new CopyingReaderStream(std::move(adaptor))) {
    SetOwnsCopyingStream(true);
}

ReaderStream::CopyingReaderStream::CopyingReaderStream(rust::Box<ReadAdaptor> adaptor)
    : adaptor_(std::move(adaptor)) {}

int ReaderStream::CopyingReaderStream::Read(void* buffer, int size) {
    return adaptor_->read(rust::Slice<uint8_t>(static_cast<uint8_t*>(buffer), size));
}

ReaderStream* NewReaderStream(rust::Box<ReadAdaptor> adaptor) {
    return new ReaderStream(std::move(adaptor));
}

void DeleteReaderStream(ReaderStream* stream) { delete stream; }

ArrayInputStream* NewArrayInputStream(const uint8_t* data, int size) {
    return new ArrayInputStream(data, size);
}

void DeleteArrayInputStream(ArrayInputStream* stream) { delete stream; }

WriterStream::WriterStream(rust::Box<WriteAdaptor> adaptor)
    : CopyingOutputStreamAdaptor(new CopyingWriterStream(std::move(adaptor))) {
    SetOwnsCopyingStream(true);
}

WriterStream::CopyingWriterStream::CopyingWriterStream(rust::Box<WriteAdaptor> adaptor)
    : adaptor_(std::move(adaptor)) {}

bool WriterStream::CopyingWriterStream::Write(const void* buffer, int size) {
    return adaptor_->write(rust::Slice<const uint8_t>(static_cast<const uint8_t*>(buffer), size));
}

WriterStream* NewWriterStream(rust::Box<WriteAdaptor> adaptor) {
    return new WriterStream(std::move(adaptor));
}

void DeleteWriterStream(WriterStream* stream) { delete stream; }

ArrayOutputStream* NewArrayOutputStream(uint8_t* data, int size) {
    return new ArrayOutputStream(data, size);
}

void DeleteArrayOutputStream(ArrayOutputStream* stream) { delete stream; }

VecOutputStream::VecOutputStream(rust::Vec<uint8_t>& target)
    : target_(target), start_position_(target.size()), position_(target.size()) {}

VecOutputStream::~VecOutputStream() { vec_u8_set_len(target_, position_); }

bool VecOutputStream::Next(void** data, int* size) {
    if (position_ == target_.capacity()) {
        target_.reserve(std::max(position_ * 2, kMinimumSize));
    }
    *data = target_.data() + position_;
    *size = target_.capacity() - position_;
    position_ = target_.capacity();
    return true;
}

void VecOutputStream::BackUp(int count) {
    GOOGLE_CHECK_GE(count, 0);
    GOOGLE_CHECK_LE(static_cast<int64_t>(count), ByteCount());
    position_ -= count;
}

int64_t VecOutputStream::ByteCount() const { return position_ - start_position_; }

VecOutputStream* NewVecOutputStream(rust::Vec<uint8_t>& target) {
    return new VecOutputStream(target);
}

void DeleteVecOutputStream(VecOutputStream* stream) { delete stream; }

CodedInputStream* NewCodedInputStream(ZeroCopyInputStream* input) {
    return new CodedInputStream(input);
}

void DeleteCodedInputStream(CodedInputStream* stream) { delete stream; }

}  // namespace io
}  // namespace protobuf_native

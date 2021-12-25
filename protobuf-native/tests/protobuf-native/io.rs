// Copyright Materialize, Inc. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not
// use this file except in compliance with the License. You may obtain a copy of
// the License in the LICENSE file at the root of this repository, or online at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations under
// the License.
//
// Portions of this file are derived from the zero copy stream unit tests in the
// Protocol Buffers project. The original source code was retrieved on December
// 26, 2021 from:
//
//     https://github.com/protocolbuffers/protobuf/blob/01e84b129361913e5613464c857734fcfe095367/src/google/protobuf/io/zero_copy_stream_unittest.cc
//
// The original source code is subject to the terms of the MIT license, a copy
// of which can be found in the LICENSE file at the root of this repository.

//! Tests for the `protobuf_native::io` module.
//!
//! Testing strategy: for each type of I/O (array, string, file, etc.) we create
//! an output stream and write some data to it, then create a corresponding
//! input stream to read the same data back and expect it to match. When the
//! data is written, it is written in several small chunks of varying sizes,
//! with a `back_up` after each chunk. It is read back similarly, but with
//! chunks separated at different points. The whole process is run with a
//! variety of block sizes for both the input and the output.

use std::io::{Seek, SeekFrom};
use std::pin::Pin;

use protobuf_native::io::{
    ReaderStream, SliceInputStream, SliceOutputStream, VecOutputStream, WriterStream,
    ZeroCopyInputStream, ZeroCopyOutputStream,
};

use crate::util;

fn write_bytes(mut output: Pin<&mut dyn ZeroCopyOutputStream>, mut bytes: &[u8]) {
    loop {
        // SAFETY: we either fill `buf` in its entirely, or call `back_up` to
        // indicate the unfilled portion, before returning or calling `next`
        // again.
        let buf = unsafe { output.as_mut().next() }.unwrap();
        if bytes.len() < buf.len() {
            util::copy_to_uninit_slice(&mut buf[..bytes.len()], bytes);
            let extra = buf.len() - bytes.len();
            output.back_up(extra);
            return;
        }
        util::copy_to_uninit_slice(buf, &bytes[..buf.len()]);
        bytes = &bytes[buf.len()..];
    }
}

fn read_bytes(mut input: Pin<&mut dyn ZeroCopyInputStream>, mut out: &mut [u8]) {
    loop {
        let buf = input.as_mut().next().unwrap();
        if buf.len() < out.len() {
            out[..buf.len()].copy_from_slice(buf);
            out = &mut out[buf.len()..];
        } else {
            out.copy_from_slice(&buf[..out.len()]);
            let extra = buf.len() - out.len();
            input.back_up(extra);
            return;
        }
    }
}

fn check_some_writes(mut output: Pin<&mut dyn ZeroCopyOutputStream>) {
    write_bytes(output.as_mut(), b"Hello world!\n");
    write_bytes(output.as_mut(), b"Some te");
    write_bytes(output.as_mut(), b"xt.  Blah blah.");
    write_bytes(output.as_mut(), &[b'x'; 100_000]);
    write_bytes(output.as_mut(), &[b'y'; 100_000]);
    write_bytes(output.as_mut(), b"01234567890123456789");
    assert_eq!(output.byte_count(), 200_055);
}

fn check_read(input: Pin<&mut dyn ZeroCopyInputStream>, expected: &[u8]) {
    let mut out = vec![0; expected.len()];
    read_bytes(input, &mut out);
    assert_eq!(out, expected);
}

fn check_some_reads(mut input: Pin<&mut dyn ZeroCopyInputStream>) {
    check_read(input.as_mut(), b"Hello world!\nSome text.  ");
    input.as_mut().skip(5).unwrap();
    check_read(input.as_mut(), b"blah.");
    input.as_mut().skip(100000 - 10).unwrap();
    check_read(input.as_mut(), &{
        let mut buf = vec![];
        buf.extend([b'x'; 10]);
        buf.extend([b'y'; 100000 - 20000]);
        buf
    });
    input.as_mut().skip(20000 - 10).unwrap();
    check_read(input.as_mut(), b"yyyyyyyyyy01234567890123456789");
    assert_eq!(input.byte_count(), 200_055);
}

#[test]
fn test_io_slice() {
    let mut buffer = vec![0; 1 << 18];
    check_some_writes(SliceOutputStream::new(&mut buffer).as_mut());
    check_some_reads(SliceInputStream::new(&buffer).as_mut());
}

#[test]
fn test_io_vec() {
    let mut buffer = vec![];
    check_some_writes(VecOutputStream::new(&mut buffer).as_mut());
    let mut input = SliceInputStream::new(&buffer);
    check_some_reads(input.as_mut());
    assert!(input.as_mut().next().is_err()); // check for EOF
}

#[test]
fn test_io_file() {
    let mut file = tempfile::tempfile().unwrap();
    check_some_writes(WriterStream::new(&mut file).as_mut());
    file.seek(SeekFrom::Start(0)).unwrap();
    check_some_reads(ReaderStream::new(&mut file).as_mut());
}

# rust-protobuf-native

This is a collection of crates that provide Rust interop for [Protocol
Buffers](https://github.com/google/protobuf), Google's data interchange format.
The following crates are available:

* [**protobuf-native**](./protobuf-native) is a high-level, safe API to
  `libprotobuf`.
* [**protobuf-sys**](./protobuf-sys) provides automatically-generated Rust
  bindings to `libprotobuf` via [autocxx].
* [**protobuf-src**](./protobuf-src) builds the `libprotobuf` library and
  `protoc` binary from the C++ source code checked out at build time from
  the GitHub repository.

## Related projects

There are two other major Protobuf projects in the Rust ecosystem:

  * [rust-protobuf] contains a `protoc` plugin for generating Rust code,
    an (incomplete) pure-Rust reimplementation of `libprotobuf`, including
    (incomplete) support for dynamic messages, and a Rust API for compiling
    protobufs.

  * [prost] contains a Rust API for compiling protobufs that uses an alternative
    code generation backend that purports to generate more idiomatic Rust.

This project is meant to supplement these existing tools, not supplant them.
The hope is that prost and rust-protobuf will support optional integration
with these crates for users who want to avoid the system `protoc`.

[autocxx]: https://github.com/google/autocxx
[rust-protobuf]: https://github.com/stepancheg/rust-protobuf
[prost]: https://github.com/tokio-rs/prost

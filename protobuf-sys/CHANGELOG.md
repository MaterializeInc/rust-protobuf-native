# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog], and this project adheres to [Semantic
Versioning].

<!-- #release:next-header -->

## [Unreleased] <!-- #release:date -->

## [0.1.2+3.19.1] - 2021-12-24

* Generate bindings for the following additional types:

  * `google::protobuf::util::JsonParseOptions`
  * `google::protobuf::util::JsonPrintOptions`
  * `google::protobuf::util::TimeUtil`

* Emit the Cargo directives to request linking with the vendored copy of
  `libprotobuf`.

## [0.1.1+3.19.1] - 2021-12-23

* Generate bindings for the `google::protobuf::io::CodedInputStream` and
  `google::protobuf::io::ZeroCopyInputStream` types.

## 0.1.0+3.19.1 - 2021-12-23

Initial release.

<!-- #release:next-url -->
[Unreleased]: https://github.com/MaterializeInc/rust-protobuf-native/compare/protobuf-sys-v0.1.2+3.19.1...HEAD
[0.1.2+3.19.1]: https://github.com/MaterializeInc/rust-protobuf-native/compare/protobuf-sys-v0.1.1+3.19.1...protobuf-sys-v0.1.2+3.19.1
[0.1.1+3.19.1]: https://github.com/MaterializeInc/rust-protobuf-native/compare/protobuf-sys-v0.1.0+3.19.1...protobuf-sys-v0.1.1+3.19.1

[Keep a Changelog]: https://keepachangelog.com/en/1.0.0/
[Semantic Versioning]: https://semver.org/spec/v2.0.0.html

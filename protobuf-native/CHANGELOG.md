# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog], and this project adheres to [Semantic
Versioning].

<!-- #release:next-header -->

## [Unreleased] <!-- #release:date -->

* Fix the file descriptor traversal in
 `SourceTreeDescriptorDatabase::build_file_descriptor_set` to avoid duplicating
 already-visited file descriptors.

## [0.2.0+3.19.1] - 2022-01-18

* Add initial bindings. The bindings in the `protobuf::io` and
  `protobuf::compiler` modules are now largely complete, while the bindings in
  the top-level `protobuf` module are very sparse. The `protobuf::util` module
  is entirely absent.

## 0.1.0+3.19.1 - 2021-12-23

Initial release.

<!-- #release:next-url -->
[Unreleased]: https://github.com/MaterializeInc/rust-protobuf-native/compare/protobuf-native-v0.2.0+3.19.1...HEAD
[0.2.0+3.19.1]: https://github.com/MaterializeInc/rust-protobuf-native/compare/protobuf-native-v0.1.0+3.19.1...protobuf-native-v0.2.0+3.19.1

[Keep a Changelog]: https://keepachangelog.com/en/1.0.0/
[Semantic Versioning]: https://semver.org/spec/v2.0.0.html

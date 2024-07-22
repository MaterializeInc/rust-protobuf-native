# Maintainer instructions

## Upgrading the vendored copies of libprotobuf and Abseil

Run `bin/update-protobuf $VERSION` .

If you need to upgrade Abseil, you'll need to edit the hardcoded version in
`bin/update-protobuf` directly.

## Cutting new releases

1. `cargo install cargo-release`, if you don't have it already.

2. Update CHANGELOG.md for the desired crate.

3. Run `cargo release -p CRATE VERSION` and verify the dry run.

   **Important:** do *not* use the auto-version bumping functionality, as in
   `cargo release -p CRATE patch`, as it will lose the build metadata.

4. Run `cargo release -p CREATE VERSION -x` to actually execute the release.

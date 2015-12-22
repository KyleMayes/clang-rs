clang-rs
========

[![crates.io](https://img.shields.io/crates/v/clang.svg)](https://crates.io/crates/clang)

Rust bindings and idiomatic wrapper for libclang.

Supported on the stable, beta, and nightly Rust channels.

**Warning:** the API of this library is subject to change.

Released under the MIT license.

### Dependencies

This crate depends on `libclang.dll` (Windows), `libclang.so` (Linux), or `libclang.dylib` (OS X).
These binaries can be downloaded [here](http://llvm.org/releases/download.html). Place the
appropriate binary on your system's path so that `rustc` can find `libclang`.

### Supported Versions

* 3.5.x - [Documentation](https://kylemayes.github.io/clang-rs/3_5/clang)
* 3.6.x - [Documentation](https://kylemayes.github.io/clang-rs/3_6/clang)
* 3.7.x - [Documentation](https://kylemayes.github.io/clang-rs/3_7/clang)

If you do not select a specific version, a common subset API will be availabile. The documentation
for this API is [here](https://kylemayes.github.io/clang-rs/all/clang).

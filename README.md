clang-rs
========

[![crates.io](https://img.shields.io/crates/v/clang.svg)](https://crates.io/crates/clang)
[![Travis CI](https://travis-ci.org/KyleMayes/clang-rs.svg?branch=master)](https://travis-ci.org/KyleMayes/clang-rs)

A somewhat idiomatic Rust wrapper for `libclang`.

Supported on the stable, beta, and nightly Rust channels.

Released under the Apache License 2.0.

### Supported Versions

To target a version of `libclang`, enable one of the following Cargo features:

* `clang_3_5` - requires `libclang` 3.5 or later
  ([Documentation](https://kylemayes.github.io/clang-rs/3_5/clang))
* `clang_3_6` - requires `libclang` 3.6 or later
  ([Documentation](https://kylemayes.github.io/clang-rs/3_6/clang))
* `clang_3_7` - requires `libclang` 3.7 or later
  ([Documentation](https://kylemayes.github.io/clang-rs/3_7/clang))
* `clang_3_8` - requires `libclang` 3.8 or later
  ([Documentation](https://kylemayes.github.io/clang-rs/3_8/clang))

If you do not enable one of these features, the API provided by `libclang` 3.5 will be available by
default.

### Dependencies

See [here](https://github.com/KyleMayes/clang-sys#dependencies) for information on this crate's
dependencies.

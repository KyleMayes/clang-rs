# clang-rs

[![Crate](https://img.shields.io/crates/v/clang.svg)](https://crates.io/crates/clang)
[![Documentation](https://docs.rs/clang/badge.svg)](https://docs.rs/clang)
[![CI](https://github.com/KyleMayes/clang-rs/workflows/CI/badge.svg?branch=master)](https://github.com/KyleMayes/clang-rs/actions?query=workflow%3ACI)

A somewhat idiomatic Rust wrapper for `libclang`.

Supported on the stable, beta, and nightly Rust channels.<br/>
Minimum supported Rust version: **1.40.0**

Released under the Apache License 2.0.

## Supported Versions

To target a version of `libclang`, enable one of the following Cargo features:

* `clang_3_5` - requires `libclang` 3.5 or later
* `clang_3_6` - requires `libclang` 3.6 or later
* `clang_3_7` - requires `libclang` 3.7 or later
* `clang_3_8` - requires `libclang` 3.8 or later
* `clang_3_9` - requires `libclang` 3.9 or later
* `clang_4_0` - requires `libclang` 4.0 or later
* `clang_5_0` - requires `libclang` 5.0 or later
* `clang_6_0` - requires `libclang` 6.0 or later
* `clang_7_0` - requires `libclang` 7.0 or later
* `clang_8_0` - requires `libclang` 8.0 or later
* `clang_9_0` - requires `libclang` 9.0 or later
* `clang_10_0` - requires `libclang` 10.0 or later

If you do not enable one of these features, the API provided by `libclang` 3.5 will be available by
default.

## Dependencies

See [here](https://github.com/KyleMayes/clang-sys#dependencies) for information on this crate's
dependencies.

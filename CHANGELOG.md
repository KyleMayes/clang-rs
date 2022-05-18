## [2.0.0] - 2022-05-18

### Changed
- Made `Clang` struct impossible to construct directly
- Disabled `Send` and `Sync` for `Clang` struct
- Make `y` and `z` components of `Version` struct optional

### Fixed
- Fix null check for `Version` struct to only check sign of major version

## [1.0.3] - 2020-11-29

### Fixed
- Fixed token annotation (`TranslationUnit::annotate`) not returning correct results

## [1.0.2] - 2020-10-01

### Fixed
- Fixed stack overflow caused by displaying errors

## [1.0.1] - 2020-08-16

### Fixed
- Fixed double free caused by `Entity::get_platform_availability`

## [1.0.0] - 2020-08-08

### Removed
- Removed `gte_clang_*` Cargo features (these were an implementation detail)

### Added
- Added support for `clang` 10.0.x

### Changed
- Bumped `clang-sys` version to `1`

## [0.24.0] - 2020-07-01

### Fixed
- Fixed undefined behavior caused by transmuting unsupported C enum values into Rust enum values
- Fixed warnings about usage of deprecated `std::mem::uninitialized` function

### Added
- Added support for `clang` 9.0.x

### Changed
- Added error conditions to `CursorKind` enum

## [0.23.0] - 2019-09-07

### Fixed
- Fixed warnings on recent versions of the Rust compiler

### Added
- Added support for compilation databases

## [0.22.1] - 2019-07-15

### Fixed
- Fixed `Entity::is_function_like_macro` calling the wrong `libclang` function

## [0.22.0] - 2019-04-20

### Added
- Added partial support for `clang` 7.0.x / 8.0.x

### Changed
- Bumped `clang-sys` version to `0.28.0`

## [0.21.0] - 2019-02-17

### Changed
- Bumped `clang-sys` version to `0.26.4`

## [0.20.0] - 2018-06-16

### Changed
- Bumped `clang-sys` version to `0.23.0`

## [0.19.0] - 2018-03-11

### Added
- Added support for `clang` 6.0.x

### Changed
- Bumped `lazy_static` version to `1.0.0`
- Bumped `libc` version to `0.2.39`

## [0.18.0] - 2018-02-17

### Added
- Added `Entity::get_result_type` method

### Changed
- Bumped `clang-sys` version to `0.21.1`

## [0.17.0] - 2017-09-16

### Added
- Added support for `clang` 5.0.x

### Changed
- Bumped `clang-sys` version to `0.20.1`

## [0.16.0] - 2017-03-14

### Added
- Added support for `clang` 4.0.x

### Changed
- Renamed `EvaluationResult::Integer` variant to `SignedInteger`

## [0.15.0] - 2017-02-16

### Added
- Added assertions that pointers returned from `libclang` functions are non-null

### Changed
- Bumped `clang-sys` version to `0.14.0`
- Changed the type of the `file` field in the `Location` struct from `File` to `Option<File>`

## [0.14.1] - 2017-01-29

### Changed
- Bumped `clang-sys` version to `0.13.0`

## [0.14.0] - 2016-11-01

### Added
- Added children to comment parameters

## [0.13.0] - 2016-10-07

### Changed
- Removed feature gates for `CallingConvention`, `EntityKind`, and `TypeKind` variants
- Bumped `clang-sys` version to `0.11.0`
- Bumped `libc` version to `0.2.16`

## [0.12.0] - 2016-8-1

### Changed
- Added `runtime` Cargo feature that links to `libclang` shared library at runtime

## [0.11.0] - 2016-7-22

### Added
- Added `documentation` module

### Changed
- Bumped `clang-sys` version to `0.9.0`

## [0.10.0] - 2016-7-19

### Added
- Added support for `clang` 3.9.x
- Added `Entity::get_child` method

### Changed
- Bumped `clang-sys` version to `0.8.0`
- Bumped `libc` version to `0.2.14`

## [0.9.5] - 2016-7-14

### Fixed
- Fixed usage of `clang` 3.9.x binaries

## [0.9.4] - 2016-6-17

### Added
- Added implementation of `std::cmp::PartialOrd` for `Diagnostic` struct

### Changed
- Bumped `clang-sys` version to `0.7.2`

## [0.9.3] - 2016-5-26

### Changed
- Bumped `clang-sys` version to `0.6.0`

## [0.9.2] - 2016-5-19

### Changed
- Bumped `clang-sys` version to `0.5.4`

## [0.9.1] - 2016-5-17

### Changed
- Bumped `clang-sys` version to `0.5.3`

## [0.9.0] - 2016-5-14

### Added
- Added location functions to `Entity` struct

### Changed
- Changed `sonar` module interface to use iterators

## [0.8.2] - 2016-5-13

### Changed
- Bumped `clang-sys` version to `0.5.2`

## [0.8.1] - 2016-5-11

### Changed
- Bumped `clang-sys` version to `0.5.1`

## [0.8.0] - 2016-5-10

### Changed
- Bumped `clang-sys` version to `0.5.0`
- Bumped `lazy_static` version to `0.2.1`
- Bumped `libc` version to `0.2.11`

## [0.7.3] - 2016-4-21

### Fixed
- Fixed `sonar` module handling of record typedefs

### Changed
- Bumped `clang-sys` version to `0.4.2`
- Bumped `lazy_static` version to `0.2.0`
- Bumped `libc` version to `0.2.10`

## [0.7.2] - 2016-4-5

### Fixed
- Removed `println!` in `sonar` module

### Changed
- Changed `Parser::arguments` parameter type

## [0.7.1] - 2016-4-4

### Fixed
- Fixed panic in `sonar` module when encountering certain kinds of typedefs

## [0.7.0] - 2016-4-3

### Changed
- Major refactoring
- Bumped `clang-sys` version to `0.4.1`
- Bumped `libc` version to `0.2.9`

## [0.6.0] - 2016-4-2

### Added
- Added preprocessor definition finding to `sonar` module

### Changed
- Changed `sonar` interface

## [0.5.4] - 2016-3-28

### Added
- Added `static` feature

### Changed
- Bumped `clang-sys` version to `0.4.0`

## [0.5.3] - 2016-3-21

### Changed
- Bumped `clang-sys` version to `0.3.1`

## [0.5.2] - 2016-3-16

### Changed
- Bumped `clang-sys` version to `0.3.0`
- Bumped `libc` version to `0.2.8`

## [0.5.1] - 2016-3-14

### Added
- Added implementations of `std::error::Error` for error enums

## [0.5.0] - 2016-3-9

### Added
- Added a `sonar` module for finding C declarations

## [0.4.0] - 2016-2-13

### Added
- Added support for `clang` 3.8.x

### Fixed
- Added missing `cfg`s on enum variants

### Changed
- Simplified internal usage of conditional compilation
- Bumped `clang-sys` version to `0.2.0`
- Bumped `libc` version to `0.2.7`

## [0.3.1] - 2016-2-5

### Changed
- Bumped `clang-sys` version to `0.1.2`

## [0.3.0] - 2015-12-27

### Removed
- Removed `sonar` module

### Added
- Added implementations of `From` to `String` for error enums
- Added integer categorization methods to `Type` struct

### Changed
- Bumped `clang-sys` version to `0.1.1`

## [0.2.2] - 2015-12-23

### Added
- Added `sonar` module for finding declarations in C translation units

## [0.2.1] - 2015-12-22

### Fixed
- Fixed passing of version features to `clang-sys`

## [0.2.0] - 2015-12-22

### Changed
- Moved FFI bindings into separate crate (`clang-sys`)
- Bumped `libc` version to `0.2.4`

## [0.1.0] - 2015-12-21
- Initial release

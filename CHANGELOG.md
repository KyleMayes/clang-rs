## [0.9.3] - UNRELEASED

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

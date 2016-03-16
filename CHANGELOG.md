## [0.1.0] - 2015-12-21
- Initial release

## [0.2.0] - 2015-12-22

### Changed
- Moved FFI bindings into separate crate (`clang-sys`)

## [0.2.1] - 2015-12-22

### Fixed
- Fixed passing of version features to `clang-sys`

## [0.2.2] - 2015-12-23

### Added
- Added `sonar` module for finding declarations in C translation units

## [0.3.0] - 2015-12-27

### Added
- Added implementations of `From` to `String` for error enums
- Added integer categorization methods to `Type` struct

### Removed
- Removed `sonar` module

## [0.3.1] - 2016-2-5

### Changed
- Bumped `clang-sys` version

## [0.4.0] - 2016-2-13

### Fixed
- Added missing `cfg`s on enum variants

### Added
- Added support for `clang` 3.8.x

### Changed
- Simplified internal usage of conditional compilation
- Bumped `clang-sys` version
- Bumped `libc` version

## [0.5.0] - 2016-3-9

### Added
- Added a `sonar` module for finding C declarations

## [0.5.1] - 2016-3-14

### Added
- Added implementations of `std::error::Error` for error enums

## [0.5.2] - 2016-3-16

// Copyright 2016 Kyle Mayes
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::ffi::{CStr, CString};
use std::path::{Path, PathBuf};

use clang_sys::*;

use libc::{c_void};

//================================================
// Macros
//================================================

// builder! ______________________________________

/// Defines a struct that builds a set of fields and bitflags.
macro_rules! builder {
    ($(#[$doc:meta])+ builder $name:ident: $underlying:ident {
        $($parameter:ident: $pty:ty), +;
    OPTIONS:
        $($(#[$odoc:meta])+ pub $option:ident: $flag:ident), +,
    }) => (
        $(#[$doc])+
        #[derive(Clone, Debug)]
        pub struct $name<'tu> {
            $($parameter: $pty), *,
            flags: ::clang_sys::$underlying,
        }

        impl<'tu> $name<'tu> {
            $($(#[$odoc])+ pub fn $option(&mut self, $option: bool) -> &mut $name<'tu> {
                if $option {
                    self.flags |= ::clang_sys::$flag;
                } else {
                    self.flags &= !::clang_sys::$flag;
                }
                self
            })+
        }
    );
}

// iter! _________________________________________

/// Returns an iterator over the values returned by `get_argument`.
macro_rules! iter {
    ($num:ident($($num_argument:expr), *), $get:ident($($get_argument:expr), *),) => ({
        let count = unsafe { $num($($num_argument), *) };
        (0..count).map(|i| unsafe { $get($($get_argument), *, i) })
    });

    ($num:ident($($num_argument:expr), *), $($get:ident($($get_argument:expr), *)), *,) => ({
        let count = unsafe { $num($($num_argument), *) };
        (0..count).map(|i| unsafe { ($($get($($get_argument), *, i)), *) })
    });
}

// iter_option! __________________________________

/// Returns an optional iterator over the values returned by `get_argument`.
macro_rules! iter_option {
    ($num:ident($($num_argument:expr), *), $get:ident($($get_argument:expr), *),) => ({
        let count = unsafe { $num($($num_argument), *) };
        if count >= 0 {
            Some((0..count).map(|i| unsafe { $get($($get_argument), *, i as c_uint) }))
        } else {
            None
        }
    });
}

// options! ______________________________________

/// Defines a struct that maps bitflags to fields.
macro_rules! options {
    ($(#[$attribute:meta])* options $name:ident: $underlying:ident {
        $($(#[$fattribute:meta])* pub $option:ident: $flag:ident), +,
    }) => (
        $(#[$attribute])*
        #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
        pub struct $name {
            $($(#[$fattribute])* pub $option: bool), +,
        }

        impl From<::clang_sys::$underlying> for $name {
            fn from(flags: ::clang_sys::$underlying) -> $name {
                $name { $($option: (flags & ::clang_sys::$flag) != 0), + }
            }
        }

        impl From<$name> for ::clang_sys::$underlying {
            fn from(options: $name) -> ::clang_sys::$underlying {
                let mut flags: ::clang_sys::$underlying = 0;
                $(if options.$option { flags |= ::clang_sys::$flag; })+
                flags
            }
        }
    );

    ($(#[$attribute:meta])* options $name:ident: $underlying:ident {
        $($(#[$fattribute:meta])* pub $option:ident: $flag:ident), +,
    }, $fname:ident: #[$feature:meta] {
        $($(#[$ffattribute:meta])* pub $foption:ident: $fflag:ident), +,
    }) => (
        #[cfg($feature)]
        mod $fname {
            options! {
                $(#[$attribute])*
                options $name: $underlying {
                    $($(#[$fattribute])* pub $option: $flag), +,
                    $($(#[$ffattribute])* pub $foption: $fflag), +,
                }
            }
        }

        #[cfg(not($feature))]
        mod $fname {
            options! {
                $(#[$attribute])*
                options $name: $underlying {
                    $($(#[$fattribute])* pub $option: $flag), +,
                }
            }
        }

        pub use $fname::{$name};
    );
}

//================================================
// Traits
//================================================

// FromError _____________________________________

/// A type that can convert a `T` into a `Result<(), Self>`.
pub trait FromError<T>: Sized where T: Sized {
    fn from_error(error: T) -> Result<(), Self>;
}

// Nullable ______________________________________

/// A type which may be null or otherwise invalid.
pub trait Nullable: Sized {
    fn map<U, F: FnOnce(Self) -> U>(self, f: F) -> Option<U>;
}

impl Nullable for *mut c_void {
    fn map<U, F: FnOnce(*mut c_void) -> U>(self, f: F) -> Option<U> {
        if !self.is_null() {
            Some(f(self))
        } else {
            None
        }
    }
}

impl Nullable for CXComment {
    fn map<U, F: FnOnce(CXComment) -> U>(self, f: F) -> Option<U> {
        if !self.ASTNode.is_null() {
            Some(f(self))
        } else {
            None
        }
    }
}

impl Nullable for CXCursor {
    fn map<U, F: FnOnce(CXCursor) -> U>(self, f: F) -> Option<U> {
        unsafe {
            let null = clang_getNullCursor();
            if clang_equalCursors(self, null) == 0 && clang_isInvalid(self.kind) == 0 {
                Some(f(self))
            } else {
                None
            }
        }
    }
}

impl Nullable for CXSourceLocation {
    fn map<U, F: FnOnce(CXSourceLocation) -> U>(self, f: F) -> Option<U> {
        unsafe {
            if clang_equalLocations(self, clang_getNullLocation()) == 0 {
                Some(f(self))
            } else {
                None
            }
        }
    }
}

impl Nullable for CXSourceRange {
    fn map<U, F: FnOnce(CXSourceRange) -> U>(self, f: F) -> Option<U> {
        unsafe {
            if clang_Range_isNull(self) == 0 {
                Some(f(self))
            } else {
                None
            }
        }
    }
}

impl Nullable for CXString {
    fn map<U, F: FnOnce(CXString) -> U>(self, f: F) -> Option<U> {
        if !self.data.is_null() {
            Some(f(self))
        } else {
            None
        }
    }
}

impl Nullable for CXType {
    fn map<U, F: FnOnce(CXType) -> U>(self, f: F) -> Option<U> {
        if self.kind != CXType_Invalid {
            Some(f(self))
        } else {
            None
        }
    }
}

impl Nullable for CXVersion {
    fn map<U, F: FnOnce(CXVersion) -> U>(self, f: F) -> Option<U> {
        if self.Major >= 0 {
            Some(f(self))
        } else {
            None
        }
    }
}

//================================================
// Functions
//================================================

pub fn addressof<T>(value: &mut T) -> *mut c_void {
    (value as *mut T) as *mut c_void
}

pub fn from_path<P: AsRef<Path>>(path: P) -> CString {
    from_string(path.as_ref().as_os_str().to_str().expect("invalid C string"))
}

pub fn to_path(clang: CXString) -> PathBuf {
    let rust_string = to_string(clang);
    PathBuf::from(rust_string)
}

pub fn from_string<S: AsRef<str>>(string: S) -> CString {
    CString::new(string.as_ref()).expect("invalid C string")
}

pub fn to_string(clang: CXString) -> String {
    unsafe {
        let c = CStr::from_ptr(clang_getCString(clang));
        let rust = c.to_str().expect("invalid Rust string").into();
        clang_disposeString(clang);
        rust
    }
}

pub fn to_string_option(clang: CXString) -> Option<String> {
    clang.map(to_string).and_then(|s| {
        if !s.is_empty() {
            Some(s)
        } else {
            None
        }
    })
}

#[cfg(feature="clang_3_8")]
pub fn to_string_set_option(clang: *mut CXStringSet) -> Option<Vec<String>> {
    unsafe {
        if clang.is_null() || (*clang).Count == 0 {
            return None;
        }

        let c = ::std::slice::from_raw_parts((*clang).Strings, (*clang).Count as usize);
        let rust = c.iter().map(|c| {
            CStr::from_ptr(clang_getCString(*c)).to_str().expect("invalid Rust string").into()
        }).collect();
        clang_disposeStringSet(clang);
        Some(rust)
    }
}

pub fn with_string<S: AsRef<str>, T, F: FnOnce(CXString) -> T>(string: S, f: F) -> T {
    let string = from_string(string);
    f(CXString { data: string.as_ptr() as *const c_void, private_flags: 0 })
}

use std::mem;
use std::ffi::{CStr, CString};
use std::path::{Path};

use clang_sys as ffi;

//================================================
// Macros
//================================================

// iter! _________________________________________

/// Returns an iterator over the values returned by `get_argument`.
macro_rules! iter {
    ($num:ident($($num_argument:expr), *), $get:ident($($get_argument:expr), *),) => ({
        let count = unsafe { ffi::$num($($num_argument), *) };
        (0..count).map(|i| unsafe { ffi::$get($($get_argument), *, i) })
    });

    ($num:ident($($num_argument:expr), *), $($get:ident($($get_argument:expr), *)), *,) => ({
        let count = unsafe { ffi::$num($($num_argument), *) };
        (0..count).map(|i| unsafe { ($(ffi::$get($($get_argument), *, i)), *) })
    });
}

// iter_option! __________________________________

/// Returns an optional iterator over the values returned by `get_argument`.
macro_rules! iter_option {
    ($num:ident($($num_argument:expr), *), $get:ident($($get_argument:expr), *),) => ({
        let count = unsafe { ffi::$num($($num_argument), *) };

        if count >= 0 {
            Some((0..count).map(|i| unsafe { ffi::$get($($get_argument), *, i as c_uint) }))
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
                $name { $($option: flags.contains(::clang_sys::$flag)), + }
            }
        }

        impl Into<::clang_sys::$underlying> for $name {
            fn into(self) -> ::clang_sys::$underlying {
                let mut flags = ::clang_sys::$underlying::empty();
                $(if self.$option { flags.insert(::clang_sys::$flag); })+
                flags
            }
        }
    );

    ($(#[$attribute:meta])* options $name:ident: $underlying:ident {
        $($(#[$fattribute:meta])* pub $option:ident: $flag:ident), +,
        CONDITIONAL: #[$condition:meta] $($(#[$cfattribute:meta])* pub $coption:ident: $cflag:ident), +,
    }) => (
        #[cfg(not($condition))]
        mod detail {
            options! {
                $(#[$attribute])*
                options $name: $underlying {
                    $($(#[$fattribute])* pub $option: $flag), +,
                }
            }
        }

        #[cfg($condition)]
        mod detail {
            options! {
                $(#[$attribute])*
                options $name: $underlying {
                    $($(#[$fattribute])* pub $option: $flag), +,
                    $($(#[$cfattribute])* pub $coption: $cflag), +,
                }
            }
        }
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
    /// Transforms this value into an `Option<U>`, mapping a null value to `None` and a non-null
    /// value to `Some(v)` where `v` is the result of applying the supplied function to this value.
    fn map<U, F: FnOnce(Self) -> U>(self, f: F) -> Option<U>;
}

macro_rules! nullable {
    ($name:ident) => (
        impl Nullable for ffi::$name {
            fn map<U, F: FnOnce(ffi::$name) -> U>(self, f: F) -> Option<U> {
                if !self.0.is_null() {
                    Some(f(self))
                } else {
                    None
                }
            }
        }
    );
}

nullable!(CXCompilationDatabase);
nullable!(CXCompileCommand);
nullable!(CXCompileCommands);
nullable!(CXCompletionString);
nullable!(CXCursorSet);
nullable!(CXDiagnostic);
nullable!(CXDiagnosticSet);
nullable!(CXFile);
nullable!(CXIdxClientASTFile);
nullable!(CXIdxClientContainer);
nullable!(CXIdxClientEntity);
nullable!(CXIdxClientFile);
nullable!(CXIndex);
nullable!(CXIndexAction);
nullable!(CXModule);
nullable!(CXModuleMapDescriptor);
nullable!(CXRemapping);
nullable!(CXTranslationUnit);
nullable!(CXVirtualFileOverlay);

impl Nullable for ffi::CXCursor {
    fn map<U, F: FnOnce(ffi::CXCursor) -> U>(self, f: F) -> Option<U> {
        unsafe {
            let null = ffi::clang_equalCursors(self, ffi::clang_getNullCursor()) != 0;

            if !null && ffi::clang_isInvalid(self.kind) == 0 {
                Some(f(self))
            } else {
                None
            }
        }
    }
}

impl Nullable for ffi::CXSourceLocation {
    fn map<U, F: FnOnce(ffi::CXSourceLocation) -> U>(self, f: F) -> Option<U> {
        unsafe {
            if ffi::clang_equalLocations(self, ffi::clang_getNullLocation()) == 0 {
                Some(f(self))
            } else {
                None
            }
        }
    }
}

impl Nullable for ffi::CXSourceRange {
    fn map<U, F: FnOnce(ffi::CXSourceRange) -> U>(self, f: F) -> Option<U> {
        unsafe {
            if ffi::clang_Range_isNull(self) == 0 {
                Some(f(self))
            } else {
                None
            }
        }
    }
}

impl Nullable for ffi::CXString {
    fn map<U, F: FnOnce(ffi::CXString) -> U>(self, f: F) -> Option<U> {
        if !self.data.is_null() {
            Some(f(self))
        } else {
            None
        }
    }
}

impl Nullable for ffi::CXType {
    fn map<U, F: FnOnce(ffi::CXType) -> U>(self, f: F) -> Option<U> {
        if self.kind != ffi::CXTypeKind::Invalid {
            Some(f(self))
        } else {
            None
        }
    }
}

impl Nullable for ffi::CXVersion {
    fn map<U, F: FnOnce(ffi::CXVersion) -> U>(self, f: F) -> Option<U> {
        if self.Major != -1 && self.Minor != -1 && self.Subminor != -1 {
            Some(f(self))
        } else {
            None
        }
    }
}

//================================================
// Functions
//================================================

pub fn from_path<P: AsRef<Path>>(path: P) -> CString {
    from_string(path.as_ref().as_os_str().to_str().expect("invalid C string"))
}

pub fn from_string<S: AsRef<str>>(string: S) -> CString {
    CString::new(string.as_ref()).expect("invalid C string")
}

pub fn to_string(clang: ffi::CXString) -> String {
    unsafe {
        let c = CStr::from_ptr(ffi::clang_getCString(clang));
        let rust = c.to_str().expect("invalid Rust string").into();
        ffi::clang_disposeString(clang);
        rust
    }
}

pub fn to_string_option(clang: ffi::CXString) -> Option<String> {
    clang.map(to_string).and_then(|s| {
        if !s.is_empty() {
            Some(s)
        } else {
            None
        }
    })
}

#[cfg(feature="gte_clang_3_8")]
pub fn to_string_set(clang: *mut ffi::CXStringSet) -> Vec<String> {
    unsafe {
        let c = slice::from_raw_parts((*clang).Strings, (*clang).Count as usize);

        let rust = c.iter().map(|c| {
            let c = std::ffi::CStr::from_ptr(ffi::clang_getCString(*c));
            c.to_str().expect("invalid Rust string").into()
        }).collect();

        ffi::clang_disposeStringSet(clang);
        rust
    }
}

pub fn with_string<S: AsRef<str>, T, F: FnOnce(ffi::CXString) -> T>(string: S, f: F) -> T {
    let string = from_string(string);
    unsafe { f(ffi::CXString { data: mem::transmute(string.as_ptr()), private_flags: 0 }) }
}

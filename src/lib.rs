#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![cfg_attr(feature="clippy", warn(clippy))]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;

extern crate libc;

use std::marker::{PhantomData};
use std::path::{Path};
use std::sync::atomic::{AtomicBool, Ordering};

use libc::{c_int, c_ulong};

pub mod ffi;

//================================================
// Macros
//================================================

// options! ______________________________________

macro_rules! options {
    ($(#[$attribute:meta])* options $name:ident: $underlying:ident {
        $($(#[$fattribute:meta])* pub $option:ident: $flag:ident), +,
    }) => (
        $(#[$attribute])*
        #[derive(Default)]
        pub struct $name {
            $($(#[$fattribute])* pub $option: bool), +,
        }

        impl Into<ffi::$underlying> for $name {
            fn into(self) -> ffi::$underlying {
                let mut flags = ffi::$underlying::empty();
                $(if self.$option { flags.insert(ffi::$flag); })+
                flags
            }
        }
    );
}

//================================================
// Structs
//================================================

// Clang _________________________________________

lazy_static! { static ref AVAILABLE: AtomicBool = AtomicBool::new(true); }

/// An empty type which prevents the use of this library from multiple threads.
pub struct Clang;

impl Clang {
    //- Constructors -----------------------------

    /// Constructs a new `Clang`.
    ///
    /// Only one instance of `Clang` is allowed at a time.
    ///
    /// # Failures
    ///
    /// * an instance of `Clang` already exists
    pub fn new() -> Result<Clang, ()> {
        if AVAILABLE.swap(false, Ordering::Relaxed) {
            Ok(Clang)
        } else {
            Err(())
        }
    }
}

impl Drop for Clang {
    fn drop(&mut self) {
        AVAILABLE.store(true, Ordering::Relaxed);
    }
}

// Index _________________________________________

/// Indicates which types of threads have background priority.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct BackgroundPriority {
    pub editing: bool,
    pub indexing: bool,
}

/// A collection of translation units.
pub struct Index<'c> {
    handle: ffi::CXIndex,
    _marker: PhantomData<&'c Clang>,
}

impl<'c> Index<'c> {
    //- Constructors -----------------------------

    /// Constructs a new `Index`.
    ///
    /// `exclude` determines whether declarations from precompiled headers are excluded and
    /// `diagnostics` determines whether diagnostics are printed while parsing source files.
    pub fn new(_: &'c Clang, exclude: bool, diagnostics: bool) -> Index<'c> {
        let handle = unsafe { ffi::clang_createIndex(exclude as c_int, diagnostics as c_int) };
        Index { handle: handle, _marker: PhantomData }
    }

    //- Accessors --------------------------------

    /// Returns which types of threads have background priority.
    pub fn get_background_priority(&self) -> BackgroundPriority {
        let flags = unsafe { ffi::clang_CXIndex_getGlobalOptions(self.handle) };
        let editing = flags.contains(ffi::CXGlobalOpt_ThreadBackgroundPriorityForEditing);
        let indexing = flags.contains(ffi::CXGlobalOpt_ThreadBackgroundPriorityForIndexing);
        BackgroundPriority { editing: editing, indexing: indexing }
    }

    //- Mutators ---------------------------------

    /// Sets which types of threads have background priority.
    pub fn set_background_priority(&mut self, priority: BackgroundPriority) {
        let mut flags = ffi::CXGlobalOptFlags::empty();

        if priority.editing {
            flags.insert(ffi::CXGlobalOpt_ThreadBackgroundPriorityForEditing);
        }

        if priority.indexing {
            flags.insert(ffi::CXGlobalOpt_ThreadBackgroundPriorityForIndexing);
        }

        unsafe { ffi::clang_CXIndex_setGlobalOptions(self.handle, flags); }
    }
}

impl<'c> Drop for Index<'c> {
    fn drop(&mut self) {
        unsafe { ffi::clang_disposeIndex(self.handle); }
    }
}

// Unsaved _______________________________________

/// The path to and unsaved contents of a previously existing file.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Unsaved {
    path: std::ffi::CString,
    contents: std::ffi::CString,
}

impl Unsaved {
    //- Constructors -----------------------------

    /// Constructs a new `Unsaved`.
    pub fn new<P: AsRef<Path>>(path: P, contents: &str) -> Unsaved {
        Unsaved { path: from_path(path), contents: from_string(contents) }
    }

    //- Accessors --------------------------------

    fn as_raw(&self) -> ffi::CXUnsavedFile {
        ffi::CXUnsavedFile {
            Filename: self.path.as_ptr(),
            Contents: self.contents.as_ptr(),
            Length: self.contents.as_bytes().len() as c_ulong,
        }
    }
}

//================================================
// Functions
//================================================

fn from_path<P>(path: P) -> std::ffi::CString where P: AsRef<Path> {
    from_string(path.as_ref().as_os_str().to_str().expect("invalid C string"))
}

fn from_string(string: &str) -> std::ffi::CString {
    std::ffi::CString::new(string).expect("invalid C string")
}

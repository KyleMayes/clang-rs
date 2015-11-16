#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![cfg_attr(feature="clippy", warn(clippy))]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;

extern crate libc;

use std::cmp;
use std::fmt;
use std::hash;
use std::mem;
use std::slice;
use std::collections::{HashMap};
use std::marker::{PhantomData};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use libc::{c_int, c_uint, c_ulong, time_t};

pub mod ffi;

//================================================
// Macros
//================================================

// iter! _________________________________________

macro_rules! iter {
    ($num:ident($($num_argument:expr), *), $get:ident($($get_argument:expr), *),) => ({
        let count = unsafe { ffi::$num($($num_argument), *) };
        (0..count).map(|i| unsafe { ffi::$get($($get_argument), *, i) })
    });
}

// options! ______________________________________

macro_rules! options {
    ($(#[$attribute:meta])* options $name:ident: $underlying:ident {
        $($(#[$fattribute:meta])* pub $option:ident: $flag:ident), +,
    }) => (
        $(#[$attribute])*
        #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
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
// Traits
//================================================

// Nullable ______________________________________

/// A type which may be null.
pub trait Nullable<T> {
    fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Option<U>;
}

//================================================
// Enums
//================================================

// MemoryUsage ___________________________________

/// Indicates the usage category of a quantity of memory.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum MemoryUsage {
    /// Expressions, declarations and types.
    Ast = 1,
    /// Various tables used by the AST.
    AstSideTables = 6,
    /// Memory allocated with `malloc` for external AST sources.
    ExternalAstSourceMalloc = 9,
    /// Memory allocated with `mmap` for external AST sources.
    ExternalAstSourceMMap = 10,
    /// Cached global code completion results.
    GlobalCodeCompletionResults = 4,
    /// Identifiers.
    Identifiers = 2,
    /// The preprocessing record.
    PreprocessingRecord = 12,
    /// Memory allocated with `malloc` for the preprocessor.
    Preprocessor = 11,
    /// Header search tables.
    PreprocessorHeaderSearch = 14,
    /// Selectors.
    Selectors = 3,
    /// The content cache used by the source manager.
    SourceManagerContentCache = 5,
    /// Data structures used by the source manager.
    SourceManagerDataStructures = 13,
    /// Memory allocated with `malloc` for the source manager.
    SourceManagerMalloc = 7,
    /// Memory allocated with `mmap` for the source manager.
    SourceManagerMMap = 8,
}

// SaveError _____________________________________

/// Indicates the type of error that prevented the saving of a translation unit to an AST file.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum SaveError {
    /// Errors in the translation unit prevented saving.
    Errors,
    /// An unknown error occurred.
    Unknown,
}

// SourceError ___________________________________

/// Indicates the type of error that prevented the loading of a translation unit from a source file.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum SourceError {
    /// An error occurred while deserializing an AST file.
    AstDeserialization,
    /// `libclang` crashed.
    Crash,
    /// An unknown error occurred.
    Unknown,
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

// File __________________________________________

/// A source file.
#[derive(Copy, Clone)]
pub struct File<'tu> {
    handle: ffi::CXFile,
    tu: &'tu TranslationUnit<'tu>,
}

impl<'tu> File<'tu> {
    //- Constructors -----------------------------

    fn from_ptr(handle: ffi::CXFile, tu: &'tu TranslationUnit<'tu>) -> File<'tu> {
        File { handle: handle, tu: tu }
    }

    //- Accessors --------------------------------

    /// Returns a unique identifier for this file.
    pub fn get_id(&self) -> (u64, u64, u64) {
        unsafe {
            let mut id = mem::uninitialized();
            ffi::clang_getFileUniqueID(self.handle, &mut id);
            (id.data[0] as u64, id.data[1] as u64, id.data[2] as u64)
        }
    }

    /// Returns the source location at the supplied line and column in this file.
    ///
    /// # Panics
    ///
    /// * `line` or `column` is `0`
    pub fn get_location(&self, line: u32, column: u32) -> SourceLocation<'tu> {
        if line == 0 || column == 0 {
            panic!("`line` or `column` is `0`");
        }

        let raw = unsafe {
            ffi::clang_getLocation(self.tu.handle, self.handle, line as c_uint, column as c_uint)
        };

        SourceLocation::from_raw(raw, self.tu)
    }

    /// Returns the module containing this file, if any.
    pub fn get_module(&self) -> Option<Module<'tu>> {
        let handle = unsafe { ffi::clang_getModuleForFile(self.tu.handle, self.handle) };
        handle.map(|h| Module::from_ptr(h, self.tu))
    }

    /// Returns the source location at the supplied character offset in this file.
    pub fn get_offset_location(&self, offset: u32) -> SourceLocation<'tu> {
        let raw = unsafe {
            ffi::clang_getLocationForOffset(self.tu.handle, self.handle, offset as c_uint)
        };

        SourceLocation::from_raw(raw, self.tu)
    }

    /// Returns the absolute path to this file.
    pub fn get_path(&self) -> PathBuf {
        let path = unsafe { ffi::clang_getFileName(self.handle) };
        Path::new(&to_string(path)).into()
    }

    /// Returns the last modification time for this file.
    pub fn get_time(&self) -> time_t {
        unsafe { ffi::clang_getFileTime(self.handle) }
    }

    /// Returns whether this file is guarded against multiple inclusions.
    pub fn is_include_guarded(&self) -> bool {
        unsafe { ffi::clang_isFileMultipleIncludeGuarded(self.tu.handle, self.handle) != 0 }
    }
}

impl<'tu> cmp::Eq for File<'tu> { }

impl<'tu> cmp::PartialEq for File<'tu> {
    fn eq(&self, other: &File<'tu>) -> bool {
        unsafe { ffi::clang_File_isEqual(self.handle, other.handle) != 0 }
    }
}

impl<'tu> fmt::Debug for File<'tu> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.debug_struct("File").field("path", &self.get_path()).finish()
    }
}

impl<'tu> hash::Hash for File<'tu> {
    fn hash<H>(&self, hasher: &mut H) where H: hash::Hasher {
        self.get_id().hash(hasher);
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

// Module ________________________________________

/// A collection of headers.
#[derive(Copy, Clone)]
pub struct Module<'tu> {
    handle: ffi::CXModule,
    tu: &'tu TranslationUnit<'tu>,
}

impl<'tu> Module<'tu> {
    //- Constructors -----------------------------

    fn from_ptr(handle: ffi::CXModule, tu: &'tu TranslationUnit<'tu>) -> Module<'tu> {
        Module { handle: handle, tu: tu }
    }

    //- Accessors --------------------------------

    /// Returns the AST file this module came from.
    pub fn get_file(&self) -> File<'tu> {
        let handle = unsafe { ffi::clang_Module_getASTFile(self.handle) };
        File::from_ptr(handle, self.tu)
    }

    /// Returns the full name of this module (e.g., `std.vector` for the `std.vector` module).
    pub fn get_full_name(&self) -> String {
        let name = unsafe { ffi::clang_Module_getFullName(self.handle) };
        to_string(name)
    }

    /// Returns the name of this module (e.g., `vector` for the `std.vector` module).
    pub fn get_name(&self) -> String {
        let name = unsafe { ffi::clang_Module_getName(self.handle) };
        to_string(name)
    }

    /// Returns the parent of this module, if any.
    pub fn get_parent(&self) -> Option<Module<'tu>> {
        let handle = unsafe { ffi::clang_Module_getParent(self.handle) };
        handle.map(|h| Module::from_ptr(h, self.tu))
    }

    /// Returns the top-level headers in this module.
    pub fn get_top_level_headers(&self) -> Vec<File<'tu>> {
        iter!(
            clang_Module_getNumTopLevelHeaders(self.tu.handle, self.handle),
            clang_Module_getTopLevelHeader(self.tu.handle, self.handle),
        ).map(|h| File::from_ptr(h, self.tu)).collect()
    }

    /// Returns whether this module is a system module.
    pub fn is_system(&self) -> bool {
        unsafe { ffi::clang_Module_isSystem(self.handle) != 0 }
    }
}

impl<'tu> fmt::Debug for Module<'tu> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.debug_struct("Module")
            .field("full_name", &self.get_full_name())
            .field("file", &self.get_file())
            .finish()
    }
}

// ParseOptions __________________________________

options! {
    /// A set of options that determines how a source file is parsed into a translation unit.
    options ParseOptions: CXTranslationUnit_Flags {
        /// Indicates whether certain code completion results will be cached when the translation
        /// unit is reparsed.
        ///
        /// This option increases the time it takes to reparse the translation unit but improves
        /// code completion performance.
        pub cache_completion_results: CXTranslationUnit_CacheCompletionResults,
        /// Indicates whether a detailed preprocessing record will be constructed which includes all
        /// macro definitions and instantiations.
        pub detailed_preprocessing_record: CXTranslationUnit_DetailedPreprocessingRecord,
        /// Indicates whether brief documentation comments will be included in code completion
        /// results.
        pub include_brief_comments_in_code_completion: CXTranslationUnit_IncludeBriefCommentsInCodeCompletion,
        /// Indicates whether the translation unit will be considered incomplete.
        ///
        /// This option suppresses certain semantic analyses and is typically used when parsing
        /// headers with the intent of creating a precompiled header.
        pub incomplete: CXTranslationUnit_Incomplete,
        /// Indicates whether function and method bodies will be skipped.
        pub skip_function_bodies: CXTranslationUnit_SkipFunctionBodies,
    }
}

// SourceLocation ________________________________

macro_rules! location {
    ($function:ident, $location:expr, $tu:expr) => ({
        let (mut file, mut line, mut column, mut offset) = mem::uninitialized();
        ffi::$function($location, &mut file, &mut line, &mut column, &mut offset);

        Location {
            file: File::from_ptr(file, $tu),
            line: line as u32,
            column: column as u32,
            offset: offset as u32,
        }
    });
}

/// The file, line, column, and character offset of a source location.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Location<'tu> {
    pub file: File<'tu>,
    pub line: u32,
    pub column: u32,
    pub offset: u32,
}

/// A location in a source file.
#[derive(Copy, Clone)]
pub struct SourceLocation<'tu> {
    raw: ffi::CXSourceLocation,
    tu: &'tu TranslationUnit<'tu>,
}

impl<'tu> SourceLocation<'tu> {
    //- Constructors -----------------------------

    fn from_raw(raw: ffi::CXSourceLocation, tu: &'tu TranslationUnit<'tu>) -> SourceLocation<'tu> {
        SourceLocation { raw: raw, tu: tu }
    }

    //- Accessors --------------------------------

    /// Returns the file, line, column and character offset of this source location.
    ///
    /// If this source location is inside a macro expansion, the location of the macro expansion is
    /// returned instead.
    pub fn get_expansion_location(&self) -> Location<'tu> {
        unsafe { location!(clang_getExpansionLocation, self.raw, self.tu) }
    }

    /// Returns the file, line, column and character offset of this source location.
    ///
    /// If this source location is inside a macro expansion, the location of the macro expansion is
    /// returned instead unless this source location is inside a macro argument. In that case, the
    /// location of the macro argument is returned.
    pub fn get_file_location(&self) -> Location<'tu> {
        unsafe { location!(clang_getFileLocation, self.raw, self.tu) }
    }

    /// Returns the file path, line, and column of this source location taking line directives into
    /// account.
    pub fn get_presumed_location(&self) -> (String, u32, u32) {
        unsafe {
            let (mut file, mut line, mut column) = mem::uninitialized();
            ffi::clang_getPresumedLocation(self.raw, &mut file, &mut line, &mut column);
            (to_string(file), line as u32, column as u32)
        }
    }

    /// Returns the file, line, column and character offset of this source location.
    pub fn get_spelling_location(&self) -> Location<'tu> {
        unsafe { location!(clang_getSpellingLocation, self.raw, self.tu) }
    }

    /// Returns whether this source location is in the main file of its translation unit.
    pub fn is_in_main_file(&self) -> bool {
        unsafe { ffi::clang_Location_isFromMainFile(self.raw) != 0 }
    }

    /// Returns whether this source location is in a system header.
    pub fn is_in_system_header(&self) -> bool {
        unsafe { ffi::clang_Location_isInSystemHeader(self.raw) != 0 }
    }
}

impl<'tu> cmp::Eq for SourceLocation<'tu> { }

impl<'tu> cmp::PartialEq for SourceLocation<'tu> {
    fn eq(&self, other: &SourceLocation<'tu>) -> bool {
        unsafe { ffi::clang_equalLocations(self.raw, other.raw) != 0 }
    }
}

impl<'tu> fmt::Debug for SourceLocation<'tu> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let location = self.get_spelling_location();
        formatter.debug_struct("SourceLocation")
            .field("file", &location.file)
            .field("line", &location.line)
            .field("column", &location.column)
            .field("offset", &location.offset)
            .finish()
    }
}

impl<'tu> hash::Hash for SourceLocation<'tu> {
    fn hash<H>(&self, hasher: &mut H) where H: hash::Hasher {
        self.get_spelling_location().hash(hasher)
    }
}

// SourceRange ___________________________________

/// A half-open range in a source file.
#[derive(Copy, Clone)]
pub struct SourceRange<'tu> {
    raw: ffi::CXSourceRange,
    tu: &'tu TranslationUnit<'tu>,
}

impl<'tu> SourceRange<'tu> {
    //- Constructors -----------------------------

    fn from_raw(raw: ffi::CXSourceRange, tu: &'tu TranslationUnit<'tu>) -> SourceRange<'tu> {
        SourceRange { raw: raw, tu: tu }
    }

    /// Constructs a new `SourceRange` that spans [`start`, `end`).
    pub fn new(start: SourceLocation<'tu>, end: SourceLocation<'tu>) -> SourceRange<'tu> {
        let raw = unsafe { ffi::clang_getRange(start.raw, end.raw) };
        SourceRange::from_raw(raw, start.tu)
    }

    //- Accessors --------------------------------

    /// Returns the exclusive end of this source range.
    pub fn get_end(&self) -> SourceLocation<'tu> {
        let raw = unsafe { ffi::clang_getRangeEnd(self.raw) };
        SourceLocation::from_raw(raw, self.tu)
    }

    /// Returns the inclusive start of this source range.
    pub fn get_start(&self) -> SourceLocation<'tu> {
        let raw = unsafe { ffi::clang_getRangeStart(self.raw) };
        SourceLocation::from_raw(raw, self.tu)
    }
}

impl<'tu> cmp::Eq for SourceRange<'tu> { }

impl<'tu> cmp::PartialEq for SourceRange<'tu> {
    fn eq(&self, other: &SourceRange<'tu>) -> bool {
        unsafe { ffi::clang_equalRanges(self.raw, other.raw) != 0 }
    }
}

impl<'tu> fmt::Debug for SourceRange<'tu> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.debug_struct("SourceRange")
            .field("start", &self.get_start())
            .field("end", &self.get_end())
            .finish()
    }
}

impl<'tu> hash::Hash for SourceRange<'tu> {
    fn hash<H>(&self, hasher: &mut H) where H: hash::Hasher {
        self.get_start().hash(hasher);
        self.get_end().hash(hasher);
    }
}

// TranslationUnit _______________________________

/// A preprocessed and parsed source file.
pub struct TranslationUnit<'i> {
    handle: ffi::CXTranslationUnit,
    _marker: PhantomData<&'i Index<'i>>,
}

impl<'i> TranslationUnit<'i> {
    //- Constructors -----------------------------

    fn from_ptr(handle: ffi::CXTranslationUnit) -> TranslationUnit<'i> {
        TranslationUnit{ handle: handle, _marker: PhantomData }
    }

    /// Constructs a new `TranslationUnit` from an AST file.
    ///
    /// # Failures
    ///
    /// * an unknown error occurs
    pub fn from_ast<F: AsRef<Path>>(
        index: &'i mut Index, file: F
    ) -> Result<TranslationUnit<'i>, ()> {
        let handle = unsafe {
            ffi::clang_createTranslationUnit(index.handle, from_path(file).as_ptr())
        };

        handle.map(TranslationUnit::from_ptr).ok_or(())
    }

    /// Constructs a new `TranslationUnit` from a source file.
    ///
    /// Any compiler argument that may be supplied to `clang` may be supplied to this function.
    /// However, the following arguments are ignored:
    ///
    /// * `-c`
    /// * `-emit-ast`
    /// * `-fsyntax-only`
    /// * `-o` and the following `<output>`
    ///
    /// # Failures
    ///
    /// * an error occurs while deserializing an AST file
    /// * `libclang` crashes
    /// * an unknown error occurs
    pub fn from_source<F: AsRef<Path>>(
        index: &'i mut Index,
        file: F,
        arguments: &[&str],
        unsaved: &[Unsaved],
        options: ParseOptions,
    ) -> Result<TranslationUnit<'i>, SourceError> {
        let arguments = arguments.iter().map(|a| from_string(a)).collect::<Vec<_>>();
        let arguments = arguments.iter().map(|a| a.as_ptr()).collect::<Vec<_>>();
        let unsaved = unsaved.iter().map(|u| u.as_raw()).collect::<Vec<_>>();

        unsafe {
            let mut handle = mem::uninitialized();

            let code = ffi::clang_parseTranslationUnit2(
                index.handle,
                from_path(file).as_ptr(),
                arguments.as_ptr(),
                arguments.len() as c_int,
                mem::transmute(unsaved.as_ptr()),
                unsaved.len() as c_uint,
                options.into(),
                &mut handle,
            );

            match code {
                ffi::CXErrorCode::Success => Ok(TranslationUnit::from_ptr(handle)),
                ffi::CXErrorCode::ASTReadError => Err(SourceError::AstDeserialization),
                ffi::CXErrorCode::Crashed => Err(SourceError::Crash),
                ffi::CXErrorCode::Failure => Err(SourceError::Unknown),
                _ => unreachable!(),
            }
        }
    }

    //- Accessors --------------------------------

    /// Returns the file at the supplied path in this translation unit, if any.
    pub fn get_file<F: AsRef<Path>>(&'i self, file: F) -> Option<File<'i>> {
        let file = unsafe { ffi::clang_getFile(self.handle, from_path(file).as_ptr()) };
        file.map(|f| File::from_ptr(f, self))
    }

    /// Returns the memory usage of this translation unit.
    pub fn get_memory_usage(&self) -> HashMap<MemoryUsage, usize> {
        unsafe {
            let raw = ffi::clang_getCXTUResourceUsage(self.handle);

            let usage = slice::from_raw_parts(raw.entries, raw.numEntries as usize).iter().map(|u| {
                (mem::transmute(u.kind), u.amount as usize)
            }).collect();

            ffi::clang_disposeCXTUResourceUsage(raw);
            usage
        }
    }

    /// Saves this translation unit to an AST file.
    ///
    /// # Failures
    ///
    /// * errors in the translation unit prevent saving
    /// * an unknown error occurs
    pub fn save<F: AsRef<Path>>(&self, file: F) -> Result<(), SaveError> {
        let code = unsafe {
            ffi::clang_saveTranslationUnit(
                self.handle, from_path(file).as_ptr(), ffi::CXSaveTranslationUnit_None
            )
        };

        match code {
            ffi::CXSaveError::None => Ok(()),
            ffi::CXSaveError::InvalidTU => Err(SaveError::Errors),
            ffi::CXSaveError::Unknown => Err(SaveError::Unknown),
            _ => unreachable!(),
        }
    }

    //- Consumers --------------------------------

    /// Consumes this translation unit and reparses the source file it was created from with the
    /// same compiler arguments that were used originally.
    ///
    /// # Failures
    ///
    /// * an error occurs while deserializing an AST file
    /// * `libclang` crashes
    /// * an unknown error occurs
    pub fn reparse(self, unsaved: &[Unsaved]) -> Result<TranslationUnit<'i>, SourceError> {
        let unsaved = unsaved.iter().map(|u| u.as_raw()).collect::<Vec<_>>();

        unsafe {
            let code = ffi::clang_reparseTranslationUnit(
                self.handle,
                unsaved.len() as c_uint,
                mem::transmute(unsaved.as_ptr()),
                ffi::CXReparse_None,
            );

            match code {
                ffi::CXErrorCode::Success => Ok(self),
                ffi::CXErrorCode::ASTReadError => Err(SourceError::AstDeserialization),
                ffi::CXErrorCode::Crashed => Err(SourceError::Crash),
                ffi::CXErrorCode::Failure => Err(SourceError::Unknown),
                _ => unreachable!(),
            }
        }
    }
}

impl<'i> Drop for TranslationUnit<'i> {
    fn drop(&mut self) {
        unsafe { ffi::clang_disposeTranslationUnit(self.handle); }
    }
}

impl<'i> fmt::Debug for TranslationUnit<'i> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let spelling = unsafe { ffi::clang_getTranslationUnitSpelling(self.handle) };
        formatter.debug_struct("TranslationUnit").field("spelling", &to_string(spelling)).finish()
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

fn to_string(handle: ffi::CXString) -> String {
    unsafe {
        let string = ::std::ffi::CStr::from_ptr(ffi::clang_getCString(handle));
        let string = string.to_str().expect("invalid Rust string").into();
        ffi::clang_disposeString(handle);
        string
    }
}

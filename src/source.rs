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

//! Source files, locations, and ranges.

use std::cmp;
use std::fmt;
use std::hash;
use std::mem;
use std::slice;
use std::path::{Path, PathBuf};

use clang_sys::*;

use libc::{c_uint, time_t};

use utility::{self, Nullable};
use super::{Entity, TranslationUnit};
use super::token::{Token};

//================================================
// Structs
//================================================

// File __________________________________________

/// A source file.
#[derive(Copy, Clone)]
pub struct File<'tu> {
    ptr: CXFile,
    tu: &'tu TranslationUnit<'tu>,
}

impl<'tu> File<'tu> {
    //- Constructors -----------------------------

    #[doc(hidden)]
    pub fn from_ptr(ptr: CXFile, tu: &'tu TranslationUnit<'tu>) -> File<'tu> {
        assert!(!ptr.is_null());
        File { ptr, tu }
    }

    //- Accessors --------------------------------

    /// Returns the absolute path to this file.
    pub fn get_path(&self) -> PathBuf {
        unsafe { Path::new(&utility::to_string(clang_getFileName(self.ptr))).into() }
    }

    /// Returns the last modification time for this file.
    pub fn get_time(&self) -> time_t {
        unsafe { clang_getFileTime(self.ptr) }
    }

    /// Returns a unique identifier for this file.
    pub fn get_id(&self) -> (u64, u64, u64) {
        unsafe {
            let mut id = mem::MaybeUninit::uninit();
            clang_getFileUniqueID(self.ptr, id.as_mut_ptr());
            let id = id.assume_init();
            (id.data[0] as u64, id.data[1] as u64, id.data[2] as u64)
        }
    }

    /// Returns the contents of this file, if this file has been loaded.
    #[cfg(feature="clang_6_0")]
    pub fn get_contents(&self) -> Option<String> {
        use std::ptr;
        use std::ffi::CStr;

        unsafe {
            let c = clang_getFileContents(self.tu.ptr, self.ptr, ptr::null_mut());
            if !c.is_null() {
                Some(CStr::from_ptr(c).to_str().expect("invalid Rust string").into())
            } else {
                None
            }
        }
    }

    /// Returns the module containing this file, if any.
    pub fn get_module(&self) -> Option<Module<'tu>> {
        let module = unsafe { clang_getModuleForFile(self.tu.ptr, self.ptr) };
        module.map(|m| Module::from_ptr(m, self.tu))
    }

    /// Returns the source ranges in this file that were skipped by the preprocessor.
    ///
    /// This will always return an empty `Vec` if the translation unit that contains this file was
    /// not constructed with a detailed preprocessing record.
    pub fn get_skipped_ranges(&self) -> Vec<SourceRange<'tu>> {
        unsafe {
            let raw = clang_getSkippedRanges(self.tu.ptr, self.ptr);
            let raws = slice::from_raw_parts((*raw).ranges, (*raw).count as usize);
            let ranges = raws.iter().map(|r| SourceRange::from_raw(*r, self.tu)).collect();
            clang_disposeSourceRangeList(raw);
            ranges
        }
    }

    /// Returns whether this file is guarded against multiple inclusions.
    pub fn is_include_guarded(&self) -> bool {
        unsafe { clang_isFileMultipleIncludeGuarded(self.tu.ptr, self.ptr) != 0 }
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

        let (line, column) = (line, column) as (c_uint, c_uint);
        let location = unsafe { clang_getLocation(self.tu.ptr, self.ptr, line, column) };
        SourceLocation::from_raw(location, self.tu)
    }

    /// Returns the source location at the supplied character offset in this file.
    pub fn get_offset_location(&self, offset: u32) -> SourceLocation<'tu> {
        let offset = offset as c_uint;
        let location = unsafe { clang_getLocationForOffset(self.tu.ptr, self.ptr, offset) };
        SourceLocation::from_raw(location, self.tu)
    }

    /// Returns the inclusion directives in this file.
    pub fn get_includes(&self) -> Vec<Entity<'tu>> {
        let mut includes = vec![];
        self.visit_includes(|e, _| {
            includes.push(e);
            true
        });
        includes
    }

    /// Returns the references to the supplied entity in this file.
    pub fn get_references(&self, entity: Entity<'tu>) -> Vec<Entity<'tu>> {
        let mut references = vec![];
        self.visit_references(entity, |e, _| {
            references.push(e);
            true
        });
        references
    }

    /// Visits the inclusion directives in this file and returns whether visitation was ended by the
    /// callback returning `false`.
    pub fn visit_includes<F: FnMut(Entity<'tu>, SourceRange<'tu>) -> bool>(&self, f: F) -> bool {
        visit(self.tu, f, |v| unsafe { clang_findIncludesInFile(self.tu.ptr, self.ptr, v) })
    }

    /// Visits the references to the supplied entity in this file and returns whether visitation was
    /// ended by the callback returning `false`.
    pub fn visit_references<F: FnMut(Entity<'tu>, SourceRange<'tu>) -> bool>(
        &self, entity: Entity<'tu>, f: F
    ) -> bool {
        visit(self.tu, f, |v| unsafe { clang_findReferencesInFile(entity.raw, self.ptr, v) })
    }
}

impl<'tu> fmt::Debug for File<'tu> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.debug_struct("File").field("path", &self.get_path()).finish()
    }
}

impl<'tu> cmp::PartialEq for File<'tu> {
    fn eq(&self, other: &File<'tu>) -> bool {
        self.get_id() == other.get_id()
    }
}

impl<'tu> cmp::Eq for File<'tu> { }

impl<'tu> hash::Hash for File<'tu> {
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        self.get_id().hash(hasher);
    }
}

// Location ______________________________________

/// The file, line, column, and character offset of a source location.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Location<'tu> {
    /// The file of the source location, if it has any.
    pub file: Option<File<'tu>>,
    /// The line of the source location.
    pub line: u32,
    /// The column of the source location.
    pub column: u32,
    /// The character offset of the source location.
    pub offset: u32,
}

// Module ________________________________________

/// A collection of headers.
#[derive(Copy, Clone)]
pub struct Module<'tu> {
    ptr: CXModule,
    tu: &'tu TranslationUnit<'tu>,
}

impl<'tu> Module<'tu> {
    //- Constructors -----------------------------

    #[doc(hidden)]
    pub fn from_ptr(ptr: CXModule, tu: &'tu TranslationUnit<'tu>) -> Module<'tu> {
        assert!(!ptr.is_null());
        Module { ptr, tu }
    }

    //- Accessors --------------------------------

    /// Returns the name of this module (e.g., `vector` for the `std.vector` module).
    pub fn get_name(&self) -> String {
        unsafe { utility::to_string(clang_Module_getName(self.ptr)) }
    }

    /// Returns the full name of this module (e.g., `std.vector` for the `std.vector` module).
    pub fn get_full_name(&self) -> String {
        unsafe { utility::to_string(clang_Module_getFullName(self.ptr)) }
    }

    /// Returns the parent of this module, if any.
    pub fn get_parent(&self) -> Option<Module<'tu>> {
        unsafe { clang_Module_getParent(self.ptr).map(|p| Module::from_ptr(p, self.tu)) }
    }

    /// Returns the AST file this module came from.
    pub fn get_file(&self) -> File<'tu> {
        unsafe { File::from_ptr(clang_Module_getASTFile(self.ptr), self.tu) }
    }

    /// Returns the top-level headers in this module.
    pub fn get_top_level_headers(&self) -> Vec<File<'tu>> {
        iter!(
            clang_Module_getNumTopLevelHeaders(self.tu.ptr, self.ptr),
            clang_Module_getTopLevelHeader(self.tu.ptr, self.ptr),
        ).map(|h| File::from_ptr(h, self.tu)).collect()
    }

    /// Returns whether this module is a system module.
    pub fn is_system(&self) -> bool {
        unsafe { clang_Module_isSystem(self.ptr) != 0 }
    }
}

impl<'tu> fmt::Debug for Module<'tu> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.debug_struct("Module")
            .field("file", &self.get_file())
            .field("full_name", &self.get_full_name())
            .finish()
    }
}

impl<'tu> cmp::PartialEq for Module<'tu> {
    fn eq(&self, other: &Module<'tu>) -> bool {
        self.get_file() == other.get_file() && self.get_full_name() == other.get_full_name()
    }
}

impl<'tu> cmp::Eq for Module<'tu> { }

// SourceLocation ________________________________

macro_rules! location {
    ($function:ident, $location:expr, $tu:expr) => ({
        fn uninit<T>() -> mem::MaybeUninit<T> { mem::MaybeUninit::uninit() }
        let (mut file, mut line, mut column, mut offset) = (uninit(), uninit(), uninit(), uninit());
        $function(
            $location,
            file.as_mut_ptr(),
            line.as_mut_ptr(),
            column.as_mut_ptr(),
            offset.as_mut_ptr(),
        );
        Location {
            file: file.assume_init().map(|f| File::from_ptr(f, $tu)),
            line: line.assume_init() as u32,
            column: column.assume_init() as u32,
            offset: offset.assume_init() as u32,
        }
    });
}

/// A location in a source file.
#[derive(Copy, Clone)]
pub struct SourceLocation<'tu> {
    raw: CXSourceLocation,
    tu: &'tu TranslationUnit<'tu>,
}

impl<'tu> SourceLocation<'tu> {
    //- Constructors -----------------------------

    #[doc(hidden)]
    pub fn from_raw(raw: CXSourceLocation, tu: &'tu TranslationUnit<'tu>) -> SourceLocation<'tu> {
        SourceLocation { raw, tu }
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
            fn uninit<T>() -> mem::MaybeUninit<T> { mem::MaybeUninit::uninit() }
            let (mut file, mut line, mut column) = (uninit(), uninit(), uninit());
            clang_getPresumedLocation(
                self.raw, file.as_mut_ptr(), line.as_mut_ptr(), column.as_mut_ptr());
            (
                utility::to_string(file.assume_init()),
                line.assume_init() as u32,
                column.assume_init() as u32
            )
        }
    }

    /// Returns the file, line, column and character offset of this source location.
    pub fn get_spelling_location(&self) -> Location<'tu> {
        unsafe { location!(clang_getSpellingLocation, self.raw, self.tu) }
    }

    /// Returns the AST entity at this source location, if any.
    pub fn get_entity(&self) -> Option<Entity<'tu>> {
        unsafe { clang_getCursor(self.tu.ptr, self.raw).map(|c| Entity::from_raw(c, self.tu)) }
    }

    /// Returns whether this source location is in the main file of its translation unit.
    pub fn is_in_main_file(&self) -> bool {
        unsafe { clang_Location_isFromMainFile(self.raw) != 0 }
    }

    /// Returns whether this source location is in a system header.
    pub fn is_in_system_header(&self) -> bool {
        unsafe { clang_Location_isInSystemHeader(self.raw) != 0 }
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

impl<'tu> cmp::PartialEq for SourceLocation<'tu> {
    fn eq(&self, other: &SourceLocation<'tu>) -> bool {
        unsafe { clang_equalLocations(self.raw, other.raw) != 0 }
    }
}

impl<'tu> cmp::Eq for SourceLocation<'tu> { }

impl<'tu> hash::Hash for SourceLocation<'tu> {
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        self.get_spelling_location().hash(hasher)
    }
}

// SourceRange ___________________________________

/// A half-open range in a source file.
#[derive(Copy, Clone)]
pub struct SourceRange<'tu> {
    raw: CXSourceRange,
    tu: &'tu TranslationUnit<'tu>,
}

impl<'tu> SourceRange<'tu> {
    //- Constructors -----------------------------

    #[doc(hidden)]
    pub fn from_raw(raw: CXSourceRange, tu: &'tu TranslationUnit<'tu>) -> SourceRange<'tu> {
        SourceRange { raw, tu }
    }

    /// Constructs a new `SourceRange` that spans [`start`, `end`).
    pub fn new(start: SourceLocation<'tu>, end: SourceLocation<'tu>) -> SourceRange<'tu> {
        unsafe { SourceRange::from_raw(clang_getRange(start.raw, end.raw), start.tu) }
    }

    //- Accessors --------------------------------

    /// Returns the inclusive start of this source range.
    pub fn get_start(&self) -> SourceLocation<'tu> {
        unsafe { SourceLocation::from_raw(clang_getRangeStart(self.raw), self.tu) }
    }

    /// Returns the exclusive end of this source range.
    pub fn get_end(&self) -> SourceLocation<'tu> {
        unsafe { SourceLocation::from_raw(clang_getRangeEnd(self.raw), self.tu) }
    }

    /// Returns whether this source range is in the main file of its translation unit.
    pub fn is_in_main_file(&self) -> bool {
        self.get_start().is_in_main_file()
    }

    /// Returns whether this source range is in a system header.
    pub fn is_in_system_header(&self) -> bool {
        self.get_start().is_in_system_header()
    }

    /// Tokenizes the source code covered by this source range and returns the resulting tokens.
    pub fn tokenize(&self) -> Vec<Token<'tu>> {
        unsafe {
            let (mut raw, mut count) = (mem::MaybeUninit::uninit(), mem::MaybeUninit::uninit());
            clang_tokenize(self.tu.ptr, self.raw, raw.as_mut_ptr(), count.as_mut_ptr());
            let (raw, count) = (raw.assume_init(), count.assume_init());
            let raws = if raw.is_null() {
                &[]
            } else {
                slice::from_raw_parts(raw, count as usize)
            };
            let tokens = raws.iter().map(|t| Token::from_raw(*t, self.tu)).collect();
            if !raw.is_null() {
                clang_disposeTokens(self.tu.ptr, raw, count);
            }
            tokens
        }
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

impl<'tu> cmp::PartialEq for SourceRange<'tu> {
    fn eq(&self, other: &SourceRange<'tu>) -> bool {
        unsafe { clang_equalRanges(self.raw, other.raw) != 0 }
    }
}

impl<'tu> cmp::Eq for SourceRange<'tu> { }

impl<'tu> hash::Hash for SourceRange<'tu> {
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        self.get_start().hash(hasher);
        self.get_end().hash(hasher);
    }
}

//================================================
// Functions
//================================================

fn visit<'tu, F, G>(tu: &'tu TranslationUnit<'tu>, f: F, g: G) -> bool
    where F: FnMut(Entity<'tu>, SourceRange<'tu>) -> bool,
          G: Fn(CXCursorAndRangeVisitor) -> CXResult
{
    trait Callback<'tu> {
        fn call(&mut self, entity: Entity<'tu>, range: SourceRange<'tu>) -> bool;
    }

    impl<'tu, F: FnMut(Entity<'tu>, SourceRange<'tu>) -> bool> Callback<'tu> for F {
        fn call(&mut self, entity: Entity<'tu>, range: SourceRange<'tu>) -> bool {
            self(entity, range)
        }
    }

    extern fn visit(data: CXClientData, cursor: CXCursor, range: CXSourceRange) -> CXVisitorResult {
        unsafe {
            let &mut (tu, ref mut callback):
                &mut (&TranslationUnit, Box<dyn Callback>) =
                    &mut *(data as *mut (&TranslationUnit, Box<dyn Callback>));

            if callback.call(Entity::from_raw(cursor, tu), SourceRange::from_raw(range, tu)) {
                CXVisit_Continue
            } else {
                CXVisit_Break
            }
        }
    }

    let mut data = (tu, Box::new(f) as Box<dyn Callback>);
    let visitor = CXCursorAndRangeVisitor { context: utility::addressof(&mut data), visit: Some(visit) };
    g(visitor) == CXResult_VisitBreak
}

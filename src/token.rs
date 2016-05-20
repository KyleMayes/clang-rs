//! Lexed pieces of source files.

use std::fmt;
use std::mem;

use clang_sys::*;

use utility;
use super::{TranslationUnit};
use super::source::{SourceLocation, SourceRange};

//================================================
// Enums
//================================================

// TokenKind _____________________________________

/// Indicates the categorization of a token.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum TokenKind {
    /// A comment token.
    Comment = 4,
    /// An identifier token.
    Identifier = 2,
    /// A keyword token.
    Keyword = 1,
    /// A literal token.
    Literal = 3,
    /// A puncuation token.
    Punctuation = 0,
}

//================================================
// Structs
//================================================

// Token _________________________________________

/// A lexed piece of a source file.
#[derive(Copy, Clone)]
pub struct Token<'tu> {
    raw: CXToken,
    tu: &'tu TranslationUnit<'tu>,
}

impl<'tu> Token<'tu> {
    //- Constructors -----------------------------

    #[doc(hidden)]
    pub fn from_raw(raw: CXToken, tu: &'tu TranslationUnit<'tu>) -> Token<'tu> {
        Token{ raw: raw, tu: tu }
    }

    //- Accessors --------------------------------

    /// Returns the categorization of this token.
    pub fn get_kind(&self) -> TokenKind {
        unsafe { mem::transmute(clang_getTokenKind(self.raw)) }
    }

    /// Returns the textual representation of this token.
    pub fn get_spelling(&self) -> String {
        unsafe { utility::to_string(clang_getTokenSpelling(self.tu.ptr, self.raw)) }
    }

    /// Returns the source location of this token.
    pub fn get_location(&self) -> SourceLocation<'tu> {
        unsafe { SourceLocation::from_raw(clang_getTokenLocation(self.tu.ptr, self.raw), self.tu) }
    }

    /// Returns the source range of this token.
    pub fn get_range(&self) -> SourceRange<'tu> {
        unsafe { SourceRange::from_raw(clang_getTokenExtent(self.tu.ptr, self.raw), self.tu) }
    }
}

impl<'tu> fmt::Debug for Token<'tu> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.debug_struct("Token")
            .field("kind", &self.get_kind())
            .field("spelling", &self.get_spelling())
            .field("range", &self.get_range())
            .finish()
    }
}

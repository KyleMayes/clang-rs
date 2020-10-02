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

use std::error::{Error};
use std::fmt;

use clang_sys::*;

use libc::{c_longlong};

use utility::{FromError};

//================================================
// Macros
//================================================

macro_rules! error {
    (
        $(#[$meta:meta])*
        pub enum $name:ident: $underlying:ty {
            $(#[$variantdoc:meta] $variant:ident = ($error:pat, $message:expr)), +,
        }
    ) => {
        $(#[$meta])*
        #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
        pub enum $name {
            $(#[$variantdoc] $variant), +
        }

        impl Error for $name { }

        impl From<$name> for String {
            fn from(error: $name) -> String {
                error.to_string()
            }
        }

        impl FromError<$underlying> for $name {
            fn from_error(error: $underlying) -> Result<(), $name> {
                match error {
                    $($error => Err($name::$variant)), +,
                    _ => Ok(()),
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match *self {
                    $($name::$variant => write!(f, $message)), +
                }
            }
        }
    };
}

//================================================
// Enums
//================================================

// AlignofError __________________________________

error! {
    /// Indicates the error that prevented determining the alignment of a type.
    pub enum AlignofError: c_longlong {
        /// The type is a dependent type.
        Dependent = (-3, "the type is a dependent type"),
        /// The type is an incomplete type.
        Incomplete = (-2, "the type is an incomplete type"),
    }
}

// OffsetofError _________________________________

error! {
    /// Indicates the error that prevented determining the offset of a field in a record type.
    pub enum OffsetofError: c_longlong {
        /// The record type is a dependent type.
        Dependent = (-3, "the record type is a dependent type"),
        /// The record type is an incomplete type.
        Incomplete = (-2, "the record type is an incomplete type"),
        /// The record type does not contain a field with the supplied name.
        Name = (-5, "the record type does not contain a field with the supplied name"),
        /// The record type has an invalid parent declaration.
        Parent = (-1, "the record type has an invalid parent declaration"),
        /// The type is undeduced.
        Undeduced = (-6, "the type is undeduced"),
    }
}

// SaveError _____________________________________

error! {
    /// Indicates the type of error that prevented the saving of a translation unit to an AST file.
    pub enum SaveError: CXSaveError {
        /// Errors in the translation unit prevented saving.
        Errors = (CXSaveError_InvalidTU, "errors in the translation unit prevented saving"),
        /// An unknown error occurred.
        Unknown = (CXSaveError_Unknown, "an unknown error occurred"),
    }
}

// SizeofError ___________________________________

error! {
    /// Indicates the error that prevented determining the size of a type.
    pub enum SizeofError: c_longlong {
        /// The type is a dependent type.
        Dependent = (-3, "the type is a dependent type"),
        /// The type is an incomplete type.
        Incomplete = (-2, "the type is an incomplete type"),
        /// The type is a variable size type.
        VariableSize = (-4, "the type is a variable size type"),
        /// The supplied field name was invalid.
        InvalidFieldName = (-5, "invalid field name"),
    }
}

// SourceError ___________________________________

error! {
    /// Indicates the type of error that prevented the loading of a translation unit from a source
    /// file.
    pub enum SourceError: CXErrorCode {
        /// An error occurred while deserializing an AST file.
        AstDeserialization = (CXError_ASTReadError, "AST deserialization failed"),
        /// `libclang` crashed.
        Crash = (CXError_Crashed, "`libclang` crashed"),
        /// An unknown error occurred.
        Unknown = (CXError_Failure, "an unknown error occurred"),
    }
}

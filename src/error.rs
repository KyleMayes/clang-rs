use std::error::{Error};
use std::fmt;

//================================================
// Macros
//================================================

macro_rules! error {
    (
        $(#[$meta:meta])*
        pub enum $name:ident {
            $(#[$variantdoc:meta] $variant:ident = $message:expr), +,
        }
    ) => {
        $(#[$meta])*
        #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
        pub enum $name {
            $(#[$variantdoc] $variant), +
        }

        impl Error for $name {
            fn description(&self) -> &str {
                match *self {
                    $($name::$variant => $message), +
                }
            }
        }

        impl From<$name> for String {
            fn from(error: $name) -> String {
                error.description().into()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "{}", self.description())
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
    pub enum AlignofError {
        /// The type is a dependent type.
        Dependent = "the type is a dependent type",
        /// The type is an incomplete type.
        Incomplete = "the type is an incomplete type",
    }
}

// OffsetofError _________________________________

error! {
    /// Indicates the error that prevented determining the offset of a field in a record type.
    pub enum OffsetofError {
        /// The record type is a dependent type.
        Dependent = "the record type is a dependent type",
        /// The record type is an incomplete type.
        Incomplete = "the record type is an incomplete type",
        /// The record type does not contain a field with the supplied name.
        Name = "the record type does not contain a field with the supplied name",
        /// The record type has an invalid parent declaration.
        Parent = "the record type has an invalid parent declaration",
    }
}

// SaveError _____________________________________

error! {
    /// Indicates the type of error that prevented the saving of a translation unit to an AST file.
    pub enum SaveError {
        /// Errors in the translation unit prevented saving.
        Errors = "errors in the translation unit prevented saving",
        /// An unknown error occurred.
        Unknown = "an unknown error occurred",
    }
}

// SizeofError ___________________________________

error! {
    /// Indicates the error that prevented determining the size of a type.
    pub enum SizeofError {
        /// The type is a dependent type.
        Dependent = "the type is a dependent type",
        /// The type is an incomplete type.
        Incomplete = "the type is an incomplete type",
        /// The type is a variable size type.
        VariableSize = "the type is a variable size type",
    }
}

// SourceError ___________________________________

error! {
    /// Indicates the type of error that prevented the loading of a translation unit from a source
    /// file.
    pub enum SourceError {
        /// An error occurred while deserializing an AST file.
        AstDeserialization = "an error occurred while deserializing an AST file",
        /// `libclang` crashed.
        Crash = "`libclang` crashed",
        /// An unknown error occurred.
        Unknown = "an unknown error occurred",
    }
}

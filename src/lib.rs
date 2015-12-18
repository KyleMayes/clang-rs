//! Bindings and idiomatic wrapper for `libclang`.

#![warn(missing_docs)]

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![cfg_attr(feature="clippy", warn(clippy))]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;

extern crate libc;

use std::fmt;
use std::hash;
use std::mem;
use std::ptr;
use std::slice;
use std::cmp::{self, Ordering};
use std::collections::{HashMap};
use std::marker::{PhantomData};
use std::path::{Path, PathBuf};
use std::rc::{Rc};
use std::sync::atomic::{self, AtomicBool};

use libc::{c_int, c_uint, c_ulong, time_t};

pub mod ffi;

/// A Unified Symbol Resolution (USR).
///
/// A USR identifies an AST entity and can be used to compare AST entities from different
/// translation units.
pub type Usr = String;

//================================================
// Macros
//================================================

// iter! _________________________________________

macro_rules! iter {
    ($num:ident($($num_argument:expr), *), $get:ident($($get_argument:expr), *)) => ({
        let count = unsafe { ffi::$num($($num_argument), *) };
        (0..count).map(|i| unsafe { ffi::$get($($get_argument), *, i) })
    });

    ($num:ident($($num_argument:expr), *), $get:ident($($get_argument:expr), *),) => ({
        iter!($num($($num_argument), *), $get($($get_argument), *))
    });

    ($num:ident($($num_argument:expr), *), $($get:ident($($get_argument:expr), *)), *,) => ({
        let count = unsafe { ffi::$num($($num_argument), *) };
        (0..count).map(|i| unsafe { ($(ffi::$get($($get_argument), *, i)), *) })
    });
}

// iter_option! __________________________________

macro_rules! iter_option {
    ($num:ident($($num_argument:expr), *), $get:ident($($get_argument:expr), *)) => ({
        let count = unsafe { ffi::$num($($num_argument), *) };

        if count >= 0 {
            Some((0..count).map(|i| unsafe { ffi::$get($($get_argument), *, i as c_uint) }))
        } else {
            None
        }
    });

    ($num:ident($($num_argument:expr), *), $get:ident($($get_argument:expr), *),) => ({
        iter_option!($num($($num_argument), *), $get($($get_argument), *))
    });
}

// options! ______________________________________

macro_rules! options {
    ($(#[$attribute:meta])* options $name:ident: $underlying:ident {
        $($(#[$fattribute:meta])* pub $option:ident: $flag:ident), +,
    }) => (
        $(#[$attribute])*
        #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
        pub struct $name {
            $($(#[$fattribute])* pub $option: bool), +,
        }

        impl From<ffi::$underlying> for $name {
            fn from(flags: ffi::$underlying) -> $name {
                $name { $($option: flags.contains(ffi::$flag)), + }
            }
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
    /// Transforms this value into an `Option<U>`, mapping a null value to `None` and a non-null
    /// value to `Some(v)` where `v` is the result of applying the supplied function to this value.
    fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Option<U>;
}

//================================================
// Enums
//================================================

// Accessibility _________________________________

/// Indicates the accessibility of a declaration or base class specifier.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum Accessibility {
    /// The declaration or base class specifier is private.
    Private = 3,
    /// The declaration or base class specifier is protected.
    Protected = 2,
    /// The declaration or base class specifier is public.
    Public = 1,
}

// AlignofError __________________________________

/// Indicates the error that prevented determining the alignment of a type.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum AlignofError {
    /// The type is a dependent type.
    Dependent,
    /// The type is an incomplete type.
    Incomplete,
}

// Availability __________________________________

/// Indicates the availability of an AST entity.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum Availability {
    /// The entity is available.
    Available = 0,
    /// The entity is available but has been deprecated and any usage of it will be a warning.
    Deprecated = 1,
    /// The entity is not available and any usage of it will be an error.
    Unavailable = 2,
    /// The entity is available but is not accessible and any usage of it will be an error.
    Inaccessible = 3,
}

// CallingConvention _____________________________

/// Indicates the calling convention specified for a function type.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum CallingConvention {
    /// The function type uses the x86 `cdecl` calling convention.
    Cdecl = 1,
    /// The function type uses the x86 `fastcall` calling convention.
    Fastcall = 3,
    /// The function type uses the x86 `pascal` calling convention.
    Pascal = 5,
    /// The function type uses the x86 `stdcall` calling convention.
    Stdcall = 2,
    /// The function type uses the x86 `thiscall` calling convention.
    Thiscall = 4,
    /// The function type uses the x86 `vectorcall` calling convention.
    Vectorcall = 12,
    /// The function type uses the ARM AACPS calling convention.
    Aapcs = 6,
    /// The function type uses the ARM AACPS-VFP calling convention.
    AapcsVfp = 7,
    /// The function type uses the calling convention for Intel OpenCL built-ins.
    IntelOcl = 9,
    /// The function type uses the x64 C calling convention as specified in the System V ABI.
    SysV64 = 11,
    /// The function type uses the x64 C calling convention as implemented on Windows.
    Win64 = 10,
    /// The function type uses a calling convention that is not exposed via this interface.
    Unexposed = 200,
}

// CompletionChunk _______________________________

/// A piece of a code completion string.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompletionChunk<'r> {
    /// An optional piece that could be part of the template but is not required.
    Optional(CompletionString<'r>),
    /// Text that describes the current parameter when code completion was run on a function call,
    /// message send, or template specialization.
    CurrentParameter(String),
    /// Informative text that should be displayed but not inserted as part of the template.
    Informative(String),
    /// Text that should be replaced by the user.
    Placeholder(String),
    /// Text that specifies the result type of the containing result.
    ResultType(String),
    /// Text that the user would be expected to type to get the containing code completion result.
    TypedText(String),
    /// Text that should be inserted.
    Text(String),
    /// A colon (`':'`).
    Colon(String),
    /// A comma (`','`).
    Comma(String),
    /// An equals sign (`'='`).
    Equals(String),
    /// A semicolon (`';'`).
    Semicolon(String),
    /// A left angle bracket (`'<'`).
    LeftAngleBracket(String),
    /// A right angle bracket (`'>'`).
    RightAngleBracket(String),
    /// A left brace (`'{'`).
    LeftBrace(String),
    /// A right brace (`'}'`).
    RightBrace(String),
    /// A left parentesis (`'('`)).
    LeftParenthesis(String),
    /// A right parenthesis (`')'`).
    RightParenthesis(String),
    /// A left square bracket (`'['`).
    LeftSquareBracket(String),
    /// A right square bracket (`']'`).
    RightSquareBracket(String),
    /// Horizontal space (e.g., `' '`).
    HorizontalSpace(String),
    /// Vertical space (e.g., `'\n'`).
    VerticalSpace(String),
}

impl<'r> CompletionChunk<'r> {
    //- Accessors --------------------------------

    /// Returns the text associated with this completion chunk.
    ///
    /// # Panics
    ///
    /// * this completion chunk is a `CompletionChunk::Optional`
    pub fn get_text(&self) -> String {
        match *self {
            CompletionChunk::Optional(_) => {
                panic!("this completion chunk is a `CompletionChunk::Optional`")
            },
            CompletionChunk::CurrentParameter(ref text) => text.clone(),
            CompletionChunk::Informative(ref text) => text.clone(),
            CompletionChunk::Placeholder(ref text) => text.clone(),
            CompletionChunk::ResultType(ref text) => text.clone(),
            CompletionChunk::TypedText(ref text) => text.clone(),
            CompletionChunk::Text(ref text) => text.clone(),
            CompletionChunk::Colon(ref text) => text.clone(),
            CompletionChunk::Comma(ref text) => text.clone(),
            CompletionChunk::Equals(ref text) => text.clone(),
            CompletionChunk::Semicolon(ref text) => text.clone(),
            CompletionChunk::LeftAngleBracket(ref text) => text.clone(),
            CompletionChunk::RightAngleBracket(ref text) => text.clone(),
            CompletionChunk::LeftBrace(ref text) => text.clone(),
            CompletionChunk::RightBrace(ref text) => text.clone(),
            CompletionChunk::LeftParenthesis(ref text) => text.clone(),
            CompletionChunk::RightParenthesis(ref text) => text.clone(),
            CompletionChunk::LeftSquareBracket(ref text) => text.clone(),
            CompletionChunk::RightSquareBracket(ref text) => text.clone(),
            CompletionChunk::HorizontalSpace(ref text) => text.clone(),
            CompletionChunk::VerticalSpace(ref text) => text.clone(),
        }
    }
}

// EntityKind ____________________________________

/// Indicates the categorization of an AST entity.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum EntityKind {
    /// A declaration whose specific type is not exposed via this interface.
    UnexposedDecl = 1,
    /// A C or C++ struct.
    StructDecl = 2,
    /// A C or C++ union.
    UnionDecl = 3,
    /// A C++ class.
    ClassDecl = 4,
    /// An enum.
    EnumDecl = 5,
    /// A C field or C++ non-static data member in a struct, union, or class.
    FieldDecl = 6,
    /// An enum constant.
    EnumConstantDecl = 7,
    /// A function.
    FunctionDecl = 8,
    /// A variable.
    VarDecl = 9,
    /// A parameter.
    ParmDecl = 10,
    /// An Objective-C `@interface`.
    ObjCInterfaceDecl = 11,
    /// An Objective-C `@interface` for a category.
    ObjCCategoryDecl = 12,
    /// An Objective-C `@protocol` declaration.
    ObjCProtocolDecl = 13,
    /// An Objective-C `@property` declaration.
    ObjCPropertyDecl = 14,
    /// An Objective-C instance variable.
    ObjCIvarDecl = 15,
    /// An Objective-C instance method.
    ObjCInstanceMethodDecl = 16,
    /// An Objective-C class method.
    ObjCClassMethodDecl = 17,
    /// An Objective-C `@implementation`.
    ObjCImplementationDecl = 18,
    /// An Objective-C `@implementation` for a category.
    ObjCCategoryImplDecl = 19,
    /// A typedef.
    TypedefDecl = 20,
    /// A C++ method.
    Method = 21,
    /// A C++ namespace.
    Namespace = 22,
    /// A linkage specification (e.g., `extern "C"`).
    LinkageSpec = 23,
    /// A C++ constructor.
    Constructor = 24,
    /// A C++ destructor.
    Destructor = 25,
    /// A C++ conversion function.
    ConversionFunction = 26,
    /// A C++ template type parameter.
    TemplateTypeParameter = 27,
    /// A C++ template non-type parameter.
    NonTypeTemplateParameter = 28,
    /// A C++ template template parameter.
    TemplateTemplateParameter = 29,
    /// A C++ function template.
    FunctionTemplate = 30,
    /// A C++ class template.
    ClassTemplate = 31,
    /// A C++ class template partial specialization.
    ClassTemplatePartialSpecialization = 32,
    /// A C++ namespace alias declaration.
    NamespaceAlias = 33,
    /// A C++ using directive.
    UsingDirective = 34,
    /// A C++ using declaration.
    UsingDeclaration = 35,
    /// A C++ type alias declaration.
    TypeAliasDecl = 36,
    /// An Objective-C `@synthesize` definition.
    ObjCSynthesizeDecl = 37,
    /// An Objective-C `@dynamic` definition.
    ObjCDynamicDecl = 38,
    /// An access specifier.
    AccessSpecifier = 39,
    /// A reference to a super class in Objective-C.
    ObjCSuperClassRef = 40,
    /// A reference to a protocol in Objective-C.
    ObjCProtocolRef = 41,
    /// A reference to a class in Objective-C.
    ObjCClassRef = 42,
    /// A reference to a type declaration.
    TypeRef = 43,
    /// A base class specifier.
    BaseSpecifier = 44,
    /// A reference to a class template, function template, template template parameter, or class
    /// template partial specialization.
    TemplateRef = 45,
    /// A reference to a namespace or namespace alias.
    NamespaceRef = 46,
    /// A reference to a member of a struct, union, or class that occurs in some non-expression
    /// context.
    MemberRef = 47,
    /// A reference to a labeled statement.
    LabelRef = 48,
    /// A reference to a set of overloaded functions or function templates that has not yet been
    /// resolved to a specific function or function template.
    OverloadedDeclRef = 49,
    /// A reference to a variable that occurs in some non-expression context.
    VariableRef = 50,
    /// An expression whose specific kind is not exposed via this interface.
    UnexposedExpr = 100,
    /// An expression that refers to some value declaration, such as a function or enumerator.
    DeclRefExpr = 101,
    /// An expression that refers to the member of a struct, union, or class.
    MemberRefExpr = 102,
    /// An expression that calls a function.
    CallExpr = 103,
    /// An expression that sends a message to an Objective-C object or class.
    ObjCMessageExpr = 104,
    /// An expression that represents a block literal.
    BlockExpr = 105,
    /// An integer literal.
    IntegerLiteral = 106,
    /// A floating point number literal.
    FloatingLiteral = 107,
    /// An imaginary number literal.
    ImaginaryLiteral = 108,
    /// A string literal.
    StringLiteral = 109,
    /// A character literal.
    CharacterLiteral = 110,
    /// A parenthesized expression.
    ParenExpr = 111,
    /// Any unary expression other than `sizeof` and `alignof`.
    UnaryOperator = 112,
    /// An array subscript expression (`[C99 6.5.2.1]`).
    ArraySubscriptExpr = 113,
    /// A built-in binary expression (e.g., `x + y`).
    BinaryOperator = 114,
    /// A compound assignment expression (e.g., `x += y`).
    CompoundAssignOperator = 115,
    /// A ternary expression.
    ConditionalOperator = 116,
    /// An explicit cast in C or a C-style cast in C++.
    CStyleCastExpr = 117,
    /// A compound literal expression (`[C99 6.5.2.5]`).
    CompoundLiteralExpr = 118,
    /// A C or C++ initializer list.
    InitListExpr = 119,
    /// A GNU address of label expression.
    AddrLabelExpr = 120,
    /// A GNU statement expression.
    StmtExpr = 121,
    /// A C11 generic selection expression.
    GenericSelectionExpr = 122,
    /// A GNU `__null` expression.
    GNUNullExpr = 123,
    /// A C++ `static_cast<>` expression.
    StaticCastExpr = 124,
    /// A C++ `dynamic_cast<>` expression.
    DynamicCastExpr = 125,
    /// A C++ `reinterpret_cast<>` expression.
    ReinterpretCastExpr = 126,
    /// A C++ `const_cast<>` expression.
    ConstCastExpr = 127,
    /// A C++ cast that uses "function" notation (e.g., `int(0.5)`).
    FunctionalCastExpr = 128,
    /// A C++ `typeid` expression.
    TypeidExpr = 129,
    /// A C++ boolean literal.
    BoolLiteralExpr = 130,
    /// A C++ `nullptr` exrepssion.
    NullPtrLiteralExpr = 131,
    /// A C++ `this` expression.
    ThisExpr = 132,
    /// A C++ `throw` expression.
    ThrowExpr = 133,
    /// A C++ `new` expression.
    NewExpr = 134,
    /// A C++ `delete` expression.
    DeleteExpr = 135,
    /// A unary expression.
    UnaryExpr = 136,
    /// An Objective-C string literal.
    ObjCStringLiteral = 137,
    /// An Objective-C `@encode` expression.
    ObjCEncodeExpr = 138,
    /// An Objective-C `@selector` expression.
    ObjCSelectorExpr = 139,
    /// An Objective-C `@protocol` expression.
    ObjCProtocolExpr = 140,
    /// An Objective-C bridged cast expression.
    ObjCBridgedCastExpr = 141,
    /// A C++11 parameter pack expansion expression.
    PackExpansionExpr = 142,
    /// A C++11 `sizeof...` expression.
    SizeOfPackExpr = 143,
    /// A C++11 lambda expression.
    LambdaExpr = 144,
    /// An Objective-C boolean literal.
    ObjCBoolLiteralExpr = 145,
    /// An Objective-C `self` expression.
    ObjCSelfExpr = 146,
    /// A statement whose specific kind is not exposed via this interface.
    UnexposedStmt = 200,
    /// A labelled statement in a function.
    LabelStmt = 201,
    /// A group of statements (e.g., a function body).
    CompoundStmt = 202,
    /// A `case` statement.
    CaseStmt = 203,
    /// A `default` statement.
    DefaultStmt = 204,
    /// An `if` statement.
    IfStmt = 205,
    /// A `switch` statement.
    SwitchStmt = 206,
    /// A `while` statement.
    WhileStmt = 207,
    /// A `do` statement.
    DoStmt = 208,
    /// A `for` statement.
    ForStmt = 209,
    /// A `goto` statement.
    GotoStmt = 210,
    /// An indirect `goto` statement.
    IndirectGotoStmt = 211,
    /// A `continue` statement.
    ContinueStmt = 212,
    /// A `break` statement.
    BreakStmt = 213,
    /// A `return` statement.
    ReturnStmt = 214,
    /// An inline assembly statement.
    AsmStmt = 215,
    /// An Objective-C `@try`-`@catch`-`@finally` statement.
    ObjCAtTryStmt = 216,
    /// An Objective-C `@catch` statement.
    ObjCAtCatchStmt = 217,
    /// An Objective-C `@finally` statement.
    ObjCAtFinallyStmt = 218,
    /// An Objective-C `@throw` statement.
    ObjCAtThrowStmt = 219,
    /// An Objective-C `@synchronized` statement.
    ObjCAtSynchronizedStmt = 220,
    /// An Objective-C autorelease pool statement.
    ObjCAutoreleasePoolStmt = 221,
    /// An Objective-C collection statement.
    ObjCForCollectionStmt = 222,
    /// A C++ catch statement.
    CatchStmt = 223,
    /// A C++ try statement.
    TryStmt = 224,
    /// A C++11 range-based for statement.
    ForRangeStmt = 225,
    /// A Windows Structured Exception Handling `__try` statement.
    SehTryStmt = 226,
    /// A Windows Structured Exception Handling `__except` statement.
    SehExceptStmt = 227,
    /// A Windows Structured Exception Handling `__finally` statement.
    SehFinallyStmt = 228,
    /// A Windows Structured Exception Handling `__leave` statement.
    SehLeaveStmt = 247,
    /// A Microsoft inline assembly statement.
    MsAsmStmt = 229,
    /// A null statement.
    NullStmt = 230,
    /// An adaptor for mixing declarations with statements and expressions.
    DeclStmt = 231,
    /// An OpenMP parallel directive.
    OmpParallelDirective = 232,
    /// An OpenMP SIMD directive.
    OmpSimdDirective = 233,
    /// An OpenMP for directive.
    OmpForDirective = 234,
    /// An OpenMP sections directive.
    OmpSectionsDirective = 235,
    /// An OpenMP section directive.
    OmpSectionDirective = 236,
    /// An OpenMP single directive.
    OmpSingleDirective = 237,
    /// An OpenMP parallel for directive.
    OmpParallelForDirective = 238,
    /// An OpenMP parallel sections directive.
    OmpParallelSectionsDirective = 239,
    /// An OpenMP task directive.
    OmpTaskDirective = 240,
    /// An OpenMP master directive.
    OmpMasterDirective = 241,
    /// An OpenMP critical directive.
    OmpCriticalDirective = 242,
    /// An OpenMP taskyield directive.
    OmpTaskyieldDirective = 243,
    /// An OpenMP barrier directive.
    OmpBarrierDirective = 244,
    /// An OpenMP taskwait directive.
    OmpTaskwaitDirective = 245,
    /// An OpenMP flush directive.
    OmpFlushDirective = 246,
    /// An OpenMP ordered directive.
    OmpOrderedDirective = 248,
    /// An OpenMP atomic directive.
    OmpAtomicDirective = 249,
    /// An OpenMP for SIMD directive.
    OmpForSimdDirective = 250,
    /// An OpenMP parallel for SIMD directive.
    OmpParallelForSimdDirective = 251,
    /// An OpenMP target directive.
    OmpTargetDirective = 252,
    /// An OpenMP teams directive.
    OmpTeamsDirective = 253,
    /// An OpenMP taskgroup directive.
    OmpTaskgroupDirective = 254,
    /// An OpenMP cancellation point directive.
    OmpCancellationPointDirective = 255,
    /// An OpenMP cancel directive.
    OmpCancelDirective = 256,
    /// The top-level AST entity which acts as the root for the other entitys.
    TranslationUnit = 300,
    /// An attribute whose specific kind is not exposed via this interface.
    UnexposedAttr = 400,
    /// An attribute applied to an Objective-C IBAction.
    IbActionAttr = 401,
    /// An attribute applied to an Objective-C IBOutlet.
    IbOutletAttr = 402,
    /// An attribute applied to an Objective-C IBOutletCollection.
    IbOutletCollectionAttr = 403,
    /// The `final` attribute.
    FinalAttr = 404,
    /// The `override` attribute.
    OverrideAttr = 405,
    /// An annotation attribute.
    AnnotateAttr = 406,
    /// An ASM label attribute.
    AsmLabelAttr = 407,
    /// An attribute that requests for packed records (e.g., `__attribute__ ((__packed__))`).
    PackedAttr = 408,
    /// An attribute that asserts a function has no side effects (e.g., `__attribute__((pure))`).
    PureAttr = 409,
    /// The `const` attribute.
    ConstAttr = 410,
    /// An attribute that allows calls to a function to be duplicated by the optimized
    /// (e.g., `__attribute__((noduplicate))`).
    NoDuplicateAttr = 411,
    /// A CUDA constant attribute.
    CudaConstantAttr = 412,
    /// A CUDA device attribute.
    CudaDeviceAttr = 413,
    /// A CUDA global attribute.
    CudaGlobalAttr = 414,
    /// A CUDA host attribute.
    CudaHostAttr = 415,
    /// A CUDA shared attribute.
    CudaSharedAttr = 416,
    /// A preprocessing directive.
    PreprocessingDirective = 500,
    /// A macro definition.
    MacroDefinition = 501,
    /// A macro expansion.
    MacroExpansion = 502,
    /// An inclusion directive.
    InclusionDirective = 503,
    /// A module import declaration.
    ModuleImportDecl = 600,
    /// A single overload in a set of overloads.
    OverloadCandidate = 700,
}

// EntityVisitResult _____________________________

/// Indicates how a entity visitation should proceed.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum EntityVisitResult {
    /// Do not continue visiting entities.
    Break = 0,
    /// Continue visiting sibling entities iteratively, skipping child entities.
    Continue = 1,
    /// Continue visiting sibling and child entities recursively, children first.
    Recurse = 2,
}

// FixIt _________________________________________

/// A suggested fix for an issue with a source file.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum FixIt<'tu> {
    /// Delete a segment of the source file.
    Deletion(SourceRange<'tu>),
    /// Insert a string into the source file.
    Insertion(SourceLocation<'tu>, String),
    /// Replace a segment of the source file with a string.
    Replacement(SourceRange<'tu>, String),
}

// Language ______________________________________

/// Indicates the language used by a declaration.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum Language {
    /// The entity uses the C programming language.
    C = 1,
    /// The entity uses the C++ programming language.
    Cpp = 3,
    /// The entity uses the Objective-C programming language.
    ObjectiveC = 2,
}

// Linkage _______________________________________

/// Indicates the linkage of an AST entity.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum Linkage {
    /// The AST entity has automatic storage (e.g., variables or parameters).
    Automatic = 1,
    /// The AST entity is a static variable or static function.
    Internal = 2,
    /// The AST entity has external linkage.
    External = 4,
    /// The AST entity has external linkage and lives in a C++ anonymous namespace.
    UniqueExternal = 3,
}

// MemoryUsage ___________________________________

/// Indicates the usage category of a quantity of memory.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum MemoryUsage {
    /// Expressions, declarations, and types.
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

// OffsetofError _________________________________

/// Indicates the error that prevented determining the offset of a field in a record type.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum OffsetofError {
    /// The record type is a dependent type.
    Dependent,
    /// The record type is an incomplete type.
    Incomplete,
    /// The record type does not contain a field with the supplied name.
    Name,
    /// The record type has an invalid parent declaration.
    Parent,
}

// RefQualifier __________________________________

/// Indicates the ref qualifier of a C++ function or method type.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum RefQualifier {
    /// The function or method has an l-value ref qualifier (`&`).
    LValue = 1,
    /// The function or method has an r-value ref qualifier (`&&`).
    RValue = 2,
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

// Severity ______________________________________

/// Indicates the severity of a diagnostic.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum Severity {
    /// The diagnostic has been suppressed (e.g., by a command-line option).
    Ignored = 0,
    /// The diagnostic is attached to the previous non-note diagnostic.
    Note = 1,
    /// The diagnostic targets suspicious code that may or may not be wrong.
    Warning = 2,
    /// The diagnostic targets ill-formed code.
    Error = 3,
    /// The diagnostic targets code that is ill-formed in such a way that parser recovery is
    /// unlikely to produce any useful results.
    Fatal = 4,
}

// SizeofError ___________________________________

/// Indicates the error that prevented determining the size of a type.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum SizeofError {
    /// The type is a dependent type.
    Dependent,
    /// The type is an incomplete type.
    Incomplete,
    /// The type is a variable size type.
    VariableSize,
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

// StorageClass __________________________________

/// Indicates the storage class of a declaration.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum StorageClass {
    /// The declaration does not specifiy a storage duration and therefore has an automatic storage
    /// duration.
    None = 1,
    /// The declaration specifies a static storage duration and external linkage.
    Extern = 2,
    /// The declaration specifies a static storage duration and internal linkage.
    Static = 3,
    /// The declaration specifies a static storage duration and external linkage but is not
    /// accessible outside the containing translation unit.
    PrivateExtern = 4,
    /// The declaration specifies a storage duration related to an OpenCL work group.
    OpenClWorkGroupLocal = 5,
    /// The declaration specifies an automatic storage duration.
    Auto = 6,
    /// The declaration specifies that it should be stored in a CPU register and have an automatic
    /// storage duration.
    Register = 7,
}

// TemplateArgument ______________________________

/// An argument to a template function specialization.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TemplateArgument<'tu> {
    /// An empty template argument (e.g., one that has not yet been deduced).
    Null,
    /// A type.
    Type(Type<'tu>),
    /// A declaration for a pointer, reference, or member pointer non-type template parameter.
    Declaration,
    /// A null pointer or null member pointer provided for a non-type template parameter.
    Nullptr,
    /// An integer.
    Integral(i64, u64),
    /// A name for a template provided for a template template parameter.
    Template,
    /// A pack expansion of a name for a template provided for a template template parameter.
    TemplateExpansion,
    /// An expression that has not yet been resolved
    Expression,
    /// A parameter pack.
    Pack,
}

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

// TypeKind ______________________________________

/// Indicates the categorization of a type.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum TypeKind {
    /// A type whose specific kind is not exposed via this interface.
    Unexposed = 1,
    /// `void`
    Void = 2,
    /// `bool` (C++) or `_Bool` (C99)
    Bool = 3,
    /// The `char` type when it is signed by default.
    CharS = 13,
    /// The `char` type when it is unsigned by default.
    CharU = 4,
    /// `signed char`
    SChar = 14,
    /// `unsigned char`
    UChar = 5,
    /// `wchar_t`
    WChar = 15,
    /// `char16_t`
    Char16 = 6,
    /// `char32_t`
    Char32 = 7,
    /// `short`
    Short = 16,
    /// `unsigned short`
    UShort = 8,
    /// `int`
    Int = 17,
    /// `unsigned int`
    UInt = 9,
    /// `long`
    Long = 18,
    /// `unsigned long`
    ULong = 10,
    /// `long long`
    LongLong = 19,
    /// `unsigned long long`
    ULongLong = 11,
    /// `__int128_t`
    Int128 = 20,
    /// `__uint128_t`
    UInt128 = 12,
    /// `float`
    Float = 21,
    /// `double`
    Double = 22,
    /// `long double`
    LongDouble = 23,
    /// `nullptr_t` (C++11)
    Nullptr = 24,
    /// A C99 complex type (e.g., `_Complex float`).
    Complex = 100,
    /// An unknown dependent type.
    Dependent = 26,
    /// The type of an unresolved overload set.
    Overload = 25,
    /// `id` (Objective-C)
    ObjCId = 27,
    /// `Class` (Objective-C)
    ObjCClass = 28,
    /// `SEL` (Objective-C)
    ObjCSel = 29,
    /// An Objective-C interface type.
    ObjCInterface = 108,
    /// An Objective-C pointer to object type.
    ObjCObjectPointer = 109,
    /// A pointer type.
    Pointer = 101,
    /// A block pointer type (e.g., `void (^)(int)`).
    BlockPointer = 102,
    /// A pointer to a record member type.
    MemberPointer = 117,
    /// An l-value reference (e.g. `int&`).
    LValueReference = 103,
    /// An r-value reference (e.g. `int&&`).
    RValueReference = 104,
    /// An enum type.
    Enum = 106,
    /// A record type such as a struct or a class.
    Record = 105,
    /// A typedef.
    Typedef = 107,
    /// A function prototype with parameter type information (e.g., `void foo(int)`).
    FunctionPrototype = 111,
    /// A function prototype without parameter type information (e.g., `void foo()`).
    FunctionNoPrototype = 110,
    /// An array type with a specified size that is an integer constant expression.
    ConstantArray = 112,
    /// An array type with a specified size that is a dependent value.
    DependentSizedArray = 116,
    /// An array type without a specified size.
    IncompleteArray = 114,
    /// An array type with a specified size that is not an integer constant expression.
    VariableArray = 115,
    /// A GCC generic vector type.
    Vector = 113,
}

//================================================
// Structs
//================================================

// Clang _________________________________________

lazy_static! { static ref AVAILABLE: AtomicBool = AtomicBool::new(true); }

/// An empty type which prevents the use of this library from multiple threads simultaneously.
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
        if AVAILABLE.swap(false, atomic::Ordering::Relaxed) {
            Ok(Clang)
        } else {
            Err(())
        }
    }

    //- Static -----------------------------------

    /// Returns the version string for the version of `libclang` in use.
    pub fn get_version() -> String {
        unsafe { to_string(ffi::clang_getClangVersion()) }
    }
}

impl Drop for Clang {
    fn drop(&mut self) {
        AVAILABLE.store(true, atomic::Ordering::Relaxed);
    }
}

// CompilationDatabase ___________________________

/// The information used to compile the source files in a project.
pub struct CompilationDatabase {
    ptr: ffi::CXCompilationDatabase,
}

impl CompilationDatabase {
    //- Constructors -----------------------------

    /// Constructs a new `CompilationDatabase` from a directory containing a `compile_commands.json`
    /// file.
    pub fn from_directory<D: AsRef<Path>>(directory: D) -> Result<CompilationDatabase, ()> {
        unsafe {
            let mut code = mem::uninitialized();

            let ptr = ffi::clang_CompilationDatabase_fromDirectory(
                from_path(directory).as_ptr(), &mut code
            );

            if code == ffi::CXCompilationDatabase_Error::NoError {
                Ok(CompilationDatabase { ptr: ptr })
            } else {
                Err(())
            }
        }
    }

    //- Accessors --------------------------------

    fn get_commands_(&self, ptr: ffi::CXCompileCommands) -> Vec<CompileCommand> {
        if !ptr.0.is_null() {
            let commands = CompileCommands::from_ptr(ptr);

            iter!(
                clang_CompileCommands_getSize(commands.ptr),
                clang_CompileCommands_getCommand(commands.ptr),
            ).map(|c| CompileCommand::from_ptr(c, commands.clone())).collect()
        } else {
            vec![]
        }
    }

    /// Returns all the compilation commands in this compilation database.
    pub fn get_all_commands(&self) -> Vec<CompileCommand> {
        unsafe {
            let ptr = ffi::clang_CompilationDatabase_getAllCompileCommands(self.ptr);
            self.get_commands_(ptr)
        }
    }

    /// Returns all the compilation commands for the supplied source file in this compilation
    /// database.
    pub fn get_commands<F: AsRef<Path>>(&self, file: F) -> Vec<CompileCommand> {
        unsafe {
            let ptr = ffi::clang_CompilationDatabase_getCompileCommands(
                self.ptr, from_path(file).as_ptr()
            );

            self.get_commands_(ptr)
        }
    }
}

impl Drop for CompilationDatabase {
    fn drop(&mut self) {
        unsafe { ffi::clang_CompilationDatabase_dispose(self.ptr); }
    }
}

// CompileCommand ________________________________

/// The information used to compile a source file in a project.
#[derive(Clone)]
pub struct CompileCommand<'d> {
    ptr: ffi::CXCompileCommand,
    parent: Rc<CompileCommands>,
    _marker: PhantomData<&'d CompilationDatabase>,
}

impl<'d> CompileCommand<'d> {
    //- Constructors -----------------------------

    fn from_ptr(ptr: ffi::CXCompileCommand, parent: Rc<CompileCommands>) -> CompileCommand<'d> {
        CompileCommand { ptr: ptr, parent: parent, _marker: PhantomData }
    }

    //- Accessors --------------------------------

    /// Returns the arguments in the compiler invocation for this compile command.
    pub fn get_arguments(&self) -> Vec<String> {
        iter!(
            clang_CompileCommand_getNumArgs(self.ptr),
            clang_CompileCommand_getArg(self.ptr),
        ).map(to_string).collect()
    }

    /// Returns the path to and contents of the source files mapped by this compile command.
    pub fn get_mapped_source_files(&self) -> Vec<(PathBuf, String)> {
        iter!(
            clang_CompileCommand_getNumMappedSources(self.ptr),
            clang_CompileCommand_getMappedSourcePath(self.ptr),
            clang_CompileCommand_getMappedSourceContent(self.ptr),
        ).map(|(p, c)| (to_string(p).into(), to_string(c))).collect()
    }

    /// Returns the working directory of this compile command.
    pub fn get_working_directory(&self) -> PathBuf {
        unsafe { to_string(ffi::clang_CompileCommand_getDirectory(self.ptr)).into() }
    }
}

// CompileCommands _______________________________

#[derive(Clone)]
struct CompileCommands {
    ptr: ffi::CXCompileCommands,
}

impl CompileCommands {
    //- Constructors -----------------------------

    fn from_ptr(ptr: ffi::CXCompileCommands) -> Rc<CompileCommands> {
        Rc::new(CompileCommands { ptr: ptr })
    }
}

impl Drop for CompileCommands {
    fn drop(&mut self) {
        unsafe { ffi::clang_CompileCommands_dispose(self.ptr); }
    }
}

// CompletionContext _____________________________

options! {
    /// Indicates which types of results were included in a set of code completion results.
    options CompletionContext: CXCompletionContext {
        /// Indicates whether all possible types were included.
        pub all_types: CXCompletionContext_AnyType,
        /// Indicates whether all possible values were included.
        pub all_values: CXCompletionContext_AnyValue,
        /// Indicates whether values that resolve to C++ class types were included.
        pub class_type_values: CXCompletionContext_CXXClassTypeValue,
        /// Indicates whether the members of a record that are accessed with the dot operator were
        /// included.
        pub dot_members: CXCompletionContext_DotMemberAccess,
        /// Indicates whether the members of a record that are accessed with the arrow operator were
        /// included.
        pub arrow_members: CXCompletionContext_ArrowMemberAccess,
        /// Indicates whether enum tags were included.
        pub enum_tags: CXCompletionContext_EnumTag,
        /// Indicates whether union tags were included.
        pub union_tags: CXCompletionContext_UnionTag,
        /// Indicates whether struct tags were included.
        pub struct_tags: CXCompletionContext_StructTag,
        /// Indicates whether C++ class names were included.
        pub class_names: CXCompletionContext_ClassTag,
        /// Indicates whether C++ namespaces and namespace aliases were included.
        pub namespaces: CXCompletionContext_Namespace,
        /// Indicates whether C++ nested name specifiers were included.
        pub nested_name_specifiers: CXCompletionContext_NestedNameSpecifier,
        /// Indicates whether macro names were included.
        pub macro_names: CXCompletionContext_MacroName,
        /// Indicates whether natural language results were included.
        pub natural_language: CXCompletionContext_NaturalLanguage,
        /// Indicates whether values that resolve to Objective-C objects were included.
        pub objc_object_values: CXCompletionContext_ObjCObjectValue,
        /// Indicates whether values that resolve to Objective-C selectors were included.
        pub objc_selector_values: CXCompletionContext_ObjCSelectorValue,
        /// Indicates whether the properties of an Objective-C object that are accessed with the dot
        /// operator were included.
        pub objc_property_members: CXCompletionContext_ObjCPropertyAccess,
        /// Indicates whether Objective-C interfaces were included.
        pub objc_interfaces: CXCompletionContext_ObjCInterface,
        /// Indicates whether Objective-C protocols were included.
        pub objc_protocols: CXCompletionContext_ObjCProtocol,
        /// Indicates whether Objective-C categories were included.
        pub objc_categories: CXCompletionContext_ObjCCategory,
        /// Indicates whether Objective-C instance messages were included.
        pub objc_instance_messages: CXCompletionContext_ObjCInstanceMessage,
        /// Indicates whether Objective-C class messages were included.
        pub objc_class_messages: CXCompletionContext_ObjCClassMessage,
        /// Indicates whether Objective-C selector names were included.
        pub objc_selector_names: CXCompletionContext_ObjCSelectorName,
    }
}

// CompletionOptions _____________________________

options! {
    /// A set of options that determines how code completion is run.
    options CompletionOptions: CXCodeComplete_Flags {
        /// Indicates whether macros will be included in code completion results.
        pub macros: CXCodeComplete_IncludeMacros,
        /// Indicates whether code patterns (e.g., for loops) will be included in code completion
        /// results.
        pub code_patterns: CXCodeComplete_IncludeCodePatterns,
        /// Indicates whether documentation comment briefs will be included in code completion
        /// results.
        pub briefs: CXCodeComplete_IncludeBriefComments,
    }
}

impl Default for CompletionOptions {
    fn default() -> CompletionOptions {
        unsafe { CompletionOptions::from(ffi::clang_defaultCodeCompleteOptions()) }
    }
}

// CompletionResult ______________________________

/// A code completion result.
#[derive(Copy, Clone)]
pub struct CompletionResult<'r> {
    raw: ffi::CXCompletionResult,
    _marker: PhantomData<&'r CompletionResults>
}

impl<'r> CompletionResult<'r> {
    //- Constructors -----------------------------

    fn from_raw(raw: ffi::CXCompletionResult) -> CompletionResult<'r> {
        CompletionResult { raw: raw, _marker: PhantomData }
    }

    //- Accessors --------------------------------

    /// Returns the categorization of the AST entity this code completion result produces.
    pub fn get_kind(&self) -> EntityKind {
        unsafe { mem::transmute(self.raw.CursorKind) }
    }

    /// Returns the completion string for this code completion result.
    pub fn get_string(&self) -> CompletionString<'r> {
        CompletionString::from_raw(self.raw.CompletionString)
    }
}

impl<'r> cmp::Eq for CompletionResult<'r> { }

impl<'r> cmp::Ord for CompletionResult<'r> {
    fn cmp(&self, other: &CompletionResult<'r>) -> Ordering {
        self.get_string().cmp(&other.get_string())
    }
}

impl<'r> cmp::PartialEq for CompletionResult<'r> {
    fn eq(&self, other: &CompletionResult<'r>) -> bool {
        self.get_kind() == other.get_kind() && self.get_string() == other.get_string()
    }
}

impl<'r> cmp::PartialOrd for CompletionResult<'r> {
    fn partial_cmp(&self, other: &CompletionResult<'r>) -> Option<Ordering> {
        self.get_string().partial_cmp(&other.get_string())
    }
}

impl<'r> fmt::Debug for CompletionResult<'r> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.debug_struct("CompletionResult")
            .field("kind", &self.get_kind())
            .field("string", &self.get_string())
            .finish()
    }
}

// CompletionResults _____________________________

/// A set of code completion results.
pub struct CompletionResults {
    ptr: *mut ffi::CXCodeCompleteResults,
}

impl CompletionResults {
    //- Constructors -----------------------------

    fn from_ptr(ptr: *mut ffi::CXCodeCompleteResults) -> CompletionResults {
        CompletionResults { ptr: ptr }
    }

    //- Accessors --------------------------------

    /// Returns the categorization of the entity that contains the code completion context for this
    /// set of code completion results and whether that entity is incomplete, if applicable.
    pub fn get_container_kind(&self) -> Option<(EntityKind, bool)> {
        unsafe {
            let mut incomplete = mem::uninitialized();

            match ffi::clang_codeCompleteGetContainerKind(self.ptr, &mut incomplete) {
                ffi::CXCursorKind::InvalidCode => None,
                other => Some((mem::transmute(other), incomplete != 0)),
            }
        }
    }

    /// Returns the code completion context for this set of code completion results, if any.
    pub fn get_context(&self) -> Option<CompletionContext> {
        unsafe {
            let bits = ffi::clang_codeCompleteGetContexts(self.ptr) as c_uint;

            if bits != 0 && bits != 4194303 {
                Some(CompletionContext::from(ffi::CXCompletionContext::from_bits_truncate(bits)))
            } else {
                None
            }
        }
    }

    /// Returns the diagnostics that were produced prior to the code completion context for this set
    /// of code completion results.
    pub fn get_diagnostics<'tu>(&self, tu: &'tu TranslationUnit<'tu>) -> Vec<Diagnostic<'tu>> {
        iter!(
            clang_codeCompleteGetNumDiagnostics(self.ptr),
            clang_codeCompleteGetDiagnostic(self.ptr),
        ).map(|d| Diagnostic::from_ptr(d, tu)).collect()
    }

    /// Returns the code completion results in this set of code completion results.
    pub fn get_results(&self) -> Vec<CompletionResult> {
        unsafe {
            let raws = slice::from_raw_parts((*self.ptr).Results, (*self.ptr).NumResults as usize);
            raws.iter().map(|r| CompletionResult::from_raw(*r)).collect()
        }
    }

    /// Returns the USR for the entity that contains the code completion context for this set of
    /// code completion results, if applicable.
    pub fn get_usr(&self) -> Option<Usr> {
        unsafe { to_string_option(ffi::clang_codeCompleteGetContainerUSR(self.ptr)) }
    }
}

impl Drop for CompletionResults {
    fn drop(&mut self) {
        unsafe { ffi::clang_disposeCodeCompleteResults(self.ptr); }
    }
}

impl fmt::Debug for CompletionResults {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.debug_struct("CompletionResults")
            .field("results", &self.get_results())
            .finish()
    }
}

// CompletionString ______________________________

/// A semantic string that describes a code completion result.
#[derive(Copy, Clone)]
pub struct CompletionString<'r> {
    raw: ffi::CXCompletionString,
    _marker: PhantomData<&'r CompletionResults>
}

impl<'r> CompletionString<'r> {
    //- Constructors -----------------------------

    fn from_raw(raw: ffi::CXCompletionString) -> CompletionString<'r> {
        CompletionString { raw: raw, _marker: PhantomData }
    }

    //- Accessors --------------------------------

    /// Returns the annotations associated with this completion string.
    pub fn get_annotations(&self) -> Vec<String> {
        iter!(
            clang_getCompletionNumAnnotations(self.raw),
            clang_getCompletionAnnotation(self.raw),
        ).map(to_string).collect()
    }

    /// Returns the availability of this completion string.
    pub fn get_availability(&self) -> Availability {
        unsafe { mem::transmute(ffi::clang_getCompletionAvailability(self.raw)) }
    }

    /// Returns the chunks of this completion string.
    pub fn get_chunks(&self) -> Vec<CompletionChunk> {
        iter!(
            clang_getNumCompletionChunks(self.raw),
            clang_getCompletionChunkKind(self.raw),
        ).enumerate().map(|(i, k)| {
            macro_rules! text {
                ($variant:ident) => ({
                    let text = unsafe { ffi::clang_getCompletionChunkText(self.raw, i as c_uint) };
                    CompletionChunk::$variant(to_string(text))
                });
            }

            match k {
                ffi::CXCompletionChunkKind::Optional => {
                    let raw = unsafe {
                        ffi::clang_getCompletionChunkCompletionString(self.raw, i as c_uint)
                    };

                    CompletionChunk::Optional(CompletionString::from_raw(raw))
                },
                ffi::CXCompletionChunkKind::CurrentParameter => text!(CurrentParameter),
                ffi::CXCompletionChunkKind::TypedText => text!(TypedText),
                ffi::CXCompletionChunkKind::Text => text!(Text),
                ffi::CXCompletionChunkKind::Placeholder => text!(Placeholder),
                ffi::CXCompletionChunkKind::Informative => text!(Informative),
                ffi::CXCompletionChunkKind::ResultType => text!(ResultType),
                ffi::CXCompletionChunkKind::Colon => text!(Colon),
                ffi::CXCompletionChunkKind::Comma => text!(Comma),
                ffi::CXCompletionChunkKind::Equal => text!(Equals),
                ffi::CXCompletionChunkKind::SemiColon => text!(Semicolon),
                ffi::CXCompletionChunkKind::LeftAngle => text!(LeftAngleBracket),
                ffi::CXCompletionChunkKind::RightAngle => text!(RightAngleBracket),
                ffi::CXCompletionChunkKind::LeftBrace => text!(LeftBrace),
                ffi::CXCompletionChunkKind::RightBrace => text!(RightBrace),
                ffi::CXCompletionChunkKind::LeftParen => text!(LeftParenthesis),
                ffi::CXCompletionChunkKind::RightParen => text!(RightParenthesis),
                ffi::CXCompletionChunkKind::LeftBracket => text!(LeftSquareBracket),
                ffi::CXCompletionChunkKind::RightBracket => text!(RightSquareBracket),
                ffi::CXCompletionChunkKind::HorizontalSpace => text!(HorizontalSpace),
                ffi::CXCompletionChunkKind::VerticalSpace => text!(VerticalSpace),
            }
        }).collect()
    }

    /// Returns the documentation comment brief associated with the declaration this completion
    /// string refers to, if applicable.
    pub fn get_comment_brief(&self) -> Option<String> {
        unsafe { to_string_option(ffi::clang_getCompletionBriefComment(self.raw)) }
    }

    /// Returns the name of the semantic parent of the declaration this completion string refers to,
    /// if applicable.
    pub fn get_parent_name(&self) -> Option<String> {
        unsafe { to_string_option(ffi::clang_getCompletionParent(self.raw, ptr::null_mut())) }
    }

    /// Returns an integer that represents how likely a user is to select this completion string as
    /// determined by internal heuristics. Smaller values indicate higher priorities.
    pub fn get_priority(&self) -> usize {
        unsafe { ffi::clang_getCompletionPriority(self.raw) as usize }
    }

    /// Returns the text of the typed text chunk for this completion string, if any.
    pub fn get_typed_text(&self) -> Option<String> {
        for chunk in self.get_chunks() {
            if let CompletionChunk::TypedText(text) = chunk {
                return Some(text);
            }
        }

        None
    }
}

impl<'r> cmp::Eq for CompletionString<'r> { }

impl<'r> cmp::Ord for CompletionString<'r> {
    fn cmp(&self, other: &CompletionString<'r>) -> Ordering {
        match self.get_priority().cmp(&other.get_priority()) {
            Ordering::Equal => self.get_typed_text().cmp(&other.get_typed_text()),
            other => other,
        }
    }
}

impl<'r> cmp::PartialEq for CompletionString<'r> {
    fn eq(&self, other: &CompletionString<'r>) -> bool {
        self.get_chunks() == other.get_chunks()
    }
}

impl<'r> cmp::PartialOrd for CompletionString<'r> {
    fn partial_cmp(&self, other: &CompletionString<'r>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'r> fmt::Debug for CompletionString<'r> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.debug_struct("CompletionString")
            .field("chunks", &self.get_chunks())
            .finish()
    }
}

// Diagnostic ____________________________________

/// A message from the compiler about an issue with a source file.
#[derive(Copy, Clone)]
pub struct Diagnostic<'tu> {
    ptr: ffi::CXDiagnostic,
    tu: &'tu TranslationUnit<'tu>,
}

impl<'tu> Diagnostic<'tu> {
    //- Constructors -----------------------------

    fn from_ptr(ptr: ffi::CXDiagnostic, tu: &'tu TranslationUnit<'tu>) -> Diagnostic<'tu> {
        Diagnostic { ptr: ptr, tu: tu }
    }

    //- Accessors --------------------------------

    /// Returns this diagnostic as a formatted string.
    pub fn format(&self, options: FormatOptions) -> String {
        unsafe { to_string(ffi::clang_formatDiagnostic(self.ptr, options.into())) }
    }

    /// Returns the child diagnostics of this diagnostic.
    pub fn get_children(&self) -> Vec<Diagnostic> {
        let raw = unsafe { ffi::clang_getChildDiagnostics(self.ptr) };

        iter!(
            clang_getNumDiagnosticsInSet(raw),
            clang_getDiagnosticInSet(raw),
        ).map(|d| Diagnostic::from_ptr(d, self.tu)).collect()
    }

    /// Returns the fix-its for this diagnostic.
    pub fn get_fix_its(&self) -> Vec<FixIt<'tu>> {
        unsafe {
            (0..ffi::clang_getDiagnosticNumFixIts(self.ptr)).map(|i| {
                let mut range = mem::uninitialized();
                let string = to_string(ffi::clang_getDiagnosticFixIt(self.ptr, i, &mut range));
                let range = SourceRange::from_raw(range, self.tu);

                if string.is_empty() {
                    FixIt::Deletion(range)
                } else if range.get_start() == range.get_end() {
                    FixIt::Insertion(range.get_start(), string)
                } else {
                    FixIt::Replacement(range, string)
                }
            }).collect()
        }
    }

    /// Returns the source location of this diagnostic.
    pub fn get_location(&self) -> SourceLocation<'tu> {
        unsafe { SourceLocation::from_raw(ffi::clang_getDiagnosticLocation(self.ptr), self.tu) }
    }

    /// Returns the source ranges of this diagnostic.
    pub fn get_ranges(&self) -> Vec<SourceRange<'tu>> {
        iter!(clang_getDiagnosticNumRanges(self.ptr), clang_getDiagnosticRange(self.ptr)).map(|r| {
            SourceRange::from_raw(r, self.tu)
        }).collect()
    }

    /// Returns the severity of this diagnostic.
    pub fn get_severity(&self) -> Severity {
        unsafe { mem::transmute(ffi::clang_getDiagnosticSeverity(self.ptr)) }
    }

    /// Returns the text of this diagnostic.
    pub fn get_text(&self) -> String {
        unsafe { to_string(ffi::clang_getDiagnosticSpelling(self.ptr)) }
    }
}

impl<'tu> fmt::Debug for Diagnostic<'tu> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.debug_struct("Diagnostic")
            .field("location", &self.get_location())
            .field("severity", &self.get_severity())
            .field("text", &self.get_text())
            .finish()
    }
}

impl<'tu> fmt::Display for Diagnostic<'tu> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.format(FormatOptions::default()))
    }
}

// Entity ________________________________________

/// An AST entity.
#[derive(Copy, Clone)]
pub struct Entity<'tu> {
    raw: ffi::CXCursor,
    tu: &'tu TranslationUnit<'tu>,
}

impl<'tu> Entity<'tu> {
    //- Constructors -----------------------------

    fn from_raw(raw: ffi::CXCursor, tu: &'tu TranslationUnit<'tu>) -> Entity<'tu> {
        Entity { raw: raw, tu: tu }
    }

    //- Accessors --------------------------------

    /// Returns the accessibility of this declaration or base class specifier, if applicable.
    pub fn get_accessibility(&self) -> Option<Accessibility> {
        unsafe {
            match ffi::clang_getCXXAccessSpecifier(self.raw) {
                ffi::CX_CXXAccessSpecifier::CXXInvalidAccessSpecifier => None,
                other => Some(mem::transmute(other)),
            }
        }
    }

    /// Returns the arguments of this function or method, if applicable.
    pub fn get_arguments(&self) -> Option<Vec<Entity<'tu>>> {
        iter_option!(
            clang_Cursor_getNumArguments(self.raw),
            clang_Cursor_getArgument(self.raw),
        ).map(|i| i.map(|a| Entity::from_raw(a, self.tu)).collect())
    }

    /// Returns the availability of this AST entity.
    pub fn get_availability(&self) -> Availability {
        unsafe { mem::transmute(ffi::clang_getCursorAvailability(self.raw)) }
    }

    /// Returns the width of this bit field, if applicable.
    pub fn get_bit_field_width(&self) -> Option<usize> {
        unsafe {
            let width = ffi::clang_getFieldDeclBitWidth(self.raw);

            if width >= 0 {
                Some(width as usize)
            } else {
                None
            }
        }
    }

    /// Returns the canonical entity for this AST entity.
    ///
    /// In the C family of languages, some types of entities can be declared multiple times. When
    /// there are multiple declarations of the same entity, only one will be considered canonical.
    pub fn get_canonical_entity(&self) -> Entity<'tu> {
        unsafe { Entity::from_raw(ffi::clang_getCanonicalCursor(self.raw), self.tu) }
    }

    /// Returns the comment associated with this AST entity, if any.
    pub fn get_comment(&self) -> Option<String> {
        unsafe { to_string_option(ffi::clang_Cursor_getRawCommentText(self.raw)) }
    }

    /// Returns the brief of the comment associated with this AST entity, if any.
    pub fn get_comment_brief(&self) -> Option<String> {
        unsafe { to_string_option(ffi::clang_Cursor_getBriefCommentText(self.raw)) }
    }

    /// Returns the source range of the comment associated with this AST entity, if any.
    pub fn get_comment_range(&self) -> Option<SourceRange<'tu>> {
        let range = unsafe { ffi::clang_Cursor_getCommentRange(self.raw) };
        range.map(|r| SourceRange::from_raw(r, self.tu))
    }

    /// Returns a completion string for this declaration or macro definition, if applicable.
    pub fn get_completion_string(&self) -> Option<CompletionString> {
        unsafe { ffi::clang_getCursorCompletionString(self.raw).map(CompletionString::from_raw) }
    }

    /// Returns the children of this AST entity.
    pub fn get_children(&self) -> Vec<Entity<'tu>> {
        let mut children = vec![];

        self.visit_children(|c, _| {
            children.push(c);
            EntityVisitResult::Continue
        });

        children
    }

    /// Returns the AST entity that describes the definition of this AST entity, if any.
    pub fn get_definition(&self) -> Option<Entity<'tu>> {
        unsafe { ffi::clang_getCursorDefinition(self.raw).map(|p| Entity::from_raw(p, self.tu)) }
    }

    /// Returns the display name of this AST entity, if any.
    ///
    /// The display name of an entity contains additional information that helps identify the
    /// entity.
    pub fn get_display_name(&self) -> Option<String> {
        unsafe { to_string_option(ffi::clang_getCursorDisplayName(self.raw)) }
    }

    /// Returns the value of this enum constant declaration, if applicable.
    pub fn get_enum_constant_value(&self) -> Option<(i64, u64)> {
        unsafe {
            if self.get_kind() == EntityKind::EnumConstantDecl {
                let signed = ffi::clang_getEnumConstantDeclValue(self.raw);
                let unsigned = ffi::clang_getEnumConstantDeclUnsignedValue(self.raw);
                Some((signed, unsigned))
            } else {
                None
            }
        }
    }

    /// Returns the underlying type of this enum declaration, if applicable.
    pub fn get_enum_underlying_type(&self) -> Option<Type<'tu>> {
        unsafe { ffi::clang_getEnumDeclIntegerType(self.raw).map(|t| Type::from_raw(t, self.tu)) }
    }

    /// Returns the file included by this inclusion directive, if applicable.
    pub fn get_file(&self) -> Option<File<'tu>> {
        unsafe { ffi::clang_getIncludedFile(self.raw).map(|f| File::from_ptr(f, self.tu)) }
    }

    /// Returns the categorization of this AST entity.
    pub fn get_kind(&self) -> EntityKind {
        unsafe { mem::transmute(ffi::clang_getCursorKind(self.raw)) }
    }

    /// Returns the language used by this declaration, if applicable.
    pub fn get_language(&self) -> Option<Language> {
        unsafe {
            match ffi::clang_getCursorLanguage(self.raw) {
                ffi::CXLanguageKind::Invalid => None,
                other => Some(mem::transmute(other)),
            }
        }
    }

    /// Returns the lexical parent of this AST entity, if any.
    pub fn get_lexical_parent(&self) -> Option<Entity<'tu>> {
        let parent = unsafe { ffi::clang_getCursorLexicalParent(self.raw) };
        parent.map(|p| Entity::from_raw(p, self.tu))
    }

    /// Returns the linkage of this AST entity, if any.
    pub fn get_linkage(&self) -> Option<Linkage> {
        unsafe {
            match ffi::clang_getCursorLinkage(self.raw) {
                ffi::CXLinkageKind::Invalid => None,
                other => Some(mem::transmute(other)),
            }
        }
    }

    /// Returns the source location of this AST entity, if any.
    pub fn get_location(&self) -> Option<SourceLocation<'tu>> {
        unsafe {
            let location = ffi::clang_getCursorLocation(self.raw);
            location.map(|l| SourceLocation::from_raw(l, self.tu))
        }
    }

    /// Returns the mangled name of this AST entity, if any.
    pub fn get_mangled_name(&self) -> Option<String> {
        unsafe { to_string_option(ffi::clang_Cursor_getMangling(self.raw)) }
    }

    /// Returns the module imported by this module import declaration, if applicable.
    pub fn get_module(&self) -> Option<Module<'tu>> {
        unsafe { ffi::clang_Cursor_getModule(self.raw).map(|m| Module::from_ptr(m, self.tu)) }
    }

    /// Returns the name of this AST entity, if any.
    pub fn get_name(&self) -> Option<String> {
        unsafe { to_string_option(ffi::clang_getCursorSpelling(self.raw)) }
    }

    /// Returns the source ranges of the name of this AST entity.
    pub fn get_name_ranges(&self) -> Vec<SourceRange<'tu>> {
        use std::ptr;

        unsafe {
            (0..).map(|i| ffi::clang_Cursor_getSpellingNameRange(self.raw, i, 0)).take_while(|r| {
                if ffi::clang_Range_isNull(*r) != 0 {
                    false
                } else {
                    let mut file = mem::uninitialized();

                    ffi::clang_getSpellingLocation(
                        ffi::clang_getRangeStart(*r),
                        &mut file,
                        ptr::null_mut(),
                        ptr::null_mut(),
                        ptr::null_mut(),
                    );

                    !file.0.is_null()
                }
            }).map(|r| SourceRange::from_raw(r, self.tu)).collect()
        }
    }

    /// Returns the overloaded declarations referenced by this overloaded declaration reference, if
    /// applicable.
    pub fn get_overloaded_declarations(&self) -> Option<Vec<Entity<'tu>>> {
        let declarations = iter!(
            clang_getNumOverloadedDecls(self.raw),
            clang_getOverloadedDecl(self.raw),
        ).map(|e| Entity::from_raw(e, self.tu)).collect::<Vec<_>>();

        if !declarations.is_empty() {
            Some(declarations)
        } else {
            None
        }
    }

    /// Returns the methods that were overridden by this method, if applicable.
    pub fn get_overridden_methods(&self) -> Option<Vec<Entity<'tu>>> {
        unsafe {
            let (mut raw, mut count) = mem::uninitialized();
            ffi::clang_getOverriddenCursors(self.raw, &mut raw, &mut count);

            if !raw.is_null() {
                let raws = slice::from_raw_parts(raw, count as usize);
                let methods = raws.iter().map(|e| Entity::from_raw(*e, self.tu)).collect();
                ffi::clang_disposeOverriddenCursors(raw);
                Some(methods)
            } else {
                None
            }
        }
    }

    /// Returns the availability of this declaration on the platforms where it is known, if
    /// applicable.
    pub fn get_platform_availability(&self) -> Option<Vec<PlatformAvailability>> {
        if !self.is_declaration() {
            return None;
        }

        unsafe {
            let mut buffer: [ffi::CXPlatformAvailability; 32] = mem::uninitialized();

            let count = ffi::clang_getCursorPlatformAvailability(
                self.raw,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                (&mut buffer).as_mut_ptr(),
                buffer.len() as c_int,
            );

            Some((0..count as usize).map(|i| PlatformAvailability::from_raw(buffer[i])).collect())
        }
    }

    /// Returns the source range of this AST entity, if any.
    pub fn get_range(&self) -> Option<SourceRange<'tu>> {
        unsafe {
            let range = ffi::clang_getCursorExtent(self.raw);
            range.map(|r| SourceRange::from_raw(r, self.tu))
        }
    }

    /// Returns the AST entity referred to by this AST entity, if any.
    pub fn get_reference(&self) -> Option<Entity<'tu>> {
        unsafe { ffi::clang_getCursorReferenced(self.raw).map(|p| Entity::from_raw(p, self.tu)) }
    }

    /// Returns the semantic parent of this AST entity, if any.
    pub fn get_semantic_parent(&self) -> Option<Entity<'tu>> {
        let parent = unsafe { ffi::clang_getCursorSemanticParent(self.raw) };
        parent.map(|p| Entity::from_raw(p, self.tu))
    }

    /// Returns the storage class of this declaration, if applicable.
    pub fn get_storage_class(&self) -> Option<StorageClass> {
        unsafe {
            match ffi::clang_Cursor_getStorageClass(self.raw) {
                ffi::CX_StorageClass::Invalid => None,
                other => Some(mem::transmute(other)),
            }
        }
    }

    /// Returns the template declaration this template specialization was instantiated from, if
    /// applicable.
    pub fn get_template(&self) -> Option<Entity<'tu>> {
        let parent = unsafe { ffi::clang_getSpecializedCursorTemplate(self.raw) };
        parent.map(|p| Entity::from_raw(p, self.tu))
    }

    /// Returns the template arguments for this template function specialization, if applicable.
    pub fn get_template_arguments(&self) -> Option<Vec<TemplateArgument<'tu>>> {
        let get_type = &ffi::clang_Cursor_getTemplateArgumentType;
        let get_signed = &ffi::clang_Cursor_getTemplateArgumentValue;
        let get_unsigned = &ffi::clang_Cursor_getTemplateArgumentUnsignedValue;

        iter_option!(
            clang_Cursor_getNumTemplateArguments(self.raw),
            clang_Cursor_getTemplateArgumentKind(self.raw),
        ).map(|i| {
            i.enumerate().map(|(i, t)| {
                match t {
                    ffi::CXTemplateArgumentKind::Null => TemplateArgument::Null,
                    ffi::CXTemplateArgumentKind::Type => {
                        let type_ = unsafe { get_type(self.raw, i as c_uint) };
                        TemplateArgument::Type(Type::from_raw(type_, self.tu))
                    },
                    ffi::CXTemplateArgumentKind::Declaration => TemplateArgument::Declaration,
                    ffi::CXTemplateArgumentKind::NullPtr => TemplateArgument::Nullptr,
                    ffi::CXTemplateArgumentKind::Integral => {
                        let signed = unsafe { get_signed(self.raw, i as c_uint) };
                        let unsigned = unsafe { get_unsigned(self.raw, i as c_uint) };
                        TemplateArgument::Integral(signed as i64, unsigned as u64)
                    },
                    ffi::CXTemplateArgumentKind::Template => TemplateArgument::Template,
                    ffi::CXTemplateArgumentKind::TemplateExpansion => TemplateArgument::TemplateExpansion,
                    ffi::CXTemplateArgumentKind::Expression => TemplateArgument::Expression,
                    ffi::CXTemplateArgumentKind::Pack => TemplateArgument::Pack,
                    _ => unreachable!(),
                }
            }).collect()
        })
    }

    /// Returns the categorization of the template specialization that would result from
    /// instantiating this template declaration, if applicable.
    pub fn get_template_kind(&self) -> Option<EntityKind> {
        unsafe {
            match ffi::clang_getTemplateCursorKind(self.raw) {
                ffi::CXCursorKind::NoDeclFound => None,
                other => Some(mem::transmute(other)),
            }
        }
    }

    /// Returns the translation unit which contains this AST entity.
    pub fn get_translation_unit(&self) -> &'tu TranslationUnit<'tu> {
        self.tu
    }

    /// Returns the type of this AST entity, if any.
    pub fn get_type(&self) -> Option<Type<'tu>> {
        unsafe { ffi::clang_getCursorType(self.raw).map(|t| Type::from_raw(t, self.tu)) }
    }

    /// Returns the underlying type of this typedef declaration, if applicable.
    pub fn get_typedef_underlying_type(&self) -> Option<Type<'tu>> {
        unsafe {
            let type_ = ffi::clang_getTypedefDeclUnderlyingType(self.raw);
            type_.map(|t| Type::from_raw(t, self.tu))
        }
    }

    /// Returns the USR for this AST entity, if any.
    pub fn get_usr(&self) -> Option<Usr> {
        unsafe { to_string_option(ffi::clang_getCursorUSR(self.raw)) }
    }

    /// Returns whether this AST entity is an anonymous record declaration.
    pub fn is_anonymous(&self) -> bool {
        unsafe { ffi::clang_Cursor_isAnonymous(self.raw) != 0 }
    }

    /// Returns whether this AST entity is a bit field.
    pub fn is_bit_field(&self) -> bool {
        unsafe { ffi::clang_Cursor_isBitField(self.raw) != 0 }
    }

    /// Returns whether this AST entity is a const method.
    pub fn is_const_method(&self) -> bool {
        unsafe { ffi::clang_CXXMethod_isConst(self.raw) != 0 }
    }

    /// Returns whether this AST entity is a declaration and also the definition of that
    /// declaration.
    pub fn is_definition(&self) -> bool {
        unsafe { ffi::clang_isCursorDefinition(self.raw) != 0 }
    }

    /// Returns whether this AST entity is a dynamic call.
    ///
    /// A dynamic call is either a call to a C++ virtual method or an Objective-C message where the
    /// receiver is an object instance, not `super` or a specific class.
    pub fn is_dynamic_call(&self) -> bool {
        unsafe { ffi::clang_Cursor_isDynamicCall(self.raw) != 0 }
    }

    /// Returns whether this AST entity is a pure virtual method.
    pub fn is_pure_virtual_method(&self) -> bool {
        unsafe { ffi::clang_CXXMethod_isPureVirtual(self.raw) != 0 }
    }

    /// Returns whether this AST entity is a static method.
    pub fn is_static_method(&self) -> bool {
        unsafe { ffi::clang_CXXMethod_isStatic(self.raw) != 0 }
    }

    /// Returns whether this AST entity is a variadic function or method.
    pub fn is_variadic(&self) -> bool {
        unsafe { ffi::clang_Cursor_isVariadic(self.raw) != 0 }
    }

    /// Returns whether this AST entity is a virtual base class specifier.
    pub fn is_virtual_base(&self) -> bool {
        unsafe { ffi::clang_isVirtualBase(self.raw) != 0 }
    }

    /// Returns whether this AST entity is a virtual method.
    pub fn is_virtual_method(&self) -> bool {
        unsafe { ffi::clang_CXXMethod_isVirtual(self.raw) != 0 }
    }

    /// Visits the children of this AST entity recursively and returns whether visitation was ended
    /// by the callback returning `EntityVisitResult::Break`.
    ///
    /// The first argument of the callback is the AST entity being visited and the second argument
    /// is the parent of that AST entity. The return value of the callback determines how visitation
    /// will proceed.
    pub fn visit_children<F: FnMut(Entity<'tu>, Entity<'tu>) -> EntityVisitResult>(
        &self, f: F
    ) -> bool {
        trait EntityCallback<'tu> {
            fn call(&mut self, entity: Entity<'tu>, parent: Entity<'tu>) -> EntityVisitResult;
        }

        impl<'tu, F: FnMut(Entity<'tu>, Entity<'tu>) -> EntityVisitResult> EntityCallback<'tu> for F {
            fn call(&mut self, entity: Entity<'tu>, parent: Entity<'tu>) -> EntityVisitResult {
                self(entity, parent)
            }
        }

        extern fn visit(
            cursor: ffi::CXCursor, parent: ffi::CXCursor, data: ffi::CXClientData
        ) -> ffi::CXChildVisitResult {
            unsafe {
                let &mut (tu, ref mut callback):
                    &mut (&TranslationUnit, Box<EntityCallback>) =
                        mem::transmute(data);

                let entity = Entity::from_raw(cursor, tu);
                let parent = Entity::from_raw(parent, tu);
                mem::transmute(callback.call(entity, parent))
            }
        }

        let mut data = (self.tu, Box::new(f) as Box<EntityCallback>);
        unsafe { ffi::clang_visitChildren(self.raw, visit, mem::transmute(&mut data)) != 0 }
    }

    //- Categorization ---------------------------

    /// Returns whether this AST entity is categorized as an attribute.
    pub fn is_attribute(&self) -> bool {
        unsafe { ffi::clang_isAttribute(self.raw.kind) != 0 }
    }

    /// Returns whether this AST entity is categorized as a declaration.
    pub fn is_declaration(&self) -> bool {
        unsafe { ffi::clang_isDeclaration(self.raw.kind) != 0 }
    }

    /// Returns whether this AST entity is categorized as an expression.
    pub fn is_expression(&self) -> bool {
        unsafe { ffi::clang_isExpression(self.raw.kind) != 0 }
    }

    /// Returns whether this AST entity is categorized as a preprocessing entity.
    pub fn is_preprocessing(&self) -> bool {
        unsafe { ffi::clang_isPreprocessing(self.raw.kind) != 0 }
    }

    /// Returns whether this AST entity is categorized as a reference.
    pub fn is_reference(&self) -> bool {
        unsafe { ffi::clang_isReference(self.raw.kind) != 0 }
    }

    /// Returns whether this AST entity is categorized as a statement.
    pub fn is_statement(&self) -> bool {
        unsafe { ffi::clang_isStatement(self.raw.kind) != 0 }
    }

    /// Returns whether the categorization of this AST entity is unexposed.
    pub fn is_unexposed(&self) -> bool {
        unsafe { ffi::clang_isUnexposed(self.raw.kind) != 0 }
    }
}

impl<'tu> cmp::Eq for Entity<'tu> { }

impl<'tu> cmp::PartialEq for Entity<'tu> {
    fn eq(&self, other: &Entity<'tu>) -> bool {
        unsafe { ffi::clang_equalCursors(self.raw, other.raw) != 0 }
    }
}

impl<'tu> fmt::Debug for Entity<'tu> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.debug_struct("Entity")
            .field("location", &self.get_location())
            .field("kind", &self.get_kind())
            .field("display_name", &self.get_display_name())
            .finish()
    }
}

impl<'tu> hash::Hash for Entity<'tu> {
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        unsafe {
            let integer = ffi::clang_hashCursor(self.raw);
            let slice = slice::from_raw_parts(mem::transmute(&integer), mem::size_of_val(&integer));
            hasher.write(slice);
        }
    }
}

// File __________________________________________

/// A source file.
#[derive(Copy, Clone)]
pub struct File<'tu> {
    ptr: ffi::CXFile,
    tu: &'tu TranslationUnit<'tu>,
}

impl<'tu> File<'tu> {
    //- Constructors -----------------------------

    fn from_ptr(ptr: ffi::CXFile, tu: &'tu TranslationUnit<'tu>) -> File<'tu> {
        File { ptr: ptr, tu: tu }
    }

    //- Accessors --------------------------------

    /// Returns a unique identifier for this file.
    pub fn get_id(&self) -> (u64, u64, u64) {
        unsafe {
            let mut id = mem::uninitialized();
            ffi::clang_getFileUniqueID(self.ptr, &mut id);
            (id.data[0] as u64, id.data[1] as u64, id.data[2] as u64)
        }
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
        let location = unsafe { ffi::clang_getLocation(self.tu.ptr, self.ptr, line, column) };
        SourceLocation::from_raw(location, self.tu)
    }

    /// Returns the module containing this file, if any.
    pub fn get_module(&self) -> Option<Module<'tu>> {
        let module = unsafe { ffi::clang_getModuleForFile(self.tu.ptr, self.ptr) };
        module.map(|m| Module::from_ptr(m, self.tu))
    }

    /// Returns the source location at the supplied character offset in this file.
    pub fn get_offset_location(&self, offset: u32) -> SourceLocation<'tu> {
        let offset = offset as c_uint;
        let location = unsafe { ffi::clang_getLocationForOffset(self.tu.ptr, self.ptr, offset) };
        SourceLocation::from_raw(location, self.tu)
    }

    /// Returns the absolute path to this file.
    pub fn get_path(&self) -> PathBuf {
        let path = unsafe { ffi::clang_getFileName(self.ptr) };
        Path::new(&to_string(path)).into()
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

    /// Returns the source ranges in this file that were skipped by the preprocessor.
    ///
    /// This will always return an empty `Vec` if the translation unit that contains this file was
    /// not constructed with a detailed preprocessing record.
    pub fn get_skipped_ranges(&self) -> Vec<SourceRange<'tu>> {
        unsafe {
            let raw = ffi::clang_getSkippedRanges(self.tu.ptr, self.ptr);
            let raws = slice::from_raw_parts((*raw).ranges, (*raw).count as usize);
            let ranges = raws.iter().map(|r| SourceRange::from_raw(*r, self.tu)).collect();
            ffi::clang_disposeSourceRangeList(raw);
            ranges
        }
    }

    /// Returns the last modification time for this file.
    pub fn get_time(&self) -> time_t {
        unsafe { ffi::clang_getFileTime(self.ptr) }
    }

    /// Returns whether this file is guarded against multiple inclusions.
    pub fn is_include_guarded(&self) -> bool {
        unsafe { ffi::clang_isFileMultipleIncludeGuarded(self.tu.ptr, self.ptr) != 0 }
    }

    /// Visits the inclusion directives in this file and returns whether visitation was ended by the
    /// callback returning `false`.
    pub fn visit_includes<F: FnMut(Entity<'tu>, SourceRange<'tu>) -> bool>(&self, f: F) -> bool {
        visit(self.tu, f, |v| unsafe { ffi::clang_findIncludesInFile(self.tu.ptr, self.ptr, v) })
    }

    /// Visits the references to the supplied entity in this file and returns whether visitation was
    /// ended by the callback returning `false`.
    pub fn visit_references<F: FnMut(Entity<'tu>, SourceRange<'tu>) -> bool>(
        &self, entity: Entity<'tu>, f: F
    ) -> bool {
        visit(self.tu, f, |v| unsafe { ffi::clang_findReferencesInFile(entity.raw, self.ptr, v) })
    }
}

impl<'tu> cmp::Eq for File<'tu> { }

impl<'tu> cmp::PartialEq for File<'tu> {
    fn eq(&self, other: &File<'tu>) -> bool {
        unsafe { ffi::clang_File_isEqual(self.ptr, other.ptr) != 0 }
    }
}

impl<'tu> fmt::Debug for File<'tu> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.debug_struct("File").field("path", &self.get_path()).finish()
    }
}

impl<'tu> hash::Hash for File<'tu> {
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        self.get_id().hash(hasher);
    }
}

// FormatOptions _________________________________

options! {
    /// A set of options that determines how a diagnostic is formatted.
    options FormatOptions: CXDiagnosticDisplayOptions {
        /// Indicates whether the diagnostic text will be prefixed by the file and line of the
        /// source location the diagnostic indicates. This prefix may also contain column and/or
        /// source range information.
        pub source_location: CXDiagnostic_DisplaySourceLocation,
        /// Indicates whether the column will be included in the source location prefix.
        pub column: CXDiagnostic_DisplayColumn,
        /// Indicates whether the source ranges will be included to the source location prefix.
        pub source_ranges: CXDiagnostic_DisplaySourceRanges,
        /// Indicates whether the option associated with the diagnostic (e.g., `-Wconversion`) will
        /// be placed in brackets after the diagnostic text if there is such an option.
        pub option: CXDiagnostic_DisplayOption,
        /// Indicates whether the category number associated with the diagnostic will be placed in
        /// brackets after the diagnostic text if there is such a category number.
        pub category_id: CXDiagnostic_DisplayCategoryId,
        /// Indicates whether the category name associated with the diagnostic will be placed in
        /// brackets after the diagnostic text if there is such a category name.
        pub category_name: CXDiagnostic_DisplayCategoryName,
    }
}

impl Default for FormatOptions {
    fn default() -> FormatOptions {
        unsafe { FormatOptions::from(ffi::clang_defaultDiagnosticDisplayOptions()) }
    }
}

// Index _________________________________________

/// A collection of translation units.
pub struct Index<'c> {
    ptr: ffi::CXIndex,
    _marker: PhantomData<&'c Clang>,
}

impl<'c> Index<'c> {
    //- Constructors -----------------------------

    /// Constructs a new `Index`.
    ///
    /// `exclude` determines whether declarations from precompiled headers are excluded and
    /// `diagnostics` determines whether diagnostics are printed while parsing source files.
    pub fn new(_: &'c Clang, exclude: bool, diagnostics: bool) -> Index<'c> {
        let ptr = unsafe { ffi::clang_createIndex(exclude as c_int, diagnostics as c_int) };
        Index { ptr: ptr, _marker: PhantomData }
    }

    //- Accessors --------------------------------

    /// Returns the thread options for this index.
    pub fn get_thread_options(&self) -> ThreadOptions {
        unsafe { ThreadOptions::from(ffi::clang_CXIndex_getGlobalOptions(self.ptr)) }
    }

    //- Mutators ---------------------------------

    /// Sets the thread options for this index.
    pub fn set_thread_options(&mut self, options: ThreadOptions) {
        unsafe { ffi::clang_CXIndex_setGlobalOptions(self.ptr, options.into()); }
    }
}

impl<'c> Drop for Index<'c> {
    fn drop(&mut self) {
        unsafe { ffi::clang_disposeIndex(self.ptr); }
    }
}

// Location ______________________________________

/// The file, line, column, and character offset of a source location.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Location<'tu> {
    /// The file of the source location.
    pub file: File<'tu>,
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
    ptr: ffi::CXModule,
    tu: &'tu TranslationUnit<'tu>,
}

impl<'tu> Module<'tu> {
    //- Constructors -----------------------------

    fn from_ptr(ptr: ffi::CXModule, tu: &'tu TranslationUnit<'tu>) -> Module<'tu> {
        Module { ptr: ptr, tu: tu }
    }

    //- Accessors --------------------------------

    /// Returns the AST file this module came from.
    pub fn get_file(&self) -> File<'tu> {
        let ptr = unsafe { ffi::clang_Module_getASTFile(self.ptr) };
        File::from_ptr(ptr, self.tu)
    }

    /// Returns the full name of this module (e.g., `std.vector` for the `std.vector` module).
    pub fn get_full_name(&self) -> String {
        let name = unsafe { ffi::clang_Module_getFullName(self.ptr) };
        to_string(name)
    }

    /// Returns the name of this module (e.g., `vector` for the `std.vector` module).
    pub fn get_name(&self) -> String {
        let name = unsafe { ffi::clang_Module_getName(self.ptr) };
        to_string(name)
    }

    /// Returns the parent of this module, if any.
    pub fn get_parent(&self) -> Option<Module<'tu>> {
        let parent = unsafe { ffi::clang_Module_getParent(self.ptr) };
        parent.map(|p| Module::from_ptr(p, self.tu))
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
        unsafe { ffi::clang_Module_isSystem(self.ptr) != 0 }
    }
}

impl<'tu> cmp::Eq for Module<'tu> { }

impl<'tu> cmp::PartialEq for Module<'tu> {
    fn eq(&self, other: &Module<'tu>) -> bool {
        self.get_file() == other.get_file() && self.get_full_name() == other.get_full_name()
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

// ParseOptions __________________________________

options! {
    /// A set of options that determines how a source file is parsed into a translation unit.
    #[derive(Default)]
    options ParseOptions: CXTranslationUnit_Flags {
        /// Indicates whether certain code completion results will be cached when the translation
        /// unit is reparsed.
        ///
        /// This option increases the time it takes to reparse the translation unit but improves
        /// code completion performance.
        pub cached_completion_results: CXTranslationUnit_CacheCompletionResults,
        /// Indicates whether a detailed preprocessing record will be constructed which tracks all
        /// macro definitions and instantiations.
        pub detailed_preprocessing_record: CXTranslationUnit_DetailedPreprocessingRecord,
        /// Indicates whether documentation comment briefs will be included in code completion
        /// results.
        pub briefs_in_completion_results: CXTranslationUnit_IncludeBriefCommentsInCodeCompletion,
        /// Indicates whether the translation unit will be considered incomplete.
        ///
        /// This option suppresses certain semantic analyses and is typically used when parsing
        /// headers with the intent of creating a precompiled header.
        pub incomplete: CXTranslationUnit_Incomplete,
        /// Indicates whether function and method bodies will be skipped.
        pub skipped_function_bodies: CXTranslationUnit_SkipFunctionBodies,
    }
}

// PlatformAvailability __________________________

/// The availability of an AST entity on a particular platform.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PlatformAvailability {
    /// The name of the platform.
    pub platform: String,
    /// Whether the AST entity is unavailable on the platform.
    pub unavailable: bool,
    /// The version of the platform in which this AST entity was introduced, if any.
    pub introduced: Option<Version>,
    /// The version of the platform in which this AST entity was deprecated, if any.
    pub deprecated: Option<Version>,
    /// The version of the platform in which this AST entity was obsoleted, if any.
    pub obsoleted: Option<Version>,
    /// A message to display to users (e.g., replacement API suggestions).
    pub message: Option<String>,
}

impl PlatformAvailability {
    //- Constructors -----------------------------

    fn from_raw(mut raw: ffi::CXPlatformAvailability) -> PlatformAvailability {
        let availability = PlatformAvailability {
            platform: to_string(raw.Platform),
            unavailable: raw.Unavailable != 0,
            introduced: raw.Introduced.map(Version::from_raw),
            deprecated: raw.Deprecated.map(Version::from_raw),
            obsoleted: raw.Obsoleted.map(Version::from_raw),
            message: to_string_option(raw.Message),
        };

        unsafe { ffi::clang_disposeCXPlatformAvailability(&mut raw); }
        availability
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

    /// Returns the AST entity at this source location, if any.
    pub fn get_entity(&self) -> Option<Entity<'tu>> {
        unsafe { ffi::clang_getCursor(self.tu.ptr, self.raw).map(|c| Entity::from_raw(c, self.tu)) }
    }

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
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
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
        let end = unsafe { ffi::clang_getRangeEnd(self.raw) };
        SourceLocation::from_raw(end, self.tu)
    }

    /// Returns the inclusive start of this source range.
    pub fn get_start(&self) -> SourceLocation<'tu> {
        let start = unsafe { ffi::clang_getRangeStart(self.raw) };
        SourceLocation::from_raw(start, self.tu)
    }

    /// Tokenizes the source code covered by this source range and returns the resulting tokens.
    pub fn tokenize(&self) -> Vec<Token<'tu>> {
        unsafe {
            let (mut raw, mut count) = mem::uninitialized();
            ffi::clang_tokenize(self.tu.ptr, self.raw, &mut raw, &mut count);
            let raws = slice::from_raw_parts(raw, count as usize);
            let tokens = raws.iter().map(|t| Token::from_raw(*t, self.tu)).collect();
            ffi::clang_disposeTokens(self.tu.ptr, raw, count);
            tokens
        }
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
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        self.get_start().hash(hasher);
        self.get_end().hash(hasher);
    }
}

// ThreadOptions _________________________________

options! {
    /// A set of options that determines which types of threads should use background priority.
    #[derive(Default)]
    options ThreadOptions: CXGlobalOptFlags {
        /// Indicates whether threads creating for editing purposes should use background priority.
        pub editing: CXGlobalOpt_ThreadBackgroundPriorityForEditing,
        /// Indicates whether threads creating for indexing purposes should use background priority.
        pub indexing: CXGlobalOpt_ThreadBackgroundPriorityForIndexing,
    }
}

// Token _________________________________________

/// A lexed piece of a source file.
#[derive(Copy, Clone)]
pub struct Token<'tu> {
    raw: ffi::CXToken,
    tu: &'tu TranslationUnit<'tu>,
}

impl<'tu> Token<'tu> {
    //- Constructors -----------------------------

    fn from_raw(raw: ffi::CXToken, tu: &'tu TranslationUnit<'tu>) -> Token<'tu> {
        Token{ raw: raw, tu: tu }
    }

    //- Accessors --------------------------------

    /// Returns the categorization of this token.
    pub fn get_kind(&self) -> TokenKind {
        unsafe { mem::transmute(ffi::clang_getTokenKind(self.raw)) }
    }

    /// Returns the source location of this token.
    pub fn get_location(&self) -> SourceLocation<'tu> {
        unsafe {
            let raw = ffi::clang_getTokenLocation(self.tu.ptr, self.raw);
            SourceLocation::from_raw(raw, self.tu)
        }
    }

    /// Returns the source range of this token.
    pub fn get_range(&self) -> SourceRange<'tu> {
        unsafe { SourceRange::from_raw(ffi::clang_getTokenExtent(self.tu.ptr, self.raw), self.tu) }
    }

    /// Returns the textual representation of this token.
    pub fn get_spelling(&self) -> String {
        unsafe { to_string(ffi::clang_getTokenSpelling(self.tu.ptr, self.raw)) }
    }
}

impl<'tu> fmt::Debug for Token<'tu> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.debug_struct("Token")
            .field("range", &self.get_range())
            .field("kind", &self.get_kind())
            .field("spelling", &self.get_spelling())
            .finish()
    }
}

// TranslationUnit _______________________________

/// A preprocessed and parsed source file.
pub struct TranslationUnit<'i> {
    ptr: ffi::CXTranslationUnit,
    _marker: PhantomData<&'i Index<'i>>,
}

impl<'i> TranslationUnit<'i> {
    //- Constructors -----------------------------

    fn from_ptr(ptr: ffi::CXTranslationUnit) -> TranslationUnit<'i> {
        TranslationUnit{ ptr: ptr, _marker: PhantomData }
    }

    /// Constructs a new `TranslationUnit` from an AST file.
    ///
    /// # Failures
    ///
    /// * an unknown error occurs
    pub fn from_ast<F: AsRef<Path>>(
        index: &'i Index, file: F
    ) -> Result<TranslationUnit<'i>, ()> {
        let ptr = unsafe { ffi::clang_createTranslationUnit(index.ptr, from_path(file).as_ptr()) };
        ptr.map(TranslationUnit::from_ptr).ok_or(())
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
        index: &'i Index,
        file: F,
        arguments: &[&str],
        unsaved: &[Unsaved],
        options: ParseOptions,
    ) -> Result<TranslationUnit<'i>, SourceError> {
        let arguments = arguments.iter().map(from_string).collect::<Vec<_>>();
        let arguments = arguments.iter().map(|a| a.as_ptr()).collect::<Vec<_>>();
        let unsaved = unsaved.iter().map(|u| u.as_raw()).collect::<Vec<_>>();

        unsafe {
            let mut ptr = mem::uninitialized();

            let code = ffi::clang_parseTranslationUnit2(
                index.ptr,
                from_path(file).as_ptr(),
                arguments.as_ptr(),
                arguments.len() as c_int,
                mem::transmute(unsaved.as_ptr()),
                unsaved.len() as c_uint,
                options.into(),
                &mut ptr,
            );

            match code {
                ffi::CXErrorCode::Success => Ok(TranslationUnit::from_ptr(ptr)),
                ffi::CXErrorCode::ASTReadError => Err(SourceError::AstDeserialization),
                ffi::CXErrorCode::Crashed => Err(SourceError::Crash),
                ffi::CXErrorCode::Failure => Err(SourceError::Unknown),
                _ => unreachable!(),
            }
        }
    }

    //- Accessors --------------------------------

    /// Returns the AST entities which correspond to the supplied tokens, if any.
    pub fn annotate(&'i self, tokens: &[Token<'i>]) -> Vec<Option<Entity<'i>>> {
        unsafe {
            let mut raws = vec![mem::uninitialized(); tokens.len()];

            ffi::clang_annotateTokens(
                self.ptr, mem::transmute(tokens.as_ptr()), tokens.len() as c_uint, raws.as_mut_ptr()
            );

            raws.iter().map(|e| e.map(|e| Entity::from_raw(e, self))).collect()
        }
    }

    /// Runs code completion at the supplied location.
    pub fn complete<F: AsRef<Path>>(
        &self,
        file: F,
        line: u32,
        column: u32,
        unsaved: &[Unsaved],
        options: CompletionOptions,
    ) -> CompletionResults {
        unsafe {
            let ptr = ffi::clang_codeCompleteAt(
                self.ptr,
                from_path(file).as_ptr(),
                line as c_uint,
                column as c_uint,
                mem::transmute(unsaved.as_ptr()),
                unsaved.len() as c_uint,
                options.into(),
            );

            CompletionResults::from_ptr(ptr)
        }
    }

    /// Returns the diagnostics for this translation unit.
    pub fn get_diagnostics(&'i self) -> Vec<Diagnostic<'i>> {
        iter!(clang_getNumDiagnostics(self.ptr), clang_getDiagnostic(self.ptr),).map(|d| {
            Diagnostic::from_ptr(d, self)
        }).collect()
    }

    /// Returns the entity for this translation unit.
    pub fn get_entity(&'i self) -> Entity<'i> {
        unsafe { Entity::from_raw(ffi::clang_getTranslationUnitCursor(self.ptr), self) }
    }

    /// Returns the file at the supplied path in this translation unit, if any.
    pub fn get_file<F: AsRef<Path>>(&'i self, file: F) -> Option<File<'i>> {
        let file = unsafe { ffi::clang_getFile(self.ptr, from_path(file).as_ptr()) };
        file.map(|f| File::from_ptr(f, self))
    }

    /// Returns the memory usage of this translation unit.
    pub fn get_memory_usage(&self) -> HashMap<MemoryUsage, usize> {
        unsafe {
            let raw = ffi::clang_getCXTUResourceUsage(self.ptr);
            let raws = slice::from_raw_parts(raw.entries, raw.numEntries as usize);
            let usage = raws.iter().map(|u| (mem::transmute(u.kind), u.amount as usize)).collect();
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
                self.ptr, from_path(file).as_ptr(), ffi::CXSaveTranslationUnit_None
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
                self.ptr,
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
        unsafe { ffi::clang_disposeTranslationUnit(self.ptr); }
    }
}

impl<'i> fmt::Debug for TranslationUnit<'i> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let spelling = unsafe { ffi::clang_getTranslationUnitSpelling(self.ptr) };
        formatter.debug_struct("TranslationUnit").field("spelling", &to_string(spelling)).finish()
    }
}

// Type __________________________________________

/// The type of an AST entity.
#[derive(Copy, Clone)]
pub struct Type<'tu> {
    raw: ffi::CXType,
    tu: &'tu TranslationUnit<'tu>,
}

impl<'tu> Type<'tu> {
    //- Constructors -----------------------------

    fn from_raw(raw: ffi::CXType, tu: &'tu TranslationUnit<'tu>) -> Type<'tu> {
        Type { raw: raw, tu: tu }
    }

    //- Accessors --------------------------------

    /// Returns the alignment of this type in bytes.
    ///
    /// # Failures
    ///
    /// * this type is a dependent type
    /// * this type is an incomplete type
    pub fn get_alignof(&self) -> Result<usize, AlignofError> {
        unsafe {
            match ffi::clang_Type_getAlignOf(self.raw) {
                -3 => Err(AlignofError::Dependent),
                -2 => Err(AlignofError::Incomplete),
                other => Ok(other as usize),
            }
        }
    }

    /// Returns the argument types for this function or method type, if applicable.
    pub fn get_argument_types(&self) -> Option<Vec<Type<'tu>>> {
        iter_option!(
            clang_getNumArgTypes(self.raw),
            clang_getArgType(self.raw),
        ).map(|i| i.map(|t| Type::from_raw(t, self.tu)).collect())
    }

    /// Returns the calling convention specified for this function type, if applicable.
    pub fn get_calling_convention(&self) -> Option<CallingConvention> {
        unsafe {
            match ffi::clang_getFunctionTypeCallingConv(self.raw) {
                ffi::CXCallingConv::Invalid => None,
                other => Some(mem::transmute(other)),
            }
        }
    }

    /// Returns the canonical type for this type.
    ///
    /// The canonical type is the underlying type with all "sugar" removed (e.g., typedefs).
    pub fn get_canonical_type(&self) -> Type<'tu> {
        unsafe { Type::from_raw(ffi::clang_getCanonicalType(self.raw), self.tu) }
    }

    /// Returns the class type for this member pointer type, if applicable.
    pub fn get_class_type(&self) -> Option<Type<'tu>> {
        unsafe { ffi::clang_Type_getClassType(self.raw).map(|t| Type::from_raw(t, self.tu)) }
    }

    /// Returns the AST entity that declared this type, if any.
    pub fn get_declaration(&self) -> Option<Entity<'tu>> {
        unsafe { ffi::clang_getTypeDeclaration(self.raw).map(|e| Entity::from_raw(e, self.tu)) }
    }

    /// Returns the display name of this type.
    pub fn get_display_name(&self) -> String {
        unsafe { to_string(ffi::clang_getTypeSpelling(self.raw)) }
    }

    /// Returns the element type for this array, complex, or vector type, if applicable.
    pub fn get_element_type(&self) -> Option<Type<'tu>> {
        unsafe { ffi::clang_getElementType(self.raw).map(|t| Type::from_raw(t, self.tu)) }
    }

    /// Returns the fields in this record type, if applicable.
    pub fn get_fields(&self) -> Option<Vec<Entity<'tu>>> {
        if self.get_kind() == TypeKind::Record {
            let mut fields = vec![];

            self.visit_fields(|e| {
                fields.push(e);
                true
            });

            Some(fields)
        } else {
            None
        }
    }

    /// Returns the offset of the field with the supplied name in this record type in bits.
    ///
    /// # Failures
    ///
    /// * this record type is a dependent type
    /// * this record record type is an incomplete type
    /// * this record type does not contain a field with the supplied name
    pub fn get_offsetof<F: AsRef<str>>(&self, field: F) -> Result<usize, OffsetofError> {
        unsafe {
            match ffi::clang_Type_getOffsetOf(self.raw, from_string(field).as_ptr()) {
                -1 => Err(OffsetofError::Parent),
                -2 => Err(OffsetofError::Incomplete),
                -3 => Err(OffsetofError::Dependent),
                -5 => Err(OffsetofError::Name),
                other => Ok(other as usize),
            }
        }
    }

    /// Returns the kind of this type.
    pub fn get_kind(&self) -> TypeKind {
        unsafe { mem::transmute(self.raw.kind) }
    }

    /// Returns the pointee type for this pointer type, if applicable.
    pub fn get_pointee_type(&self) -> Option<Type<'tu>> {
        unsafe { ffi::clang_getPointeeType(self.raw).map(|t| Type::from_raw(t, self.tu)) }
    }

    /// Returns the ref qualifier for this C++ function or method type, if applicable.
    pub fn get_ref_qualifier(&self) -> Option<RefQualifier> {
        unsafe {
            match ffi::clang_Type_getCXXRefQualifier(self.raw) {
                ffi::CXRefQualifierKind::None => None,
                other => Some(mem::transmute(other)),
            }
        }
    }

    /// Returns the result type for this function or method type, if applicable.
    pub fn get_result_type(&self) -> Option<Type<'tu>> {
        unsafe { ffi::clang_getResultType(self.raw).map(|t| Type::from_raw(t, self.tu)) }
    }

    /// Returns the size of this constant array or vector type, if applicable.
    pub fn get_size(&self) -> Option<usize> {
        let size = unsafe { ffi::clang_getNumElements(self.raw) };

        if size >= 0 {
            Some(size as usize)
        } else {
            None
        }
    }

    /// Returns the size of this type in bytes.
    ///
    /// # Failures
    ///
    /// * this type is a dependent type
    /// * this type is an incomplete type
    /// * this type is a variable size type
    pub fn get_sizeof(&self) -> Result<usize, SizeofError> {
        unsafe {
            match ffi::clang_Type_getSizeOf(self.raw) {
                -2 => Err(SizeofError::Incomplete),
                -3 => Err(SizeofError::Dependent),
                -4 => Err(SizeofError::VariableSize),
                other => Ok(other as usize),
            }
        }
    }

    /// Returns the template argument types for this template class specialization type, if
    /// applicable.
    pub fn get_template_argument_types(&self) -> Option<Vec<Option<Type<'tu>>>> {
        iter_option!(
            clang_Type_getNumTemplateArguments(self.raw),
            clang_Type_getTemplateArgumentAsType(self.raw),
        ).map(|i| i.map(|t| t.map(|t| Type::from_raw(t, self.tu))).collect())
    }

    /// Returns whether this type is qualified with const.
    pub fn is_const_qualified(&self) -> bool {
        unsafe { ffi::clang_isConstQualifiedType(self.raw) != 0 }
    }

    /// Returns whether this type is plain old data (POD).
    pub fn is_pod(&self) -> bool {
        unsafe { ffi::clang_isPODType(self.raw) != 0 }
    }

    /// Returns whether this type is qualified with restrict.
    pub fn is_restrict_qualified(&self) -> bool {
        unsafe { ffi::clang_isRestrictQualifiedType(self.raw) != 0 }
    }

    /// Returns whether this type is a variadic function type.
    pub fn is_variadic(&self) -> bool {
        unsafe { ffi::clang_isFunctionTypeVariadic(self.raw) != 0 }
    }

    /// Returns whether this type is qualified with volatile.
    pub fn is_volatile_qualified(&self) -> bool {
        unsafe { ffi::clang_isVolatileQualifiedType(self.raw) != 0 }
    }

    /// Visits the fields in this record type, returning `None` if this type is not a record type
    /// and returning `Some(b)` otherwise where `b` indicates whether visitation was ended by the
    /// callback returning `false`.
    pub fn visit_fields<F: FnMut(Entity<'tu>) -> bool>(&self, f: F) -> Option<bool> {
        if self.get_kind() != TypeKind::Record {
            return None;
        }

        trait Callback<'tu> {
            fn call(&mut self, field: Entity<'tu>) -> bool;
        }

        impl<'tu, F: FnMut(Entity<'tu>) -> bool> Callback<'tu> for F {
            fn call(&mut self, field: Entity<'tu>) -> bool {
                self(field)
            }
        }

        extern fn visit(cursor: ffi::CXCursor, data: ffi::CXClientData) -> ffi::CXVisitorResult {
            unsafe {
                let &mut (tu, ref mut callback):
                    &mut (&TranslationUnit, Box<Callback>) =
                        mem::transmute(data);

                if callback.call(Entity::from_raw(cursor, tu)) {
                    ffi::CXVisitorResult::Continue
                } else {
                    ffi::CXVisitorResult::Break
                }
            }
        }

        let mut data = (self.tu, Box::new(f) as Box<Callback>);

        unsafe {
            let data = mem::transmute(&mut data);
            Some(ffi::clang_Type_visitFields(self.raw, visit, data) == ffi::CXVisitorResult::Break)
        }
    }
}

impl<'tu> cmp::Eq for Type<'tu> { }

impl<'tu> cmp::PartialEq for Type<'tu> {
    fn eq(&self, other: &Type<'tu>) -> bool {
        unsafe { ffi::clang_equalTypes(self.raw, other.raw) != 0 }
    }
}

impl<'tu> fmt::Debug for Type<'tu> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.debug_struct("Type")
            .field("kind", &self.get_kind())
            .field("display_name", &self.get_display_name())
            .finish()
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
    pub fn new<P: AsRef<Path>, C: AsRef<str>>(path: P, contents: C) -> Unsaved {
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

// Version _______________________________________

/// A version number in the form `x.y.z`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Version {
    /// The `x` component of the version number.
    pub x: i32,
    /// The `y` component of the version number.
    pub y: i32,
    /// The `z` component of the version number.
    pub z: i32,
}

impl Version {
    //- Constructors -----------------------------

    fn from_raw(raw: ffi::CXVersion) -> Version {
        Version { x: raw.Major as i32, y: raw.Minor as i32, z: raw.Subminor as i32 }
    }
}

//================================================
// Functions
//================================================

fn from_path<P: AsRef<Path>>(path: P) -> std::ffi::CString {
    from_string(path.as_ref().as_os_str().to_str().expect("invalid C string"))
}

fn from_string<S: AsRef<str>>(string: S) -> std::ffi::CString {
    std::ffi::CString::new(string.as_ref()).expect("invalid C string")
}

fn to_string(clang: ffi::CXString) -> String {
    unsafe {
        let c = std::ffi::CStr::from_ptr(ffi::clang_getCString(clang));
        let rust = c.to_str().expect("invalid Rust string").into();
        ffi::clang_disposeString(clang);
        rust
    }
}

fn to_string_option(clang: ffi::CXString) -> Option<String> {
    clang.map(to_string).and_then(|s| {
        if !s.is_empty() {
            Some(s)
        } else {
            None
        }
    })
}

fn visit<'tu, F, G>(tu: &'tu TranslationUnit<'tu>, f: F, g: G) -> bool
    where F: FnMut(Entity<'tu>, SourceRange<'tu>) -> bool,
          G: Fn(ffi::CXCursorAndRangeVisitor) -> ffi::CXResult
{
    trait Callback<'tu> {
        fn call(&mut self, entity: Entity<'tu>, range: SourceRange<'tu>) -> bool;
    }

    impl<'tu, F: FnMut(Entity<'tu>, SourceRange<'tu>) -> bool> Callback<'tu> for F {
        fn call(&mut self, entity: Entity<'tu>, range: SourceRange<'tu>) -> bool {
            self(entity, range)
        }
    }

    extern fn visit(
        data: ffi::CXClientData, cursor: ffi::CXCursor, range: ffi::CXSourceRange
    ) -> ffi::CXVisitorResult {
        unsafe {
            let &mut (tu, ref mut callback):
                &mut (&TranslationUnit, Box<Callback>) =
                    mem::transmute(data);

            if callback.call(Entity::from_raw(cursor, tu), SourceRange::from_raw(range, tu)) {
                ffi::CXVisitorResult::Continue
            } else {
                ffi::CXVisitorResult::Break
            }
        }
    }

    let mut data = (tu, Box::new(f) as Box<Callback>);

    let visitor = ffi::CXCursorAndRangeVisitor {
        context: unsafe { mem::transmute(&mut data) }, visit: visit
    };

    g(visitor) == ffi::CXResult::VisitBreak
}

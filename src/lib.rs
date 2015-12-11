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
    ($num:ident($($num_argument:expr), *), $get:ident($($get_argument:expr), *)) => ({
        let count = unsafe { ffi::$num($($num_argument), *) };
        (0..count).map(|i| unsafe { ffi::$get($($get_argument), *, i) })
    });

    ($num:ident($($num_argument:expr), *), $get:ident($($get_argument:expr), *),) => ({
        iter!($num($($num_argument), *), $get($($get_argument), *))
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
    /// The function type uses the x86 `stdcall` calling convention.
    Stdcall = 2,
    /// The function type uses the x86 `thiscall` calling convention.
    Thiscall = 4,
    /// The function type uses the x86 `vectorcall` calling convention.
    Vectorcall = 12,
    /// The function type uses the x86 `pascal` calling convention.
    Pascal = 5,
    /// The function type uses the ARM AACPS calling convention.
    Aapcs = 6,
    /// The function type uses the ARM AACPS-VFP calling convention.
    AapcsVfp = 7,
    /// The function type uses the calling convention for Intel OpenCL built-ins.
    IntelOcl = 9,
    /// The function type uses the x64 C calling convention as implemented on Windows.
    Win64 = 10,
    /// The function type uses the x64 C calling convention as specified in the System V ABI.
    SysV64= 11,
    /// The function type uses a calling convention that is not exposed via this interface.
    Unexposed = 200,
}

// EntityKind ____________________________________

/// Indicates the kind of an AST entity.
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
    ObjCSuperClassRef = 40,
    ObjCProtocolRef = 41,
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
    IBActionAttr = 401,
    IBOutletAttr = 402,
    IBOutletCollectionAttr = 403,
    FinalAttr = 404,
    OverrideAttr = 405,
    AnnotateAttr = 406,
    AsmLabelAttr = 407,
    PackedAttr = 408,
    PureAttr = 409,
    ConstAttr = 410,
    NoDuplicateAttr = 411,
    CudaConstantAttr = 412,
    CudaDeviceAttr = 413,
    CudaGlobalAttr = 414,
    CudaHostAttr = 415,
    CudaSharedAttr = 416,
    PreprocessingDirective = 500,
    MacroDefinition = 501,
    MacroExpansion = 502,
    InclusionDirective = 503,
    ModuleImportDecl = 600,
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

// Language ______________________________________

/// Indicates the language used by an AST entity.
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
    LValue = 1,
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

// TypeKind ______________________________________

/// Indicates the kind of a type.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum TypeKind {
    /// A type whose specific kind is not exposed via this interface.
    Unexposed = 1,
    Void = 2,
    Bool = 3,
    /// The `char` type when it is signed by default.
    CharS = 13,
    /// The `char` type when it is unsigned by default.
    CharU = 4,
    /// The `signed char` type.
    SChar = 14,
    /// The `unsigned char` type.
    UChar = 5,
    WChar = 15,
    Char16 = 6,
    Char32 = 7,
    Short = 16,
    UShort = 8,
    Int = 17,
    UInt = 9,
    Long = 18,
    ULong = 10,
    LongLong = 19,
    ULongLong = 11,
    Int128 = 20,
    UInt128 = 12,
    Float = 21,
    Double = 22,
    LongDouble = 23,
    Nullptr = 24,
    Overload = 25,
    Dependent = 26,
    ObjCId = 27,
    ObjCClass = 28,
    ObjCSel = 29,
    Complex = 100,
    Pointer = 101,
    BlockPointer = 102,
    LValueReference = 103,
    RValueReference = 104,
    Record = 105,
    Enum = 106,
    Typedef = 107,
    ObjCInterface = 108,
    ObjCObjectPointer = 109,
    FunctionNoPrototype = 110,
    FunctionPrototype = 111,
    ConstantArray = 112,
    Vector = 113,
    IncompleteArray = 114,
    VariableArray = 115,
    DependentSizedArray = 116,
    MemberPointer = 117,
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
        if AVAILABLE.swap(false, Ordering::Relaxed) {
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
        AVAILABLE.store(true, Ordering::Relaxed);
    }
}

// Diagnostic ____________________________________

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
    pub fn get_display_name(&self) -> Option<String> {
        unsafe { to_string_option(ffi::clang_getCursorDisplayName(self.raw)) }
    }

    /// Returns the kind of this AST entity.
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

    /// Returns the translation unit which contains this AST entity.
    pub fn get_translation_unit(&self) -> &'tu TranslationUnit<'tu> {
        self.tu
    }

    /// Returns the type of this AST entity, if any.
    pub fn get_type(&self) -> Option<Type<'tu>> {
        unsafe { ffi::clang_getCursorType(self.raw).map(|t| Type::from_raw(t, self.tu)) }
    }

    /// Returns whether this AST entity is an anonymous record declaration.
    pub fn is_anonymous(&self) -> bool {
        unsafe { ffi::clang_Cursor_isAnonymous(self.raw) != 0 }
    }

    /// Returns whether this AST entity is an attribute.
    pub fn is_attribute(&self) -> bool {
        unsafe { ffi::clang_isAttribute(self.raw.kind) != 0 }
    }

    /// Returns whether this AST entity is a bit field.
    pub fn is_bit_field(&self) -> bool {
        unsafe { ffi::clang_Cursor_isBitField(self.raw) != 0 }
    }

    /// Returns whether this AST entity is a const method.
    pub fn is_const_method(&self) -> bool {
        unsafe { ffi::clang_CXXMethod_isConst(self.raw) != 0 }
    }

    /// Returns whether this AST entity is a declaration.
    pub fn is_declaration(&self) -> bool {
        unsafe { ffi::clang_isDeclaration(self.raw.kind) != 0 }
    }

    /// Returns whether this AST entity is a dynamic call.
    ///
    /// A dynamic call is either a call to a C++ virtual method or an Objective-C message where the
    /// receiver is an object instance, not `super` or a specific class.
    pub fn is_dynamic_call(&self) -> bool {
        unsafe { ffi::clang_Cursor_isDynamicCall(self.raw) != 0 }
    }

    /// Returns whether this AST entity is an expression.
    pub fn is_expression(&self) -> bool {
        unsafe { ffi::clang_isExpression(self.raw.kind) != 0 }
    }

    /// Returns whether this AST entity is a preprocessing entity.
    pub fn is_preprocessing(&self) -> bool {
        unsafe { ffi::clang_isPreprocessing(self.raw.kind) != 0 }
    }

    /// Returns whether this AST entity is a pure virtual method.
    pub fn is_pure_virtual_method(&self) -> bool {
        unsafe { ffi::clang_CXXMethod_isPureVirtual(self.raw) != 0 }
    }

    /// Returns whether this AST entity is categorized as a reference.
    pub fn is_reference(&self) -> bool {
        unsafe { ffi::clang_isReference(self.raw.kind) != 0 }
    }

    /// Returns whether this AST entity is a statement.
    pub fn is_statement(&self) -> bool {
        unsafe { ffi::clang_isStatement(self.raw.kind) != 0 }
    }

    /// Returns whether this AST entity is a static method.
    pub fn is_static_method(&self) -> bool {
        unsafe { ffi::clang_CXXMethod_isStatic(self.raw) != 0 }
    }

    /// Returns whether the specific kind of this AST entity is unexposed.
    pub fn is_unexposed(&self) -> bool {
        unsafe { ffi::clang_isUnexposed(self.raw.kind) != 0 }
    }

    /// Returns whether this AST entity is a variadic function or method.
    pub fn is_variadic(&self) -> bool {
        unsafe { ffi::clang_Cursor_isVariadic(self.raw) != 0 }
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

        extern fn visit<'tu>(
            cursor: ffi::CXCursor, parent: ffi::CXCursor, data: ffi::CXClientData
        ) -> ffi::CXChildVisitResult {
            unsafe {
                let &mut (tu, ref mut callback):
                    &mut (&'tu TranslationUnit<'tu>, Box<EntityCallback<'tu>>) =
                        mem::transmute(data);

                let entity = Entity::from_raw(cursor, tu);
                let parent = Entity::from_raw(parent, tu);
                mem::transmute(callback.call(entity, parent))
            }
        }

        let mut data = (self.tu, Box::new(f) as Box<EntityCallback>);
        unsafe { ffi::clang_visitChildren(self.raw, visit, mem::transmute(&mut data)) != 0 }
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

    /// Returns the last modification time for this file.
    pub fn get_time(&self) -> time_t {
        unsafe { ffi::clang_getFileTime(self.ptr) }
    }

    /// Returns whether this file is guarded against multiple inclusions.
    pub fn is_include_guarded(&self) -> bool {
        unsafe { ffi::clang_isFileMultipleIncludeGuarded(self.tu.ptr, self.ptr) != 0 }
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
        pub display_source_location: CXDiagnostic_DisplaySourceLocation,
        /// Indicates whether the column will be included in the source location prefix.
        pub display_column: CXDiagnostic_DisplayColumn,
        /// Indicates whether the source ranges will be included to the source location prefix.
        pub display_source_ranges: CXDiagnostic_DisplaySourceRanges,
        /// Indicates whether the option associated with the diagnostic (e.g., `-Wconversion`) will
        /// be placed in brackets after the diagnostic text if there is such an option.
        pub display_option: CXDiagnostic_DisplayOption,
        /// Indicates whether the category number associated with the diagnostic will be placed in
        /// brackets after the diagnostic text if there is such a category number.
        pub display_category_id: CXDiagnostic_DisplayCategoryId,
        /// Indicates whether the category name associated with the diagnostic will be placed in
        /// brackets after the diagnostic text if there is such a category name.
        pub display_category_name: CXDiagnostic_DisplayCategoryName,
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
        index: &'i mut Index, file: F
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
        index: &'i mut Index,
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

    /// Returns the diagnostics for this translation unit.
    pub fn get_diagnostics<>(&'i self) -> Vec<Diagnostic<'i>> {
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

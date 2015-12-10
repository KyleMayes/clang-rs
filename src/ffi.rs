#![allow(non_upper_case_globals, non_snake_case)]

use libc::{c_char, c_int, c_longlong, c_uint, c_ulong, c_ulonglong, c_void, time_t};

use super::{Nullable};

//================================================
// Typedefs
//================================================

pub type CXClientData = *mut c_void;
pub type CXCursorVisitor = extern fn(CXCursor, CXCursor, CXClientData) -> CXChildVisitResult;
pub type CXFieldVisitor = extern fn(CXCursor, CXClientData) -> CXVisitorResult;
pub type CXInclusionVisitor = extern fn(CXFile, *mut CXSourceLocation, c_uint, CXClientData);

//================================================
// Enums
//================================================

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum CXAvailabilityKind {
    Available = 0,
    Deprecated = 1,
    NotAvailable = 2,
    NotAccessible = 3,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum CXCallingConv {
    Default = 0,
    C = 1,
    X86StdCall = 2,
    X86FastCall = 3,
    X86ThisCall = 4,
    X86Pascal = 5,
    AAPCS = 6,
    AAPCS_VFP = 7,
    IntelOclBicc = 9,
    X86_64Win64 = 10,
    X86_64SysV = 11,
    X86VectorCall = 12,
    Invalid = 100,
    Unexposed = 200,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum CXChildVisitResult {
    Break = 0,
    Continue = 1,
    Recurse = 2,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum CXCompilationDatabase_Error {
    NoError = 0,
    CanNotLoadDatabase = 1,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum CXCompletionChunkKind {
    Optional = 0,
    TypedText = 1,
    Text = 2,
    Placeholder = 3,
    Informative = 4,
    CurrentParameter = 5,
    LeftParen = 6,
    RightParen = 7,
    LeftBracket = 8,
    RightBracket = 9,
    LeftBrace = 10,
    RightBrace = 11,
    LeftAngle = 12,
    RightAngle = 13,
    Comma = 14,
    ResultType = 15,
    Colon = 16,
    SemiColon = 17,
    Equal = 18,
    HorizontalSpace = 19,
    VerticalSpace = 20,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum CXCursorKind {
    UnexposedDecl = 1,
    StructDecl = 2,
    UnionDecl = 3,
    ClassDecl = 4,
    EnumDecl = 5,
    FieldDecl = 6,
    EnumConstantDecl = 7,
    FunctionDecl = 8,
    VarDecl = 9,
    ParmDecl = 10,
    ObjCInterfaceDecl = 11,
    ObjCCategoryDecl = 12,
    ObjCProtocolDecl = 13,
    ObjCPropertyDecl = 14,
    ObjCIvarDecl = 15,
    ObjCInstanceMethodDecl = 16,
    ObjCClassMethodDecl = 17,
    ObjCImplementationDecl = 18,
    ObjCCategoryImplDecl = 19,
    TypedefDecl = 20,
    CXXMethod = 21,
    Namespace = 22,
    LinkageSpec = 23,
    Constructor = 24,
    Destructor = 25,
    ConversionFunction = 26,
    TemplateTypeParameter = 27,
    NonTypeTemplateParameter = 28,
    TemplateTemplateParameter = 29,
    FunctionTemplate = 30,
    ClassTemplate = 31,
    ClassTemplatePartialSpecialization = 32,
    NamespaceAlias = 33,
    UsingDirective = 34,
    UsingDeclaration = 35,
    TypeAliasDecl = 36,
    ObjCSynthesizeDecl = 37,
    ObjCDynamicDecl = 38,
    CXXAccessSpecifier = 39,
    ObjCSuperClassRef = 40,
    ObjCProtocolRef = 41,
    ObjCClassRef = 42,
    TypeRef = 43,
    CXXBaseSpecifier = 44,
    TemplateRef = 45,
    NamespaceRef = 46,
    MemberRef = 47,
    LabelRef = 48,
    OverloadedDeclRef = 49,
    VariableRef = 50,
    InvalidFile = 70,
    NoDeclFound = 71,
    NotImplemented = 72,
    InvalidCode = 73,
    UnexposedExpr = 100,
    DeclRefExpr = 101,
    MemberRefExpr = 102,
    CallExpr = 103,
    ObjCMessageExpr = 104,
    BlockExpr = 105,
    IntegerLiteral = 106,
    FloatingLiteral = 107,
    ImaginaryLiteral = 108,
    StringLiteral = 109,
    CharacterLiteral = 110,
    ParenExpr = 111,
    UnaryOperator = 112,
    ArraySubscriptExpr = 113,
    BinaryOperator = 114,
    CompoundAssignOperator = 115,
    ConditionalOperator = 116,
    CStyleCastExpr = 117,
    CompoundLiteralExpr = 118,
    InitListExpr = 119,
    AddrLabelExpr = 120,
    StmtExpr = 121,
    GenericSelectionExpr = 122,
    GNUNullExpr = 123,
    CXXStaticCastExpr = 124,
    CXXDynamicCastExpr = 125,
    CXXReinterpretCastExpr = 126,
    CXXConstCastExpr = 127,
    CXXFunctionalCastExpr = 128,
    CXXTypeidExpr = 129,
    CXXBoolLiteralExpr = 130,
    CXXNullPtrLiteralExpr = 131,
    CXXThisExpr = 132,
    CXXThrowExpr = 133,
    CXXNewExpr = 134,
    CXXDeleteExpr = 135,
    UnaryExpr = 136,
    ObjCStringLiteral = 137,
    ObjCEncodeExpr = 138,
    ObjCSelectorExpr = 139,
    ObjCProtocolExpr = 140,
    ObjCBridgedCastExpr = 141,
    PackExpansionExpr = 142,
    SizeOfPackExpr = 143,
    LambdaExpr = 144,
    ObjCBoolLiteralExpr = 145,
    ObjCSelfExpr = 146,
    UnexposedStmt = 200,
    LabelStmt = 201,
    CompoundStmt = 202,
    CaseStmt = 203,
    DefaultStmt = 204,
    IfStmt = 205,
    SwitchStmt = 206,
    WhileStmt = 207,
    DoStmt = 208,
    ForStmt = 209,
    GotoStmt = 210,
    IndirectGotoStmt = 211,
    ContinueStmt = 212,
    BreakStmt = 213,
    ReturnStmt = 214,
    /// Duplicate of `GccAsmStmt`.
    AsmStmt = 215,
    ObjCAtTryStmt = 216,
    ObjCAtCatchStmt = 217,
    ObjCAtFinallyStmt = 218,
    ObjCAtThrowStmt = 219,
    ObjCAtSynchronizedStmt = 220,
    ObjCAutoreleasePoolStmt = 221,
    ObjCForCollectionStmt = 222,
    CXXCatchStmt = 223,
    CXXTryStmt = 224,
    CXXForRangeStmt = 225,
    SEHTryStmt = 226,
    SEHExceptStmt = 227,
    SEHFinallyStmt = 228,
    MSAsmStmt = 229,
    NullStmt = 230,
    DeclStmt = 231,
    OMPParallelDirective = 232,
    OMPSimdDirective = 233,
    OMPForDirective = 234,
    OMPSectionsDirective = 235,
    OMPSectionDirective = 236,
    OMPSingleDirective = 237,
    OMPParallelForDirective = 238,
    OMPParallelSectionsDirective = 239,
    OMPTaskDirective = 240,
    OMPMasterDirective = 241,
    OMPCriticalDirective = 242,
    OMPTaskyieldDirective = 243,
    OMPBarrierDirective = 244,
    OMPTaskwaitDirective = 245,
    OMPFlushDirective = 246,
    SEHLeaveStmt = 247,
    OMPOrderedDirective = 248,
    OMPAtomicDirective = 249,
    OMPForSimdDirective = 250,
    OMPParallelForSimdDirective = 251,
    OMPTargetDirective = 252,
    OMPTeamsDirective = 253,
    OMPTaskgroupDirective = 254,
    OMPCancellationPointDirective = 255,
    OMPCancelDirective = 256,
    TranslationUnit = 300,
    UnexposedAttr = 400,
    IBActionAttr = 401,
    IBOutletAttr = 402,
    IBOutletCollectionAttr = 403,
    CXXFinalAttr = 404,
    CXXOverrideAttr = 405,
    AnnotateAttr = 406,
    AsmLabelAttr = 407,
    PackedAttr = 408,
    PureAttr = 409,
    ConstAttr = 410,
    NoDuplicateAttr = 411,
    CUDAConstantAttr = 412,
    CUDADeviceAttr = 413,
    CUDAGlobalAttr = 414,
    CUDAHostAttr = 415,
    CUDASharedAttr = 416,
    PreprocessingDirective = 500,
    MacroDefinition = 501,
    /// Duplicate of `MacroInstantiation`.
    MacroExpansion = 502,
    InclusionDirective = 503,
    ModuleImportDecl = 600,
    OverloadCandidate = 700,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum CXDiagnosticSeverity {
    Ignored = 0,
    Note = 1,
    Warning = 2,
    Error = 3,
    Fatal = 4,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum CXErrorCode {
    Success = 0,
    Failure = 1,
    Crashed = 2,
    InvalidArguments = 3,
    ASTReadError = 4,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum CXIdxAttrKind {
    Unexposed = 0,
    IBAction = 1,
    IBOutlet = 2,
    IBOutletCollection = 3,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum CXIdxEntityCXXTemplateKind {
    NonTemplate = 0,
    Template = 1,
    TemplatePartialSpecialization = 2,
    TemplateSpecialization = 3,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum CXIdxEntityKind {
    Unexposed = 0,
    Typedef = 1,
    Function = 2,
    Variable = 3,
    Field = 4,
    EnumConstant = 5,
    ObjCClass = 6,
    ObjCProtocol = 7,
    ObjCCategory = 8,
    ObjCInstanceMethod = 9,
    ObjCClassMethod = 10,
    ObjCProperty = 11,
    ObjCIvar = 12,
    Enum = 13,
    Struct = 14,
    Union = 15,
    CXXClass = 16,
    CXXNamespace = 17,
    CXXNamespaceAlias = 18,
    CXXStaticVariable = 19,
    CXXStaticMethod = 20,
    CXXInstanceMethod = 21,
    CXXConstructor = 22,
    CXXDestructor = 23,
    CXXConversionFunction = 24,
    CXXTypeAlias = 25,
    CXXInterface = 26,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum CXIdxEntityLanguage {
    None = 0,
    C = 1,
    ObjC = 2,
    CXX = 3,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum CXIdxEntityRefKind {
    Direct = 1,
    Implicit = 2,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum CXIdxObjCContainerKind {
    ForwardRef = 0,
    Interface = 1,
    Implementation = 2,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum CXLanguageKind {
    Invalid = 0,
    C = 1,
    ObjC = 2,
    CPlusPlus = 3,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum CXLinkageKind {
    Invalid = 0,
    NoLinkage = 1,
    Internal = 2,
    UniqueExternal = 3,
    External = 4,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum CXLoadDiag_Error {
    None = 0,
    Unknown = 1,
    CannotLoad = 2,
    InvalidFile = 3,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum CXRefQualifierKind {
    None = 0,
    LValue = 1,
    RValue = 2,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum CXResult {
    Success = 0,
    Invalid = 1,
    VisitBreak = 2,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum CXSaveError {
    None = 0,
    Unknown = 1,
    TranslationErrors = 2,
    InvalidTU = 3,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum CXTUResourceUsageKind {
    AST = 1,
    Identifiers = 2,
    Selectors = 3,
    GlobalCompletionResults = 4,
    SourceManagerContentCache = 5,
    AST_SideTables = 6,
    SourceManager_Membuffer_Malloc = 7,
    SourceManager_Membuffer_MMap = 8,
    ExternalASTSource_Membuffer_Malloc = 9,
    ExternalASTSource_Membuffer_MMap = 10,
    Preprocessor = 11,
    PreprocessingRecord = 12,
    SourceManager_DataStructures = 13,
    Preprocessor_HeaderSearch = 14,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum CXTemplateArgumentKind {
    Null = 0,
    Type = 1,
    Declaration = 2,
    NullPtr = 3,
    Integral = 4,
    Template = 5,
    TemplateExpansion = 6,
    Expression = 7,
    Pack = 8,
    Invalid = 9,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum CXTokenKind {
    Punctuation = 0,
    Keyword = 1,
    Identifier = 2,
    Literal = 3,
    Comment = 4,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum CXTypeKind {
    Invalid = 0,
    Unexposed = 1,
    Void = 2,
    Bool = 3,
    Char_U = 4,
    UChar = 5,
    Char16 = 6,
    Char32 = 7,
    UShort = 8,
    UInt = 9,
    ULong = 10,
    ULongLong = 11,
    UInt128 = 12,
    Char_S = 13,
    SChar = 14,
    WChar = 15,
    Short = 16,
    Int = 17,
    Long = 18,
    LongLong = 19,
    Int128 = 20,
    Float = 21,
    Double = 22,
    LongDouble = 23,
    NullPtr = 24,
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
    FunctionNoProto = 110,
    FunctionProto = 111,
    ConstantArray = 112,
    Vector = 113,
    IncompleteArray = 114,
    VariableArray = 115,
    DependentSizedArray = 116,
    MemberPointer = 117,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum CXTypeLayoutError {
    Invalid = -1,
    Incomplete = -2,
    Dependent = -3,
    NotConstantSize = -4,
    InvalidFieldName = -5,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum CXVisitorResult {
    Break = 0,
    Continue = 1,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum CX_CXXAccessSpecifier {
    CXXInvalidAccessSpecifier = 0,
    CXXPublic = 1,
    CXXProtected = 2,
    CXXPrivate = 3,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum CX_StorageClass {
    SC_Invalid = 0,
    SC_None = 1,
    SC_Extern = 2,
    SC_Static = 3,
    SC_PrivateExtern = 4,
    SC_OpenCLWorkGroupLocal = 5,
    SC_Auto = 6,
    SC_Register = 7,
}

//================================================
// Flags
//================================================

bitflags! {
    #[repr(C)]
    flags CXCodeComplete_Flags: c_uint {
        const CXCodeComplete_IncludeMacros = 1,
        const CXCodeComplete_IncludeCodePatterns = 2,
        const CXCodeComplete_IncludeBriefComments = 4,
    }
}

bitflags! {
    #[repr(C)]
    flags CXCompletionContext: c_uint {
        const CXCompletionContext_Unexposed = 0,
        const CXCompletionContext_AnyType = 1,
        const CXCompletionContext_AnyValue = 2,
        const CXCompletionContext_ObjCObjectValue = 4,
        const CXCompletionContext_ObjCSelectorValue = 8,
        const CXCompletionContext_CXXClassTypeValue = 16,
        const CXCompletionContext_DotMemberAccess = 32,
        const CXCompletionContext_ArrowMemberAccess = 64,
        const CXCompletionContext_ObjCPropertyAccess = 128,
        const CXCompletionContext_EnumTag = 256,
        const CXCompletionContext_UnionTag = 512,
        const CXCompletionContext_StructTag = 1024,
        const CXCompletionContext_ClassTag = 2048,
        const CXCompletionContext_Namespace = 4096,
        const CXCompletionContext_NestedNameSpecifier = 8192,
        const CXCompletionContext_ObjCInterface = 16384,
        const CXCompletionContext_ObjCProtocol = 32768,
        const CXCompletionContext_ObjCCategory = 65536,
        const CXCompletionContext_ObjCInstanceMessage = 131072,
        const CXCompletionContext_ObjCClassMessage = 262144,
        const CXCompletionContext_ObjCSelectorName = 524288,
        const CXCompletionContext_MacroName = 1048576,
        const CXCompletionContext_NaturalLanguage = 2097152,
        const CXCompletionContext_Unknown = 4194303,
    }
}

bitflags! {
    #[repr(C)]
    flags CXDiagnosticDisplayOptions: c_uint {
        const CXDiagnostic_DisplaySourceLocation = 1,
        const CXDiagnostic_DisplayColumn = 2,
        const CXDiagnostic_DisplaySourceRanges = 4,
        const CXDiagnostic_DisplayOption = 8,
        const CXDiagnostic_DisplayCategoryId = 16,
        const CXDiagnostic_DisplayCategoryName = 32,
    }
}

bitflags! {
    #[repr(C)]
    flags CXGlobalOptFlags: c_uint {
        const CXGlobalOpt_None = 0,
        const CXGlobalOpt_ThreadBackgroundPriorityForIndexing = 1,
        const CXGlobalOpt_ThreadBackgroundPriorityForEditing = 2,
        const CXGlobalOpt_ThreadBackgroundPriorityForAll = 3,
    }
}

bitflags! {
    #[repr(C)]
    flags CXIdxDeclInfoFlags: c_uint {
        const CXIdxDeclFlag_Skipped = 1,
    }
}

bitflags! {
    #[repr(C)]
    flags CXIndexOptFlags: c_uint {
        const CXIndexOptNone = 0,
        const CXIndexOptSuppressRedundantRefs = 1,
        const CXIndexOptIndexFunctionLocalSymbols = 2,
        const CXIndexOptIndexImplicitTemplateInstantiations = 4,
        const CXIndexOptSuppressWarnings = 8,
        const CXIndexOptSkipParsedBodiesInSession = 16,
    }
}

bitflags! {
    #[repr(C)]
    flags CXNameRefFlags: c_uint {
        const CXNameRange_WantQualifier = 1,
        const CXNameRange_WantTemplateArgs = 2,
        const CXNameRange_WantSinglePiece = 4
    }
}

bitflags! {
    #[repr(C)]
    flags CXObjCDeclQualifierKind: c_uint {
        const CXObjCDeclQualifier_None = 0,
        const CXObjCDeclQualifier_In = 1,
        const CXObjCDeclQualifier_Inout = 2,
        const CXObjCDeclQualifier_Out = 4,
        const CXObjCDeclQualifier_Bycopy = 8,
        const CXObjCDeclQualifier_Byref = 16,
        const CXObjCDeclQualifier_Oneway = 32,
    }
}

bitflags! {
    #[repr(C)]
    flags CXObjCPropertyAttrKind: c_uint {
        const CXObjCPropertyAttr_noattr = 0,
        const CXObjCPropertyAttr_readonly = 1,
        const CXObjCPropertyAttr_getter = 2,
        const CXObjCPropertyAttr_assign = 4,
        const CXObjCPropertyAttr_readwrite = 8,
        const CXObjCPropertyAttr_retain = 16,
        const CXObjCPropertyAttr_copy = 32,
        const CXObjCPropertyAttr_nonatomic = 64,
        const CXObjCPropertyAttr_setter = 128,
        const CXObjCPropertyAttr_atomic = 256,
        const CXObjCPropertyAttr_weak = 512,
        const CXObjCPropertyAttr_strong = 1024,
        const CXObjCPropertyAttr_unsafe_unretained = 2048,
    }
}

bitflags! {
    #[repr(C)]
    flags CXReparse_Flags: c_uint {
        const CXReparse_None = 0,
    }
}

bitflags! {
    #[repr(C)]
    flags CXSaveTranslationUnit_Flags: c_uint {
        const CXSaveTranslationUnit_None = 0,
    }
}

bitflags! {
    #[repr(C)]
    flags CXTranslationUnit_Flags: c_uint {
        const CXTranslationUnit_None = 0,
        const CXTranslationUnit_DetailedPreprocessingRecord = 1,
        const CXTranslationUnit_Incomplete = 2,
        const CXTranslationUnit_PrecompiledPreamble = 4,
        const CXTranslationUnit_CacheCompletionResults = 8,
        const CXTranslationUnit_ForSerialization = 16,
        const CXTranslationUnit_CXXChainedPCH = 32,
        const CXTranslationUnit_SkipFunctionBodies = 64,
        const CXTranslationUnit_IncludeBriefCommentsInCodeCompletion = 128,
    }
}

//================================================
// Structs
//================================================

// Opaque ________________________________________

macro_rules! opaque {
    ($name:ident) => (
        #[derive(Copy, Clone, Debug)]
        #[repr(C)]
        pub struct $name(pub *mut c_void);

        impl Nullable<$name> for $name {
            fn map<U, F: FnOnce($name) -> U>(self, f: F) -> Option<U> {
                if !self.0.is_null() {
                    Some(f(self))
                } else {
                    None
                }
            }
        }
    );
}

opaque!(CXCompilationDatabase);
opaque!(CXCompileCommand);
opaque!(CXCompileCommands);
opaque!(CXCompletionString);
opaque!(CXCursorSet);
opaque!(CXDiagnostic);
opaque!(CXDiagnosticSet);
opaque!(CXFile);
opaque!(CXIdxClientASTFile);
opaque!(CXIdxClientContainer);
opaque!(CXIdxClientEntity);
opaque!(CXIdxClientFile);
opaque!(CXIndex);
opaque!(CXIndexAction);
opaque!(CXModule);
opaque!(CXModuleMapDescriptor);
opaque!(CXRemapping);
opaque!(CXTranslationUnit);
opaque!(CXVirtualFileOverlay);

// Transparent ___________________________________

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CXCodeCompleteResults {
    pub Results: *mut CXCompletionResult,
    pub NumResults: c_uint,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CXCompletionResult {
    pub CursorKind: CXCursorKind,
    pub CompletionString: *mut c_void,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CXCursor {
    pub kind: CXCursorKind,
    pub xdata: c_int,
    pub data: [*const c_void; 3],
}

impl Nullable<CXCursor> for CXCursor {
    fn map<U, F: FnOnce(CXCursor) -> U>(self, f: F) -> Option<U> {
        unsafe {
            let null = clang_equalCursors(self, clang_getNullCursor()) != 0;

            if !null && clang_isInvalid(self.kind) == 0 {
                Some(f(self))
            } else {
                None
            }
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CXCursorAndRangeVisitor {
    pub context: *mut c_void,
    pub visit: extern fn(*mut c_void, CXCursor, CXSourceRange) -> CXVisitorResult,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CXFileUniqueID {
    pub data: [c_ulonglong; 3],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CXIdxAttrInfo {
    pub kind: CXIdxAttrKind,
    pub cursor: CXCursor,
    pub loc: CXIdxLoc,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CXIdxBaseClassInfo {
    pub base: *const CXIdxEntityInfo,
    pub cursor: CXCursor,
    pub loc: CXIdxLoc,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CXIdxCXXClassDeclInfo {
    pub declInfo: *const CXIdxDeclInfo,
    pub bases: *const *const CXIdxBaseClassInfo,
    pub numBases: c_uint,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CXIdxContainerInfo {
    pub cursor: CXCursor,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CXIdxDeclInfo {
    pub entityInfo: *const CXIdxEntityInfo,
    pub cursor: CXCursor,
    pub loc: CXIdxLoc,
    pub semanticContainer: *const CXIdxContainerInfo,
    pub lexicalContainer: *const CXIdxContainerInfo,
    pub isRedeclaration: c_int,
    pub isDefinition: c_int,
    pub isContainer: c_int,
    pub declAsContainer: *const CXIdxContainerInfo,
    pub isImplicit: c_int,
    pub attributes: *const *const CXIdxAttrInfo,
    pub numAttributes: c_uint,
    pub flags: c_uint,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CXIdxEntityInfo {
    pub kind: CXIdxEntityKind,
    pub templateKind: CXIdxEntityCXXTemplateKind,
    pub lang: CXIdxEntityLanguage,
    pub name: *const c_char,
    pub USR: *const c_char,
    pub cursor: CXCursor,
    pub attributes: *const *const CXIdxAttrInfo,
    pub numAttributes: c_uint,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CXIdxEntityRefInfo {
    pub kind: CXIdxEntityRefKind,
    pub cursor: CXCursor,
    pub loc: CXIdxLoc,
    pub referencedEntity: *const CXIdxEntityInfo,
    pub parentEntity: *const CXIdxEntityInfo,
    pub container: *const CXIdxContainerInfo,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CXIdxIBOutletCollectionAttrInfo {
    pub attrInfo: *const CXIdxAttrInfo,
    pub objcClass: *const CXIdxEntityInfo,
    pub classCursor: CXCursor,
    pub classLoc: CXIdxLoc,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CXIdxImportedASTFileInfo {
    pub file: *mut c_void,
    pub module: *mut c_void,
    pub loc: CXIdxLoc,
    pub isImplicit: c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CXIdxIncludedFileInfo {
    pub hashLoc: CXIdxLoc,
    pub filename: *const c_char,
    pub file: *mut c_void,
    pub isImport: c_int,
    pub isAngled: c_int,
    pub isModuleImport: c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CXIdxLoc {
    pub ptr_data: [*mut c_void; 2],
    pub int_data: c_uint,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CXIdxObjCCategoryDeclInfo {
    pub containerInfo: *const CXIdxObjCContainerDeclInfo,
    pub objcClass: *const CXIdxEntityInfo,
    pub classCursor: CXCursor,
    pub classLoc: CXIdxLoc,
    pub protocols: *const CXIdxObjCProtocolRefListInfo,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CXIdxObjCContainerDeclInfo {
    pub declInfo: *const CXIdxDeclInfo,
    pub kind: CXIdxObjCContainerKind,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CXIdxObjCInterfaceDeclInfo {
    pub containerInfo: *const CXIdxObjCContainerDeclInfo,
    pub superInfo: *const CXIdxBaseClassInfo,
    pub protocols: *const CXIdxObjCProtocolRefListInfo,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CXIdxObjCPropertyDeclInfo {
    pub declInfo: *const CXIdxDeclInfo,
    pub getter: *const CXIdxEntityInfo,
    pub setter: *const CXIdxEntityInfo,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CXIdxObjCProtocolRefInfo {
    pub protocol: *const CXIdxEntityInfo,
    pub cursor: CXCursor,
    pub loc: CXIdxLoc,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CXIdxObjCProtocolRefListInfo {
    pub protocols: *const *const CXIdxObjCProtocolRefInfo,
    pub numProtocols: c_uint,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CXPlatformAvailability {
    pub Platform: c_int,
    pub Introduced: CXVersion,
    pub Deprecated: CXVersion,
    pub Obsoleted: CXVersion,
    pub Unavailable: c_int,
    pub Message: c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CXSourceLocation {
    pub ptr_data: [*const c_void; 2],
    pub int_data: c_uint,
}

impl Nullable<CXSourceLocation> for CXSourceLocation {
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

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CXSourceRange {
    pub ptr_data: [*const c_void; 2],
    pub begin_int_data: c_uint,
    pub end_int_data: c_uint,
}

impl Nullable<CXSourceRange> for CXSourceRange {
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

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CXSourceRangeList {
    pub count: c_uint,
    pub ranges: *mut CXSourceRange,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CXString {
    pub data: *const c_void,
    pub private_flags: c_uint,
}

impl Nullable<CXString> for CXString {
    fn map<U, F: FnOnce(CXString) -> U>(self, f: F) -> Option<U> {
        if !self.data.is_null() {
            Some(f(self))
        } else {
            None
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CXTUResourceUsage {
    pub data: *mut c_void,
    pub numEntries: c_uint,
    pub entries: *mut CXTUResourceUsageEntry,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CXTUResourceUsageEntry {
    pub kind: CXTUResourceUsageKind,
    pub amount: c_ulong,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CXToken {
    pub int_data: [c_uint; 4],
    pub ptr_data: *mut c_void,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CXType {
    pub kind: CXTypeKind,
    pub data: [*mut c_void; 2],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CXUnsavedFile {
    pub Filename: *const c_char,
    pub Contents: *const c_char,
    pub Length: c_ulong,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CXVersion {
    pub Major: c_int,
    pub Minor: c_int,
    pub Subminor: c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct IndexerCallbacks {
    pub abortQuery: extern fn(CXClientData, *mut c_void) -> c_int,
    pub diagnostic: extern fn(CXClientData, CXDiagnosticSet, *mut c_void),
    pub enteredMainFile: extern fn(CXClientData, CXFile, *mut c_void) -> CXIdxClientFile,
    pub ppIncludedFile: extern fn(CXClientData, *const CXIdxIncludedFileInfo) -> CXIdxClientFile,
    pub importedASTFile: extern fn(CXClientData, *const CXIdxImportedASTFileInfo) -> CXIdxClientASTFile,
    pub startedTranslationUnit: extern fn(CXClientData, *mut c_void) -> CXIdxClientContainer,
    pub indexDeclaration: extern fn(CXClientData, *const CXIdxDeclInfo),
    pub indexEntityReference: extern fn(CXClientData, *const CXIdxEntityRefInfo),
}

//================================================
// Functions
//================================================

#[link(name="clang")]
extern {
    pub fn clang_CXCursorSet_contains(set: CXCursorSet, cursor: CXCursor) -> c_uint;
    pub fn clang_CXCursorSet_insert(set: CXCursorSet, cursor: CXCursor) -> c_uint;
    pub fn clang_CXIndex_getGlobalOptions(index: CXIndex) -> CXGlobalOptFlags;
    pub fn clang_CXIndex_setGlobalOptions(index: CXIndex, flags: CXGlobalOptFlags);
    pub fn clang_CXXMethod_isConst(cursor: CXCursor) -> c_uint;
    pub fn clang_CXXMethod_isPureVirtual(cursor: CXCursor) -> c_uint;
    pub fn clang_CXXMethod_isStatic(cursor: CXCursor) -> c_uint;
    pub fn clang_CXXMethod_isVirtual(cursor: CXCursor) -> c_uint;
    pub fn clang_CompilationDatabase_dispose(database: CXCompilationDatabase);
    pub fn clang_CompilationDatabase_fromDirectory(directory: *const c_char, error: *mut CXCompilationDatabase_Error) -> CXCompilationDatabase;
    pub fn clang_CompilationDatabase_getAllCompileCommands(database: CXCompilationDatabase) -> CXCompileCommands;
    pub fn clang_CompilationDatabase_getCompileCommands(database: CXCompilationDatabase, filename: *const c_char) -> CXCompileCommands;
    pub fn clang_CompileCommand_getArg(command: CXCompileCommand, index: c_uint) -> CXString;
    pub fn clang_CompileCommand_getDirectory(command: CXCompileCommand) -> CXString;
    pub fn clang_CompileCommand_getMappedSourceContent(command: CXCompileCommand, index: c_uint) -> CXString;
    pub fn clang_CompileCommand_getMappedSourcePath(command: CXCompileCommand, index: c_uint) -> CXString;
    pub fn clang_CompileCommand_getNumArgs(command: CXCompileCommand) -> c_uint;
    pub fn clang_CompileCommand_getNumMappedSources(command: CXCompileCommand) -> c_uint;
    pub fn clang_CompileCommands_dispose(command: CXCompileCommands);
    pub fn clang_CompileCommands_getCommand(command: CXCompileCommands, index: c_uint) -> CXCompileCommand;
    pub fn clang_CompileCommands_getSize(command: CXCompileCommands) -> c_uint;
    pub fn clang_Cursor_getArgument(cursor: CXCursor, index: c_uint) -> CXCursor;
    pub fn clang_Cursor_getBriefCommentText(cursor: CXCursor) -> CXString;
    pub fn clang_Cursor_getCommentRange(cursor: CXCursor) -> CXSourceRange;
    pub fn clang_Cursor_getMangling(cursor: CXCursor) -> CXString;
    pub fn clang_Cursor_getModule(cursor: CXCursor) -> CXModule;
    pub fn clang_Cursor_getNumArguments(cursor: CXCursor) -> c_int;
    pub fn clang_Cursor_getNumTemplateArguments(cursor: CXCursor) -> c_int;
    pub fn clang_Cursor_getObjCDeclQualifiers(cursor: CXCursor) -> CXObjCDeclQualifierKind;
    pub fn clang_Cursor_getObjCPropertyAttributes(cursor: CXCursor, reserved: c_uint) -> CXObjCPropertyAttrKind;
    pub fn clang_Cursor_getObjCSelectorIndex(cursor: CXCursor) -> c_int;
    pub fn clang_Cursor_getOffsetOfField(cursor: CXCursor) -> c_longlong;
    pub fn clang_Cursor_getRawCommentText(cursor: CXCursor) -> CXString;
    pub fn clang_Cursor_getReceiverType(cursor: CXCursor) -> CXType;
    pub fn clang_Cursor_getSpellingNameRange(cursor: CXCursor, index: c_uint, reserved: c_uint) -> CXSourceRange;
    pub fn clang_Cursor_getStorageClass(cursor: CXCursor) -> CX_StorageClass;
    pub fn clang_Cursor_getTemplateArgumentKind(cursor: CXCursor, index: c_uint) -> CXTemplateArgumentKind;
    pub fn clang_Cursor_getTemplateArgumentType(cursor: CXCursor, index: c_uint) -> CXType;
    pub fn clang_Cursor_getTemplateArgumentUnsignedValue(cursor: CXCursor, index: c_uint) -> c_ulonglong;
    pub fn clang_Cursor_getTemplateArgumentValue(cursor: CXCursor, index: c_uint) -> c_longlong;
    pub fn clang_Cursor_getTranslationUnit(cursor: CXCursor) -> CXTranslationUnit;
    pub fn clang_Cursor_isAnonymous(cursor: CXCursor) -> c_uint;
    pub fn clang_Cursor_isBitField(cursor: CXCursor) -> c_uint;
    pub fn clang_Cursor_isDynamicCall(cursor: CXCursor) -> c_int;
    pub fn clang_Cursor_isNull(cursor: CXCursor) -> c_int;
    pub fn clang_Cursor_isObjCOptional(cursor: CXCursor) -> c_uint;
    pub fn clang_Cursor_isVariadic(cursor: CXCursor) -> c_uint;
    pub fn clang_File_isEqual(left: CXFile, right: CXFile) -> c_int;
    pub fn clang_IndexAction_create(index: CXIndex) -> CXIndexAction;
    pub fn clang_IndexAction_dispose(index: CXIndexAction);
    pub fn clang_Location_isFromMainFile(location: CXSourceLocation) -> c_int;
    pub fn clang_Location_isInSystemHeader(location: CXSourceLocation) -> c_int;
    pub fn clang_ModuleMapDescriptor_create(reserved: c_uint) -> CXModuleMapDescriptor;
    pub fn clang_ModuleMapDescriptor_dispose(module: CXModuleMapDescriptor);
    pub fn clang_ModuleMapDescriptor_setFrameworkModuleName(module: CXModuleMapDescriptor, name: *const c_char) -> CXErrorCode;
    pub fn clang_ModuleMapDescriptor_setUmbrellaHeader(module: CXModuleMapDescriptor, name: *const c_char) -> CXErrorCode;
    pub fn clang_ModuleMapDescriptor_writeToBuffer(module: CXModuleMapDescriptor, reserved: c_uint, buffer: *mut *mut c_char, n_buffer: *mut c_uint) -> CXErrorCode;
    pub fn clang_Module_getASTFile(module: CXModule) -> CXFile;
    pub fn clang_Module_getFullName(module: CXModule) -> CXString;
    pub fn clang_Module_getName(module: CXModule) -> CXString;
    pub fn clang_Module_getNumTopLevelHeaders(tu: CXTranslationUnit, module: CXModule) -> c_uint;
    pub fn clang_Module_getParent(module: CXModule) -> CXModule;
    pub fn clang_Module_getTopLevelHeader(tu: CXTranslationUnit, module: CXModule, index: c_uint) -> CXFile;
    pub fn clang_Module_isSystem(module: CXModule) -> c_int;
    pub fn clang_Range_isNull(range: CXSourceRange) -> c_int;
    pub fn clang_Type_getAlignOf(type_: CXType) -> c_longlong;
    pub fn clang_Type_getCXXRefQualifier(type_: CXType) -> CXRefQualifierKind;
    pub fn clang_Type_getClassType(type_: CXType) -> CXType;
    pub fn clang_Type_getNumTemplateArguments(type_: CXType) -> c_int;
    pub fn clang_Type_getOffsetOf(type_: CXType, field: *const c_char) -> c_longlong;
    pub fn clang_Type_getSizeOf(type_: CXType) -> c_longlong;
    pub fn clang_Type_getTemplateArgumentAsType(type_: CXType, index: c_uint) -> CXType;
    pub fn clang_Type_visitFields(type_: CXType, visitor: CXFieldVisitor, data: CXClientData) -> CXVisitorResult;
    pub fn clang_VirtualFileOverlay_addFileMapping(overlay: CXVirtualFileOverlay, virtual_: *const c_char, real: *const c_char) -> CXErrorCode;
    pub fn clang_VirtualFileOverlay_create(reserved: c_uint) -> CXVirtualFileOverlay;
    pub fn clang_VirtualFileOverlay_dispose(overlay: CXVirtualFileOverlay);
    pub fn clang_VirtualFileOverlay_setCaseSensitivity(overlay: CXVirtualFileOverlay, sensitive: c_int) -> CXErrorCode;
    pub fn clang_VirtualFileOverlay_writeToBuffer(overlay: CXVirtualFileOverlay, reserved: c_uint, buffer: *mut *mut c_char, n_buffer: *mut c_uint) -> CXErrorCode;
    pub fn clang_annotateTokens(tu: CXTranslationUnit, tokens: *mut CXToken, n_tokens: c_uint, cursors: *mut CXCursor);
    pub fn clang_codeCompleteAt(tu: CXTranslationUnit, file: *const c_char, line: c_uint, column: c_uint, unsaved: *mut CXUnsavedFile, n_unsaved: c_uint, flags: CXCodeComplete_Flags) -> *mut CXCodeCompleteResults;
    pub fn clang_codeCompleteGetContainerKind(results: *mut CXCodeCompleteResults, incomplete: *mut c_uint) -> CXCursorKind;
    pub fn clang_codeCompleteGetContainerUSR(results: *mut CXCodeCompleteResults) -> CXString;
    pub fn clang_codeCompleteGetContexts(results: *mut CXCodeCompleteResults) -> c_ulonglong;
    pub fn clang_codeCompleteGetDiagnostic(results: *mut CXCodeCompleteResults, index: c_uint) -> CXDiagnostic;
    pub fn clang_codeCompleteGetNumDiagnostics(results: *mut CXCodeCompleteResults) -> c_uint;
    pub fn clang_codeCompleteGetObjCSelector(results: *mut CXCodeCompleteResults) -> CXString;
    pub fn clang_constructUSR_ObjCCategory(class: *const c_char, category: *const c_char) -> CXString;
    pub fn clang_constructUSR_ObjCClass(class: *const c_char) -> CXString;
    pub fn clang_constructUSR_ObjCIvar(name: *const c_char, usr: CXString) -> CXString;
    pub fn clang_constructUSR_ObjCMethod(name: *const c_char, instance: c_uint, usr: CXString) -> CXString;
    pub fn clang_constructUSR_ObjCProperty(property: *const c_char, usr: CXString) -> CXString;
    pub fn clang_constructUSR_ObjCProtocol(protocol: *const c_char) -> CXString;
    pub fn clang_createCXCursorSet() -> CXCursorSet;
    pub fn clang_createIndex(exclude: c_int, display: c_int) -> CXIndex;
    pub fn clang_createTranslationUnit(index: CXIndex, file: *const c_char) -> CXTranslationUnit;
    pub fn clang_createTranslationUnit2(index: CXIndex, file: *const c_char, tu: *mut CXTranslationUnit) -> CXErrorCode;
    pub fn clang_createTranslationUnitFromSourceFile(index: CXIndex, file: *const c_char, n_arguments: c_int, arguments: *const *const c_char, n_unsaved: c_uint, unsaved: *mut CXUnsavedFile) -> CXTranslationUnit;
    pub fn clang_defaultCodeCompleteOptions() -> CXCodeComplete_Flags;
    pub fn clang_defaultDiagnosticDisplayOptions() -> CXDiagnosticDisplayOptions;
    pub fn clang_defaultEditingTranslationUnitOptions() -> CXTranslationUnit_Flags;
    pub fn clang_defaultReparseOptions(tu: CXTranslationUnit) -> CXReparse_Flags;
    pub fn clang_defaultSaveOptions(tu: CXTranslationUnit) -> CXSaveTranslationUnit_Flags;
    pub fn clang_disposeCXCursorSet(set: CXCursorSet);
    pub fn clang_disposeCXPlatformAvailability(availability: *mut CXPlatformAvailability);
    pub fn clang_disposeCXTUResourceUsage(usage: CXTUResourceUsage);
    pub fn clang_disposeCodeCompleteResults(results: *mut CXCodeCompleteResults);
    pub fn clang_disposeDiagnostic(diagnostic: CXDiagnostic);
    pub fn clang_disposeDiagnosticSet(diagnostic: CXDiagnosticSet);
    pub fn clang_disposeIndex(index: CXIndex);
    pub fn clang_disposeOverriddenCursors(cursors: *mut CXCursor);
    pub fn clang_disposeSourceRangeList(list: *mut CXSourceRangeList);
    pub fn clang_disposeString(string: CXString);
    pub fn clang_disposeTokens(tu: CXTranslationUnit, tokens: *mut CXToken, n_tokens: c_uint);
    pub fn clang_disposeTranslationUnit(tu: CXTranslationUnit);
    pub fn clang_enableStackTraces();
    pub fn clang_equalCursors(left: CXCursor, right: CXCursor) -> c_uint;
    pub fn clang_equalLocations(left: CXSourceLocation, right: CXSourceLocation) -> c_uint;
    pub fn clang_equalRanges(left: CXSourceRange, right: CXSourceRange) -> c_uint;
    pub fn clang_equalTypes(left: CXType, right: CXType) -> c_uint;
    pub fn clang_executeOnThread(function: extern fn(*mut c_void), data: *mut c_void, stack: c_uint);
    pub fn clang_findIncludesInFile(tu: CXTranslationUnit, file: CXFile, cursor: CXCursorAndRangeVisitor) -> CXResult;
    pub fn clang_findReferencesInFile(cursor: CXCursor, file: CXFile, cursor: CXCursorAndRangeVisitor) -> CXResult;
    pub fn clang_formatDiagnostic(diagnostic: CXDiagnostic, flags: CXDiagnosticDisplayOptions) -> CXString;
    pub fn clang_free(buffer: *mut c_void);
    pub fn clang_getArgType(type_: CXType, index: c_uint) -> CXType;
    pub fn clang_getArrayElementType(type_: CXType) -> CXType;
    pub fn clang_getArraySize(type_: CXType) -> c_longlong;
    pub fn clang_getBuildSessionTimestamp() -> c_ulonglong;
    pub fn clang_getCString(string: CXString) -> *const c_char;
    pub fn clang_getCXTUResourceUsage(tu: CXTranslationUnit) -> CXTUResourceUsage;
    pub fn clang_getCXXAccessSpecifier(cursor: CXCursor) -> CX_CXXAccessSpecifier;
    pub fn clang_getCanonicalCursor(cursor: CXCursor) -> CXCursor;
    pub fn clang_getCanonicalType(type_: CXType) -> CXType;
    pub fn clang_getChildDiagnostics(diagnostic: CXDiagnostic) -> CXDiagnosticSet;
    pub fn clang_getClangVersion() -> CXString;
    pub fn clang_getCompletionAnnotation(string: CXCompletionString, index: c_uint) -> CXString;
    pub fn clang_getCompletionAvailability(string: CXCompletionString) -> CXAvailabilityKind;
    pub fn clang_getCompletionBriefComment(string: CXCompletionString) -> CXString;
    pub fn clang_getCompletionChunkCompletionString(string: CXCompletionString, index: c_uint) -> CXCompletionString;
    pub fn clang_getCompletionChunkKind(string: CXCompletionString, index: c_uint) -> CXCompletionChunkKind;
    pub fn clang_getCompletionChunkText(string: CXCompletionString, index: c_uint) -> CXString;
    pub fn clang_getCompletionNumAnnotations(string: CXCompletionString) -> c_uint;
    pub fn clang_getCompletionParent(string: CXCompletionString, kind: *mut CXCursorKind) -> CXString;
    pub fn clang_getCompletionPriority(string: CXCompletionString) -> c_uint;
    pub fn clang_getCursor(tu: CXTranslationUnit, location: CXSourceLocation) -> CXCursor;
    pub fn clang_getCursorAvailability(cursor: CXCursor) -> CXAvailabilityKind;
    pub fn clang_getCursorCompletionString(cursor: CXCursor) -> CXCompletionString;
    pub fn clang_getCursorDefinition(cursor: CXCursor) -> CXCursor;
    pub fn clang_getCursorDisplayName(cursor: CXCursor) -> CXString;
    pub fn clang_getCursorExtent(cursor: CXCursor) -> CXSourceRange;
    pub fn clang_getCursorKind(cursor: CXCursor) -> CXCursorKind;
    pub fn clang_getCursorKindSpelling(kind: CXCursorKind) -> CXString;
    pub fn clang_getCursorLanguage(cursor: CXCursor) -> CXLanguageKind;
    pub fn clang_getCursorLexicalParent(cursor: CXCursor) -> CXCursor;
    pub fn clang_getCursorLinkage(cursor: CXCursor) -> CXLinkageKind;
    pub fn clang_getCursorLocation(cursor: CXCursor) -> CXSourceLocation;
    pub fn clang_getCursorPlatformAvailability(cursor: CXCursor, deprecated: *mut c_int, deprecated_message: *mut CXString, unavailable: *mut c_int, unavailable_message: *mut CXString, availability: *mut CXPlatformAvailability, n_availability: c_int) -> c_int;
    pub fn clang_getCursorReferenceNameRange(cursor: CXCursor, flags: CXNameRefFlags, index: c_uint) -> CXSourceRange;
    pub fn clang_getCursorReferenced(cursor: CXCursor) -> CXCursor;
    pub fn clang_getCursorResultType(cursor: CXCursor) -> CXType;
    pub fn clang_getCursorSemanticParent(cursor: CXCursor) -> CXCursor;
    pub fn clang_getCursorSpelling(cursor: CXCursor) -> CXString;
    pub fn clang_getCursorType(cursor: CXCursor) -> CXType;
    pub fn clang_getCursorUSR(cursor: CXCursor) -> CXString;
    pub fn clang_getDeclObjCTypeEncoding(cursor: CXCursor) -> CXString;
    pub fn clang_getDefinitionSpellingAndExtent(cursor: CXCursor, start: *mut *const c_char, end: *mut *const c_char, start_line: *mut c_uint, start_column: *mut c_uint, end_line: *mut c_uint, end_column: *mut c_uint);
    pub fn clang_getDiagnostic(tu: CXTranslationUnit, index: c_uint) -> CXDiagnostic;
    pub fn clang_getDiagnosticCategory(diagnostic: CXDiagnostic) -> c_uint;
    pub fn clang_getDiagnosticCategoryName(category: c_uint) -> CXString;
    pub fn clang_getDiagnosticCategoryText(diagnostic: CXDiagnostic) -> CXString;
    pub fn clang_getDiagnosticFixIt(diagnostic: CXDiagnostic, index: c_uint, range: *mut CXSourceRange) -> CXString;
    pub fn clang_getDiagnosticInSet(diagnostic: CXDiagnosticSet, index: c_uint) -> CXDiagnostic;
    pub fn clang_getDiagnosticLocation(diagnostic: CXDiagnostic) -> CXSourceLocation;
    pub fn clang_getDiagnosticNumFixIts(diagnostic: CXDiagnostic) -> c_uint;
    pub fn clang_getDiagnosticNumRanges(diagnostic: CXDiagnostic) -> c_uint;
    pub fn clang_getDiagnosticOption(diagnostic: CXDiagnostic, option: *mut CXString) -> CXString;
    pub fn clang_getDiagnosticRange(diagnostic: CXDiagnostic, index: c_uint) -> CXSourceRange;
    pub fn clang_getDiagnosticSetFromTU(tu: CXTranslationUnit) -> CXDiagnosticSet;
    pub fn clang_getDiagnosticSeverity(diagnostic: CXDiagnostic) -> CXDiagnosticSeverity;
    pub fn clang_getDiagnosticSpelling(diagnostic: CXDiagnostic) -> CXString;
    pub fn clang_getElementType(type_: CXType) -> CXType;
    pub fn clang_getEnumConstantDeclUnsignedValue(cursor: CXCursor) -> c_ulonglong;
    pub fn clang_getEnumConstantDeclValue(cursor: CXCursor) -> c_longlong;
    pub fn clang_getEnumDeclIntegerType(cursor: CXCursor) -> CXType;
    pub fn clang_getExpansionLocation(location: CXSourceLocation, file: *mut CXFile, line: *mut c_uint, column: *mut c_uint, offset: *mut c_uint);
    pub fn clang_getFieldDeclBitWidth(cursor: CXCursor) -> c_int;
    pub fn clang_getFile(tu: CXTranslationUnit, file: *const c_char) -> CXFile;
    pub fn clang_getFileLocation(location: CXSourceLocation, file: *mut CXFile, line: *mut c_uint, column: *mut c_uint, offset: *mut c_uint);
    pub fn clang_getFileName(file: CXFile) -> CXString;
    pub fn clang_getFileTime(file: CXFile) -> time_t;
    pub fn clang_getFileUniqueID(file: CXFile, id: *mut CXFileUniqueID) -> c_int;
    pub fn clang_getFunctionTypeCallingConv(type_: CXType) -> CXCallingConv;
    pub fn clang_getIBOutletCollectionType(cursor: CXCursor) -> CXType;
    pub fn clang_getIncludedFile(cursor: CXCursor) -> CXFile;
    pub fn clang_getInclusions(tu: CXTranslationUnit, visitor: CXInclusionVisitor, data: CXClientData);
    pub fn clang_getInstantiationLocation(location: CXSourceLocation, file: *mut CXFile, line: *mut c_uint, column: *mut c_uint, offset: *mut c_uint);
    pub fn clang_getLocation(tu: CXTranslationUnit, file: CXFile, line: c_uint, column: c_uint) -> CXSourceLocation;
    pub fn clang_getLocationForOffset(tu: CXTranslationUnit, file: CXFile, offset: c_uint) -> CXSourceLocation;
    pub fn clang_getModuleForFile(tu: CXTranslationUnit, file: CXFile) -> CXModule;
    pub fn clang_getNullCursor() -> CXCursor;
    pub fn clang_getNullLocation() -> CXSourceLocation;
    pub fn clang_getNullRange() -> CXSourceRange;
    pub fn clang_getNumArgTypes(type_: CXType) -> c_int;
    pub fn clang_getNumCompletionChunks(string: CXCompletionString) -> c_uint;
    pub fn clang_getNumDiagnostics(tu: CXTranslationUnit) -> c_uint;
    pub fn clang_getNumDiagnosticsInSet(diagnostic: CXDiagnosticSet) -> c_uint;
    pub fn clang_getNumElements(type_: CXType) -> c_longlong;
    pub fn clang_getNumOverloadedDecls(cursor: CXCursor) -> c_uint;
    pub fn clang_getOverloadedDecl(cursor: CXCursor, index: c_uint) -> CXCursor;
    pub fn clang_getOverriddenCursors(cursor: CXCursor, cursors: *mut *mut CXCursor, n_cursors: *mut c_uint);
    pub fn clang_getPointeeType(type_: CXType) -> CXType;
    pub fn clang_getPresumedLocation(location: CXSourceLocation, file: *mut CXString, line: *mut c_uint, column: *mut c_uint);
    pub fn clang_getRange(start: CXSourceLocation, end: CXSourceLocation) -> CXSourceRange;
    pub fn clang_getRangeEnd(range: CXSourceRange) -> CXSourceLocation;
    pub fn clang_getRangeStart(range: CXSourceRange) -> CXSourceLocation;
    pub fn clang_getRemappings(file: *const c_char) -> CXRemapping;
    pub fn clang_getRemappingsFromFileList(files: *mut *const c_char, n_files: c_uint) -> CXRemapping;
    pub fn clang_getResultType(type_: CXType) -> CXType;
    pub fn clang_getSkippedRanges(tu: CXTranslationUnit, file: CXFile) -> *mut CXSourceRangeList;
    pub fn clang_getSpecializedCursorTemplate(cursor: CXCursor) -> CXCursor;
    pub fn clang_getSpellingLocation(location: CXSourceLocation, file: *mut CXFile, line: *mut c_uint, column: *mut c_uint, offset: *mut c_uint);
    pub fn clang_getTUResourceUsageName(kind: CXTUResourceUsageKind) -> *const c_char;
    pub fn clang_getTemplateCursorKind(cursor: CXCursor) -> CXCursorKind;
    pub fn clang_getTokenExtent(tu: CXTranslationUnit, token: CXToken) -> CXSourceRange;
    pub fn clang_getTokenKind(token: CXToken) -> CXTokenKind;
    pub fn clang_getTokenLocation(tu: CXTranslationUnit, token: CXToken) -> CXSourceLocation;
    pub fn clang_getTokenSpelling(tu: CXTranslationUnit, token: CXToken) -> CXString;
    pub fn clang_getTranslationUnitCursor(tu: CXTranslationUnit) -> CXCursor;
    pub fn clang_getTranslationUnitSpelling(tu: CXTranslationUnit) -> CXString;
    pub fn clang_getTypeDeclaration(type_: CXType) -> CXCursor;
    pub fn clang_getTypeKindSpelling(type_: CXTypeKind) -> CXString;
    pub fn clang_getTypeSpelling(type_: CXType) -> CXString;
    pub fn clang_getTypedefDeclUnderlyingType(cursor: CXCursor) -> CXType;
    pub fn clang_hashCursor(cursor: CXCursor) -> c_uint;
    pub fn clang_indexLoc_getCXSourceLocation(location: CXIdxLoc) -> CXSourceLocation;
    pub fn clang_indexLoc_getFileLocation(location: CXIdxLoc, index_file: *mut CXIdxClientFile, file: *mut CXFile, line: *mut c_uint, column: *mut c_uint, offset: *mut c_uint);
    pub fn clang_indexSourceFile(index: CXIndexAction, data: CXClientData, callbacks: *mut IndexerCallbacks, n_callbacks: c_uint, index_flags: CXIndexOptFlags, file: *const c_char, arguments: *const *const c_char, n_arguments: c_int, unsaved: *mut CXUnsavedFile, n_unsaved: c_uint, tu: *mut CXTranslationUnit, tu_flags: CXTranslationUnit_Flags) -> CXErrorCode;
    pub fn clang_indexTranslationUnit(index: CXIndexAction, data: CXClientData, callbacks: *mut IndexerCallbacks, n_callbacks: c_uint, flags: CXIndexOptFlags, tu: CXTranslationUnit) -> c_int;
    pub fn clang_index_getCXXClassDeclInfo(info: *const CXIdxDeclInfo) -> *const CXIdxCXXClassDeclInfo;
    pub fn clang_index_getClientContainer(info: *const CXIdxContainerInfo) -> CXIdxClientContainer;
    pub fn clang_index_getClientEntity(info: *const CXIdxEntityInfo) -> CXIdxClientEntity;
    pub fn clang_index_getIBOutletCollectionAttrInfo(info: *const CXIdxAttrInfo) -> *const CXIdxIBOutletCollectionAttrInfo;
    pub fn clang_index_getObjCCategoryDeclInfo(info: *const CXIdxDeclInfo) -> *const CXIdxObjCCategoryDeclInfo;
    pub fn clang_index_getObjCContainerDeclInfo(info: *const CXIdxDeclInfo) -> *const CXIdxObjCContainerDeclInfo;
    pub fn clang_index_getObjCInterfaceDeclInfo(info: *const CXIdxDeclInfo) -> *const CXIdxObjCInterfaceDeclInfo;
    pub fn clang_index_getObjCPropertyDeclInfo(info: *const CXIdxDeclInfo) -> *const CXIdxObjCPropertyDeclInfo;
    pub fn clang_index_getObjCProtocolRefListInfo(info: *const CXIdxDeclInfo) -> *const CXIdxObjCProtocolRefListInfo;
    pub fn clang_index_isEntityObjCContainerKind(info: CXIdxEntityKind) -> c_int;
    pub fn clang_index_setClientContainer(info: *const CXIdxContainerInfo, container: CXIdxClientContainer);
    pub fn clang_index_setClientEntity(info: *const CXIdxEntityInfo, entity: CXIdxClientEntity);
    pub fn clang_isAttribute(kind: CXCursorKind) -> c_uint;
    pub fn clang_isConstQualifiedType(type_: CXType) -> c_uint;
    pub fn clang_isCursorDefinition(cursor: CXCursor) -> c_uint;
    pub fn clang_isDeclaration(kind: CXCursorKind) -> c_uint;
    pub fn clang_isExpression(kind: CXCursorKind) -> c_uint;
    pub fn clang_isFileMultipleIncludeGuarded(tu: CXTranslationUnit, file: CXFile) -> c_uint;
    pub fn clang_isFunctionTypeVariadic(type_: CXType) -> c_uint;
    pub fn clang_isInvalid(kind: CXCursorKind) -> c_uint;
    pub fn clang_isPODType(type_: CXType) -> c_uint;
    pub fn clang_isPreprocessing(kind: CXCursorKind) -> c_uint;
    pub fn clang_isReference(kind: CXCursorKind) -> c_uint;
    pub fn clang_isRestrictQualifiedType(type_: CXType) -> c_uint;
    pub fn clang_isStatement(kind: CXCursorKind) -> c_uint;
    pub fn clang_isTranslationUnit(kind: CXCursorKind) -> c_uint;
    pub fn clang_isUnexposed(kind: CXCursorKind) -> c_uint;
    pub fn clang_isVirtualBase(cursor: CXCursor) -> c_uint;
    pub fn clang_isVolatileQualifiedType(type_: CXType) -> c_uint;
    pub fn clang_loadDiagnostics(file: *const c_char, error: *mut CXLoadDiag_Error, message: *mut CXString) -> CXDiagnosticSet;
    pub fn clang_parseTranslationUnit(index: CXIndex, file: *const c_char, arguments: *const *const c_char, n_arguments: c_int, unsaved: *mut CXUnsavedFile, n_unsaved: c_uint, flags: CXTranslationUnit_Flags) -> CXTranslationUnit;
    pub fn clang_parseTranslationUnit2(index: CXIndex, file: *const c_char, arguments: *const *const c_char, n_arguments: c_int, unsaved: *mut CXUnsavedFile, n_unsaved: c_uint, flags: CXTranslationUnit_Flags, tu: *mut CXTranslationUnit) -> CXErrorCode;
    pub fn clang_remap_dispose(remapping: CXRemapping);
    pub fn clang_remap_getFilenames(remapping: CXRemapping, index: c_uint, original: *mut CXString, transformed: *mut CXString);
    pub fn clang_remap_getNumFiles(remapping: CXRemapping) -> c_uint;
    pub fn clang_reparseTranslationUnit(tu: CXTranslationUnit, n_unsaved: c_uint, unsaved: *mut CXUnsavedFile, flags: CXReparse_Flags) -> CXErrorCode;
    pub fn clang_saveTranslationUnit(tu: CXTranslationUnit, file: *const c_char, options: CXSaveTranslationUnit_Flags) -> CXSaveError;
    pub fn clang_sortCodeCompletionResults(results: *mut CXCompletionResult, n_results: c_uint);
    pub fn clang_toggleCrashRecovery(recovery: c_uint);
    pub fn clang_tokenize(tu: CXTranslationUnit, range: CXSourceRange, tokens: *mut *mut CXToken, n_tokens: *mut c_uint);
    pub fn clang_visitChildren(cursor: CXCursor, visitor: CXCursorVisitor, data: CXClientData) -> c_uint;
}

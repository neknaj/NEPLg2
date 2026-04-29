//! Hierarchical diagnostic code registry.
//!
//! Diagnostic codes are compiler-owned enum values.  The dotted string form is
//! only the serialization/display spelling used by CLI, web, and doctests.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagnosticCode {
    Loader(LoaderDiagnosticCode),
    Lexer(LexerDiagnosticCode),
    Parser(ParserDiagnosticCode),
    Resolve(ResolveDiagnosticCode),
    Type(TypeDiagnosticCode),
    Effect(EffectDiagnosticCode),
    Resource(ResourceDiagnosticCode),
    Backend(BackendDiagnosticCode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LoaderDiagnosticCode {
    TargetMultipleDirective,
    TargetUnknown,
    SourceFailure,
    ConditionalGateInvalid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LexerDiagnosticCode {
    DirectiveUnknown,
    TokenUnknown,
    IndentArgumentInvalid,
    IndentTabsNotAllowed,
    RawBlockExpectedIndent,
    PubPrefixInvalid,
    IndentWidthMismatch,
    IndentLevelMismatch,
    StringInvalidEscape,
    StringUnterminated,
    CharInvalid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParserDiagnosticCode {
    TokenExpected,
    TokenUnexpected,
    IdentifierExpected,
    TypeExprInvalid,
    IdentifierReservedKeyword,
    ExternSignatureInvalid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResolveDiagnosticCode {
    ImportAmbiguous,
    IdentifierUndefined,
    ShadowNoShadowViolation,
    ShadowNoShadowConflict,
    ItemNameConflict,
    AliasTargetNotFound,
    EntryFunctionMissingOrAmbiguous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TypeDiagnosticCode {
    VariableUndefined,
    ReturnTypeMismatch,
    AnnotationMismatch,
    OverloadAmbiguous,
    OverloadNoMatch,
    MatchScrutineeNotEnum,
    MatchDuplicateArm,
    MatchNonExhaustive,
    MutationImmutable,
    FieldInvalidAccess,
    IntrinsicUnknown,
    LiteralIntInvalid,
    LiteralCharOutOfRange,
    PipeInvalid,
    StackExtraValues,
    BlockStackInconsistent,
    NestedGenericFunctionUnsupported,
    RawBlockInvalidPlacement,
    FunctionValueCapturingUnsupported,
    IndirectCallRequiresFunctionValue,
    CallCaptureArityMismatch,
    VariableNotCallable,
    OverloadTypeArgsMismatch,
    ArgumentMismatch,
    FunctionRefRequiresCallable,
    VariableTypeArgsNotAllowed,
    AssignmentMismatch,
    AssignmentUndefinedVariable,
    IfArityMismatch,
    IfConditionMismatch,
    WhileArityMismatch,
    WhileConditionMismatch,
    WhileBodyMismatch,
    MatchVariantUnknown,
    MatchPayloadBindingInvalid,
    MatchArmsMismatch,
    IntrinsicTypeArgArityMismatch,
    IntrinsicArgArityMismatch,
    IntrinsicArgTypeMismatch,
    CopyImplTargetNotCopy,
    CopyImplRequiresClone,
    TraitMethodTypeArgsUnsupported,
    TraitMethodNotFound,
    ArgumentArityMismatch,
    TraitBoundUnsatisfied,
    DerefInvalid,
    AssignmentArityMismatch,
    CallReductionLimitExceeded,
    TraitBoundUnknown,
    ExternWasiTargetMismatch,
    ExternSignatureNotFunction,
    EnumTypeParamBoundsUnsupported,
    StructTypeParamBoundsUnsupported,
    TraitTypeParamsUnsupported,
    TraitMethodTypeParamsUnsupported,
    ImplInherentUnsupported,
    ImplTypeParamsUnsupported,
    TraitUnknown,
    ImplTargetNotConcrete,
    FunctionSignatureNotFunction,
    FunctionSignatureOverloadNotFound,
    ImplDuplicateMethod,
    ImplMethodNotInTrait,
    ImplMethodSignatureMismatch,
    ImplMissingTraitMethod,
    ImplDuplicateForTraitTarget,
    TraitCapabilityUnknown,
    MatchPatternUnsupported,
    MatchWildcardNotLast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EffectDiagnosticCode {
    OverloadMismatch,
    PureCallsImpure,
    RawBodyMultipleActive,
    RawBodyTargetMismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResourceDiagnosticCode {
    Move(ResourceMoveDiagnosticCode),
    Borrow(ResourceBorrowDiagnosticCode),
    Raw(ResourceRawDiagnosticCode),
    Lower(ResourceLowerDiagnosticCode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResourceMoveDiagnosticCode {
    UseMoved,
    UsePossiblyMoved,
    DropMoved,
    DropPossiblyMoved,
    LoopPossiblyMoved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResourceBorrowDiagnosticCode {
    MoveFromShared,
    UseDuringUnique,
    AssignDuringShared,
    AssignDuringUnique,
    DropDuringShared,
    DropDuringUnique,
    UniqueDuringShared,
    BorrowDuringUnique,
    BorrowMoved,
    BorrowPossiblyMoved,
    ReturnEscape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResourceRawDiagnosticCode {
    OwnershipViolation,
    IdentityEscape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResourceLowerDiagnosticCode {
    Incomplete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BackendDiagnosticCode {
    Wasm(WasmDiagnosticCode),
    Llvm(LlvmDiagnosticCode),
    TraitCallUnresolved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WasmDiagnosticCode {
    ExternSignatureUnsupported,
    FunctionSignatureUnsupported,
    ReturnValueMissing,
    RawLineParseError,
    LlvmIrBodyUnsupported,
    StringLiteralNotFound,
    VariableUnknown,
    FunctionValueUnknown,
    FunctionUnknown,
    IndirectSignatureMissing,
    IndirectSignatureUnsupported,
    IntrinsicUnknown,
    EnumPayloadTypeUnsupported,
    StructFieldTypeUnsupported,
    TupleElementTypeUnsupported,
    IntrinsicArityMismatch,
    FieldSelectorUnsupported,
    FieldValueTypeUnsupported,
    LoweredSignatureMissing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LlvmDiagnosticCode {
    RawBodyMismatch,
    VariableUnknown,
    FunctionValueUnknown,
    FunctionUnknown,
    IntrinsicUnknown,
    HirUnsupported,
}

pub const ALL_DIAGNOSTIC_CODES: &[DiagnosticCode] = &[
    DiagnosticCode::Loader(LoaderDiagnosticCode::TargetMultipleDirective),
    DiagnosticCode::Loader(LoaderDiagnosticCode::TargetUnknown),
    DiagnosticCode::Loader(LoaderDiagnosticCode::SourceFailure),
    DiagnosticCode::Loader(LoaderDiagnosticCode::ConditionalGateInvalid),
    DiagnosticCode::Resolve(ResolveDiagnosticCode::ImportAmbiguous),
    DiagnosticCode::Lexer(LexerDiagnosticCode::DirectiveUnknown),
    DiagnosticCode::Lexer(LexerDiagnosticCode::TokenUnknown),
    DiagnosticCode::Lexer(LexerDiagnosticCode::IndentArgumentInvalid),
    DiagnosticCode::Lexer(LexerDiagnosticCode::IndentTabsNotAllowed),
    DiagnosticCode::Lexer(LexerDiagnosticCode::RawBlockExpectedIndent),
    DiagnosticCode::Lexer(LexerDiagnosticCode::PubPrefixInvalid),
    DiagnosticCode::Lexer(LexerDiagnosticCode::IndentWidthMismatch),
    DiagnosticCode::Lexer(LexerDiagnosticCode::IndentLevelMismatch),
    DiagnosticCode::Lexer(LexerDiagnosticCode::StringInvalidEscape),
    DiagnosticCode::Lexer(LexerDiagnosticCode::StringUnterminated),
    DiagnosticCode::Lexer(LexerDiagnosticCode::CharInvalid),
    DiagnosticCode::Parser(ParserDiagnosticCode::TokenExpected),
    DiagnosticCode::Parser(ParserDiagnosticCode::TokenUnexpected),
    DiagnosticCode::Parser(ParserDiagnosticCode::IdentifierExpected),
    DiagnosticCode::Parser(ParserDiagnosticCode::TypeExprInvalid),
    DiagnosticCode::Parser(ParserDiagnosticCode::IdentifierReservedKeyword),
    DiagnosticCode::Parser(ParserDiagnosticCode::ExternSignatureInvalid),
    DiagnosticCode::Resolve(ResolveDiagnosticCode::IdentifierUndefined),
    DiagnosticCode::Type(TypeDiagnosticCode::VariableUndefined),
    DiagnosticCode::Type(TypeDiagnosticCode::ReturnTypeMismatch),
    DiagnosticCode::Type(TypeDiagnosticCode::AnnotationMismatch),
    DiagnosticCode::Type(TypeDiagnosticCode::OverloadAmbiguous),
    DiagnosticCode::Type(TypeDiagnosticCode::OverloadNoMatch),
    DiagnosticCode::Type(TypeDiagnosticCode::MatchScrutineeNotEnum),
    DiagnosticCode::Type(TypeDiagnosticCode::MatchDuplicateArm),
    DiagnosticCode::Type(TypeDiagnosticCode::MatchNonExhaustive),
    DiagnosticCode::Type(TypeDiagnosticCode::MutationImmutable),
    DiagnosticCode::Type(TypeDiagnosticCode::FieldInvalidAccess),
    DiagnosticCode::Type(TypeDiagnosticCode::IntrinsicUnknown),
    DiagnosticCode::Type(TypeDiagnosticCode::LiteralIntInvalid),
    DiagnosticCode::Type(TypeDiagnosticCode::LiteralCharOutOfRange),
    DiagnosticCode::Type(TypeDiagnosticCode::PipeInvalid),
    DiagnosticCode::Type(TypeDiagnosticCode::StackExtraValues),
    DiagnosticCode::Type(TypeDiagnosticCode::BlockStackInconsistent),
    DiagnosticCode::Type(TypeDiagnosticCode::NestedGenericFunctionUnsupported),
    DiagnosticCode::Type(TypeDiagnosticCode::RawBlockInvalidPlacement),
    DiagnosticCode::Resolve(ResolveDiagnosticCode::ShadowNoShadowViolation),
    DiagnosticCode::Resolve(ResolveDiagnosticCode::ShadowNoShadowConflict),
    DiagnosticCode::Type(TypeDiagnosticCode::FunctionValueCapturingUnsupported),
    DiagnosticCode::Type(TypeDiagnosticCode::IndirectCallRequiresFunctionValue),
    DiagnosticCode::Type(TypeDiagnosticCode::VariableNotCallable),
    DiagnosticCode::Effect(EffectDiagnosticCode::OverloadMismatch),
    DiagnosticCode::Type(TypeDiagnosticCode::OverloadTypeArgsMismatch),
    DiagnosticCode::Type(TypeDiagnosticCode::ArgumentMismatch),
    DiagnosticCode::Type(TypeDiagnosticCode::FunctionRefRequiresCallable),
    DiagnosticCode::Type(TypeDiagnosticCode::VariableTypeArgsNotAllowed),
    DiagnosticCode::Effect(EffectDiagnosticCode::PureCallsImpure),
    DiagnosticCode::Type(TypeDiagnosticCode::AssignmentMismatch),
    DiagnosticCode::Type(TypeDiagnosticCode::AssignmentUndefinedVariable),
    DiagnosticCode::Type(TypeDiagnosticCode::IfArityMismatch),
    DiagnosticCode::Type(TypeDiagnosticCode::IfConditionMismatch),
    DiagnosticCode::Type(TypeDiagnosticCode::WhileArityMismatch),
    DiagnosticCode::Type(TypeDiagnosticCode::WhileConditionMismatch),
    DiagnosticCode::Type(TypeDiagnosticCode::WhileBodyMismatch),
    DiagnosticCode::Type(TypeDiagnosticCode::MatchVariantUnknown),
    DiagnosticCode::Type(TypeDiagnosticCode::MatchPayloadBindingInvalid),
    DiagnosticCode::Type(TypeDiagnosticCode::MatchArmsMismatch),
    DiagnosticCode::Type(TypeDiagnosticCode::IntrinsicTypeArgArityMismatch),
    DiagnosticCode::Type(TypeDiagnosticCode::IntrinsicArgArityMismatch),
    DiagnosticCode::Type(TypeDiagnosticCode::IntrinsicArgTypeMismatch),
    DiagnosticCode::Type(TypeDiagnosticCode::CopyImplTargetNotCopy),
    DiagnosticCode::Type(TypeDiagnosticCode::CopyImplRequiresClone),
    DiagnosticCode::Resource(ResourceDiagnosticCode::Borrow(
        ResourceBorrowDiagnosticCode::MoveFromShared,
    )),
    DiagnosticCode::Resource(ResourceDiagnosticCode::Borrow(
        ResourceBorrowDiagnosticCode::UseDuringUnique,
    )),
    DiagnosticCode::Resource(ResourceDiagnosticCode::Move(
        ResourceMoveDiagnosticCode::UseMoved,
    )),
    DiagnosticCode::Resource(ResourceDiagnosticCode::Move(
        ResourceMoveDiagnosticCode::UsePossiblyMoved,
    )),
    DiagnosticCode::Resource(ResourceDiagnosticCode::Borrow(
        ResourceBorrowDiagnosticCode::AssignDuringShared,
    )),
    DiagnosticCode::Resource(ResourceDiagnosticCode::Borrow(
        ResourceBorrowDiagnosticCode::AssignDuringUnique,
    )),
    DiagnosticCode::Resource(ResourceDiagnosticCode::Borrow(
        ResourceBorrowDiagnosticCode::DropDuringShared,
    )),
    DiagnosticCode::Resource(ResourceDiagnosticCode::Borrow(
        ResourceBorrowDiagnosticCode::DropDuringUnique,
    )),
    DiagnosticCode::Resource(ResourceDiagnosticCode::Move(
        ResourceMoveDiagnosticCode::DropMoved,
    )),
    DiagnosticCode::Resource(ResourceDiagnosticCode::Move(
        ResourceMoveDiagnosticCode::DropPossiblyMoved,
    )),
    DiagnosticCode::Resource(ResourceDiagnosticCode::Borrow(
        ResourceBorrowDiagnosticCode::UniqueDuringShared,
    )),
    DiagnosticCode::Resource(ResourceDiagnosticCode::Borrow(
        ResourceBorrowDiagnosticCode::BorrowDuringUnique,
    )),
    DiagnosticCode::Resource(ResourceDiagnosticCode::Borrow(
        ResourceBorrowDiagnosticCode::BorrowMoved,
    )),
    DiagnosticCode::Resource(ResourceDiagnosticCode::Borrow(
        ResourceBorrowDiagnosticCode::BorrowPossiblyMoved,
    )),
    DiagnosticCode::Resource(ResourceDiagnosticCode::Move(
        ResourceMoveDiagnosticCode::LoopPossiblyMoved,
    )),
    DiagnosticCode::Type(TypeDiagnosticCode::TraitMethodTypeArgsUnsupported),
    DiagnosticCode::Type(TypeDiagnosticCode::TraitMethodNotFound),
    DiagnosticCode::Type(TypeDiagnosticCode::ArgumentArityMismatch),
    DiagnosticCode::Type(TypeDiagnosticCode::TraitBoundUnsatisfied),
    DiagnosticCode::Type(TypeDiagnosticCode::DerefInvalid),
    DiagnosticCode::Type(TypeDiagnosticCode::AssignmentArityMismatch),
    DiagnosticCode::Type(TypeDiagnosticCode::CallReductionLimitExceeded),
    DiagnosticCode::Type(TypeDiagnosticCode::TraitBoundUnknown),
    DiagnosticCode::Type(TypeDiagnosticCode::ExternWasiTargetMismatch),
    DiagnosticCode::Type(TypeDiagnosticCode::ExternSignatureNotFunction),
    DiagnosticCode::Resolve(ResolveDiagnosticCode::ItemNameConflict),
    DiagnosticCode::Type(TypeDiagnosticCode::EnumTypeParamBoundsUnsupported),
    DiagnosticCode::Type(TypeDiagnosticCode::StructTypeParamBoundsUnsupported),
    DiagnosticCode::Type(TypeDiagnosticCode::TraitTypeParamsUnsupported),
    DiagnosticCode::Type(TypeDiagnosticCode::TraitMethodTypeParamsUnsupported),
    DiagnosticCode::Type(TypeDiagnosticCode::ImplInherentUnsupported),
    DiagnosticCode::Type(TypeDiagnosticCode::ImplTypeParamsUnsupported),
    DiagnosticCode::Type(TypeDiagnosticCode::TraitUnknown),
    DiagnosticCode::Type(TypeDiagnosticCode::ImplTargetNotConcrete),
    DiagnosticCode::Type(TypeDiagnosticCode::FunctionSignatureNotFunction),
    DiagnosticCode::Resolve(ResolveDiagnosticCode::AliasTargetNotFound),
    DiagnosticCode::Type(TypeDiagnosticCode::FunctionSignatureOverloadNotFound),
    DiagnosticCode::Type(TypeDiagnosticCode::ImplDuplicateMethod),
    DiagnosticCode::Type(TypeDiagnosticCode::ImplMethodNotInTrait),
    DiagnosticCode::Type(TypeDiagnosticCode::ImplMethodSignatureMismatch),
    DiagnosticCode::Type(TypeDiagnosticCode::ImplMissingTraitMethod),
    DiagnosticCode::Resolve(ResolveDiagnosticCode::EntryFunctionMissingOrAmbiguous),
    DiagnosticCode::Type(TypeDiagnosticCode::ImplDuplicateForTraitTarget),
    DiagnosticCode::Effect(EffectDiagnosticCode::RawBodyMultipleActive),
    DiagnosticCode::Effect(EffectDiagnosticCode::RawBodyTargetMismatch),
    DiagnosticCode::Type(TypeDiagnosticCode::TraitCapabilityUnknown),
    DiagnosticCode::Type(TypeDiagnosticCode::MatchPatternUnsupported),
    DiagnosticCode::Type(TypeDiagnosticCode::MatchWildcardNotLast),
    DiagnosticCode::Resource(ResourceDiagnosticCode::Borrow(
        ResourceBorrowDiagnosticCode::ReturnEscape,
    )),
    DiagnosticCode::Resource(ResourceDiagnosticCode::Raw(
        ResourceRawDiagnosticCode::OwnershipViolation,
    )),
    DiagnosticCode::Resource(ResourceDiagnosticCode::Raw(
        ResourceRawDiagnosticCode::IdentityEscape,
    )),
    DiagnosticCode::Resource(ResourceDiagnosticCode::Lower(
        ResourceLowerDiagnosticCode::Incomplete,
    )),
    DiagnosticCode::Backend(BackendDiagnosticCode::Wasm(
        WasmDiagnosticCode::ExternSignatureUnsupported,
    )),
    DiagnosticCode::Backend(BackendDiagnosticCode::Wasm(
        WasmDiagnosticCode::FunctionSignatureUnsupported,
    )),
    DiagnosticCode::Backend(BackendDiagnosticCode::Wasm(
        WasmDiagnosticCode::ReturnValueMissing,
    )),
    DiagnosticCode::Backend(BackendDiagnosticCode::Wasm(
        WasmDiagnosticCode::RawLineParseError,
    )),
    DiagnosticCode::Backend(BackendDiagnosticCode::Wasm(
        WasmDiagnosticCode::LlvmIrBodyUnsupported,
    )),
    DiagnosticCode::Backend(BackendDiagnosticCode::Wasm(
        WasmDiagnosticCode::StringLiteralNotFound,
    )),
    DiagnosticCode::Backend(BackendDiagnosticCode::Wasm(
        WasmDiagnosticCode::VariableUnknown,
    )),
    DiagnosticCode::Backend(BackendDiagnosticCode::Wasm(
        WasmDiagnosticCode::FunctionValueUnknown,
    )),
    DiagnosticCode::Backend(BackendDiagnosticCode::Wasm(
        WasmDiagnosticCode::FunctionUnknown,
    )),
    DiagnosticCode::Backend(BackendDiagnosticCode::Wasm(
        WasmDiagnosticCode::IndirectSignatureMissing,
    )),
    DiagnosticCode::Backend(BackendDiagnosticCode::Wasm(
        WasmDiagnosticCode::IndirectSignatureUnsupported,
    )),
    DiagnosticCode::Backend(BackendDiagnosticCode::Wasm(
        WasmDiagnosticCode::IntrinsicUnknown,
    )),
    DiagnosticCode::Backend(BackendDiagnosticCode::Wasm(
        WasmDiagnosticCode::EnumPayloadTypeUnsupported,
    )),
    DiagnosticCode::Backend(BackendDiagnosticCode::Wasm(
        WasmDiagnosticCode::StructFieldTypeUnsupported,
    )),
    DiagnosticCode::Backend(BackendDiagnosticCode::Wasm(
        WasmDiagnosticCode::TupleElementTypeUnsupported,
    )),
    DiagnosticCode::Backend(BackendDiagnosticCode::Wasm(
        WasmDiagnosticCode::IntrinsicArityMismatch,
    )),
    DiagnosticCode::Backend(BackendDiagnosticCode::Wasm(
        WasmDiagnosticCode::FieldSelectorUnsupported,
    )),
    DiagnosticCode::Backend(BackendDiagnosticCode::Wasm(
        WasmDiagnosticCode::FieldValueTypeUnsupported,
    )),
    DiagnosticCode::Backend(BackendDiagnosticCode::Wasm(
        WasmDiagnosticCode::LoweredSignatureMissing,
    )),
    DiagnosticCode::Backend(BackendDiagnosticCode::Llvm(
        LlvmDiagnosticCode::RawBodyMismatch,
    )),
    DiagnosticCode::Backend(BackendDiagnosticCode::Llvm(
        LlvmDiagnosticCode::VariableUnknown,
    )),
    DiagnosticCode::Backend(BackendDiagnosticCode::Llvm(
        LlvmDiagnosticCode::FunctionValueUnknown,
    )),
    DiagnosticCode::Backend(BackendDiagnosticCode::Llvm(
        LlvmDiagnosticCode::FunctionUnknown,
    )),
    DiagnosticCode::Backend(BackendDiagnosticCode::Llvm(
        LlvmDiagnosticCode::IntrinsicUnknown,
    )),
    DiagnosticCode::Backend(BackendDiagnosticCode::Llvm(
        LlvmDiagnosticCode::HirUnsupported,
    )),
    DiagnosticCode::Backend(BackendDiagnosticCode::TraitCallUnresolved),
];

impl DiagnosticCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            DiagnosticCode::Loader(code) => code.as_str(),
            DiagnosticCode::Lexer(code) => code.as_str(),
            DiagnosticCode::Parser(code) => code.as_str(),
            DiagnosticCode::Resolve(code) => code.as_str(),
            DiagnosticCode::Type(code) => code.as_str(),
            DiagnosticCode::Effect(code) => code.as_str(),
            DiagnosticCode::Resource(code) => code.as_str(),
            DiagnosticCode::Backend(code) => code.as_str(),
        }
    }

    pub const fn message(self) -> &'static str {
        match self {
            DiagnosticCode::Loader(code) => code.message(),
            DiagnosticCode::Lexer(code) => code.message(),
            DiagnosticCode::Parser(code) => code.message(),
            DiagnosticCode::Resolve(code) => code.message(),
            DiagnosticCode::Type(code) => code.message(),
            DiagnosticCode::Effect(code) => code.message(),
            DiagnosticCode::Resource(code) => code.message(),
            DiagnosticCode::Backend(code) => code.message(),
        }
    }
}

impl LoaderDiagnosticCode {
    const fn as_str(self) -> &'static str {
        match self {
            LoaderDiagnosticCode::TargetMultipleDirective => "loader.target.multiple_directive",
            LoaderDiagnosticCode::TargetUnknown => "loader.target.unknown",
            LoaderDiagnosticCode::SourceFailure => "loader.source.failure",
            LoaderDiagnosticCode::ConditionalGateInvalid => "loader.conditional_gate.invalid",
        }
    }

    const fn message(self) -> &'static str {
        match self {
            LoaderDiagnosticCode::TargetMultipleDirective => {
                "multiple #target directives are not allowed"
            }
            LoaderDiagnosticCode::TargetUnknown => "unknown target in #target",
            LoaderDiagnosticCode::SourceFailure => "loader error",
            LoaderDiagnosticCode::ConditionalGateInvalid => "invalid conditional compilation gate",
        }
    }
}

impl LexerDiagnosticCode {
    const fn as_str(self) -> &'static str {
        match self {
            LexerDiagnosticCode::DirectiveUnknown => "lexer.directive.unknown",
            LexerDiagnosticCode::TokenUnknown => "lexer.token.unknown",
            LexerDiagnosticCode::IndentArgumentInvalid => "lexer.indent.argument_invalid",
            LexerDiagnosticCode::IndentTabsNotAllowed => "lexer.indent.tabs_not_allowed",
            LexerDiagnosticCode::RawBlockExpectedIndent => "lexer.raw_block.expected_indent",
            LexerDiagnosticCode::PubPrefixInvalid => "lexer.pub_prefix.invalid",
            LexerDiagnosticCode::IndentWidthMismatch => "lexer.indent.width_mismatch",
            LexerDiagnosticCode::IndentLevelMismatch => "lexer.indent.level_mismatch",
            LexerDiagnosticCode::StringInvalidEscape => "lexer.string.invalid_escape",
            LexerDiagnosticCode::StringUnterminated => "lexer.string.unterminated",
            LexerDiagnosticCode::CharInvalid => "lexer.char.invalid",
        }
    }

    const fn message(self) -> &'static str {
        match self {
            LexerDiagnosticCode::DirectiveUnknown => "unknown directive",
            LexerDiagnosticCode::TokenUnknown => "unknown token",
            LexerDiagnosticCode::IndentArgumentInvalid => "invalid #indent argument",
            LexerDiagnosticCode::IndentTabsNotAllowed => "tabs are not allowed for indentation",
            LexerDiagnosticCode::RawBlockExpectedIndent => {
                "expected indented block after raw directive"
            }
            LexerDiagnosticCode::PubPrefixInvalid => "pub prefix is only allowed for #import",
            LexerDiagnosticCode::IndentWidthMismatch => {
                "indentation is not aligned to #indent width"
            }
            LexerDiagnosticCode::IndentLevelMismatch => {
                "indentation level does not match any previous indent"
            }
            LexerDiagnosticCode::StringInvalidEscape => "invalid escape in string literal",
            LexerDiagnosticCode::StringUnterminated => "unterminated string literal",
            LexerDiagnosticCode::CharInvalid => "invalid char literal",
        }
    }
}

impl ParserDiagnosticCode {
    const fn as_str(self) -> &'static str {
        match self {
            ParserDiagnosticCode::TokenExpected => "parser.token.expected",
            ParserDiagnosticCode::TokenUnexpected => "parser.token.unexpected",
            ParserDiagnosticCode::IdentifierExpected => "parser.identifier.expected",
            ParserDiagnosticCode::TypeExprInvalid => "parser.type_expr.invalid",
            ParserDiagnosticCode::IdentifierReservedKeyword => "parser.identifier.reserved_keyword",
            ParserDiagnosticCode::ExternSignatureInvalid => "parser.extern_signature.invalid",
        }
    }

    const fn message(self) -> &'static str {
        match self {
            ParserDiagnosticCode::TokenExpected => "expected token",
            ParserDiagnosticCode::TokenUnexpected => "unexpected token",
            ParserDiagnosticCode::IdentifierExpected => "expected identifier",
            ParserDiagnosticCode::TypeExprInvalid => "invalid type expression",
            ParserDiagnosticCode::IdentifierReservedKeyword => {
                "reserved keyword cannot be used as identifier"
            }
            ParserDiagnosticCode::ExternSignatureInvalid => "invalid #extern signature",
        }
    }
}

impl ResolveDiagnosticCode {
    const fn as_str(self) -> &'static str {
        match self {
            ResolveDiagnosticCode::ImportAmbiguous => "resolve.import.ambiguous",
            ResolveDiagnosticCode::IdentifierUndefined => "resolve.identifier.undefined",
            ResolveDiagnosticCode::ShadowNoShadowViolation => "resolve.shadow.no_shadow_violation",
            ResolveDiagnosticCode::ShadowNoShadowConflict => "resolve.shadow.no_shadow_conflict",
            ResolveDiagnosticCode::ItemNameConflict => "resolve.item.name_conflict",
            ResolveDiagnosticCode::AliasTargetNotFound => "resolve.alias.target_not_found",
            ResolveDiagnosticCode::EntryFunctionMissingOrAmbiguous => {
                "resolve.entry_function.missing_or_ambiguous"
            }
        }
    }

    const fn message(self) -> &'static str {
        match self {
            ResolveDiagnosticCode::ImportAmbiguous => "ambiguous import",
            ResolveDiagnosticCode::IdentifierUndefined => "undefined identifier",
            ResolveDiagnosticCode::ShadowNoShadowViolation => "cannot shadow non-shadowable symbol",
            ResolveDiagnosticCode::ShadowNoShadowConflict => "noshadow declaration conflicts",
            ResolveDiagnosticCode::ItemNameConflict => "name already used by another item",
            ResolveDiagnosticCode::AliasTargetNotFound => "alias target not found",
            ResolveDiagnosticCode::EntryFunctionMissingOrAmbiguous => {
                "entry function is missing or ambiguous"
            }
        }
    }
}

impl TypeDiagnosticCode {
    const fn as_str(self) -> &'static str {
        match self {
            TypeDiagnosticCode::VariableUndefined => "type.variable.undefined",
            TypeDiagnosticCode::ReturnTypeMismatch => "type.return.mismatch",
            TypeDiagnosticCode::AnnotationMismatch => "type.annotation.mismatch",
            TypeDiagnosticCode::OverloadAmbiguous => "type.overload.ambiguous",
            TypeDiagnosticCode::OverloadNoMatch => "type.overload.no_match",
            TypeDiagnosticCode::MatchScrutineeNotEnum => "type.match.scrutinee_not_enum",
            TypeDiagnosticCode::MatchDuplicateArm => "type.match.duplicate_arm",
            TypeDiagnosticCode::MatchNonExhaustive => "type.match.non_exhaustive",
            TypeDiagnosticCode::MutationImmutable => "type.mutation.immutable",
            TypeDiagnosticCode::FieldInvalidAccess => "type.field.invalid_access",
            TypeDiagnosticCode::IntrinsicUnknown => "type.intrinsic.unknown",
            TypeDiagnosticCode::LiteralIntInvalid => "type.literal.int_invalid",
            TypeDiagnosticCode::LiteralCharOutOfRange => "type.literal.char_out_of_range",
            TypeDiagnosticCode::PipeInvalid => "type.pipe.invalid",
            TypeDiagnosticCode::StackExtraValues => "type.stack.extra_values",
            TypeDiagnosticCode::BlockStackInconsistent => "type.block.stack_inconsistent",
            TypeDiagnosticCode::NestedGenericFunctionUnsupported => {
                "type.nested_function.generic_unsupported"
            }
            TypeDiagnosticCode::RawBlockInvalidPlacement => "type.raw_block.invalid_placement",
            TypeDiagnosticCode::FunctionValueCapturingUnsupported => {
                "type.function_value.capturing_unsupported"
            }
            TypeDiagnosticCode::IndirectCallRequiresFunctionValue => {
                "type.indirect_call.requires_function_value"
            }
            TypeDiagnosticCode::CallCaptureArityMismatch => "type.call.capture_arity_mismatch",
            TypeDiagnosticCode::VariableNotCallable => "type.call.variable_not_callable",
            TypeDiagnosticCode::OverloadTypeArgsMismatch => "type.overload.type_args_mismatch",
            TypeDiagnosticCode::ArgumentMismatch => "type.argument.mismatch",
            TypeDiagnosticCode::FunctionRefRequiresCallable => {
                "type.function_ref.requires_callable"
            }
            TypeDiagnosticCode::VariableTypeArgsNotAllowed => "type.variable.type_args_not_allowed",
            TypeDiagnosticCode::AssignmentMismatch => "type.assignment.mismatch",
            TypeDiagnosticCode::AssignmentUndefinedVariable => "type.assignment.undefined_variable",
            TypeDiagnosticCode::IfArityMismatch => "type.if.arity_mismatch",
            TypeDiagnosticCode::IfConditionMismatch => "type.if.condition_mismatch",
            TypeDiagnosticCode::WhileArityMismatch => "type.while.arity_mismatch",
            TypeDiagnosticCode::WhileConditionMismatch => "type.while.condition_mismatch",
            TypeDiagnosticCode::WhileBodyMismatch => "type.while.body_mismatch",
            TypeDiagnosticCode::MatchVariantUnknown => "type.match.variant_unknown",
            TypeDiagnosticCode::MatchPayloadBindingInvalid => "type.match.payload_binding_invalid",
            TypeDiagnosticCode::MatchArmsMismatch => "type.match.arms_mismatch",
            TypeDiagnosticCode::IntrinsicTypeArgArityMismatch => {
                "type.intrinsic.type_arg_arity_mismatch"
            }
            TypeDiagnosticCode::IntrinsicArgArityMismatch => "type.intrinsic.arg_arity_mismatch",
            TypeDiagnosticCode::IntrinsicArgTypeMismatch => "type.intrinsic.arg_type_mismatch",
            TypeDiagnosticCode::CopyImplTargetNotCopy => "type.copy_impl.target_not_copy",
            TypeDiagnosticCode::CopyImplRequiresClone => "type.copy_impl.requires_clone",
            TypeDiagnosticCode::TraitMethodTypeArgsUnsupported => {
                "type.trait_method.type_args_unsupported"
            }
            TypeDiagnosticCode::TraitMethodNotFound => "type.trait_method.not_found",
            TypeDiagnosticCode::ArgumentArityMismatch => "type.argument.arity_mismatch",
            TypeDiagnosticCode::TraitBoundUnsatisfied => "type.trait_bound.unsatisfied",
            TypeDiagnosticCode::DerefInvalid => "type.deref.invalid",
            TypeDiagnosticCode::AssignmentArityMismatch => "type.assignment.arity_mismatch",
            TypeDiagnosticCode::CallReductionLimitExceeded => "type.call_reduction.limit_exceeded",
            TypeDiagnosticCode::TraitBoundUnknown => "type.trait_bound.unknown",
            TypeDiagnosticCode::ExternWasiTargetMismatch => "type.extern.wasi_target_mismatch",
            TypeDiagnosticCode::ExternSignatureNotFunction => "type.extern.signature_not_function",
            TypeDiagnosticCode::EnumTypeParamBoundsUnsupported => {
                "type.enum.type_param_bounds_unsupported"
            }
            TypeDiagnosticCode::StructTypeParamBoundsUnsupported => {
                "type.struct.type_param_bounds_unsupported"
            }
            TypeDiagnosticCode::TraitTypeParamsUnsupported => "type.trait.type_params_unsupported",
            TypeDiagnosticCode::TraitMethodTypeParamsUnsupported => {
                "type.trait_method.type_params_unsupported"
            }
            TypeDiagnosticCode::ImplInherentUnsupported => "type.impl.inherent_unsupported",
            TypeDiagnosticCode::ImplTypeParamsUnsupported => "type.impl.type_params_unsupported",
            TypeDiagnosticCode::TraitUnknown => "type.trait.unknown",
            TypeDiagnosticCode::ImplTargetNotConcrete => "type.impl.target_not_concrete",
            TypeDiagnosticCode::FunctionSignatureNotFunction => {
                "type.function.signature_not_function"
            }
            TypeDiagnosticCode::FunctionSignatureOverloadNotFound => {
                "type.function.signature_overload_not_found"
            }
            TypeDiagnosticCode::ImplDuplicateMethod => "type.impl.duplicate_method",
            TypeDiagnosticCode::ImplMethodNotInTrait => "type.impl.method_not_in_trait",
            TypeDiagnosticCode::ImplMethodSignatureMismatch => {
                "type.impl.method_signature_mismatch"
            }
            TypeDiagnosticCode::ImplMissingTraitMethod => "type.impl.missing_trait_method",
            TypeDiagnosticCode::ImplDuplicateForTraitTarget => {
                "type.impl.duplicate_for_trait_target"
            }
            TypeDiagnosticCode::TraitCapabilityUnknown => "type.trait_capability.unknown",
            TypeDiagnosticCode::MatchPatternUnsupported => "type.match.pattern_unsupported",
            TypeDiagnosticCode::MatchWildcardNotLast => "type.match.wildcard_not_last",
        }
    }

    const fn message(self) -> &'static str {
        match self {
            TypeDiagnosticCode::VariableUndefined => "undefined variable",
            TypeDiagnosticCode::ReturnTypeMismatch => "return type does not match signature",
            TypeDiagnosticCode::AnnotationMismatch => "type annotation mismatch",
            TypeDiagnosticCode::OverloadAmbiguous => "ambiguous overload",
            TypeDiagnosticCode::OverloadNoMatch => "function signature does not match any overload",
            TypeDiagnosticCode::MatchScrutineeNotEnum => "match scrutinee must be an enum",
            TypeDiagnosticCode::MatchDuplicateArm => "duplicate match arm",
            TypeDiagnosticCode::MatchNonExhaustive => "non-exhaustive match",
            TypeDiagnosticCode::MutationImmutable => "immutable mutation",
            TypeDiagnosticCode::FieldInvalidAccess => "cannot access field on this type",
            TypeDiagnosticCode::IntrinsicUnknown => "unknown intrinsic",
            TypeDiagnosticCode::LiteralIntInvalid => "invalid integer literal",
            TypeDiagnosticCode::LiteralCharOutOfRange => {
                "char literal is outside current i32-backed codegen range"
            }
            TypeDiagnosticCode::PipeInvalid => "pipe usage error",
            TypeDiagnosticCode::StackExtraValues => "expression left extra values on the stack",
            TypeDiagnosticCode::BlockStackInconsistent => "block leaves inconsistent stack state",
            TypeDiagnosticCode::NestedGenericFunctionUnsupported => {
                "nested generic functions are not supported yet"
            }
            TypeDiagnosticCode::RawBlockInvalidPlacement => {
                "raw backend block is only allowed as a function body"
            }
            TypeDiagnosticCode::FunctionValueCapturingUnsupported => {
                "capturing function cannot be used as a function value yet"
            }
            TypeDiagnosticCode::IndirectCallRequiresFunctionValue => {
                "indirect call requires a function value"
            }
            TypeDiagnosticCode::CallCaptureArityMismatch => {
                "internal error: capture arity mismatch"
            }
            TypeDiagnosticCode::VariableNotCallable => "variable is not callable",
            TypeDiagnosticCode::OverloadTypeArgsMismatch => {
                "type arguments do not match any overload"
            }
            TypeDiagnosticCode::ArgumentMismatch => "argument type mismatch",
            TypeDiagnosticCode::FunctionRefRequiresCallable => {
                "only callable symbols can be referenced with '@'"
            }
            TypeDiagnosticCode::VariableTypeArgsNotAllowed => {
                "type arguments are not allowed for variables"
            }
            TypeDiagnosticCode::AssignmentMismatch => "type mismatch in assignment",
            TypeDiagnosticCode::AssignmentUndefinedVariable => "undefined variable for assignment",
            TypeDiagnosticCode::IfArityMismatch => "if expects three arguments",
            TypeDiagnosticCode::IfConditionMismatch => "if condition must be bool",
            TypeDiagnosticCode::WhileArityMismatch => "while expects two arguments",
            TypeDiagnosticCode::WhileConditionMismatch => "while condition must be bool",
            TypeDiagnosticCode::WhileBodyMismatch => "while body must be unit",
            TypeDiagnosticCode::MatchVariantUnknown => "unknown enum variant in match",
            TypeDiagnosticCode::MatchPayloadBindingInvalid => "variant has no payload to bind",
            TypeDiagnosticCode::MatchArmsMismatch => "match arms have incompatible types",
            TypeDiagnosticCode::IntrinsicTypeArgArityMismatch => "callsite_span expects 1 type arg",
            TypeDiagnosticCode::IntrinsicArgArityMismatch => "intrinsic expects 1 argument",
            TypeDiagnosticCode::IntrinsicArgTypeMismatch => "intrinsic argument type mismatch",
            TypeDiagnosticCode::CopyImplTargetNotCopy => "copy impl target type is not copyable",
            TypeDiagnosticCode::CopyImplRequiresClone => {
                "copy impl requires clone impl for the same target type"
            }
            TypeDiagnosticCode::TraitMethodTypeArgsUnsupported => {
                "type arguments are not supported for trait methods yet"
            }
            TypeDiagnosticCode::TraitMethodNotFound => "unknown method for trait",
            TypeDiagnosticCode::ArgumentArityMismatch => "argument count mismatch",
            TypeDiagnosticCode::TraitBoundUnsatisfied => "type does not satisfy trait bound",
            TypeDiagnosticCode::DerefInvalid => "cannot dereference non-reference type",
            TypeDiagnosticCode::AssignmentArityMismatch => "assignment expects one argument",
            TypeDiagnosticCode::CallReductionLimitExceeded => {
                "call reduction exceeded maximum iterations"
            }
            TypeDiagnosticCode::TraitBoundUnknown => "unknown trait bound",
            TypeDiagnosticCode::ExternWasiTargetMismatch => {
                "WASI import is only allowed for #target wasi"
            }
            TypeDiagnosticCode::ExternSignatureNotFunction => {
                "extern signature must be a function type"
            }
            TypeDiagnosticCode::EnumTypeParamBoundsUnsupported => {
                "enum type parameter bounds are not supported yet"
            }
            TypeDiagnosticCode::StructTypeParamBoundsUnsupported => {
                "struct type parameter bounds are not supported yet"
            }
            TypeDiagnosticCode::TraitTypeParamsUnsupported => {
                "trait type parameters are not supported yet"
            }
            TypeDiagnosticCode::TraitMethodTypeParamsUnsupported => {
                "trait methods cannot have type parameters yet"
            }
            TypeDiagnosticCode::ImplInherentUnsupported => "inherent impl is not supported yet",
            TypeDiagnosticCode::ImplTypeParamsUnsupported => {
                "impl type parameters are not supported yet"
            }
            TypeDiagnosticCode::TraitUnknown => "unknown trait",
            TypeDiagnosticCode::ImplTargetNotConcrete => "impl target type must be concrete",
            TypeDiagnosticCode::FunctionSignatureNotFunction => {
                "function signature must be a function type"
            }
            TypeDiagnosticCode::FunctionSignatureOverloadNotFound => {
                "function signature does not match any overload"
            }
            TypeDiagnosticCode::ImplDuplicateMethod => "duplicate method in impl",
            TypeDiagnosticCode::ImplMethodNotInTrait => "method is not found in the target trait",
            TypeDiagnosticCode::ImplMethodSignatureMismatch => {
                "impl method signature does not match trait"
            }
            TypeDiagnosticCode::ImplMissingTraitMethod => "missing required trait method in impl",
            TypeDiagnosticCode::ImplDuplicateForTraitTarget => {
                "duplicate impl for same trait and target type"
            }
            TypeDiagnosticCode::TraitCapabilityUnknown => "unknown trait capability",
            TypeDiagnosticCode::MatchPatternUnsupported => {
                "match arm pattern is not supported for this scrutinee type"
            }
            TypeDiagnosticCode::MatchWildcardNotLast => "wildcard match arm must be last",
        }
    }
}

impl EffectDiagnosticCode {
    const fn as_str(self) -> &'static str {
        match self {
            EffectDiagnosticCode::OverloadMismatch => "effect.overload.mismatch",
            EffectDiagnosticCode::PureCallsImpure => "effect.pure.calls_impure",
            EffectDiagnosticCode::RawBodyMultipleActive => "effect.raw_body.multiple_active",
            EffectDiagnosticCode::RawBodyTargetMismatch => "effect.raw_body.target_mismatch",
        }
    }

    const fn message(self) -> &'static str {
        match self {
            EffectDiagnosticCode::OverloadMismatch => {
                "overloaded functions must have the same effect"
            }
            EffectDiagnosticCode::PureCallsImpure => "pure context cannot call impure function",
            EffectDiagnosticCode::RawBodyMultipleActive => {
                "multiple active raw bodies in one function"
            }
            EffectDiagnosticCode::RawBodyTargetMismatch => {
                "raw body does not match the active target"
            }
        }
    }
}

impl ResourceDiagnosticCode {
    const fn as_str(self) -> &'static str {
        match self {
            ResourceDiagnosticCode::Move(code) => code.as_str(),
            ResourceDiagnosticCode::Borrow(code) => code.as_str(),
            ResourceDiagnosticCode::Raw(code) => code.as_str(),
            ResourceDiagnosticCode::Lower(code) => code.as_str(),
        }
    }

    const fn message(self) -> &'static str {
        match self {
            ResourceDiagnosticCode::Move(code) => code.message(),
            ResourceDiagnosticCode::Borrow(code) => code.message(),
            ResourceDiagnosticCode::Raw(code) => code.message(),
            ResourceDiagnosticCode::Lower(code) => code.message(),
        }
    }
}

impl ResourceMoveDiagnosticCode {
    const fn as_str(self) -> &'static str {
        match self {
            ResourceMoveDiagnosticCode::UseMoved => "resource.move.use_moved",
            ResourceMoveDiagnosticCode::UsePossiblyMoved => "resource.move.use_possibly_moved",
            ResourceMoveDiagnosticCode::DropMoved => "resource.move.drop_moved",
            ResourceMoveDiagnosticCode::DropPossiblyMoved => "resource.move.drop_possibly_moved",
            ResourceMoveDiagnosticCode::LoopPossiblyMoved => "resource.move.loop_possibly_moved",
        }
    }

    const fn message(self) -> &'static str {
        match self {
            ResourceMoveDiagnosticCode::UseMoved => "use of moved value",
            ResourceMoveDiagnosticCode::UsePossiblyMoved => "use of potentially moved value",
            ResourceMoveDiagnosticCode::DropMoved => "drop of moved value",
            ResourceMoveDiagnosticCode::DropPossiblyMoved => "drop of potentially moved value",
            ResourceMoveDiagnosticCode::LoopPossiblyMoved => "potentially moved value in loop",
        }
    }
}

impl ResourceBorrowDiagnosticCode {
    const fn as_str(self) -> &'static str {
        match self {
            ResourceBorrowDiagnosticCode::MoveFromShared => "resource.borrow.move_from_shared",
            ResourceBorrowDiagnosticCode::UseDuringUnique => "resource.borrow.use_during_unique",
            ResourceBorrowDiagnosticCode::AssignDuringShared => {
                "resource.borrow.assign_during_shared"
            }
            ResourceBorrowDiagnosticCode::AssignDuringUnique => {
                "resource.borrow.assign_during_unique"
            }
            ResourceBorrowDiagnosticCode::DropDuringShared => "resource.borrow.drop_during_shared",
            ResourceBorrowDiagnosticCode::DropDuringUnique => "resource.borrow.drop_during_unique",
            ResourceBorrowDiagnosticCode::UniqueDuringShared => {
                "resource.borrow.unique_during_shared"
            }
            ResourceBorrowDiagnosticCode::BorrowDuringUnique => {
                "resource.borrow.borrow_during_unique"
            }
            ResourceBorrowDiagnosticCode::BorrowMoved => "resource.borrow.borrow_moved",
            ResourceBorrowDiagnosticCode::BorrowPossiblyMoved => {
                "resource.borrow.borrow_possibly_moved"
            }
            ResourceBorrowDiagnosticCode::ReturnEscape => "resource.borrow.return_escape",
        }
    }

    const fn message(self) -> &'static str {
        match self {
            ResourceBorrowDiagnosticCode::MoveFromShared => {
                "cannot move out of shared borrowed value"
            }
            ResourceBorrowDiagnosticCode::UseDuringUnique => "use of uniquely borrowed value",
            ResourceBorrowDiagnosticCode::AssignDuringShared => {
                "cannot assign to shared borrowed value"
            }
            ResourceBorrowDiagnosticCode::AssignDuringUnique => {
                "cannot assign to uniquely borrowed value"
            }
            ResourceBorrowDiagnosticCode::DropDuringShared => "cannot drop shared borrowed value",
            ResourceBorrowDiagnosticCode::DropDuringUnique => "cannot drop uniquely borrowed value",
            ResourceBorrowDiagnosticCode::UniqueDuringShared => {
                "cannot uniquely borrow shared borrowed value"
            }
            ResourceBorrowDiagnosticCode::BorrowDuringUnique => {
                "cannot borrow uniquely borrowed value"
            }
            ResourceBorrowDiagnosticCode::BorrowMoved => "borrow of moved value",
            ResourceBorrowDiagnosticCode::BorrowPossiblyMoved => {
                "borrow of potentially moved value"
            }
            ResourceBorrowDiagnosticCode::ReturnEscape => "borrowed value escapes its scope",
        }
    }
}

impl ResourceRawDiagnosticCode {
    const fn as_str(self) -> &'static str {
        match self {
            ResourceRawDiagnosticCode::OwnershipViolation => "resource.raw.ownership_violation",
            ResourceRawDiagnosticCode::IdentityEscape => "resource.raw.identity_escape",
        }
    }

    const fn message(self) -> &'static str {
        match self {
            ResourceRawDiagnosticCode::OwnershipViolation => "raw memory place ownership violation",
            ResourceRawDiagnosticCode::IdentityEscape => {
                "raw address identity escapes the pure surface"
            }
        }
    }
}

impl ResourceLowerDiagnosticCode {
    const fn as_str(self) -> &'static str {
        match self {
            ResourceLowerDiagnosticCode::Incomplete => "resource.lower.incomplete",
        }
    }

    const fn message(self) -> &'static str {
        match self {
            ResourceLowerDiagnosticCode::Incomplete => {
                "resource ir lowering lost static check input"
            }
        }
    }
}

impl BackendDiagnosticCode {
    const fn as_str(self) -> &'static str {
        match self {
            BackendDiagnosticCode::Wasm(code) => code.as_str(),
            BackendDiagnosticCode::Llvm(code) => code.as_str(),
            BackendDiagnosticCode::TraitCallUnresolved => "backend.codegen.trait_call_unresolved",
        }
    }

    const fn message(self) -> &'static str {
        match self {
            BackendDiagnosticCode::Wasm(code) => code.message(),
            BackendDiagnosticCode::Llvm(code) => code.message(),
            BackendDiagnosticCode::TraitCallUnresolved => {
                "unresolved trait call remained after monomorphize"
            }
        }
    }
}

impl WasmDiagnosticCode {
    const fn as_str(self) -> &'static str {
        match self {
            WasmDiagnosticCode::ExternSignatureUnsupported => {
                "backend.wasm.extern_signature_unsupported"
            }
            WasmDiagnosticCode::FunctionSignatureUnsupported => {
                "backend.wasm.function_signature_unsupported"
            }
            WasmDiagnosticCode::ReturnValueMissing => "backend.wasm.return_value_missing",
            WasmDiagnosticCode::RawLineParseError => "backend.wasm.raw_line_parse_error",
            WasmDiagnosticCode::LlvmIrBodyUnsupported => "backend.wasm.llvm_ir_body_unsupported",
            WasmDiagnosticCode::StringLiteralNotFound => "backend.wasm.string_literal_not_found",
            WasmDiagnosticCode::VariableUnknown => "backend.wasm.variable_unknown",
            WasmDiagnosticCode::FunctionValueUnknown => "backend.wasm.function_value_unknown",
            WasmDiagnosticCode::FunctionUnknown => "backend.wasm.function_unknown",
            WasmDiagnosticCode::IndirectSignatureMissing => {
                "backend.wasm.indirect_signature_missing"
            }
            WasmDiagnosticCode::IndirectSignatureUnsupported => {
                "backend.wasm.indirect_signature_unsupported"
            }
            WasmDiagnosticCode::IntrinsicUnknown => "backend.wasm.intrinsic_unknown",
            WasmDiagnosticCode::EnumPayloadTypeUnsupported => {
                "backend.wasm.enum_payload_type_unsupported"
            }
            WasmDiagnosticCode::StructFieldTypeUnsupported => {
                "backend.wasm.struct_field_type_unsupported"
            }
            WasmDiagnosticCode::TupleElementTypeUnsupported => {
                "backend.wasm.tuple_element_type_unsupported"
            }
            WasmDiagnosticCode::IntrinsicArityMismatch => "backend.wasm.intrinsic_arity_mismatch",
            WasmDiagnosticCode::FieldSelectorUnsupported => {
                "backend.wasm.field_selector_unsupported"
            }
            WasmDiagnosticCode::FieldValueTypeUnsupported => {
                "backend.wasm.field_value_type_unsupported"
            }
            WasmDiagnosticCode::LoweredSignatureMissing => "backend.wasm.lowered_signature_missing",
        }
    }

    const fn message(self) -> &'static str {
        match self {
            WasmDiagnosticCode::ExternSignatureUnsupported => {
                "unsupported extern signature for wasm"
            }
            WasmDiagnosticCode::FunctionSignatureUnsupported => {
                "unsupported function signature for wasm"
            }
            WasmDiagnosticCode::ReturnValueMissing => "function expected to return value",
            WasmDiagnosticCode::RawLineParseError => "invalid raw wasm line",
            WasmDiagnosticCode::LlvmIrBodyUnsupported => {
                "llvm ir block cannot be compiled by wasm backend"
            }
            WasmDiagnosticCode::StringLiteralNotFound => "string literal not found during codegen",
            WasmDiagnosticCode::VariableUnknown => "unknown variable",
            WasmDiagnosticCode::FunctionValueUnknown => "unknown function value",
            WasmDiagnosticCode::FunctionUnknown => "unknown function",
            WasmDiagnosticCode::IndirectSignatureMissing => {
                "missing wasm signature for indirect call"
            }
            WasmDiagnosticCode::IndirectSignatureUnsupported => {
                "unsupported indirect call signature for wasm"
            }
            WasmDiagnosticCode::IntrinsicUnknown => "unknown codegen intrinsic",
            WasmDiagnosticCode::EnumPayloadTypeUnsupported => "unsupported enum payload type",
            WasmDiagnosticCode::StructFieldTypeUnsupported => {
                "unsupported struct field type for codegen"
            }
            WasmDiagnosticCode::TupleElementTypeUnsupported => {
                "unsupported tuple element type for codegen"
            }
            WasmDiagnosticCode::IntrinsicArityMismatch => {
                "codegen intrinsic argument count mismatch"
            }
            WasmDiagnosticCode::FieldSelectorUnsupported => {
                "unsupported field selector for codegen"
            }
            WasmDiagnosticCode::FieldValueTypeUnsupported => {
                "unsupported field value type for codegen"
            }
            WasmDiagnosticCode::LoweredSignatureMissing => {
                "missing lowered wasm function signature"
            }
        }
    }
}

impl LlvmDiagnosticCode {
    const fn as_str(self) -> &'static str {
        match self {
            LlvmDiagnosticCode::RawBodyMismatch => "backend.llvm.raw_body_mismatch",
            LlvmDiagnosticCode::VariableUnknown => "backend.llvm.variable_unknown",
            LlvmDiagnosticCode::FunctionValueUnknown => "backend.llvm.function_value_unknown",
            LlvmDiagnosticCode::FunctionUnknown => "backend.llvm.function_unknown",
            LlvmDiagnosticCode::IntrinsicUnknown => "backend.llvm.intrinsic_unknown",
            LlvmDiagnosticCode::HirUnsupported => "backend.llvm.hir_unsupported",
        }
    }

    const fn message(self) -> &'static str {
        match self {
            LlvmDiagnosticCode::RawBodyMismatch => "raw body does not match the LLVM backend",
            LlvmDiagnosticCode::VariableUnknown => "unknown variable in LLVM codegen",
            LlvmDiagnosticCode::FunctionValueUnknown => "unknown function value in LLVM codegen",
            LlvmDiagnosticCode::FunctionUnknown => "unknown function in LLVM codegen",
            LlvmDiagnosticCode::IntrinsicUnknown => "unknown intrinsic in LLVM codegen",
            LlvmDiagnosticCode::HirUnsupported => "unsupported HIR in LLVM codegen",
        }
    }
}

pub fn message(code: DiagnosticCode) -> &'static str {
    code.message()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn diagnostic_codes_have_unique_serialized_names() {
        let mut serialized = BTreeSet::new();
        for &code in ALL_DIAGNOSTIC_CODES {
            let text = code.as_str();
            assert!(!text.is_empty(), "empty diagnostic code for {:?}", code);
            assert!(
                text.contains('.'),
                "non-hierarchical diagnostic code: {text}"
            );
            assert!(serialized.insert(text), "duplicate diagnostic code: {text}");
            assert!(
                !code.message().is_empty(),
                "empty diagnostic message for {text}"
            );
        }
    }
}

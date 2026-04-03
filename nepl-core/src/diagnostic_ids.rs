//! 診断IDの定義テーブル。
//!
//! 数値IDは表示層（CLI/LSP/Web）で共通利用し、短いIDから詳細説明へ
//! 参照できるようにするための土台です。

/// 診断ID。
///
/// 表示時は `D{number}` 形式（例: `D1001`）で扱います。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum DiagnosticId {
    /// #target が複数回指定された。
    MultipleTargetDirective = 1001,
    /// #target の値が不正。
    UnknownTargetDirective = 1002,
    /// VFS/Loader でソース取得に失敗。
    LoaderFailure = 1003,
    /// open import が複数候補で曖昧。
    AmbiguousImport = 1101,
    /// 字句解析で未知のディレクティブ。
    LexerUnknownDirective = 1201,
    /// 字句解析で未知トークン。
    LexerUnknownToken = 1202,
    /// インデントにタブを使用。
    LexerIndentTabsNotAllowed = 1203,
    /// `#wasm` / `#llvmir` 後のインデントブロック不足。
    LexerExpectedIndentedBlock = 1204,
    /// `pub` 接頭辞の不正利用。
    LexerInvalidPubDirectivePrefix = 1205,
    /// インデント幅が `#indent` 設定に不一致。
    LexerIndentWidthMismatch = 1206,
    /// 既存のインデント階層に一致しない dedent。
    LexerIndentLevelMismatch = 1207,
    /// 文字列エスケープが不正。
    LexerInvalidStringEscape = 1208,
    /// 文字列リテラルが未終端。
    LexerUnterminatedStringLiteral = 1209,
    /// パーサでトークン期待に失敗。
    ParserExpectedToken = 2001,
    /// パーサで予期しないトークン。
    ParserUnexpectedToken = 2002,
    /// パーサで識別子期待に失敗。
    ParserExpectedIdentifier = 2003,
    /// パーサで型式の解釈に失敗。
    ParserInvalidTypeExpr = 2004,
    /// 予約語を識別子として使用。
    ParserReservedKeywordIdentifier = 2005,
    /// #extern シグネチャが不正。
    ParserInvalidExternSignature = 2006,
    /// 未定義識別子。
    TypeUndefinedIdentifier = 3001,
    /// 未定義変数。
    TypeUndefinedVariable = 3002,
    /// 返り値型がシグネチャと不一致。
    TypeReturnTypeMismatch = 3003,
    /// 型注釈不一致。
    TypeAnnotationMismatch = 3004,
    /// オーバーロード解決が曖昧。
    TypeAmbiguousOverload = 3005,
    /// オーバーロード候補に一致なし。
    TypeNoMatchingOverload = 3006,
    /// match 対象が enum ではない。
    TypeMatchScrutineeMustBeEnum = 3007,
    /// match の arm が重複。
    TypeDuplicateMatchArm = 3008,
    /// match が非網羅。
    TypeNonExhaustiveMatch = 3009,
    /// 不変変数への代入。
    TypeImmutableMutation = 3010,
    /// 不正なフィールドアクセス。
    TypeInvalidFieldAccess = 3011,
    /// 不明 intrinsic。
    TypeUnknownIntrinsic = 3012,
    /// pipe 使用エラー。
    TypePipeError = 3013,
    /// non-shadowable への shadow 試行。
    TypeNoShadowViolation = 3014,
    /// no-shadow 宣言の衝突。
    TypeNoShadowConflict = 3015,
    /// 式が余剰値をスタックに残した。
    TypeStackExtraValues = 3016,
    /// capture 付き関数値は未対応。
    TypeCapturingFunctionValueUnsupported = 3017,
    /// 関数値でない値への間接呼び出し。
    TypeIndirectCallRequiresFunctionValue = 3018,
    /// 呼び出し不可能な変数を呼ぼうとした。
    TypeVariableNotCallable = 3019,
    /// オーバーロード群の effect が不一致。
    TypeOverloadEffectMismatch = 3020,
    /// オーバーロード解決時の型引数不一致。
    TypeOverloadTypeArgsMismatch = 3021,
    /// 関数引数の型が一致しない。
    TypeArgumentTypeMismatch = 3022,
    /// `@` は callable 記号にのみ適用可能。
    TypeAtRequiresCallable = 3023,
    /// 変数に型引数は適用できない。
    TypeVariableTypeArgsNotAllowed = 3024,
    /// pure 文脈から impure 関数を呼んだ。
    TypePureCallsImpureFunction = 3025,
    /// 代入時の型不一致。
    TypeAssignmentTypeMismatch = 3036,
    /// 代入対象の変数が未定義。
    TypeAssignmentUndefinedVariable = 3037,
    /// if の引数個数不一致。
    TypeIfArityMismatch = 3038,
    /// if の条件式型不一致。
    TypeIfConditionTypeMismatch = 3039,
    /// while の引数個数不一致。
    TypeWhileArityMismatch = 3040,
    /// while の条件式型不一致。
    TypeWhileConditionTypeMismatch = 3041,
    /// while の本体式型不一致。
    TypeWhileBodyTypeMismatch = 3042,
    /// match で未知のバリアントを指定。
    TypeMatchUnknownVariant = 3043,
    /// payload を持たないバリアントへの束縛。
    TypeMatchPayloadBindingInvalid = 3044,
    /// match の arm 型不一致。
    TypeMatchArmsTypeMismatch = 3045,
    /// intrinsic の型引数個数不一致。
    TypeIntrinsicTypeArgArityMismatch = 3046,
    /// intrinsic の引数個数不一致。
    TypeIntrinsicArgArityMismatch = 3047,
    /// intrinsic の引数型不一致。
    TypeIntrinsicArgTypeMismatch = 3048,
    /// Copy 実装対象が Copy 可能でない。
    TypeCopyImplTargetNotCopy = 3049,
    /// Copy 実装には Clone 実装が必要。
    TypeCopyImplRequiresClone = 3050,
    /// shared borrow 中の値を move しようとした。
    TypeMoveFromSharedBorrowedValue = 3051,
    /// unique borrow 中の値を使用しようとした。
    TypeUseUniquelyBorrowedValue = 3052,
    /// move 済み値を使用しようとした。
    TypeUseMovedValue = 3053,
    /// move された可能性のある値を使用しようとした。
    TypeUsePossiblyMovedValue = 3054,
    /// shared borrow 中の値へ代入しようとした。
    TypeAssignSharedBorrowedValue = 3055,
    /// unique borrow 中の値へ代入しようとした。
    TypeAssignUniquelyBorrowedValue = 3056,
    /// shared borrow 中の値を drop しようとした。
    TypeDropSharedBorrowedValue = 3057,
    /// unique borrow 中の値を drop しようとした。
    TypeDropUniquelyBorrowedValue = 3058,
    /// move 済み値を drop しようとした。
    TypeDropMovedValue = 3059,
    /// move された可能性のある値を drop しようとした。
    TypeDropPossiblyMovedValue = 3060,
    /// shared borrow 中の値を unique borrow しようとした。
    TypeUniqueBorrowSharedBorrowedValue = 3061,
    /// unique borrow 中の値を borrow しようとした。
    TypeBorrowUniquelyBorrowedValue = 3062,
    /// move 済み値を borrow しようとした。
    TypeBorrowMovedValue = 3063,
    /// move された可能性のある値を borrow しようとした。
    TypeBorrowPossiblyMovedValue = 3064,
    /// ループ反復で move される可能性がある。
    TypeLoopPotentiallyMovedValue = 3065,
    /// trait メソッド参照に型引数は未対応。
    TypeTraitMethodTypeArgsNotSupported = 3066,
    /// trait に存在しないメソッドを参照。
    TypeTraitMethodNotFound = 3067,
    /// 関数/コンストラクタ呼び出しの引数個数不一致。
    TypeArgumentArityMismatch = 3068,
    /// trait 境界を満たさない型引数/受信型。
    TypeTraitBoundUnsatisfied = 3069,
    /// 参照型でない値への deref。
    TypeInvalidDeref = 3070,
    /// 代入演算子の引数個数不一致。
    TypeAssignmentArityMismatch = 3071,
    /// 呼び出し簡約の反復上限超過。
    TypeCallReductionLimitExceeded = 3072,
    /// 存在しない trait 境界を指定。
    TypeUnknownTraitBound = 3073,
    /// #extern で WASI import を wasi 以外で使用。
    TypeWasiImportTargetMismatch = 3074,
    /// #extern のシグネチャが関数型でない。
    TypeExternSignatureMustBeFunction = 3075,
    /// 既存 item 名との衝突。
    TypeItemNameConflict = 3076,
    /// enum 型引数の trait 境界は未対応。
    TypeEnumTypeParamBoundsUnsupported = 3077,
    /// struct 型引数の trait 境界は未対応。
    TypeStructTypeParamBoundsUnsupported = 3078,
    /// trait 自体の型引数は未対応。
    TypeTraitTypeParamsUnsupported = 3079,
    /// trait method の型引数は未対応。
    TypeTraitMethodTypeParamsUnsupported = 3080,
    /// inherent impl は未対応。
    TypeInherentImplUnsupported = 3081,
    /// impl 型引数は未対応。
    TypeImplTypeParamsUnsupported = 3082,
    /// 不明 trait。
    TypeUnknownTrait = 3083,
    /// impl 対象型が concrete でない。
    TypeImplTargetMustBeConcrete = 3084,
    /// function signature が関数型でない。
    TypeFunctionSignatureMustBeFunction = 3085,
    /// 関数 alias の対象が未定義。
    TypeAliasTargetNotFound = 3086,
    /// function body 検証時に対応する overload が見つからない。
    TypeFunctionSignatureOverloadNotFound = 3087,
    /// impl 内でメソッド名が重複。
    TypeDuplicateImplMethod = 3088,
    /// trait に存在しないメソッドを impl。
    TypeImplMethodNotFoundInTrait = 3089,
    /// impl メソッドのシグネチャが trait と不一致。
    TypeImplMethodSignatureMismatch = 3090,
    /// trait の必須メソッドが impl に不足。
    TypeImplMissingTraitMethod = 3091,
    /// entry 関数が未定義または曖昧。
    TypeEntryFunctionMissingOrAmbiguous = 3092,
    /// 同一 trait と同一対象型への impl が重複。
    TypeDuplicateImplForTraitTarget = 3093,
    /// 1関数内で有効化された raw body が複数ある。
    TypeMultipleActiveRawBodies = 3094,
    /// raw body と target の組み合わせが不正。
    TypeRawBodyTargetMismatch = 3095,
    /// trait capability 名が不正。
    TypeUnknownTraitCapability = 3096,
    /// WASM backend が extern シグネチャを lower できない。
    CodegenWasmUnsupportedExternSignature = 4001,
    /// WASM backend が関数シグネチャを lower できない。
    CodegenWasmUnsupportedFunctionSignature = 4002,
    /// 戻り値必須関数が値を返さない。
    CodegenWasmMissingReturnValue = 4003,
    /// `#wasm` 生命令の解析に失敗。
    CodegenWasmRawLineParseError = 4004,
    /// WASM backend で `#llvmir` 本体を受理。
    CodegenWasmLlvmIrBodyNotSupported = 4005,
    /// lower 対象の文字列リテラルがテーブルに存在しない。
    CodegenWasmStringLiteralNotFound = 4006,
    /// codegen 時に未知の変数参照。
    CodegenWasmUnknownVariable = 4007,
    /// codegen 時に未知の関数値参照。
    CodegenWasmUnknownFunctionValue = 4008,
    /// codegen 時に未知の関数呼び出し。
    CodegenWasmUnknownFunction = 4009,
    /// 間接呼び出し用のシグネチャが type section に存在しない。
    CodegenWasmMissingIndirectSignature = 4010,
    /// 間接呼び出しシグネチャが WASM で未対応。
    CodegenWasmUnsupportedIndirectSignature = 4011,
    /// codegen intrinsic が未定義。
    CodegenWasmUnknownIntrinsic = 4012,
    /// enum payload 型が WASM lower 非対応。
    CodegenWasmUnsupportedEnumPayloadType = 4013,
    /// struct field 型が WASM lower 非対応。
    CodegenWasmUnsupportedStructFieldType = 4014,
    /// tuple element 型が WASM lower 非対応。
    CodegenWasmUnsupportedTupleElementType = 4015,
}

impl DiagnosticId {
    /// 数値IDへ変換します。
    pub const fn as_u32(self) -> u32 {
        self as u32
    }

    /// 数値IDから列挙値へ変換します。
    pub const fn from_u32(id: u32) -> Option<DiagnosticId> {
        match id {
            1001 => Some(DiagnosticId::MultipleTargetDirective),
            1002 => Some(DiagnosticId::UnknownTargetDirective),
            1003 => Some(DiagnosticId::LoaderFailure),
            1101 => Some(DiagnosticId::AmbiguousImport),
            1201 => Some(DiagnosticId::LexerUnknownDirective),
            1202 => Some(DiagnosticId::LexerUnknownToken),
            1203 => Some(DiagnosticId::LexerIndentTabsNotAllowed),
            1204 => Some(DiagnosticId::LexerExpectedIndentedBlock),
            1205 => Some(DiagnosticId::LexerInvalidPubDirectivePrefix),
            1206 => Some(DiagnosticId::LexerIndentWidthMismatch),
            1207 => Some(DiagnosticId::LexerIndentLevelMismatch),
            1208 => Some(DiagnosticId::LexerInvalidStringEscape),
            1209 => Some(DiagnosticId::LexerUnterminatedStringLiteral),
            2001 => Some(DiagnosticId::ParserExpectedToken),
            2002 => Some(DiagnosticId::ParserUnexpectedToken),
            2003 => Some(DiagnosticId::ParserExpectedIdentifier),
            2004 => Some(DiagnosticId::ParserInvalidTypeExpr),
            2005 => Some(DiagnosticId::ParserReservedKeywordIdentifier),
            2006 => Some(DiagnosticId::ParserInvalidExternSignature),
            3001 => Some(DiagnosticId::TypeUndefinedIdentifier),
            3002 => Some(DiagnosticId::TypeUndefinedVariable),
            3003 => Some(DiagnosticId::TypeReturnTypeMismatch),
            3004 => Some(DiagnosticId::TypeAnnotationMismatch),
            3005 => Some(DiagnosticId::TypeAmbiguousOverload),
            3006 => Some(DiagnosticId::TypeNoMatchingOverload),
            3007 => Some(DiagnosticId::TypeMatchScrutineeMustBeEnum),
            3008 => Some(DiagnosticId::TypeDuplicateMatchArm),
            3009 => Some(DiagnosticId::TypeNonExhaustiveMatch),
            3010 => Some(DiagnosticId::TypeImmutableMutation),
            3011 => Some(DiagnosticId::TypeInvalidFieldAccess),
            3012 => Some(DiagnosticId::TypeUnknownIntrinsic),
            3013 => Some(DiagnosticId::TypePipeError),
            3014 => Some(DiagnosticId::TypeNoShadowViolation),
            3015 => Some(DiagnosticId::TypeNoShadowConflict),
            3016 => Some(DiagnosticId::TypeStackExtraValues),
            3017 => Some(DiagnosticId::TypeCapturingFunctionValueUnsupported),
            3018 => Some(DiagnosticId::TypeIndirectCallRequiresFunctionValue),
            3019 => Some(DiagnosticId::TypeVariableNotCallable),
            3020 => Some(DiagnosticId::TypeOverloadEffectMismatch),
            3021 => Some(DiagnosticId::TypeOverloadTypeArgsMismatch),
            3022 => Some(DiagnosticId::TypeArgumentTypeMismatch),
            3023 => Some(DiagnosticId::TypeAtRequiresCallable),
            3024 => Some(DiagnosticId::TypeVariableTypeArgsNotAllowed),
            3025 => Some(DiagnosticId::TypePureCallsImpureFunction),
            3036 => Some(DiagnosticId::TypeAssignmentTypeMismatch),
            3037 => Some(DiagnosticId::TypeAssignmentUndefinedVariable),
            3038 => Some(DiagnosticId::TypeIfArityMismatch),
            3039 => Some(DiagnosticId::TypeIfConditionTypeMismatch),
            3040 => Some(DiagnosticId::TypeWhileArityMismatch),
            3041 => Some(DiagnosticId::TypeWhileConditionTypeMismatch),
            3042 => Some(DiagnosticId::TypeWhileBodyTypeMismatch),
            3043 => Some(DiagnosticId::TypeMatchUnknownVariant),
            3044 => Some(DiagnosticId::TypeMatchPayloadBindingInvalid),
            3045 => Some(DiagnosticId::TypeMatchArmsTypeMismatch),
            3046 => Some(DiagnosticId::TypeIntrinsicTypeArgArityMismatch),
            3047 => Some(DiagnosticId::TypeIntrinsicArgArityMismatch),
            3048 => Some(DiagnosticId::TypeIntrinsicArgTypeMismatch),
            3049 => Some(DiagnosticId::TypeCopyImplTargetNotCopy),
            3050 => Some(DiagnosticId::TypeCopyImplRequiresClone),
            3051 => Some(DiagnosticId::TypeMoveFromSharedBorrowedValue),
            3052 => Some(DiagnosticId::TypeUseUniquelyBorrowedValue),
            3053 => Some(DiagnosticId::TypeUseMovedValue),
            3054 => Some(DiagnosticId::TypeUsePossiblyMovedValue),
            3055 => Some(DiagnosticId::TypeAssignSharedBorrowedValue),
            3056 => Some(DiagnosticId::TypeAssignUniquelyBorrowedValue),
            3057 => Some(DiagnosticId::TypeDropSharedBorrowedValue),
            3058 => Some(DiagnosticId::TypeDropUniquelyBorrowedValue),
            3059 => Some(DiagnosticId::TypeDropMovedValue),
            3060 => Some(DiagnosticId::TypeDropPossiblyMovedValue),
            3061 => Some(DiagnosticId::TypeUniqueBorrowSharedBorrowedValue),
            3062 => Some(DiagnosticId::TypeBorrowUniquelyBorrowedValue),
            3063 => Some(DiagnosticId::TypeBorrowMovedValue),
            3064 => Some(DiagnosticId::TypeBorrowPossiblyMovedValue),
            3065 => Some(DiagnosticId::TypeLoopPotentiallyMovedValue),
            3066 => Some(DiagnosticId::TypeTraitMethodTypeArgsNotSupported),
            3067 => Some(DiagnosticId::TypeTraitMethodNotFound),
            3068 => Some(DiagnosticId::TypeArgumentArityMismatch),
            3069 => Some(DiagnosticId::TypeTraitBoundUnsatisfied),
            3070 => Some(DiagnosticId::TypeInvalidDeref),
            3071 => Some(DiagnosticId::TypeAssignmentArityMismatch),
            3072 => Some(DiagnosticId::TypeCallReductionLimitExceeded),
            3073 => Some(DiagnosticId::TypeUnknownTraitBound),
            3074 => Some(DiagnosticId::TypeWasiImportTargetMismatch),
            3075 => Some(DiagnosticId::TypeExternSignatureMustBeFunction),
            3076 => Some(DiagnosticId::TypeItemNameConflict),
            3077 => Some(DiagnosticId::TypeEnumTypeParamBoundsUnsupported),
            3078 => Some(DiagnosticId::TypeStructTypeParamBoundsUnsupported),
            3079 => Some(DiagnosticId::TypeTraitTypeParamsUnsupported),
            3080 => Some(DiagnosticId::TypeTraitMethodTypeParamsUnsupported),
            3081 => Some(DiagnosticId::TypeInherentImplUnsupported),
            3082 => Some(DiagnosticId::TypeImplTypeParamsUnsupported),
            3083 => Some(DiagnosticId::TypeUnknownTrait),
            3084 => Some(DiagnosticId::TypeImplTargetMustBeConcrete),
            3085 => Some(DiagnosticId::TypeFunctionSignatureMustBeFunction),
            3086 => Some(DiagnosticId::TypeAliasTargetNotFound),
            3087 => Some(DiagnosticId::TypeFunctionSignatureOverloadNotFound),
            3088 => Some(DiagnosticId::TypeDuplicateImplMethod),
            3089 => Some(DiagnosticId::TypeImplMethodNotFoundInTrait),
            3090 => Some(DiagnosticId::TypeImplMethodSignatureMismatch),
            3091 => Some(DiagnosticId::TypeImplMissingTraitMethod),
            3092 => Some(DiagnosticId::TypeEntryFunctionMissingOrAmbiguous),
            3093 => Some(DiagnosticId::TypeDuplicateImplForTraitTarget),
            3094 => Some(DiagnosticId::TypeMultipleActiveRawBodies),
            3095 => Some(DiagnosticId::TypeRawBodyTargetMismatch),
            3096 => Some(DiagnosticId::TypeUnknownTraitCapability),
            4001 => Some(DiagnosticId::CodegenWasmUnsupportedExternSignature),
            4002 => Some(DiagnosticId::CodegenWasmUnsupportedFunctionSignature),
            4003 => Some(DiagnosticId::CodegenWasmMissingReturnValue),
            4004 => Some(DiagnosticId::CodegenWasmRawLineParseError),
            4005 => Some(DiagnosticId::CodegenWasmLlvmIrBodyNotSupported),
            4006 => Some(DiagnosticId::CodegenWasmStringLiteralNotFound),
            4007 => Some(DiagnosticId::CodegenWasmUnknownVariable),
            4008 => Some(DiagnosticId::CodegenWasmUnknownFunctionValue),
            4009 => Some(DiagnosticId::CodegenWasmUnknownFunction),
            4010 => Some(DiagnosticId::CodegenWasmMissingIndirectSignature),
            4011 => Some(DiagnosticId::CodegenWasmUnsupportedIndirectSignature),
            4012 => Some(DiagnosticId::CodegenWasmUnknownIntrinsic),
            4013 => Some(DiagnosticId::CodegenWasmUnsupportedEnumPayloadType),
            4014 => Some(DiagnosticId::CodegenWasmUnsupportedStructFieldType),
            4015 => Some(DiagnosticId::CodegenWasmUnsupportedTupleElementType),
            _ => None,
        }
    }

    /// 診断IDに対応する短い説明を返します。
    pub const fn message(self) -> &'static str {
        match self {
            DiagnosticId::MultipleTargetDirective => "multiple #target directives are not allowed",
            DiagnosticId::UnknownTargetDirective => "unknown target in #target",
            DiagnosticId::LoaderFailure => "loader error",
            DiagnosticId::AmbiguousImport => "ambiguous import",
            DiagnosticId::LexerUnknownDirective => "unknown directive",
            DiagnosticId::LexerUnknownToken => "unknown token",
            DiagnosticId::LexerIndentTabsNotAllowed => {
                "tabs are not allowed for indentation"
            }
            DiagnosticId::LexerExpectedIndentedBlock => {
                "expected indented block after raw directive"
            }
            DiagnosticId::LexerInvalidPubDirectivePrefix => {
                "pub prefix is only allowed for #import"
            }
            DiagnosticId::LexerIndentWidthMismatch => {
                "indentation is not aligned to #indent width"
            }
            DiagnosticId::LexerIndentLevelMismatch => {
                "indentation level does not match any previous indent"
            }
            DiagnosticId::LexerInvalidStringEscape => "invalid escape in string literal",
            DiagnosticId::LexerUnterminatedStringLiteral => "unterminated string literal",
            DiagnosticId::ParserExpectedToken => "expected token",
            DiagnosticId::ParserUnexpectedToken => "unexpected token",
            DiagnosticId::ParserExpectedIdentifier => "expected identifier",
            DiagnosticId::ParserInvalidTypeExpr => "invalid type expression",
            DiagnosticId::ParserReservedKeywordIdentifier => {
                "reserved keyword cannot be used as identifier"
            }
            DiagnosticId::ParserInvalidExternSignature => "invalid #extern signature",
            DiagnosticId::TypeUndefinedIdentifier => "undefined identifier",
            DiagnosticId::TypeUndefinedVariable => "undefined variable",
            DiagnosticId::TypeReturnTypeMismatch => "return type does not match signature",
            DiagnosticId::TypeAnnotationMismatch => "type annotation mismatch",
            DiagnosticId::TypeAmbiguousOverload => "ambiguous overload",
            DiagnosticId::TypeNoMatchingOverload => "function signature does not match any overload",
            DiagnosticId::TypeMatchScrutineeMustBeEnum => "match scrutinee must be an enum",
            DiagnosticId::TypeDuplicateMatchArm => "duplicate match arm",
            DiagnosticId::TypeNonExhaustiveMatch => "non-exhaustive match",
            DiagnosticId::TypeImmutableMutation => "immutable mutation",
            DiagnosticId::TypeInvalidFieldAccess => "cannot access field on this type",
            DiagnosticId::TypeUnknownIntrinsic => "unknown intrinsic",
            DiagnosticId::TypePipeError => "pipe usage error",
            DiagnosticId::TypeNoShadowViolation => "cannot shadow non-shadowable symbol",
            DiagnosticId::TypeNoShadowConflict => "noshadow declaration conflicts",
            DiagnosticId::TypeStackExtraValues => "expression left extra values on the stack",
            DiagnosticId::TypeCapturingFunctionValueUnsupported => {
                "capturing function cannot be used as a function value yet"
            }
            DiagnosticId::TypeIndirectCallRequiresFunctionValue => {
                "indirect call requires a function value"
            }
            DiagnosticId::TypeVariableNotCallable => "variable is not callable",
            DiagnosticId::TypeOverloadEffectMismatch => {
                "overloaded functions must have the same effect"
            }
            DiagnosticId::TypeOverloadTypeArgsMismatch => {
                "type arguments do not match any overload"
            }
            DiagnosticId::TypeArgumentTypeMismatch => "argument type mismatch",
            DiagnosticId::TypeAtRequiresCallable => {
                "only callable symbols can be referenced with '@'"
            }
            DiagnosticId::TypeVariableTypeArgsNotAllowed => {
                "type arguments are not allowed for variables"
            }
            DiagnosticId::TypePureCallsImpureFunction => {
                "pure context cannot call impure function"
            }
            DiagnosticId::TypeAssignmentTypeMismatch => "type mismatch in assignment",
            DiagnosticId::TypeAssignmentUndefinedVariable => {
                "undefined variable for assignment"
            }
            DiagnosticId::TypeIfArityMismatch => "if expects three arguments",
            DiagnosticId::TypeIfConditionTypeMismatch => "if condition must be bool",
            DiagnosticId::TypeWhileArityMismatch => "while expects two arguments",
            DiagnosticId::TypeWhileConditionTypeMismatch => "while condition must be bool",
            DiagnosticId::TypeWhileBodyTypeMismatch => "while body must be unit",
            DiagnosticId::TypeMatchUnknownVariant => "unknown enum variant in match",
            DiagnosticId::TypeMatchPayloadBindingInvalid => {
                "variant has no payload to bind"
            }
            DiagnosticId::TypeMatchArmsTypeMismatch => {
                "match arms have incompatible types"
            }
            DiagnosticId::TypeIntrinsicTypeArgArityMismatch => {
                "callsite_span expects 1 type arg"
            }
            DiagnosticId::TypeIntrinsicArgArityMismatch => "intrinsic expects 1 argument",
            DiagnosticId::TypeIntrinsicArgTypeMismatch => {
                "intrinsic argument type mismatch"
            }
            DiagnosticId::TypeCopyImplTargetNotCopy => {
                "copy impl target type is not copyable"
            }
            DiagnosticId::TypeCopyImplRequiresClone => {
                "copy impl requires clone impl for the same target type"
            }
            DiagnosticId::TypeMoveFromSharedBorrowedValue => {
                "cannot move out of shared borrowed value"
            }
            DiagnosticId::TypeUseUniquelyBorrowedValue => "use of uniquely borrowed value",
            DiagnosticId::TypeUseMovedValue => "use of moved value",
            DiagnosticId::TypeUsePossiblyMovedValue => "use of potentially moved value",
            DiagnosticId::TypeAssignSharedBorrowedValue => {
                "cannot assign to shared borrowed value"
            }
            DiagnosticId::TypeAssignUniquelyBorrowedValue => {
                "cannot assign to uniquely borrowed value"
            }
            DiagnosticId::TypeDropSharedBorrowedValue => "cannot drop shared borrowed value",
            DiagnosticId::TypeDropUniquelyBorrowedValue => {
                "cannot drop uniquely borrowed value"
            }
            DiagnosticId::TypeDropMovedValue => "drop of moved value",
            DiagnosticId::TypeDropPossiblyMovedValue => "drop of potentially moved value",
            DiagnosticId::TypeUniqueBorrowSharedBorrowedValue => {
                "cannot uniquely borrow shared borrowed value"
            }
            DiagnosticId::TypeBorrowUniquelyBorrowedValue => {
                "cannot borrow uniquely borrowed value"
            }
            DiagnosticId::TypeBorrowMovedValue => "borrow of moved value",
            DiagnosticId::TypeBorrowPossiblyMovedValue => {
                "borrow of potentially moved value"
            }
            DiagnosticId::TypeLoopPotentiallyMovedValue => "potentially moved value in loop",
            DiagnosticId::TypeTraitMethodTypeArgsNotSupported => {
                "type arguments are not supported for trait methods yet"
            }
            DiagnosticId::TypeTraitMethodNotFound => "unknown method for trait",
            DiagnosticId::TypeArgumentArityMismatch => "argument count mismatch",
            DiagnosticId::TypeTraitBoundUnsatisfied => "type does not satisfy trait bound",
            DiagnosticId::TypeInvalidDeref => "cannot dereference non-reference type",
            DiagnosticId::TypeAssignmentArityMismatch => "assignment expects one argument",
            DiagnosticId::TypeCallReductionLimitExceeded => {
                "call reduction exceeded maximum iterations"
            }
            DiagnosticId::TypeUnknownTraitBound => "unknown trait bound",
            DiagnosticId::TypeWasiImportTargetMismatch => {
                "WASI import is only allowed for #target wasi"
            }
            DiagnosticId::TypeExternSignatureMustBeFunction => {
                "extern signature must be a function type"
            }
            DiagnosticId::TypeItemNameConflict => "name already used by another item",
            DiagnosticId::TypeEnumTypeParamBoundsUnsupported => {
                "enum type parameter bounds are not supported yet"
            }
            DiagnosticId::TypeStructTypeParamBoundsUnsupported => {
                "struct type parameter bounds are not supported yet"
            }
            DiagnosticId::TypeTraitTypeParamsUnsupported => {
                "trait type parameters are not supported yet"
            }
            DiagnosticId::TypeTraitMethodTypeParamsUnsupported => {
                "trait methods cannot have type parameters yet"
            }
            DiagnosticId::TypeInherentImplUnsupported => "inherent impl is not supported yet",
            DiagnosticId::TypeImplTypeParamsUnsupported => {
                "impl type parameters are not supported yet"
            }
            DiagnosticId::TypeUnknownTrait => "unknown trait",
            DiagnosticId::TypeImplTargetMustBeConcrete => {
                "impl target type must be concrete"
            }
            DiagnosticId::TypeFunctionSignatureMustBeFunction => {
                "function signature must be a function type"
            }
            DiagnosticId::TypeAliasTargetNotFound => "alias target not found",
            DiagnosticId::TypeFunctionSignatureOverloadNotFound => {
                "function signature does not match any overload"
            }
            DiagnosticId::TypeDuplicateImplMethod => "duplicate method in impl",
            DiagnosticId::TypeImplMethodNotFoundInTrait => {
                "method is not found in the target trait"
            }
            DiagnosticId::TypeImplMethodSignatureMismatch => {
                "impl method signature does not match trait"
            }
            DiagnosticId::TypeImplMissingTraitMethod => "missing required trait method in impl",
            DiagnosticId::TypeEntryFunctionMissingOrAmbiguous => {
                "entry function is missing or ambiguous"
            }
            DiagnosticId::TypeDuplicateImplForTraitTarget => {
                "duplicate impl for same trait and target type"
            }
            DiagnosticId::TypeMultipleActiveRawBodies => {
                "multiple active raw bodies in one function"
            }
            DiagnosticId::TypeRawBodyTargetMismatch => {
                "raw body does not match the active target"
            }
            DiagnosticId::TypeUnknownTraitCapability => "unknown trait capability",
            DiagnosticId::CodegenWasmUnsupportedExternSignature => {
                "unsupported extern signature for wasm"
            }
            DiagnosticId::CodegenWasmUnsupportedFunctionSignature => {
                "unsupported function signature for wasm"
            }
            DiagnosticId::CodegenWasmMissingReturnValue => {
                "function expected to return value"
            }
            DiagnosticId::CodegenWasmRawLineParseError => "invalid raw wasm line",
            DiagnosticId::CodegenWasmLlvmIrBodyNotSupported => {
                "llvm ir block cannot be compiled by wasm backend"
            }
            DiagnosticId::CodegenWasmStringLiteralNotFound => {
                "string literal not found during codegen"
            }
            DiagnosticId::CodegenWasmUnknownVariable => "unknown variable",
            DiagnosticId::CodegenWasmUnknownFunctionValue => "unknown function value",
            DiagnosticId::CodegenWasmUnknownFunction => "unknown function",
            DiagnosticId::CodegenWasmMissingIndirectSignature => {
                "missing wasm signature for indirect call"
            }
            DiagnosticId::CodegenWasmUnsupportedIndirectSignature => {
                "unsupported indirect call signature for wasm"
            }
            DiagnosticId::CodegenWasmUnknownIntrinsic => "unknown codegen intrinsic",
            DiagnosticId::CodegenWasmUnsupportedEnumPayloadType => {
                "unsupported enum payload type"
            }
            DiagnosticId::CodegenWasmUnsupportedStructFieldType => {
                "unsupported struct field type for codegen"
            }
            DiagnosticId::CodegenWasmUnsupportedTupleElementType => {
                "unsupported tuple element type for codegen"
            }
        }
    }

}

/// 既存呼び出し互換: 数値IDから短い説明を返します。
pub fn message(id: u32) -> Option<&'static str> {
    DiagnosticId::from_u32(id).map(|d| d.message())
}

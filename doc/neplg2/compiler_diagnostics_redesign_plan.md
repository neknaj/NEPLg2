# NEPLg2 compiler diagnostic redesign plan

作成日: 2026-04-29
更新日: 2026-04-29

## 目的

NEPLg2 の Rust compiler diagnostic を、現在の型検査、effect 検査、Resource IR、self-host compiler に合う形へ再設計する。

旧来の数値 ID は履歴的な分類であり、Resource IR の owner obligation、initialized cell、borrow lifetime、raw identity boundary の意味を十分に表せない。後方互換は不要とし、診断の内部表現は数値や自由文字列ではなく、階層化された enum で管理する。

関連 issue:

- [ISS-20260429T040748194Z-RUST-COMPILER-DIAGNOSTICS-ARE-NOT-AL-1617747D](../../issues/items/ISS-20260429T040748194Z-RUST-COMPILER-DIAGNOSTICS-ARE-NOT-AL-1617747D.md): Rust compiler diagnostics が Resource IR / self-host model と揃っていない。
- [ISS-20260425T000000Z-RV-CORE-009-58589A3F](../../issues/items/ISS-20260425T000000Z-RV-CORE-009-58589A3F.md): Resource IR 上の move / borrow / drop 検査。
- [ISS-20260427T132406497Z-CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS-DC67DF04](../../issues/items/ISS-20260427T132406497Z-CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS-DC67DF04.md): raw memory effect / ownership boundary。
- [ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF](../../issues/items/ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF.md): `MemPtr` / owner token / initialized cell の分離。
- [NEPLg2 静的検査の複雑化解消計画](./static_check_complexity_reduction_plan.md): Stage 4/5 の Resource IR authoritative gate。

## 現状の問題

### 1. 数値 ID は静的検査の分類として不十分

数値 ID は、型、effect、borrow、Resource IR、backend の意味分類をひとつの平坦な番号空間に押し込む。特に raw memory、owner obligation、initialized cell、storage destructive operation が同じ bucket に寄ると、回帰テストは通っても原因分類が失われる。

この状態では、次の区別が機械的に維持できない。

- type error と effect boundary error。
- raw identity escape と ordinary impure call。
- owner obligation leak と initialized cell violation。
- borrow lifetime escape と active borrow conflict。
- Resource IR lowering 欠落と Resource IR checker violation。

### 2. 文字列 code を内部主キーにすると網羅性が効かない

安定文字列は CLI、web、JSON、doctest の外部 contract として必要である。一方で、Rust core 内部の分岐を文字列で行うと、typo や未処理 variant を compiler が検出できない。

内部表現は必ず `DiagnosticCode` と下位 enum にする。文字列は `as_str()` による表示・シリアライズだけに限定する。

### 3. Diagnostic construction が各 pass に散らばっている

lexer、parser、typecheck、Resource IR gate、backend が、それぞれ message と diagnostic kind を直接組み立てている。Resource IR では typed diagnostic を持っているのに、compiler diagnostic へ変換する場所で意味分類を粗くしてしまう危険がある。

新しい設計では、各 pass は可能な限り enum variant を直接返し、表示層は enum の `match` を通して文字列へ変換する。

## 目標仕様

### DiagnosticCode

Rust core の診断主識別子は次の階層 enum とする。

- `DiagnosticCode::Loader(LoaderDiagnosticCode)`
- `DiagnosticCode::Lexer(LexerDiagnosticCode)`
- `DiagnosticCode::Parser(ParserDiagnosticCode)`
- `DiagnosticCode::Resolve(ResolveDiagnosticCode)`
- `DiagnosticCode::Type(TypeDiagnosticCode)`
- `DiagnosticCode::Effect(EffectDiagnosticCode)`
- `DiagnosticCode::Resource(ResourceDiagnosticCode)`
- `DiagnosticCode::Backend(BackendDiagnosticCode)`

Resource diagnostic はさらに次へ分ける。

- `ResourceDiagnosticCode::Move(ResourceMoveDiagnosticCode)`
- `ResourceDiagnosticCode::Borrow(ResourceBorrowDiagnosticCode)`
- `ResourceDiagnosticCode::Raw(ResourceRawDiagnosticCode)`
- `ResourceDiagnosticCode::Lower(ResourceLowerDiagnosticCode)`

この階層は、追加・削除・分類変更時に Rust の `match` 網羅性検査が効くことを目的にする。`DiagnosticCode::as_str()` と `DiagnosticCode::message()` はすべての variant を明示的に `match` し、wildcard arm は使わない。

### Diagnostic value

Rust core の `Diagnostic` は次を持つ。

- `severity`
- `code: Option<DiagnosticCode>`
- `message`
- `primary`
- `secondary`

数値 ID field は持たない。後方互換用の field も置かない。

将来の拡張で note/help/related label を追加する場合も、診断種別は `DiagnosticCode` から導く。補助的な説明文は表示値であり、識別子にしない。

### 外部表現

CLI、web、JSON、doctest は `DiagnosticCode::as_str()` の結果を使う。

例:

- `resolve.identifier.undefined`
- `effect.pure.calls_impure`
- `resource.borrow.return_escape`
- `resource.raw.ownership_violation`
- `resource.lower.incomplete`
- `backend.wasm.variable_unknown`

外部 contract は stable string code だが、内部の分岐・保持・比較は enum で行う。

### Resource IR diagnostic mapping

Resource IR の diagnostic は、compiler.rs の ad-hoc な番号写像ではなく、次の順で扱う。

1. Resource IR checker が typed diagnostic kind を返す。
2. compiler gate が `DiagnosticCode::Resource(...)` または `DiagnosticCode::Effect(...)` へ写像する。
3. CLI / web / doctest が `as_str()` を使って表示・検査する。

代表例:

| Resource IR diagnostic | DiagnosticCode |
|---|---|
| lowering completeness lost input | `Resource(Lower(Incomplete))` |
| raw storage ownership violation | `Resource(Raw(OwnershipViolation))` |
| use after move | `Resource(Move(UseMoved))` |
| possible use after move | `Resource(Move(UsePossiblyMoved))` |
| active borrow escapes return | `Resource(Borrow(ReturnEscape))` |
| unique borrow while shared borrow is active | `Resource(Borrow(UniqueDuringShared))` |
| pure context calls impure function | `Effect(PureCallsImpure)` |

## 実装計画

### Stage D0: 数値 ID の削除と enum registry 導入

目的: 後方互換なしで、診断の主識別子を階層 enum へ移す。

作業:

- `diagnostic_ids.rs` を削除し、`diagnostic_codes.rs` を正にする。
- `Diagnostic` から数値 ID field を削除する。
- `Diagnostic::with_code` は `DiagnosticCode` だけを受け取る。
- Rust call site は `DiagnosticCode::Category(SubCode::Variant)` を渡す。
- CLI / web / doctest は `as_str()` の結果だけを表示・検査する。
- 旧メタデータ名は受け付けない。

完了条件:

- Rust core / CLI / web / nodesrc / active doctest に旧数値 ID 依存がない。
- `with_code` に自由文字列を渡せない。
- registry consistency test が code string の重複と空 message を検出する。

進捗:

- 2026-04-29: Stage D0 の実装で active code path から数値 ID field、`diagnostic_ids.rs`、`with_id`、旧 `diag_id` metadata を削除した。以後の残作業は D1 以降の builder / note / typed Resource IR mapping として扱う。
- 2026-04-29: CI build で残っていた `nepl-language` / `nepl-lsp` の旧 `DiagnosticId` 参照を削除した。editor/LSP diagnostic は数値 `id` を持たず、`DiagnosticCode::as_str()` 由来の stable string `code` だけを外部へ渡す。

### Stage D1: Diagnostic builder の導入

目的: call site が message と diagnostic kind を直接組み合わせる範囲を減らす。

作業:

- `DiagnosticSpec` または `DiagnosticBuilder` を導入する。
- stage/category は enum から導ける形にする。
- note/help/related label の保存先を設ける。
- lexer/parser/typecheck の代表診断から builder へ移行する。

進捗:

- 2026-04-29: `DiagnosticSpec` と `Diagnostic::error_code` / `error_with_code` / `warning_code` / `warning_with_code` を追加し、compiler-owned enum code を診断生成時点で渡す builder 経路を導入した。これにより、少なくとも移行済み call site では `Diagnostic::error(...).with_code(...)` の後付け組み合わせを避けられる。
- 2026-04-29: `Diagnostic` に `notes` / `helps` を追加し、CLI / web / language / LSP の外部境界で保持するようにした。補助説明は識別子ではなく structured display value として扱う。
- 2026-04-29: Resource IR gate の lowering / raw ownership / borrow conflict / raw identity escape 変換を code-first constructor へ移行した。動的な詳細文は現時点では message に残し、次の D1 follow-up で note/help へ段階的に分離する。
- 2026-04-29: `compiler.rs` に残っていた unresolved trait call、lowered entry 解決、target directive の compiler boundary 診断を code-first constructor へ移行した。これにより `compiler.rs` 内の active diagnostic construction は `Diagnostic::error(...).with_code(...)` を使わず、enum code を生成時点で渡す形に揃った。
- 2026-04-29: `lexer.rs` に module-local な `lexer_error` / `parser_error` helper を導入し、lexer 内の active diagnostic construction を code-first constructor へ移行した。indent / raw block / directive / string / char / unknown token 診断は、後付け `.with_code(...)` ではなく生成時点で `LexerDiagnosticCode` または `ParserDiagnosticCode` を確定する。
- 2026-04-29: `codegen_wasm.rs` / `codegen_llvm.rs` の backend diagnostic helper を code-first constructor へ移行した。backend は個別 call site で `Diagnostic` を直接組み立てず helper へ `BackendDiagnosticCode` を渡す構造なので、この boundary で code を生成時点に固定する。
- 2026-04-29: `parser.rs` の shared `error_with_code` / `push_error_with_code` と、再帰上限、no-progress recovery、raw block、intrinsic、tuple、match scrutinee の parser recovery boundary を code-first constructor へ移行した。この時点では layout/type expression/extern signature 系の直接 `.with_code(...)` が残っていたため、後続 D1 で同じ方針に揃える対象として切り出した。
- 2026-04-29: `parser.rs` に残っていた layout、type expression、identifier、mlstr、extern signature 診断を code-first constructor へ移行し、parser module 内の `.with_code(...)` を 0 件にした。
- 2026-04-29: `typecheck/effect_check.rs` の pure context / raw body effect 診断を code-first constructor へ移行し、未コード化だった raw body 多重有効化診断も `EffectDiagnosticCode::RawBodyMultipleActive` へ接続した。これにより effect checker boundary では `.with_code(...)` とコード無し raw-body effect error が残らない。
- 2026-04-29: `typecheck/diagnostics.rs` を追加し、typecheck 内部で `TypeDiagnosticCode` / `EffectDiagnosticCode` を code-first constructor へ渡す helper を共有化した。call application、selected callable、trait method call、indirect call、constructor、field accessor、field access、selected trait bound の boundary を移行し、コード無しだった capture arity invariant も `TypeDiagnosticCode::CallCaptureArityMismatch` へ接続した。
- 2026-04-29: `typecheck/match_check.rs` の enum / scalar match 診断を `type_error(...)` 経由へ移行した。scrutinee type、wildcard order、duplicate arm、non-exhaustive、payload binding、unsupported literal pattern、arm result mismatch は生成時点で `TypeDiagnosticCode` が確定する。
- 2026-04-29: `typecheck/control_apply.rs` の `if` / `while` arity、condition、body type 診断を `type_error(...)` 経由へ移行した。control special function boundary では診断生成時点で `TypeDiagnosticCode` が確定する。
- 2026-04-29: `typecheck/ascription.rs` の annotation mismatch 診断を `type_error(...)` 経由へ移行した。char literal の u8 range mismatch と一般の type annotation mismatch は生成時点で `TypeDiagnosticCode::AnnotationMismatch` が確定する。
- 2026-04-29: `typecheck/assignment_apply.rs` の assignment arity、deref、assignment mismatch、undefined set、immutable mutation、assignment target undefined 診断を `type_error(...)` 経由へ移行した。assignment boundary では生成時点で `TypeDiagnosticCode` が確定する。
- 2026-04-29: `typecheck/diagnostics.rs` に `resolve_error(...)` を追加し、`typecheck/driver_entry.rs` の entry missing / ambiguous 診断を code-first 化した。entry 解決境界では生成時点で `ResolveDiagnosticCode::EntryFunctionMissingOrAmbiguous` が確定する。
- 2026-04-29: `typecheck/function_check.rs` の function signature、parameter count、return type、pending trait bound 診断を `type_error(...)` 経由へ移行した。function checking boundary では生成時点で `TypeDiagnosticCode` が確定する。
- 2026-04-29: `typecheck/traits.rs` の trait bound arity / unknown trait bound 診断を `type_error(...)` 経由へ移行した。trait bound collection boundary では生成時点で `TypeDiagnosticCode` が確定する。
- 2026-04-29: `typecheck/overload_selection.rs` の explicit type arg mismatch、no matching overload、ambiguous overload 診断を `type_error(...)` 経由へ移行した。overload selection boundary では生成時点で `TypeDiagnosticCode` が確定する。
- 2026-04-29: `typecheck/call_reduction.rs` の call reduction defensive diagnostics を `type_error(...)` 経由へ移行した。call reduction 内部不変条件の破綻を報告する場合も生成時点で `TypeDiagnosticCode::CallReductionLimitExceeded` が確定する。

### Stage D2: Resource IR diagnostic の typed mapping 強化

目的: Stage 4/5 の Resource IR gate が、意味分類を失わず compiler diagnostic へ接続されるようにする。

作業:

- Resource IR diagnostic kind ごとに `DiagnosticCode` を返す関数を持たせる。
- owner / cell / borrow / raw effect / lowering を別 variant として保つ。
- raw identity escape と ordinary impure call を分ける。
- borrow lifetime escape と active borrow conflict を分ける。

進捗:

- 2026-04-29: `ResourceEffectBoundaryDiagnostic::RawAddressEscapeFromInternalAlloc` を `Effect(PureCallsImpure)` から分離し、`Resource(Raw(IdentityEscape))` / `resource.raw.identity_escape` として compiler diagnostic へ写像するようにした。raw identity escape の compile_fail regression は `effect.pure.calls_impure` ではなく `resource.raw.identity_escape` を期待する。
- `UnsafeMemoryInPureFunction` は現行 stdlib の raw-memory-backed API 移行と衝突するため、これまで通り shadow-only に残す。ordinary impure call や raw body I/O は `effect.pure.calls_impure` のまま維持する。

### Stage D3: CLI / JSON / web 表示の整理

目的: 人間向け表示と機械判定を同じ diagnostic value から安定生成する。

作業:

- CLI 表示は `error[resource.borrow.return_escape]: ...` の形式に統一する。
- web diagnostic object は `code` と `code_message` を出す。
- JSON diagnostic output は enum 由来の stable string code を primary key にする。

### Stage D4: test migration

目的: regression が粗い bucket ではなく、意味的な diagnostic code を固定する。

作業:

- doctest metadata は `diag_code` / `diag_codes` だけを受け付ける。
- active compile_fail tests は stable code を期待値にする。
- 新規 Resource IR / effect / owner / borrow regression は code を必須にする。

### Stage D5: self-host parity

目的: Rust core と NEPLg2 self-host compiler が同じ diagnostic contract を使う。

作業:

- `SelfhostDiagnostic` の code 命名を Rust registry と揃える。
- self-host reporter JSON と Rust CLI JSON を比較できる形にする。
- parser / resolver / checker の diagnostic code parity tests を追加する。

## 静的検査大規模修正との関係

この再設計は `static_check_complexity_reduction_plan.md` の Stage 4/5 を止めるものではない。むしろ、Stage 4/5 の Resource IR gate を、意味分類を失わない enum diagnostic に接続するための前提である。

方針:

- memory safety / type safety / effect safety の gate は弱めない。
- 既に authoritative 化した Resource IR gate は維持する。
- 新規 gate を追加する時は、対応する `DiagnosticCode` variant を同時に設計する。
- 後方互換用の数値 ID は追加しない。
- self-host 実装開始前に、Rust と self-host が同じ stable string code contract を使える状態にする。

## 完了条件

1. Rust core diagnostic の内部識別子が enum である。
2. 数値 ID field、旧 ID module、旧テストメタデータが active code path に残らない。
3. `DiagnosticCode::as_str()` / `message()` が wildcard なしの `match` で管理される。
4. Resource IR diagnostic が move / borrow / raw / lowering / effect の意味分類を失わない。
5. CLI / web / nodesrc test が enum 由来の stable string code を主識別子として扱う。
6. self-host `SelfhostDiagnostic` と Rust core diagnostic が同じ code contract で比較できる。

# NEPLg2 compiler diagnostic redesign plan

作成日: 2026-04-29
更新日: 2026-05-12

## 目的

NEPLg2 の Rust compiler diagnostic を、現在の型検査、effect 検査、Resource IR、self-host compiler に合う形へ再設計する。

旧来の数値 ID は履歴的な分類であり、Resource IR の owner obligation、initialized cell、borrow lifetime、raw identity boundary の意味を十分に表せない。後方互換は不要とし、診断の内部表現は数値や自由文字列ではなく、階層化された enum で管理する。

関連 issue:

- [ISS-20260429T040748194Z-RUST-COMPILER-DIAGNOSTICS-ARE-NOT-AL-1617747D](../../issues/items/ISS-20260429T040748194Z-RUST-COMPILER-DIAGNOSTICS-ARE-NOT-AL-1617747D.md): Rust compiler diagnostics が Resource IR / self-host model と揃っていない。
- [ISS-20260425T000000Z-RV-CORE-009-58589A3F](../../issues/items/ISS-20260425T000000Z-RV-CORE-009-58589A3F.md): Resource IR 上の move / borrow / drop 検査。
- [ISS-20260427T132406497Z-CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS-DC67DF04](../../issues/items/ISS-20260427T132406497Z-CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS-DC67DF04.md): raw memory effect / ownership boundary。
- [ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF](../../issues/items/ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF.md): `MemPtr` / owner token / initialized cell の分離。
- [ISS-20260429T100747827Z-WASM-INDIRECT-SIGNATURE-MISSING-DIAG-DBB86ABB](../../issues/items/ISS-20260429T100747827Z-WASM-INDIRECT-SIGNATURE-MISSING-DIAG-DBB86ABB.md): wasm indirect signature missing diagnostic の到達可能性。
- [ISS-20260507T094645212Z-DIAGNOSTIC-VALUE-STILL-PERMITS-CODEL-3654AAD9](../../issues/items/ISS-20260507T094645212Z-DIAGNOSTIC-VALUE-STILL-PERMITS-CODEL-3654AAD9.md): `Diagnostic` value が code-less 診断を型として許していた。
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
- `code: DiagnosticCode`
- `message`
- `primary`
- `secondary`

数値 ID field は持たない。後方互換用の field も置かない。

`code` は必須である。診断が外部へ出る時点だけでなく、compiler 内部の `Diagnostic` value 自体が必ず `DiagnosticCode` を持つ。code-less な `Diagnostic::error(...)` / `Diagnostic::warning(...)` constructor は公開しない。

将来の拡張で note/help/related label を追加する場合も、診断種別は `DiagnosticCode` から導く。補助的な説明文は表示値であり、識別子にしない。

### 外部表現

CLI、web、JSON、doctest は `DiagnosticCode::as_str()` の結果を使う。

例:

- `resolve.identifier.undefined`
- `effect.pure.calls_impure`
- `resource.borrow.return_escape`
- `resource.cell.moved`
- `resource.owner.leak`
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
| raw cell is moved/uninit/dropped | `Resource(Cell(...))` |
| storage owner obligation leak/double free | `Resource(Owner(...))` |
| raw address identity escapes pure surface | `Resource(Raw(IdentityEscape))` |
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
- 2026-04-29: Resource IR gate の lowering / raw cell / owner obligation / borrow conflict / raw identity escape 変換を code-first constructor へ移行した。動的な詳細文は現時点では message に残し、次の D1 follow-up で note/help へ段階的に分離する。
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
- 2026-04-29: `typecheck/block_check.rs` の block stack extra values と nested function trait-bound arity 診断を `type_error(...)` 経由へ移行した。block stack / nested bound boundary では生成時点で `TypeDiagnosticCode` が確定する。
- 2026-04-29: `typecheck/prefix_check.rs` の function value capture、`@` function reference、variable type argument、expected function value overload ambiguity 診断を `type_error(...)` 経由へ移行した。prefix expression の関数値選択境界では生成時点で `TypeDiagnosticCode` が確定する。
- 2026-04-29: `typecheck/prefix_check.rs` の trait method type args、unknown trait method、undefined identifier 診断を `type_error(...)` / `resolve_error(...)` 経由へ移行した。prefix expression の trait method / identifier resolution boundary では生成時点で `TypeDiagnosticCode` または `ResolveDiagnosticCode` が確定する。
- 2026-04-29: `typecheck/prefix_check.rs` の no-shadow violation / conflict、immutable mutation、undefined set target 診断を `resolve_error(...)` / `type_error(...)` 経由へ移行した。prefix expression の declaration / mutation boundary では生成時点で resolve/type の分類が確定する。
- 2026-04-29: `typecheck/prefix_check.rs` の impure intrinsic in pure context、unknown intrinsic、intrinsic arity/type mismatch、field/ref intrinsic、set_field mismatch 診断を `effect_error(...)` / `type_error(...)` 経由へ移行した。prefix intrinsic boundary では生成時点で effect/type の分類が確定する。
- 2026-04-29: `typecheck/prefix_check.rs` の pipe pending、source missing、target mismatch、target missing、unreduced left-hand side 診断を `type_error(...)` 経由へ移行した。pipe boundary では生成時点で `TypeDiagnosticCode::PipeInvalid` が確定する。
- 2026-04-29: `typecheck/prefix_check.rs` の integer literal parse failure と char literal i32-backed range violation に `LiteralIntInvalid` / `LiteralCharOutOfRange` を追加し、`type_error(...)` 経由へ移行した。同時に `parse_i32_literal` の overflow cast を範囲検査へ修正した。
- 2026-04-29: `typecheck/block_check.rs` の block-local no-shadow、nested generic function、nested function signature、raw block placement、block stack invariant 診断を `resolve_error(...)` / `type_error(...)` 経由へ移行した。code-less だった raw block placement と block stack invariant には専用 `TypeDiagnosticCode` を追加した。
- 2026-04-29: `typecheck/driver.rs` の extern directive と enum/struct declaration 境界を `type_error(...)` / `resolve_error(...)` 経由へ移行した。重複 enum/struct 宣言を無診断で skip していた conflict 分岐も修正し、`ItemNameConflict` を出すようにした。
- 2026-04-29: `typecheck/driver.rs` の trait declaration 境界に残っていた unknown capability と trait method type parameters 診断を `type_error(...)` 経由へ移行した。trait safety の capability/associated method shape は生成時点で `TypeDiagnosticCode` を確定する。
- 2026-04-29: `typecheck/driver.rs` の impl collection / impl validation 境界を `type_error(...)` 経由へ移行した。前段で拒否した impl を `rejected_impl_spans` として保持し、後段 validation が同じ impl を再診断しないようにしたため、inherent impl / unknown trait / trait type argument count mismatch の重複診断も解消された。impl method shape / signature / missing method 診断も生成時点で `TypeDiagnosticCode` を確定する。
- 2026-04-29: `typecheck/driver.rs` の function / alias hoist 境界を `type_error(...)` / `resolve_error(...)` 経由へ移行した。function signature、overload ambiguity、alias target、item conflict、no-shadow conflict / violation、function type parameter bound mismatch は生成時点で `TypeDiagnosticCode` または `ResolveDiagnosticCode` を確定する。
- 2026-04-29: `passes/move_check/raw_state.rs` の raw memory cell diagnostics を code-first helper へ移行した。non-Copy raw load / store / dealloc / realloc / byte write / bulk copy の violation は、後付け `.with_code(...)` ではなく生成時点で `ResourceDiagnosticCode::Cell(...)` を確定する。
- 2026-04-29: `passes/move_check/context_state.rs` の move / borrow diagnostics を code-first helper へ移行した。use/drop/possibly moved/loop escape と shared/unique borrow conflict/borrow escape は、生成時点で `ResourceMoveDiagnosticCode` または `ResourceBorrowDiagnosticCode` を確定する。
- 2026-04-29: `passes/move_check/visitor.rs` の deref borrow violation と loop body merge diagnostics を code-first helper へ移行した。non-Copy deref は `ResourceBorrowDiagnosticCode::MoveFromShared`、while body で発生する possibly moved state は `ResourceMoveDiagnosticCode::LoopPossiblyMoved` を生成時点で確定し、2 系統の while lowering 経路は同じ helper で診断する。
- 2026-04-29: `passes/codegen_precheck.rs` の wasm / llvm precheck diagnostics を code-first helper へ移行した。wasm backend precheck は `WasmDiagnosticCode`、LLVM precheck の型境界は `TypeDiagnosticCode` を生成時点で確定する。`IndirectSignatureMissing` は現行の signature set 生成では到達しにくいことが判明したため、別 issue で signature source 分離または variant 整理を追跡する。
- 2026-04-29: `target_precheck.rs` / `target_gate.rs` の raw body target、target directive、conditional gate diagnostics を code-first helper へ移行した。`#target` の fallback 走査では「有効 target が見つかったか」と「target directive を見たか」を分離し、unknown target を module directives と root items の両方で重複診断しないようにした。
- 2026-04-29: `resolve.rs` の open import ambiguity diagnostic を code-first constructor へ移行した。host-side module graph の visible map construction でも `ResolveDiagnosticCode::ImportAmbiguous` を生成時点で確定する。
- 2026-04-29: `wasm_shared.rs` の raw wasm line parse diagnostic を code-first helper へ移行した。raw wasm body precheck は `WasmDiagnosticCode::RawLineParseError` を生成時点で確定する。
- 2026-04-29: `compiler.rs` の LLVM target が wasm artifact pipeline に渡った場合の境界診断と、生成済み wasm validation failure を code-first constructor へ移行した。validation offset から推定する function body 位置は別 warning ではなく `backend.wasm.validation_failed` diagnostic の note として保持する。併せて typecheck の shadow warning / same-signature callable shadow warning を `ResolveDiagnosticCode` へ分類し、active compiler pass call site からコード無し `Diagnostic::error(...)` / `Diagnostic::warning(...)` / 後付け `.with_code(...)` を除去した。
- 2026-04-30: `nepl-language` の editor / LSP 解析境界に残っていた target directive diagnostic を code-first constructor へ移行し、`Diagnostic::with_code` API 自体を削除した。これ以降、後付け diagnostic code は Rust の型検査で使えない。code を持つ diagnostic は `error_with_code` / `warning_with_code` または category helper で生成時点に分類を確定する。
- 2026-04-30: `nepl-web` の wasm analysis 境界に残っていた target directive / loader failure diagnostic も code-first helper へ移行した。`nodesrc/test_diagnostic_code_first_boundary.js` を CI source policy に追加し、`nepl-core` / `nepl-language` / `nepl-lsp` / `nepl-web` に `.with_code(...)` や `Diagnostic::with_code` API が戻らないことを軽量に検査する。
- 2026-04-30: `nodesrc/test_diagnostic_code_first_boundary.js` の対象を active Rust source tree 全体へ拡張した。`nepl-core/src`、`nepl-language/src`、`nepl-lsp/src`、`nepl-web/src` を再帰走査し、`.with_code(...)` / `fn with_code` の再導入に加えて、compiler pass 側の code-less `Diagnostic::error(...)` / `Diagnostic::warning(...)` も拒否する。
- 2026-05-07: `ALL_DIAGNOSTIC_CODES` が手動 registry であるにもかかわらず、source policy は enum variant と registry の対応を検査していなかった。`TypeDiagnosticCode::CallCaptureArityMismatch` と `ResourceOwnerDiagnosticCode::Reserved` が registry から漏れていたため追加し、`nodesrc/test_diagnostic_code_first_boundary.js` で各 leaf diagnostic enum variant が `ALL_DIAGNOSTIC_CODES` に正確に 1 回登録されることを検査するようにした。
- 2026-05-07: `Diagnostic.code` を `Option<DiagnosticCode>` から必須 `DiagnosticCode` へ変更し、code-less な `Diagnostic::error(...)` / `Diagnostic::warning(...)` constructor を削除した。`DiagnosticSpec` / code-first constructor 以外では `Diagnostic` を構築できないため、診断 code の欠落は source policy ではなく Rust の型検査でも止まる。`nepl-language` の `EditorDiagnostic.code` も必須 stable string にし、web serialization は `code` / `code_message` を常に出す。
- 2026-05-07: CLI renderer に残っていた旧 `Option<DiagnosticCode>` 前提の `d.code.map(...)` を削除し、`d.code.as_str()` を直接表示するようにした。`nodesrc/test_diagnostic_code_first_boundary.js` は `nepl-cli/src` も監視対象に含め、`.code.map(...)` を拒否する。これにより、CLI / web / language / LSP の外部境界すべてで mandatory diagnostic code contract を source policy が監視する。
- 2026-05-07: `ISS-20260507T144641729Z-PUBLIC-MONOMORPHIZE-API-PANICS-ON-UN-4492668C` を解決した。公開 `monomorphize` API は unresolved trait call で panic せず、`MonomorphizeResult { module, unresolved_trait_calls }` を返す。compiler pipeline はこの構造化結果を `BackendDiagnosticCode::TraitCallUnresolved` へ写像するため、monomorphize 境界でも panic ではなく typed diagnostic contract が authority になる。`nodesrc/test_monomorphize_unresolved_api_policy.js` は panic-based unresolved trait handling と二重公開 API の再導入を拒否する。

### Stage D2: Resource IR diagnostic の typed mapping 強化

目的: Stage 4/5 の Resource IR gate が、意味分類を失わず compiler diagnostic へ接続されるようにする。

作業:

- Resource IR diagnostic kind ごとに `DiagnosticCode` を返す関数を持たせる。
- owner / cell / borrow / raw effect / lowering を別 variant として保つ。
- raw identity escape と ordinary impure call を分ける。
- borrow lifetime escape と active borrow conflict を分ける。

進捗:

- 2026-04-29: `ResourceEffectBoundaryDiagnostic::RawAddressEscapeFromInternalAlloc` を `Effect(PureCallsImpure)` から分離し、`Resource(Raw(IdentityEscape))` / `resource.raw.identity_escape` として compiler diagnostic へ写像するようにした。raw identity escape の compile_fail regression は `effect.pure.calls_impure` ではなく `resource.raw.identity_escape` を期待する。
- 2026-05-06: `ResourceEffectBoundaryDiagnostic::UnsafeMemoryInPureFunction` は Resource IR compiler gate から `Effect(PureCallsImpure)` / `effect.pure.calls_impure` へ error として写像する。Stage 6 まで残る移行中許可は、`stdlib/core/mem.nepl` など compiler-owned raw-memory-boundary capability を持つ source に限定する。
- 2026-04-30: `ResourceDiagnosticCode::Cell(...)` と `ResourceDiagnosticCode::Owner(...)` を追加し、Resource IR の `CellState` / `OwnerState` 診断を `resource.raw.ownership_violation` bucket へ潰さないようにした。raw-memory-backed 旧 move checker の non-Copy raw cell diagnostics も `resource.cell.*` へ移行したため、`resource.raw.*` は raw identity escape など raw provenance / unsafe boundary そのものへ限定する。
- 2026-05-12: `EffectOp::Unknown` と `ResourceEffectBoundaryDiagnostic::UnknownEffect` の理由を自由文字列から `UnknownEffectReason` enum へ移行した。unknown effect は `resource.lower.incomplete` へ写像される lowering incompleteness であり、原因分類は message 文字列ではなく typed enum / exhaustive match で保持する。
- 2026-05-12: cell / owner / borrow / effect boundary の Resource IR diagnostic 型に `diagnostic_code()` を持たせ、compiler gate 側の private helper ではなく diagnostic kind 自身が stable `DiagnosticCode` への写像を所有する形へ寄せた。compiler gate は dynamic message と allow 判定だけを担当し、分類は Resource IR diagnostic enum の exhaustive `match` で決まる。
- 2026-05-12: lowering coverage / drop elaboration plan / HIR bridge の diagnostic 型にも `diagnostic_code()` を追加し、`resource.lower.incomplete` の所有を Resource IR 側へ移した。併せて `ResourceCoverageDiagnostic::UnknownPlace` の操作分類を自由文字列から `ResourceCoveragePlaceOperation` enum へ移行し、coverage gate の unknown-place 原因も exhaustive `match` で管理する。

2026-04-30 追記:

[静的検査設計確認 2026-04-30](./static_check_design_verification_20260430.md) で、現在の Resource IR gate mapping を再確認した。

`ResourceCheckDiagnostic::CellUnavailable` と `ResourceOwnerDiagnostic::*` は、compiler diagnostic で `resource.cell.*` / `resource.owner.*` へ分離済みである。旧 D3100 相当の互換 bucket は残さず、原因分類を enum と stable string code の両方で保持する。

D2 の完了条件は次の通り。

- `ResourceDiagnosticCode` に cell / owner category を追加済みであること。
- raw memory 上で起きた violation でも、原因が initialized cell state なのか owner/free obligation なのかを stable code 上で失わない。
- `UnsafeMemoryInPureFunction` は `effect.pure.calls_impure` として hard error へ接続済みであり、raw-memory-boundary capability は stdlib migration の限定許可として扱う。
- `resource.raw.*` は raw identity escape、raw capability/provenance boundary、pointer provenance そのものの問題に限定する。
- `resource.cell.*` と `resource.owner.*` の `as_str()` / `message()` は wildcard なしの exhaustive match で管理する。
- self-host S3 以降の diagnostic category も、この Rust taxonomy と同じ構造で追加する。

### Stage D3: CLI / JSON / web 表示の整理

目的: 人間向け表示と機械判定を同じ diagnostic value から安定生成する。

作業:

- CLI 表示は `error[resource.borrow.return_escape]: ...` の形式に統一する。
- web diagnostic object は `code` と `code_message` を出す。
- JSON diagnostic output は enum 由来の stable string code を primary key にする。

進捗:

- 2026-04-30: `nepl-web` の wasm analysis object は `code` / `code_message` を出していたが、playground editor の `EditorUpdatePayload` が `code` を落としていたため、`web/src/editor-core/language-analysis.ts` の `EditorDiagnostic` に `code` / `codeMessage` を追加した。analysis snapshot から editor payload、差分 remap 後の payload まで stable code を保持する。`nodesrc/test_editor_diagnostic_code_contract.js` を CI source policy に追加し、web 側表示 contract が code を失わないことを固定する。
- 2026-05-13: `nepl-language` / `nepl-lsp` 側の editor diagnostic も `code_message` を保持するようにした。`EditorDiagnostic.code_message` は `DiagnosticCode::message()` から生成し、LSP diagnostic の `data.code_message` へ転送する。これにより web / language / LSP の外部境界で stable code と enum-derived canonical message の両方を保持する。`nodesrc/test_diagnostic_code_first_boundary.js` でこの contract を監視する。
- 2026-05-13: `nepl-language` / `nepl-web` の editor analysis target resolver で、unknown `#target` が `module.directives` と `module.root.items` の両方から二重診断される問題を修正した。fallback root scan は valid target の有無ではなく `saw_target_directive` の有無で判断する。`loader.target.unknown` は単一 source violation につき 1 件だけ出るため、Stage D3 の diagnostic event count と stable code regression が信頼できる。

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

進捗:

- 2026-04-29: self-host 側に `SelfhostDiagnosticCode` 階層 enum を導入し、`SelfhostDiagnostic.code` を自由文字列から typed code へ移行した。stable string は `selfhost_diag_code_name` の match 変換だけで生成し、reporter / JSON はその表示値を使う。lexer、parser、loader、module graph、module path、CLI driver/file_io の既存 diagnostic 生成箇所は typed constructor へ移行済み。parser / resolver / checker の Rust parity をさらに詰める作業は、各 stage 実装時の diagnostic variant 追加と parity fixture で継続する。

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

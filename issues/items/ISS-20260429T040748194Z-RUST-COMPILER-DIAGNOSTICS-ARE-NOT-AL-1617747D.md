---
id: ISS-20260429T040748194Z-RUST-COMPILER-DIAGNOSTICS-ARE-NOT-AL-1617747D
title: "Rust compiler diagnostics are not aligned with Resource IR and self-host model"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-29
updated: 2026-04-29
target: "nepl-core/src/diagnostic.rs, nepl-core/src/diagnostic_codes.rs, nepl-core/src/compiler.rs, nepl-cli/src/main.rs, nodesrc/tests.js, stdlib/neplg2/core/infra/diag.nepl, doc/neplg2/compiler_diagnostics_redesign_plan.md"
---

# ISS-20260429T040748194Z-RUST-COMPILER-DIAGNOSTICS-ARE-NOT-AL-1617747D: Rust compiler diagnostics are not aligned with Resource IR and self-host model

## 概要

Rust core diagnostics relied on hand-maintained numeric IDs and free-form Diagnostic construction. Resource IR errors were forced into coarse buckets, while the self-host compiler already uses stable string codes, labels, and notes. The two models were diverging.

## 対象

- `nepl-core/src/diagnostic.rs, nepl-core/src/diagnostic_codes.rs, nepl-core/src/compiler.rs, nepl-cli/src/main.rs, nodesrc/tests.js, stdlib/neplg2/core/infra/diag.nepl, doc/neplg2/compiler_diagnostics_redesign_plan.md`

## 根拠

- `nepl-core/src/diagnostic.rs` には自由文字列の `code` field と数値 ID field が混在しており、Rust の `match` 網羅性検査が効かない。
- `nepl-core/src/compiler.rs` の Resource IR gate は `ResourceCheckDiagnostic` / `ResourceOwnerDiagnostic` / `ResourceBorrowDiagnostic` / `ResourceEffectBoundaryDiagnostic` を粗い diagnostic bucket へ写像しており、Resource IR 側の意味分類が compiler diagnostic で粗くなる。
- `stdlib/neplg2/core/infra/diag.nepl` の self-host diagnostic は string `code`、message、primary label、note を中心にしており、Rust core の数値 ID 中心モデルと既に分岐している。
- `nodesrc/parser.js` / `nodesrc/tests.js` は数値 ID metadata を検査できるが、stable string diagnostic code を regression として固定する仕組みがない。

## 問題

Rust core diagnostics relied on hand-maintained numeric IDs and free-form Diagnostic construction. Resource IR errors were forced into coarse buckets, while the self-host compiler already uses stable string codes, labels, and notes. The two models were diverging.

## 影響

Static check gates can only be connected through ad-hoc mapping, regression tests pin accidental buckets, and self-host parity cannot compare Rust and NEPL diagnostics without another translation layer.

## 修正方針

Introduce a diagnostics redesign plan: hierarchical enum diagnostic codes inside Rust, stable string serialization only at the boundary, typed diagnostic kinds/builders per compiler stage, Resource IR diagnostic mapping through semantic categories, richer notes/help/related labels, and registry consistency checks.

詳細設計と実装段階は [NEPLg2 compiler diagnostic redesign plan](../../doc/neplg2/compiler_diagnostics_redesign_plan.md) に定義する。

この issue は局所的な数値 ID 追加を避けるための親 issue とする。後方互換は不要とし、新しい Resource IR / effect / borrow / owner diagnostic は `DiagnosticCode` enum を内部主識別子にする。

## 検証

Add registry consistency tests, CLI rendering tests, doctest support for diagnostic codes, and focused Resource IR diagnostic mapping regressions.

## 対応結果

2026-04-29 の Stage D0 実装で、Rust core の診断主識別子を `DiagnosticCode` と下位 enum へ移行した。`diagnostic_ids.rs`、数値 ID field、`with_id`、自由文字列を受け取る `with_code` は active code path から削除した。

CLI / web / nodesrc doctest は enum から `as_str()` で得た stable code を表示・検査する。旧メタデータ名は受け付けず、active compile_fail tests は `diag_code` / `diag_codes` へ移行する。

enum 化により、`#indent xx` が parser の generic token error として分類されていた不整合も見つかったため、`lexer.indent.argument_invalid` として発生段階に合う code へ分離した。

検証:

- `cargo check -p nepl-core --tests`
- `cargo test -p nepl-core diagnostic_codes -- --nocapture`
- `cargo test -p nepl-core --test codegen_diagnostics -- --nocapture`
- `trunk build`
- `node nodesrc/run_doctest.js -i tests/compiler/compile_fail_diag_location.n.md -n 1`
- `node nodesrc/tests.js -i tests/compiler/compile_fail_diag_location.n.md --no-tree -o tmp/agent1-diagnostic-code-location.json -j 1`
- `node nodesrc/test_llvm_runner_return_value.js`
- `node nodesrc/issues.js check`

この issue はまだ open のままとする。Stage D1 以降で builder、note/help、Resource IR typed mapping、self-host parity を続けて追跡する。

## 2026-04-29 Stage D2 raw identity escape code 追記

Resource IR の `RawAddressEscapeFromInternalAlloc` が ordinary な `effect.pure.calls_impure` に潰れていたため、`ResourceRawDiagnosticCode::IdentityEscape` を追加し、compiler gate では `resource.raw.identity_escape` として出すようにした。

これにより、pure context で impure I/O を呼ぶ診断と、internal allocation の raw address identity が public surface へ漏れる診断が enum 上でも doctest 上でも分離された。`UnsafeMemoryInPureFunction` は stdlib raw-memory-backed API 移行中のため、今回も explicit match で shadow-only に残している。

検証:

- `cargo test -p nepl-core diagnostic_codes -- --nocapture`
- `cargo test -p nepl-core compiler::tests::resource_effect_gate -- --nocapture`
- `cargo check -p nepl-core --tests`
- `trunk build`
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/agent1-resource-raw-identity-code-move-effect.json -j 1`

## 2026-04-29 Stage D0 nepl-language/LSP 追記

GitHub Actions run `25091893184` の bootstrap build で、`nepl-language/src/lib.rs` が削除済みの `nepl_core::diagnostic_ids::DiagnosticId`、`Diagnostic.id`、`Diagnostic::with_id` を参照していることが判明した。これは Stage D0 の active code path 移行漏れであり、後方互換 layer を戻さずに修正する。

`EditorDiagnostic` から数値 `id` field を削除し、`Diagnostic.code.map(DiagnosticCode::as_str)` で stable string code を渡すようにした。`nepl-lsp` の `textDocument/publishDiagnostics` も `code` に数値 ID ではなく同じ stable string code を出す。target directive 用の editor-side diagnostic は `LoaderDiagnosticCode` の enum variant で構築する。

検証:

- `cargo build`
- `cargo check -p nepl-language -p nepl-lsp`
- `cargo test -p nepl-language -- --nocapture`
- `cargo test -p nepl-lsp -- --nocapture`

## 2026-04-29 Stage D1 code-first builder 追記

`Diagnostic::error(...).with_code(...)` が active code path に広く残っているため、diagnostic code と message の組み合わせが call site ごとの後付けになる問題が残っていた。これは enum registry 導入後も、不適切な code/message の組み合わせを作れる設計上の弱さである。

`DiagnosticSpec` と `Diagnostic::error_code` / `error_with_code` / `warning_code` / `warning_with_code` を追加し、compiler-owned enum code を診断生成時点で渡す code-first constructor を導入した。`Diagnostic` には `notes` / `helps` を追加し、CLI / web / language / LSP の外部境界でも保持するようにした。

代表移行として、Resource IR gate の lowering coverage、raw cell ownership、owner obligation、borrow conflict、raw identity escape の compiler diagnostic 変換を `error_with_code` へ移行した。Resource IR の typed diagnostic が compiler diagnostic へ入る境界では、code が構築時に必ず決まる形になった。

この issue はまだ open のままとする。D1 follow-up で lexer/parser/typecheck の代表診断も builder へ移行し、D3 で JSON 表示の note/help contract をさらに固定する。

検証:

- `cargo test -p nepl-core diagnostic -- --nocapture`
- `cargo check -p nepl-core -p nepl-cli -p nepl-language -p nepl-lsp --tests`
- `trunk build`

## 2026-04-29 Stage D1 compiler boundary follow-up 追記

`compiler.rs` の Resource IR gate は code-first constructor へ移行済みだったが、同じ compiler pipeline 内の unresolved trait call、lowered entry 解決、target directive 診断には `Diagnostic::error(...).with_code(...)` が残っていた。

今回の対応で、これらを `Diagnostic::error_with_code(...)` へ移行した。`BackendDiagnosticCode::TraitCallUnresolved`、`ResolveDiagnosticCode::EntryFunctionMissingOrAmbiguous`、`LoaderDiagnosticCode::TargetMultipleDirective`、`LoaderDiagnosticCode::TargetUnknown` を import し、診断生成時点で enum code が確定する形にした。secondary label は diagnostic value の構築後に付与するが、primary code は builder で必ず渡す。

これにより `nepl-core/src/compiler.rs` から `.with_code(...)` は消え、Resource IR gate 以外の compiler boundary も Stage D1 の code-first 方針に揃った。lexer/parser/typecheck など他 module の残件はこの issue の後続 D1 として維持する。

検証:

- `rg -n "\\.with_code" nepl-core/src/compiler.rs`: no matches
- `cargo fmt --check -p nepl-core`: pass
- `cargo test -p nepl-core compiler::tests:: -- --nocapture`: pass
- `cargo check -p nepl-core --tests`: pass
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass

## 2026-04-29 Stage D1 driver extern / declaration boundary follow-up 追記

`typecheck/driver.rs` には extern directive と enum/struct declaration 境界で `Diagnostic::error(...).with_code(...)` が残っていた。また enum/struct の重複名検出は、先に `continue` する分岐があるため duplicate enum / duplicate struct を無診断で捨てる経路があった。

今回の対応で extern WASI target mismatch / extern signature mismatch / enum type parameter bounds / struct type parameter bounds を `type_error(...)` helper 経由へ移行し、enum/struct item name conflict を `resolve_error(...)` helper 経由へ移行した。重複 enum/struct は無診断 skip ではなく `ResolveDiagnosticCode::ItemNameConflict` を出す。source から到達できる declaration diagnostics は `tests/compiler/driver_declaration_diagnostics.n.md` でも固定する。

検証:

- `cargo fmt --check -p nepl-core`: pass
- `cargo test -p nepl-core diagnostic_codes -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 wasi_import_rejected_on_wasm_target -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 extern_signature_not_function_has_type_code -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 enum_type_param_bounds_have_type_code -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 struct_type_param_bounds_have_type_code -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 duplicate_enum_name_has_resolve_code -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 duplicate_struct_name_has_resolve_code -- --nocapture`: pass
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc/run_doctest.js -i tests/compiler/driver_declaration_diagnostics.n.md -n 1 --dist web/dist`: pass。`type.enum.type_param_bounds_unsupported` が出ることを確認した。
- `node nodesrc/run_doctest.js -i tests/compiler/driver_declaration_diagnostics.n.md -n 2 --dist web/dist`: pass。`type.struct.type_param_bounds_unsupported` が出ることを確認した。
- `node nodesrc/run_doctest.js -i tests/compiler/driver_declaration_diagnostics.n.md -n 3 --dist web/dist`: pass。`resolve.item.name_conflict` が出ることを確認した。
- `node nodesrc/run_doctest.js -i tests/compiler/driver_declaration_diagnostics.n.md -n 4 --dist web/dist`: pass。`resolve.item.name_conflict` が出ることを確認した。
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass

## 2026-04-29 Stage D1 driver trait declaration boundary follow-up 追記

`typecheck/driver.rs` には trait declaration の unknown capability と trait method type parameters unsupported 診断で `Diagnostic::error(...).with_code(...)` が残っていた。trait capability と trait method shape は trait safety と impl validation の前提なので、diagnostic code を後付けせず生成時点で `TypeDiagnosticCode` を確定する必要がある。

今回の対応で `TraitCapabilityUnknown` と `TraitMethodTypeParamsUnsupported` を `type_error(...)` helper 経由へ移行した。Rust regression と `tests/compiler/driver_trait_diagnostics.n.md` で unknown capability / trait method type params の stable code を固定する。

検証:

- `cargo fmt --check -p nepl-core`: pass
- `cargo test -p nepl-core diagnostic_codes -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 trait_unknown_capability_has_type_code -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 trait_method_type_params_have_type_code -- --nocapture`: pass
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc/run_doctest.js -i tests/compiler/driver_trait_diagnostics.n.md -n 1 --dist web/dist`: pass。`type.trait_capability.unknown` が出ることを確認した。
- `node nodesrc/run_doctest.js -i tests/compiler/driver_trait_diagnostics.n.md -n 2 --dist web/dist`: pass。`type.trait_method.type_params_unsupported` が出ることを確認した。
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass

## 2026-04-29 Stage D1 driver impl collection / validation boundary follow-up 追記

`typecheck/driver.rs` には impl collection と impl validation の境界に `Diagnostic::error(...).with_code(...)` が残っていた。さらに collection 前段で structural に拒否した inherent impl / unknown trait / trait type argument count mismatch を validation 後段でも再診断しており、stable code regression があっても同一原因の重複診断を見逃す状態だった。

今回の対応で impl collection の inherent impl、unknown trait、trait type argument count mismatch、generic target、copy target、duplicate impl、copy requires clone を `type_error(...)` helper 経由へ移行した。前段で拒否した impl は `rejected_impl_spans` に記録し、validation 後段は同じ impl を再診断しない。validation 後段の duplicate method、impl method type params、method not in trait、signature mismatch、missing trait method も `type_error(...)` helper 経由に揃えた。

回帰として、重複していた inherent impl / unknown trait / trait type argument count mismatch は Rust test で `TypeDiagnosticCode` の出現回数を 1 件に固定した。impl method validation 系の stable code は Rust test と `tests/compiler/driver_impl_diagnostics.n.md` の doctest で固定する。

検証:

- `cargo fmt --check -p nepl-core`: pass
- `cargo test -p nepl-core impl_ -- --nocapture`: pass
- `cargo test -p nepl-core diagnostic_codes -- --nocapture`: pass
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc/run_doctest.js -i tests/compiler/driver_impl_diagnostics.n.md -n 1 --dist web/dist`: pass。`type.impl.inherent_unsupported` が 1 件だけ出ることを確認した。
- `node nodesrc/run_doctest.js -i tests/compiler/driver_impl_diagnostics.n.md -n 2 --dist web/dist`: pass。`type.impl.duplicate_method` が出ることを確認した。
- `node nodesrc/run_doctest.js -i tests/compiler/driver_impl_diagnostics.n.md -n 3 --dist web/dist`: pass。`type.trait_method.type_params_unsupported` が出ることを確認した。
- `node nodesrc/run_doctest.js -i tests/compiler/driver_impl_diagnostics.n.md -n 4 --dist web/dist`: pass。`type.impl.method_not_in_trait` が出ることを確認した。
- `node nodesrc/run_doctest.js -i tests/compiler/driver_impl_diagnostics.n.md -n 5 --dist web/dist`: pass。`type.impl.method_signature_mismatch` が出ることを確認した。
- `node nodesrc/run_doctest.js -i tests/compiler/driver_impl_diagnostics.n.md -n 6 --dist web/dist`: pass。`type.impl.missing_trait_method` が出ることを確認した。
- `node nodesrc/run_doctest.js -i tests/compiler/driver_impl_diagnostics.n.md -n 7 --dist web/dist`: pass。`type.trait.unknown` が 1 件だけ出ることを確認した。
- `node nodesrc/run_doctest.js -i tests/compiler/driver_impl_diagnostics.n.md -n 8 --dist web/dist`: pass。`type.trait.type_params_unsupported` が 1 件だけ出ることを確認した。
- `node nodesrc/run_doctest.js -i tests/compiler/driver_impl_diagnostics.n.md -n 9 --dist web/dist`: pass。`type.impl.duplicate_for_trait_target` が出ることを確認した。

## 2026-04-29 Stage D1 driver function / alias hoist boundary follow-up 追記

`typecheck/driver.rs` の function hoist、function alias hoist、function body checking の overload 照合、function type parameter bound には `Diagnostic::error(...).with_code(...)` が残っていた。ここは name resolution と type checking が混在する境界なので、message 文字列ではなく `ResolveDiagnosticCode` と `TypeDiagnosticCode` を生成時点で選ぶ必要がある。

今回の対応で function item name conflict、overload ambiguity、no-shadow violation / conflict、function signature not function、alias item name conflict、alias target not found、function signature overload not found、function type parameter bound mismatch を `resolve_error(...)` / `type_error(...)` helper 経由へ移行した。`typecheck/driver.rs` から `Diagnostic::error(...).with_code(...)` と `DiagnosticCode` import は消えている。

回帰として、function alias target missing、function alias name conflict、function vs enum name conflict を Rust test で stable resolve code に固定した。既存の function signature、overload ambiguity、no-shadow、trait bound type argument count regression もこの境界の確認に使う。

検証:

- `cargo fmt --check -p nepl-core`: pass
- `rg -n "Diagnostic::error|\\.with_code|\\bDiagnosticCode\\b" nepl-core/src/typecheck/driver.rs`: no matches
- `cargo test -p nepl-core --test neplg2 function_alias -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 name_conflict_enum_fn_has_resolve_code -- --nocapture`: pass
- `cargo test -p nepl-core --test functions function_signature_not_function_has_type_code -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 overloads_ambiguous_return_type_is_error -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 trait_bound_type_arg_count_mismatch_has_type_code -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 let_noshadow_shadow_has_resolve_code -- --nocapture`: pass
- `cargo test -p nepl-core diagnostic_codes -- --nocapture`: pass
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc/run_doctest.js -i tests/compiler/functions.n.md -n 7 --dist web/dist`: pass。`resolve.alias.target_not_found` が出ることを確認した。
- `node nodesrc/run_doctest.js -i tests/compiler/shadowing.n.md -n 20 --dist web/dist`: pass。`resolve.shadow.no_shadow_violation` が出ることを確認した。
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass
- `cargo test -p nepl-core --test neplg2 -- --nocapture`: failed 89/97。失敗 8 件は `ISS-20260429T071452715Z-RESOURCE-IR-GATE-REGRESSES-NEPLG2-GE-E2DCC26B` の Resource IR raw ownership / owner obligation leak 既知件で、今回の diagnostic construction 変更とは別件。

## 2026-04-29 Stage D1 raw move check diagnostics follow-up 追記

`passes/move_check/raw_state.rs` には raw memory ownership violation を報告する箇所が 7 箇所あり、すべて `Diagnostic::error(...).with_code(...)` で `resource.raw.ownership_violation` を後付けしていた。raw memory ownership はメモリ安全に直結するため、message 文字列ではなく `ResourceRawDiagnosticCode::OwnershipViolation` を生成時点で確定する必要がある。

今回の対応で module-local `raw_ownership_error(...)` helper を追加し、non-Copy raw load / store / dealloc / realloc / byte write / bulk copy source / bulk copy destination の violation を `Diagnostic::error_with_code(...)` 経由へ移行した。Rust regression は message 部分ではなく `DiagnosticCode::Resource(ResourceDiagnosticCode::Raw(ResourceRawDiagnosticCode::OwnershipViolation))` を直接検査する。

検証:

- `cargo fmt --check -p nepl-core`: pass
- `cargo test -p nepl-core --test move_check move_raw_aggregate_non_copy_field_move_blocks_whole_load -- --nocapture`: pass
- `cargo test -p nepl-core diagnostic_codes -- --nocapture`: pass
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc/run_doctest.js -i tests/compiler/move_effect.n.md -n 17 --dist web/dist`: pass。`resource.raw.ownership_violation` が出ることを確認した。
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass

## 2026-04-29 Stage D1 move / borrow context diagnostics follow-up 追記

`passes/move_check/context_state.rs` には variable state と field move state を根拠にした move / borrow diagnostics が多数残っており、`Diagnostic::error(...).with_code(...)` で `ResourceMoveDiagnosticCode` / `ResourceBorrowDiagnosticCode` を後付けしていた。ここは move/borrow/lifetime 相当の静的検査の中核なので、message ではなく enum code を生成時点で確定する必要がある。

今回の対応で module-local `resource_move_error(...)` / `resource_borrow_error(...)` helper を追加し、use moved / use possibly moved / drop moved / drop possibly moved / return escape / move from shared / unique use / assign during borrow / drop during borrow / borrow moved / unique during shared / borrow during unique を `Diagnostic::error_with_code(...)` 経由へ移行した。Rust regression は代表的な move / borrow diagnostics で message ではなく `DiagnosticCode::Resource(...)` を直接検査する。

検証:

- `cargo fmt --check -p nepl-core`: pass
- `cargo test -p nepl-core --test move_check move_use_after_move -- --nocapture`: pass
- `cargo test -p nepl-core --test move_check move_in_branch -- --nocapture`: pass
- `cargo test -p nepl-core --test move_check move_live_reference_blocks_move -- --nocapture`: pass
- `cargo test -p nepl-core --test move_check -- --nocapture`: pass
- `cargo test -p nepl-core diagnostic_codes -- --nocapture`: pass
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `rg -n "Diagnostic::error|\\.with_code" nepl-core/src/passes/move_check/context_state.rs`: helper 内の `Diagnostic::error_with_code` 以外に no matches
- `node nodesrc/run_doctest.js -i tests/compiler/move_check.n.md -n 2 --dist web/dist`: pass。`resource.move.use_moved` が出ることを確認した。
- `node nodesrc/run_doctest.js -i tests/compiler/move_check.n.md -n 8 --dist web/dist`: pass。`resource.borrow.move_from_shared` が出ることを確認した。
- `node nodesrc/run_doctest.js -i tests/compiler/move_check.n.md -n 34 --dist web/dist`: pass。field projection 経由でも `resource.borrow.move_from_shared` が出ることを確認した。
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass

## 2026-04-29 Stage D1 parser direct diagnostics follow-up 追記

前回の parser recovery boundary 移行後、`parser.rs` には layout block、type expression、identifier、mlstr、extern signature など 42 箇所の直接 `.with_code(...)` が残っていた。

今回の対応で module-level `parser_error(...)` helper を追加し、戻り値で `Diagnostic` を返す layout / extern signature 系と、`self.diagnostics` に push する type expression / identifier / mlstr 系の直接構築を code-first constructor へ移行した。

これにより `nepl-core/src/parser.rs` から `.with_code(...)` は消えた。parser 内では `ParserDiagnosticCode` を生成時点で確定し、外部表示用 string code は `DiagnosticCode::as_str()` 境界だけに残る。

検証:

- `cargo fmt --check -p nepl-core`: pass
- `rg -n "\\.with_code" nepl-core/src/parser.rs`: no matches
- `cargo test -p nepl-core parser -- --nocapture`: pass
- `cargo check -p nepl-core --tests`: pass
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass

## 2026-04-29 Stage D1 entry resolve boundary follow-up 追記

`typecheck/driver_entry.rs` には `#entry` が missing / ambiguous の場合に `Diagnostic::error(...).with_code(...)` を直接組み立てる処理が残っていた。entry 解決は resolve diagnostic の境界であり、message string ではなく `ResolveDiagnosticCode::EntryFunctionMissingOrAmbiguous` を生成時点で確定する必要がある。

今回の対応で `typecheck/diagnostics.rs` に `resolve_error(...)` helper を追加し、entry resolve diagnostic を code-first にした。回帰テストとして missing entry が `DiagnosticCode::Resolve(EntryFunctionMissingOrAmbiguous)` を返す Rust test を追加した。

検証:

- `cargo fmt --check -p nepl-core`: pass
- `rg -n "\\.with_code|Diagnostic::error\\(" nepl-core/src/typecheck/driver_entry.rs`: no matches
- `cargo test -p nepl-core --test neplg2 missing_entry_function_has_resolve_code -- --nocapture`: pass
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/neplg2.n.md --no-tree -o tmp/agent1-entry-diagnostics-after-trunk.json -j 1`: failed 44/45。失敗は `tests/compiler/neplg2.n.md::doctest#33` の `List` RawMemoryLoadCell Uninit で、`ISS-20260429T071452715Z-RESOURCE-IR-GATE-REGRESSES-NEPLG2-GE-E2DCC26B` に記録済み。
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass

## 2026-04-29 Stage D1 parser recovery boundary follow-up 追記

`parser.rs` の shared `error_with_code` / `push_error_with_code` は名前上 code-aware だったが、内部実装は `Diagnostic::error(...).with_code(...)` で code を後付けしていた。また parser の再帰上限、no-progress recovery、raw block、intrinsic、tuple、match scrutinee の回復境界にも直接 `.with_code(...)` が残っていた。

今回の対応で shared helper を `Diagnostic::error_with_code(...)` へ移行し、上記の parser recovery boundary を `push_error_with_code(...)` 経由に統一した。これにより、parser の回復診断は生成時点で `ParserDiagnosticCode` が確定する。

parser には layout block、type expression、extern signature など 42 箇所の直接 `.with_code(...)` が残っている。これはこの issue の後続 D1 として残し、次の parser commit で module 内の残件を継続して削る。

検証:

- `cargo fmt --check -p nepl-core`: pass
- `cargo test -p nepl-core parser -- --nocapture`: pass
- `cargo check -p nepl-core --tests`: pass
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass

## 2026-04-29 Stage D1 backend boundary follow-up 追記

`codegen_wasm.rs` / `codegen_llvm.rs` の backend diagnostic helper が `Diagnostic::error(...).with_code(...)` を使っており、backend diagnostic は helper 境界で code を後付けする構造だった。

今回の対応で、WASM / LLVM の共通 diagnostic helper を `Diagnostic::error_with_code(...)` へ移行した。backend の各 call site は引き続き `DiagnosticCode::Backend(...)` を helper に渡すが、`Diagnostic` value は生成時点で enum code を持つ。

これにより backend boundary の active diagnostic construction から `.with_code(...)` は消えた。parser/typecheck などの大量の直接構築はこの issue の後続 D1 として維持する。

検証:

- `cargo fmt --check -p nepl-core`: pass
- `rg -n "\\.with_code" nepl-core/src/codegen_wasm.rs nepl-core/src/codegen_llvm.rs`: no matches
- `cargo test -p nepl-core --test codegen_diagnostics -- --nocapture`: pass
- `cargo check -p nepl-core --tests`: pass
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass

## 2026-04-29 Stage D1 lexer boundary follow-up 追記

`compiler.rs` の D1 移行後も、lexer は `Diagnostic::error(...).with_code(...)` を多数使っており、indent、raw block、directive、string/char literal、unknown token の各診断で code と message が後付け結合のままだった。

今回の対応で `nepl-core/src/lexer.rs` に `lexer_error` / `parser_error` helper を導入し、lexer 内の active diagnostic construction を `Diagnostic::error_with_code(...)` 経由に統一した。`#extern` の構文診断だけは lexer 入口で発生するが、意味分類は既存通り `ParserDiagnosticCode::ExternSignatureInvalid` に固定する。

これにより `lexer.rs` から `.with_code(...)` は消え、Stage D1 の「生成時点で enum code を確定する」方針が compiler boundary だけでなく lexer boundary にも適用された。parser/typecheck の残件はこの issue の後続 D1 として維持する。

検証:

- `cargo fmt --check -p nepl-core`: pass
- `rg -n "\\.with_code" nepl-core/src/lexer.rs`: no matches
- `cargo test -p nepl-core lexer -- --nocapture`: pass
- `cargo check -p nepl-core --tests`: pass
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass

## 2026-04-29 Stage D1 effect checker boundary follow-up 追記

`typecheck/effect_check.rs` には pure raw body / pure context の effect safety 診断で `Diagnostic::error(...).with_code(...)` が残っていた。また、同じ effect checker boundary の raw body 多重有効化診断は message だけで発行され、既に registry に存在する `EffectDiagnosticCode::RawBodyMultipleActive` に接続されていなかった。

今回の対応で module-local `effect_error(...)` helper を追加し、pure context が impure function / raw memory helper / raw memory instruction へ到達する診断を生成時点で `EffectDiagnosticCode::PureCallsImpure` に固定した。raw body が wasm / llvm の複数 active body を持つ場合も `EffectDiagnosticCode::RawBodyMultipleActive` を必ず持つ。

これにより effect checker boundary から `.with_code(...)` は消え、raw body の安全境界エラーがコード無し診断として外部へ漏れない。typecheck 全体には call application / match / prefix / driver などの D1 残件が残るため、この issue は open のまま継続する。

検証:

- `cargo fmt --check -p nepl-core`: pass
- `rg -n "\\.with_code" nepl-core/src/typecheck/effect_check.rs`: no matches
- `cargo test -p nepl-core --test effects -- --nocapture`: pass
- `cargo check -p nepl-core --tests`: pass
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass

## 2026-04-29 Stage D1 typecheck call boundary follow-up 追記

typecheck の call application 周辺には、`function_apply.rs` / `selected_call_apply.rs` / `trait_call_apply.rs` / `indirect_apply.rs` / `constructor_apply.rs` / `field_apply.rs` / `field_access.rs` / `trait_bound_apply.rs` に直接 `Diagnostic::error(...).with_code(...)` が残っていた。さらに `selected_call_apply.rs` の capture arity invariant は message だけの診断であり、enum registry による分類を持たなかった。

今回の対応で `typecheck/diagnostics.rs` を追加し、typecheck 内部から `TypeDiagnosticCode` / `EffectDiagnosticCode` を code-first constructor へ渡す `type_error(...)` / `effect_error(...)` helper を共有化した。call application、selected callable、trait method call、indirect call、constructor、field accessor、field access、selected trait bound の boundary は、この helper 経由で診断生成時点に enum code を確定する。

コード無しだった capture arity invariant は `TypeDiagnosticCode::CallCaptureArityMismatch` / `type.call.capture_arity_mismatch` として registry に追加した。これにより、call boundary では overload/argument/effect/trait-bound/field-access の分類が後付けにならず、internal invariant diagnostic も stable code を持つ。

検証:

- `cargo fmt --check -p nepl-core`: pass
- `rg -n "\\.with_code|Diagnostic::error\\(" nepl-core/src/typecheck/function_apply.rs nepl-core/src/typecheck/selected_call_apply.rs nepl-core/src/typecheck/trait_call_apply.rs nepl-core/src/typecheck/indirect_apply.rs nepl-core/src/typecheck/constructor_apply.rs nepl-core/src/typecheck/field_apply.rs nepl-core/src/typecheck/field_access.rs nepl-core/src/typecheck/trait_bound_apply.rs nepl-core/src/typecheck/effect_check.rs`: no matches
- `cargo test -p nepl-core diagnostic_codes -- --nocapture`: pass
- `cargo test -p nepl-core --test effects --test functions --test overload -- --nocapture`: `effects` pass, `overload` pass, `functions` は `print_i32__i32__unit__imp` の RawMemoryLoadCell ownership violation で別件 failure。`ISS-20260429T064519915Z-STDIO-PRINT-I32-TRIGGERS-RAWMEMORYLO-B90E5FA7` として起票した。
- `cargo check -p nepl-core --tests`: pass

## 2026-04-29 Stage D1 match checker boundary follow-up 追記

`typecheck/match_check.rs` には enum match / scalar match の診断が直接 `Diagnostic::error(...).with_code(...)` を組み立てる形で残っていた。match は型安全の中心的な boundary であり、scrutinee type、arm pattern、payload binding、網羅性、arm result type の分類が後付けになると、self-host parity や regression が粗い bucket に戻りやすい。

今回の対応で `match_check.rs` は既存の typecheck shared helper `type_error(...)` を使うようにした。`MatchScrutineeNotEnum`、`MatchWildcardNotLast`、`MatchDuplicateArm`、`MatchVariantUnknown`、`MatchPayloadBindingInvalid`、`MatchNonExhaustive`、`MatchPatternUnsupported`、`MatchArmsMismatch` は、診断生成時点で `TypeDiagnosticCode` として確定する。

これにより match checker boundary から直接 `.with_code(...)` と `Diagnostic::error(...)` は消えた。typecheck 全体では `prefix_check.rs` / `driver.rs` / `block_check.rs` などの D1 残件が残るため、この issue は open のまま継続する。

検証:

- `cargo fmt --check -p nepl-core`: pass
- `rg -n "\\.with_code|Diagnostic::error\\(" nepl-core/src/typecheck/match_check.rs`: no matches
- `cargo test -p nepl-core match -- --nocapture`: pass
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/match_literal_patterns.n.md -i tests/compiler/match_enum_wildcard_patterns.n.md --no-tree -o tmp/agent1-match-diagnostics-after-trunk.json -j 1`: pass
- `cargo check -p nepl-core --tests`: pass
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass

## 2026-04-29 Stage D1 control checker boundary follow-up 追記

`typecheck/control_apply.rs` には `if` / `while` の special function boundary で `Diagnostic::error(...).with_code(...)` が残っていた。control boundary は条件型、body 型、arity を確定する型安全の基本経路なので、診断 code を後付けにしない。

今回の対応で `if` arity / condition mismatch、`while` arity / condition mismatch / body mismatch を `type_error(...)` helper 経由へ移行した。これにより `control_apply.rs` から直接 `.with_code(...)` と `Diagnostic::error(...)` は消えた。

検証:

- `cargo fmt --check -p nepl-core`: pass
- `rg -n "\\.with_code|Diagnostic::error\\(" nepl-core/src/typecheck/control_apply.rs`: no matches
- `cargo test -p nepl-core --test if -- --nocapture`: pass
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/if.n.md --no-tree -o tmp/agent1-control-diagnostics-after-trunk.json -j 1`: pass
- `cargo check -p nepl-core --tests`: pass
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass

## 2026-04-29 Stage D1 ascription boundary follow-up 追記

`typecheck/ascription.rs` には char literal の `u8` range mismatch と、一般の type annotation mismatch で `Diagnostic::error(...).with_code(...)` が残っていた。type annotation は期待型を確定させる boundary なので、mismatch の分類を後付けにしない。

今回の対応で `AnnotationMismatch` を `type_error(...)` helper 経由へ移行した。これにより `ascription.rs` から直接 `.with_code(...)` と `Diagnostic::error(...)` は消えた。

検証:

- `cargo fmt --check -p nepl-core`: pass
- `rg -n "\\.with_code|Diagnostic::error\\(" nepl-core/src/typecheck/ascription.rs`: no matches
- `cargo test -p nepl-core --test char -- --nocapture`: pass
- `cargo test -p nepl-core --test typeannot -- --nocapture`: pass
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/plan.n.md -i tests/compiler/generics.n.md --no-tree -o tmp/agent1-ascription-diagnostics-after-trunk.json -j 1`: pass
- `cargo check -p nepl-core --tests`: pass
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass

## 2026-04-29 Stage D1 assignment boundary follow-up 追記

`typecheck/assignment_apply.rs` には `let` / `set` / deref boundary の診断で `Diagnostic::error(...).with_code(...)` が残っていた。assignment boundary は型不一致、未定義 set、不変変数 mutation、非 reference deref を分類するため、diagnostic code を後付けにしない。

今回の対応で `AssignmentArityMismatch`、`DerefInvalid`、`AssignmentMismatch`、`VariableUndefined`、`MutationImmutable`、`AssignmentUndefinedVariable` を `type_error(...)` helper 経由へ移行した。これにより `assignment_apply.rs` から直接 `.with_code(...)` と `Diagnostic::error(...)` は消えた。

検証:

- `cargo fmt --check -p nepl-core`: pass
- `rg -n "\\.with_code|Diagnostic::error\\(" nepl-core/src/typecheck/assignment_apply.rs`: no matches
- `cargo test -p nepl-core --test move_check -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 set_type_mismatch_is_error -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 -- --nocapture`: failed 52/60。失敗は assignment diagnostic ではなく Resource IR owner obligation leak / RawMemoryLoadCell Uninit で、`ISS-20260429T071452715Z-RESOURCE-IR-GATE-REGRESSES-NEPLG2-GE-E2DCC26B` として分離した。
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/move_check.n.md -i tests/compiler/move_effect.n.md -i tests/compiler/neplg2.n.md --no-tree -o tmp/agent1-assignment-diagnostics-after-trunk.json -j 1`: failed 206/207。失敗は `tests/compiler/neplg2.n.md::doctest#33` の `List` RawMemoryLoadCell Uninit で、`ISS-20260429T071452715Z-RESOURCE-IR-GATE-REGRESSES-NEPLG2-GE-E2DCC26B` に含めて追跡する。
- `node nodesrc/run_doctest.js -i tests/compiler/neplg2.n.md -n 3 --dist web/dist`: pass。`type.assignment.mismatch` が出ることを確認した。
- `cargo check -p nepl-core --tests`: pass
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass

## 2026-04-29 Stage D1 function checker boundary follow-up 追記

`typecheck/function_check.rs` には function signature、parameter count、return type、pending trait bound の診断で `Diagnostic::error(...).with_code(...)` が残っていた。function checking boundary は関数型安全と trait bound の基本境界なので、diagnostic code を後付けにしない。

今回の対応で `FunctionSignatureNotFunction`、`ArgumentArityMismatch`、`ReturnTypeMismatch`、`TraitBoundUnsatisfied` を `type_error(...)` helper 経由へ移行した。Rust 回帰テストでは各 function diagnostic が `DiagnosticCode::Type(...)` の enum code を返すことを確認する。

検証:

- `cargo fmt --check -p nepl-core`: pass
- `rg -n "\\.with_code|Diagnostic::error\\(" nepl-core/src/typecheck/function_check.rs`: no matches
- `cargo test -p nepl-core --test functions has_type_code -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 trait_bound_missing_impl_is_error -- --nocapture`: pass
- `cargo test -p nepl-core --test functions function_ -- --nocapture`: failed 15/16。失敗は stdio `print_i32` の RawMemoryLoadCell MaybeMoved で、`ISS-20260429T064519915Z-STDIO-PRINT-I32-TRIGGERS-RAWMEMORYLO-B90E5FA7` に記録済み。
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc/run_doctest.js -i tests/compiler/neplg2.n.md -n 43 --dist web/dist`: pass。`type.trait_bound.unsatisfied` が出ることを確認した。
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass

## 2026-04-29 Stage D1 trait bound collection follow-up 追記

`typecheck/traits.rs` には type parameter の trait bound 収集中に、trait bound arity mismatch と unknown trait bound の診断を `Diagnostic::error(...).with_code(...)` で後付けする処理が残っていた。trait bound は generic type safety の前提なので、収集段階で `TypeDiagnosticCode` を確定する。

今回の対応で `TraitTypeParamsUnsupported` と `TraitBoundUnknown` を `type_error(...)` helper 経由へ移行した。Rust 回帰テストでは unknown trait bound と trait bound type argument count mismatch の enum code を確認する。

検証:

- `cargo fmt --check -p nepl-core`: pass
- `rg -n "\\.with_code|Diagnostic::error\\(" nepl-core/src/typecheck/traits.rs`: no matches
- `cargo test -p nepl-core --test neplg2 trait_bound_ -- --nocapture`: pass
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc/run_doctest.js -i tests/compiler/neplg2.n.md -n 45 --dist web/dist`: pass。`type.trait_bound.unknown` が出ることを確認した。
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass

## 2026-04-29 Stage D1 overload selection boundary follow-up 追記

`typecheck/overload_selection.rs` には explicit type argument mismatch、no matching overload、ambiguous overload の診断で `Diagnostic::error(...).with_code(...)` が残っていた。overload selection は型推論後に候補を分類する境界なので、診断 code を後付けにしない。

今回の対応で `OverloadTypeArgsMismatch`、`OverloadNoMatch`、`OverloadAmbiguous` を `type_error(...)` helper 経由へ移行した。Rust 回帰テストでは no match / type args mismatch / ambiguous overload がそれぞれ `DiagnosticCode::Type(...)` の enum code を返すことを確認する。

検証:

- `cargo fmt --check -p nepl-core`: pass
- `rg -n "\\.with_code|Diagnostic::error\\(" nepl-core/src/typecheck/overload_selection.rs`: no matches
- `cargo test -p nepl-core --test neplg2 overload -- --nocapture`: pass
- `cargo test -p nepl-core --test overload -- --nocapture`: pass
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc/run_doctest.js -i tests/compiler/neplg2.n.md -n 39 --dist web/dist`: pass。`type.overload.ambiguous` が出ることを確認した。
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass

## 2026-04-29 Stage D1 call reduction boundary follow-up 追記

`typecheck/call_reduction.rs` には call reduction の内部防衛診断で `Diagnostic::error(...).with_code(...)` が残っていた。これは通常の user-facing overload mismatch ではなく、call reduction が非関数を reduction 対象にした場合や進捗不能 loop を検出した場合の invariant diagnostic だが、同じく diagnostic code を後付けにしない。

今回の対応で `CallReductionLimitExceeded` を `type_error(...)` helper 経由へ移行した。直接発火させる fixture は内部 invariant 破綻を作る必要があるため追加せず、call reduction を通る overload / grouped call / function diagnostic regression で挙動維持を確認した。

検証:

- `cargo fmt --check -p nepl-core`: pass
- `rg -n "\\.with_code|Diagnostic::error\\(" nepl-core/src/typecheck/call_reduction.rs`: no matches
- `cargo test -p nepl-core --test neplg2 overload -- --nocapture`: pass
- `cargo test -p nepl-core --test overload -- --nocapture`: pass
- `cargo test -p nepl-core --test functions has_type_code -- --nocapture`: pass
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc/run_doctest.js -i tests/compiler/neplg2.n.md -n 39 --dist web/dist`: pass。call reduction を含む overload diagnostic path が維持されることを確認した。
- `git diff --check`: pass

## 2026-04-29 Stage D1 block stack / nested bound follow-up 追記

`typecheck/block_check.rs` には block stack extra values と nested function trait-bound arity mismatch の型診断で `Diagnostic::error(...).with_code(...)` が残っていた。block stack と nested function bound collection は型安全の境界なので、診断 code を後付けにしない。

今回の対応で `StackExtraValues` と `TraitTypeParamsUnsupported` を `type_error(...)` helper 経由へ移行した。`block_check.rs` には shadow/resolve と raw block placement の直接診断がまだ残るため、これは別 boundary として続ける。

検証:

- `cargo fmt --check -p nepl-core`: pass
- `cargo test -p nepl-core --test block_if_semantics -- --nocapture`: pass
- `cargo test -p nepl-core --test block_single_line -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 trait_bound_type_arg_count_mismatch_has_type_code -- --nocapture`: pass
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc/run_doctest.js -i tests/compiler/block_semicolon_return.n.md -n 5 --dist web/dist`: pass。`type.stack.extra_values` が出ることを確認した。
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass

## 2026-04-29 Stage D1 prefix function value/reference boundary follow-up 追記

`typecheck/prefix_check.rs` には function value capture、`@` function reference、variable type argument、expected function value overload ambiguity の診断で `Diagnostic::error(...).with_code(...)` が残っていた。prefix expression の関数値選択は、関数を値として扱えるか、変数を callable として参照していないか、型引数を渡せる対象かを決める型安全境界なので、diagnostic code を後付けにしない。

今回の対応で `FunctionValueCapturingUnsupported`、`FunctionRefRequiresCallable`、`VariableTypeArgsNotAllowed`、`OverloadAmbiguous` を `type_error(...)` helper 経由へ移行した。Rust 回帰テストでは enum code を直接確認し、Markdown doctest では stable string `diag_code` を固定した。`prefix_check.rs` には trait method、resolve、shadow、intrinsic、pipe などの直接診断がまだ残るため、別 boundary として継続する。

検証:

- `cargo fmt --check -p nepl-core`: pass
- `cargo test -p nepl-core --test functions has_type_code -- --nocapture`: pass
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc/run_doctest.js -i tests/compiler/functions.n.md -n 18 --dist web/dist`: pass。`type.function_value.capturing_unsupported` が出ることを確認した。
- `node nodesrc/run_doctest.js -i tests/compiler/functions.n.md -n 19 --dist web/dist`: pass。`type.function_ref.requires_callable` が出ることを確認した。
- `node nodesrc/run_doctest.js -i tests/compiler/functions.n.md -n 20 --dist web/dist`: pass。`type.variable.type_args_not_allowed` が出ることを確認した。
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass

## 2026-04-29 Stage D1 prefix trait method / identifier resolve boundary follow-up 追記

`typecheck/prefix_check.rs` には trait method type args、unknown trait method、undefined identifier の診断で `Diagnostic::error(...).with_code(...)` が残っていた。prefix expression の名前解決境界は、`Trait::method` の型診断と通常 identifier の resolve 診断を分けるため、diagnostic code を後付けにしない。

今回の対応で `TraitMethodTypeArgsUnsupported` と `TraitMethodNotFound` を `type_error(...)` 経由へ、`IdentifierUndefined` を `resolve_error(...)` 経由へ移行した。Rust 回帰テストでは type / resolve の enum code を直接確認する helper を整理し、既存 doctest では stable string `diag_code` を確認する。

検証:

- `cargo fmt --check -p nepl-core`: pass
- `cargo test -p nepl-core --test neplg2 missing_entry_function_has_resolve_code -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 undefined_identifier_has_resolve_code -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 trait_method_type_args_unsupported_has_type_code -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 trait_method_not_found_has_type_code -- --nocapture`: pass
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc/run_doctest.js -i tests/compiler/overload.n.md -n 39 --dist web/dist`: pass。`type.trait_method.type_args_unsupported` が出ることを確認した。
- `node nodesrc/run_doctest.js -i tests/compiler/overload.n.md -n 40 --dist web/dist`: pass。`type.trait_method.not_found` が出ることを確認した。
- `node nodesrc/run_doctest.js -i tests/compiler/compile_fail_diag_location.n.md -n 1 --dist web/dist`: pass。`resolve.identifier.undefined` が出ることを確認した。

## 2026-04-29 Stage D1 prefix declaration / mutation boundary follow-up 追記

`typecheck/prefix_check.rs` には no-shadow violation / conflict、immutable mutation、undefined set target の診断で `Diagnostic::error(...).with_code(...)` が残っていた。declaration / mutation boundary では resolve error と type error の分類が混ざりやすいため、helper を分けて code を生成時点に確定する。

今回の対応で `ShadowNoShadowViolation` と `ShadowNoShadowConflict` を `resolve_error(...)` 経由へ、`MutationImmutable` と `VariableUndefined` を `type_error(...)` 経由へ移行した。Rust 回帰テストでは immutable set、undefined set、no-shadow violation の enum code を直接確認する。

検証:

- `cargo fmt --check -p nepl-core`: pass
- `cargo test -p nepl-core --test neplg2 set_immutable_variable_has_type_code -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 set_undefined_variable_has_type_code -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 let_noshadow_shadow_has_resolve_code -- --nocapture`: pass
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc/run_doctest.js -i tests/compiler/shadowing.n.md -n 18 --dist web/dist`: pass。`resolve.shadow.no_shadow_violation` が出ることを確認した。
- `node nodesrc/run_doctest.js -i tests/compiler/move_effect.n.md -n 92 --dist web/dist`: pass。`type.variable.undefined` が出ることを確認した。

## 2026-04-29 Stage D1 prefix intrinsic / effect boundary follow-up 追記

`typecheck/prefix_check.rs` には impure intrinsic in pure context、unknown intrinsic、intrinsic arity/type mismatch、field/ref intrinsic、set_field mismatch の診断で `Diagnostic::error(...).with_code(...)` が残っていた。intrinsic boundary は effect safety と type safety の両方に関わるため、`EffectDiagnosticCode` と `TypeDiagnosticCode` を helper で明確に分ける。

今回の対応で `PureCallsImpure` を `effect_error(...)` 経由へ、`IntrinsicTypeArgArityMismatch`、`IntrinsicUnknown`、`FieldInvalidAccess`、`AssignmentMismatch`、`IntrinsicArgArityMismatch`、`IntrinsicArgTypeMismatch` を `type_error(...)` 経由へ移行した。Rust 回帰テストでは unknown intrinsic、intrinsic argument mismatch、callsite_span type arg arity を enum code で確認し、raw load intrinsic の effect code path も確認した。

検証:

- `cargo fmt --check -p nepl-core`: pass
- `cargo test -p nepl-core --test neplg2 unknown_intrinsic_has_type_code -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 intrinsic_arg_type_mismatch_has_type_code -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 callsite_span_type_arg_arity_has_type_code -- --nocapture`: pass
- `cargo test -p nepl-core --test effects pure_raw_load_intrinsic_is_rejected_outside_core_mem -- --nocapture`: pass
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc/run_doctest.js -i tests/compiler/codegen_diagnostics.n.md -n 1 --dist web/dist`: pass。`type.intrinsic.unknown` が出ることを確認した。
- `node nodesrc/run_doctest.js -i tests/compiler/codegen_diagnostics.n.md -n 2 --dist web/dist`: pass。`type.field.invalid_access` が出ることを確認した。
- `node nodesrc/run_doctest.js -i tests/compiler/intrinsic.n.md -n 7 --dist web/dist`: pass。`type.intrinsic.arg_type_mismatch` が出ることを確認した。
- `node nodesrc/run_doctest.js -i tests/compiler/move_effect.n.md -n 15 --dist web/dist`: pass。`effect.pure.calls_impure` が出ることを確認した。

## 2026-04-29 Stage D1 prefix pipe boundary follow-up 追記

`typecheck/prefix_check.rs` には pipe pending、source missing、target mismatch、target missing、unreduced left-hand side の診断で `Diagnostic::error(...).with_code(...)` が残っていた。pipe は prefix expression の stack reduction と call injection の境界なので、`PipeInvalid` を後付けせず生成時点で確定する。

今回の対応で pipe boundary の全 `PipeInvalid` 診断を `type_error(...)` 経由へ移行した。既存 Rust tests の `pipe_requires_callable_target` と `pipe_target_missing_after_annotation_is_error` は error-only から enum code 直接確認へ強化した。

検証:

- `cargo fmt --check -p nepl-core`: pass
- `cargo test -p nepl-core --test neplg2 pipe_requires_callable_target -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 pipe_target_missing_after_annotation_is_error -- --nocapture`: pass
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc/run_doctest.js -i tests/compiler/neplg2.n.md -n 22 --dist web/dist`: pass。`type.pipe.invalid` が出ることを確認した。
- `node nodesrc/run_doctest.js -i tests/compiler/neplg2.n.md -n 25 --dist web/dist`: pass。`type.pipe.invalid` が出ることを確認した。

## 2026-04-29 Stage D1 prefix literal boundary follow-up 追記

`typecheck/prefix_check.rs` には invalid integer literal と char literal range violation の診断が code なしの `Diagnostic::error(...)` として残っていた。また、`parse_i32_literal` は `i128` から `i32` へ `as` cast しており、巨大な整数 literal が wrap して通る根本バグがあった。

今回の対応で `TypeDiagnosticCode::LiteralIntInvalid` と `TypeDiagnosticCode::LiteralCharOutOfRange` を追加し、literal boundary を `type_error(...)` 経由へ移行した。`parse_i32_literal` は `i32::try_from(...)` による範囲検査へ変更し、overflow literal をエラーにする。Rust regression では source 経由の巨大 integer literal と、AST 直接構築の char 範囲外を enum code で確認する。

検証:

- `cargo fmt --check -p nepl-core`: pass
- `cargo test -p nepl-core diagnostic_codes -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 invalid_integer_literal_has_type_code -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 invalid_ast_char_literal_has_type_code -- --nocapture`: pass
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc/run_doctest.js -i tests/compiler/literal_diagnostics.n.md -n 1 --dist web/dist`: pass。`type.literal.int_invalid` が出ることを確認した。

## 2026-04-29 Stage D1 block scope / raw placement boundary follow-up 追記

`typecheck/block_check.rs` には block-local no-shadow、nested generic function unsupported、nested function signature mismatch、raw block placement、block stack invariant の診断で `Diagnostic::error(...)` / `.with_code(...)` が残っていた。特に raw block placement と block stack invariant は code-less 診断であり、raw backend block を関数 body 以外に置いた場合や block stack の不整合を stable enum code で監視できなかった。

今回の対応で no-shadow 系を `resolve_error(...)`、nested function / raw block / block stack 系を `type_error(...)` helper 経由へ移行した。`TypeDiagnosticCode::NestedGenericFunctionUnsupported`、`RawBlockInvalidPlacement`、`BlockStackInconsistent` を追加し、既存の block stack extra values は `StackExtraValues` に接続した。Rust regression では nested generic function と nested raw block が enum code を返すことを固定する。

検証:

- `cargo fmt --check -p nepl-core`: pass
- `cargo test -p nepl-core diagnostic_codes -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 nested_generic_function_has_type_code -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 nested_raw_block_has_type_code -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 let_noshadow_shadow_has_resolve_code -- --nocapture`: pass
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc/run_doctest.js -i tests/compiler/block_diagnostics.n.md -n 1 --dist web/dist`: pass。`type.nested_function.generic_unsupported` が出ることを確認した。
- `node nodesrc/run_doctest.js -i tests/compiler/block_diagnostics.n.md -n 2 --dist web/dist`: pass。`type.raw_block.invalid_placement` が出ることを確認した。
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass

## 2026-04-29 Stage D1 move visitor diagnostics follow-up 追記

`passes/move_check/visitor.rs` には non-Copy deref の borrow violation と while body merge の loop possibly moved diagnostics が `Diagnostic::error(...).with_code(...)` として残っていた。ここは実際の HIR traversal 中に move / borrow state を観測する境界なので、message 文字列に後から code を貼るのではなく、診断生成時点で `ResourceBorrowDiagnosticCode` / `ResourceMoveDiagnosticCode` を確定する必要がある。

今回の対応で module-local `resource_move_error(...)` / `resource_borrow_error(...)` helper を追加した。non-Copy deref は `ResourceBorrowDiagnosticCode::MoveFromShared`、while body merge は `ResourceMoveDiagnosticCode::LoopPossiblyMoved` を `Diagnostic::error_with_code(...)` 経由で生成する。さらに builtin `while` call 経路と `HirExprKind::While` 経路の重複判定を `report_loop_possibly_moved(...)` へまとめ、同じ state merge 条件から同じ enum code が出るようにした。

Rust regression は `move_in_loop` で message 部分ではなく `DiagnosticCode::Resource(ResourceDiagnosticCode::Move(ResourceMoveDiagnosticCode::LoopPossiblyMoved))` を直接検査するように強化した。web/doctest 側では loop と non-Copy deref の stable string code を確認する。

検証:

- `cargo fmt --check -p nepl-core`: pass
- `rg -n "Diagnostic::error\(|\.with_code\(" nepl-core/src/passes/move_check/visitor.rs`: no matches
- `cargo test -p nepl-core --test move_check -- --nocapture`: pass
- `cargo test -p nepl-core diagnostic_codes -- --nocapture`: pass
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc/run_doctest.js -i tests/compiler/move_check.n.md -n 4 --dist web/dist`: pass。`resource.move.loop_possibly_moved` が出ることを確認した。
- `node nodesrc/run_doctest.js -i tests/compiler/move_check.n.md -n 33 --dist web/dist`: pass。`resource.borrow.move_from_shared` が出ることを確認した。
- `node nodesrc/run_doctest.js -i tests/compiler/move_check.n.md -n 49 --dist web/dist`: pass。`resource.move.loop_possibly_moved` が出ることを確認した。
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass

## 2026-04-29 Stage D1 codegen precheck diagnostics follow-up 追記

`passes/codegen_precheck.rs` には wasm backend precheck と LLVM precheck の診断で `Diagnostic::error(...).with_code(...)` が残っていた。backend precheck は target-specific な `WasmDiagnosticCode` と type boundary の `TypeDiagnosticCode` が混在するため、message 文字列に後から code を付けるのではなく、helper を分けて生成時点で code を確定する必要がある。

今回の対応で module-local `wasm_error(...)` / `type_error(...)` helper を追加した。wasm extern/function signature、return value missing、LLVM IR body unsupported、indirect call signature、unknown wasm intrinsic は `WasmDiagnosticCode` を、LLVM precheck の return mismatch と unknown intrinsic は `TypeDiagnosticCode` を `Diagnostic::error_with_code(...)` 経由で生成する。

回帰として `codegen_diagnostics.rs` に precheck を直接叩く enum code regression を追加した。source 経由では typecheck に先に捕まる診断があるため、precheck 境界そのものの code を固定する単体テストとしている。

調査中に、現行 `collect_wasm_signature_set` が supported indirect call signature を前段で set に追加するため `WasmDiagnosticCode::IndirectSignatureMissing` が公開 precheck 経路から到達しにくいことを確認した。これは今回の code-first 移行とは別の backend precheck 設計問題として、`ISS-20260429T100747827Z-WASM-INDIRECT-SIGNATURE-MISSING-DIAG-DBB86ABB` で追跡する。

検証:

- `cargo fmt --check -p nepl-core`: pass
- `rg -n "Diagnostic::error\(|\.with_code\(" nepl-core/src/passes/codegen_precheck.rs`: no matches
- `cargo test -p nepl-core --test codegen_diagnostics -- --nocapture`: pass
- `cargo test -p nepl-core diagnostic_codes -- --nocapture`: pass
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc/run_doctest.js -i tests/compiler/codegen_diagnostics.n.md -n 3 --dist web/dist`: pass。`backend.wasm.raw_line_parse_error` が出ることを確認した。
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass

## 2026-04-29 Stage D1 target precheck / gate diagnostics follow-up 追記

`target_precheck.rs` / `target_gate.rs` には raw body target、target directive、conditional gate diagnostics の `Diagnostic::error(...).with_code(...)` が残っていた。target boundary は loader / effect diagnostics が混ざるため、helper を分けて生成時点で `LoaderDiagnosticCode` または `EffectDiagnosticCode` を確定する必要がある。

今回の対応で `target_precheck.rs` に `effect_error(...)` / `loader_error(...)` helper を追加し、raw body multiple active / target mismatch と target directive diagnostics を `Diagnostic::error_with_code(...)` 経由へ移行した。`target_gate.rs` の invalid conditional gate diagnostic も code-first constructor へ移行した。

検証中に `#target wasi2` が `loader.target.unknown` を 2 件出すことを確認した。原因は `resolve_target` と `precheck_module_target_directives` が「有効 target が見つかったか」を fallback 走査の条件にしており、unknown target directive を見ても fallback root item 走査を再実行していたためだった。`found valid target` と `saw target directive` を分離し、unknown target は 1 件だけ診断するようにした。

回帰として duplicate / unknown target directive の Rust test を loader enum code 直接検査へ強化し、unknown target の code count を 1 件に固定した。web/doctest でも target directive と raw body target/effect diagnostics の stable string code を確認する。

検証:

- `cargo fmt --check -p nepl-core`: pass
- `rg -n "Diagnostic::error\(|\.with_code\(" nepl-core/src/target_precheck.rs nepl-core/src/target_gate.rs`: no matches
- `cargo test -p nepl-core --test neplg2 invalid_iftarget -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 invalid_ifprofile -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 target_directive -- --nocapture`: pass
- `cargo test -p nepl-core diagnostic_codes -- --nocapture`: pass
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc/run_doctest.js -i tests/compiler/raw_body_precheck.n.md -n 1 --dist web/dist`: pass。`effect.raw_body.target_mismatch` が出ることを確認した。
- `node nodesrc/run_doctest.js -i tests/compiler/raw_body_precheck.n.md -n 3 --dist web/dist`: pass。`effect.raw_body.multiple_active` が出ることを確認した。
- `node nodesrc/run_doctest.js -i tests/compiler/neplg2.n.md -n 36 --dist web/dist`: pass。`loader.target.multiple_directive` が出ることを確認した。
- `node nodesrc/run_doctest.js -i tests/compiler/neplg2.n.md -n 37 --dist web/dist`: pass。`loader.target.unknown` が 1 件だけ出ることを確認した。
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass

## 2026-04-29 Stage D1 resolve diagnostic follow-up 追記

`resolve.rs` の host-side `build_visible_map` には open import ambiguity diagnostic の `Diagnostic::error(...).with_code(...)` が残っていた。module graph / visible map 構築は resolver の責務なので、message 文字列に後から code を貼るのではなく、`ResolveDiagnosticCode::ImportAmbiguous` を生成時点で確定する必要がある。

今回の対応で open import ambiguity diagnostic を `Diagnostic::error_with_code(...)` へ移行した。`build_visible_map_reports_ambiguous_open` regression は message 部分ではなく `DiagnosticCode::Resolve(ResolveDiagnosticCode::ImportAmbiguous)` を直接検査するように強化した。

検証:

- `cargo fmt --check -p nepl-core`: pass
- `rg -n "Diagnostic::error\(|\.with_code\(" nepl-core/src/resolve.rs`: no matches
- `cargo test -p nepl-core --test resolve build_visible_map_reports_ambiguous_open -- --nocapture`: pass
- `cargo test -p nepl-core diagnostic_codes -- --nocapture`: pass
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass

## 2026-04-29 Stage D1 wasm shared diagnostic follow-up 追記

`wasm_shared.rs` の raw wasm body precheck には raw line parse error の `Diagnostic::error(...).with_code(...)` が残っていた。raw wasm line parse error は backend wasm precheck の責務なので、`WasmDiagnosticCode::RawLineParseError` を生成時点で確定する必要がある。

今回の対応で module-local `wasm_error(...)` helper を追加し、raw wasm line parse error を `Diagnostic::error_with_code(...)` 経由へ移行した。web/doctest 側の `backend.wasm.raw_line_parse_error` regression で外部 stable string code も確認する。

検証:

- `cargo fmt --check -p nepl-core`: pass
- `rg -n "Diagnostic::error\(|\.with_code\(" nepl-core/src/wasm_shared.rs`: no matches
- `cargo test -p nepl-core diagnostic_codes -- --nocapture`: pass
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc/run_doctest.js -i tests/compiler/codegen_diagnostics.n.md -n 3 --dist web/dist`: pass。`backend.wasm.raw_line_parse_error` が出ることを確認した。
- `node nodesrc/run_doctest.js -i tests/compiler/raw_body_precheck.n.md -n 4 --dist web/dist`: pass。`backend.wasm.raw_line_parse_error` が出ることを確認した。
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass

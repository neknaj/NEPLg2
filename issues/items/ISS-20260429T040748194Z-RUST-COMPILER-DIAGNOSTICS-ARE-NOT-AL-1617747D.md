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

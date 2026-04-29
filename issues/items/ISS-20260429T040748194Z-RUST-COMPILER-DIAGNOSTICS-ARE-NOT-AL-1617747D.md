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

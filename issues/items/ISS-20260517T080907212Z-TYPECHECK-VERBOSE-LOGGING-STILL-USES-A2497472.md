---
id: ISS-20260517T080907212Z-TYPECHECK-VERBOSE-LOGGING-STILL-USES-A2497472
title: "typecheck verbose logging still uses stale symbol allowlists"
area: core
status: fixed
resolved: true
priority: P1
type: maintenance
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/typecheck/{call_reduction,prefix_check,constructor_apply,driver,function_check,function_apply}.rs; nepl-core/src/resource_primitives/{compiler_memory,memory_helper}.rs; nodesrc/test_static_check_boundary_responsibility.js; doc/neplg2/static_check_complexity_reduction_plan.md"
---

# ISS-20260517T080907212Z-TYPECHECK-VERBOSE-LOGGING-STILL-USES-A2497472: typecheck verbose logging still uses stale symbol allowlists

## 概要

typecheck の verbose logging に get/is_none/must_hm/make_hm/new/DefaultHash32/A/use_a/Result/partition などの個別シンボル名フィルタが残っている。これは静的検査本体の判定ではないが、typecheck のデバッグ経路が特定 stdlib/test symbol に依存し、汎用的な検査器・証明器の設計方針とずれる。

## 対象

- `nepl-core/src/typecheck/{call_reduction,prefix_check,constructor_apply,driver,function_check,function_apply}.rs; nepl-core/src/resource_primitives/{compiler_memory,memory_helper}.rs; nodesrc/test_static_check_boundary_responsibility.js; doc/neplg2/static_check_complexity_reduction_plan.md`

## 根拠

- `call_reduction.rs` は `reduce_calls_guarded` の stack/result trace を `get` / `is_none` / `must_hm` / `make_hm` / `new` / `DefaultHash32` / `A` / `use_a` だけへ絞っていた。
- `prefix_check.rs` も value/callable push trace を同じテスト・stdlib 名だけへ絞っていた。
- `constructor_apply.rs`、`driver.rs`、`function_apply.rs`、`function_check.rs` にはそれぞれ `Result::Ok`、`new`、`Result`、`partition` だけを対象にした verbose trace が残っていた。
- 監査テスト更新時に、`resource_primitives.rs` が shared compiler memory primitive の追加で責務上限を超えていることも確認した。

## 問題

typecheck の verbose logging に get/is_none/must_hm/make_hm/new/DefaultHash32/A/use_a/Result/partition などの個別シンボル名フィルタが残っている。これは静的検査本体の判定ではないが、typecheck のデバッグ経路が特定 stdlib/test symbol に依存し、汎用的な検査器・証明器の設計方針とずれる。

## 影響

静的検査の調査時に特定 symbol だけが詳細ログを得るため、抽象化・trait・Resource IR の一般的な問題が同じ粒度で観測されない。将来の修正でこの種の個別名依存が静的証明ロジックへ流入しても見逃しやすい。

## 修正方針

verbose logging は CompileOptions verbose / module-level log macro に一本化し、個別 stdlib/test symbol 名による allowlist を削除する。必要なログは名前で絞らず、処理段階と型・stack 状態を汎用的に出力する。再発防止として nodesrc policy で typecheck 配下の古い個別名フィルタを禁止する。

## 検証

cargo fmt -p nepl-core --check; cargo check -p nepl-core; focused typecheck tests; node nodesrc/test_static_check_boundary_responsibility.js; node nodesrc/issues.js check --dir issues; git diff --check

## 対応

2026-05-17 に修正した。typecheck の verbose trace は特定 symbol 名の allowlist を使わず、`crate::log::is_verbose()` と処理段階だけで汎用的に出力する形へ揃えた。`reduce_calls_guarded` の stack/result trace、prefix value/callable push、enum constructor apply、global function registration、function result summary、function apply trace は、名前で分岐せず同じ構造の情報を出す。

再発防止として `nodesrc/test_static_check_boundary_responsibility.js` に stale name filter の禁止 policy を追加した。さらに同テストで露出した `resource_primitives.rs` の肥大化は、`resource_primitives/compiler_memory.rs` と `resource_primitives/memory_helper.rs` へ分割し、compiler memory type/field contract と memory helper primitive registry の責務を別 module にした。

## 回帰テスト

- `cargo fmt -p nepl-core --check`: passed
- `cargo check -p nepl-core`: passed
- `cargo test -p nepl-core compiler_memory_type_field_specs_are_kind_owned --lib -- --nocapture`: passed
- `cargo test -p nepl-core memory_helper_primitive_classifies_suffixed_symbols --lib -- --nocapture`: passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `node nodesrc/issues.js check --dir issues`: passed
- `git diff --check`: passed with CRLF normalization warnings only

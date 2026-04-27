---
id: ISS-20260427T172347729Z-MOVE-CHECK-RUST-HARNESS-DROPS-LOADER-AA922EEB
title: "Loader-based Rust harnesses drop source capabilities"
area: core
status: fixed
resolved: true
priority: P1
type: test
created: 2026-04-27
updated: 2026-04-28
target: "nepl-core/tests/move_check.rs, nepl-core/tests/drop.rs, nepl-core/tests/recursive_type.rs, nepl-core/tests/repro_recursive.rs, nepl-core/tests/debug_loader.rs, nepl-core/tests/neplg2.rs"
---

# ISS-20260427T172347729Z-MOVE-CHECK-RUST-HARNESS-DROPS-LOADER-AA922EEB: Loader-based Rust harnesses drop source capabilities

## 概要

Loader で stdlib を読み込む Rust test harness の一部が `compile_module` を直接呼び、Loader が作った `SourceMap` / `SourceCapabilities` を compile pipeline へ渡していなかった。`core/mem` raw memory boundary が `SourceCapabilities` 管理になった後、この harness は audited stdlib の `core/mem` raw body を user raw body と同じ扱いで拒否していた。

## 対象

- `nepl-core/tests/move_check.rs`
- `nepl-core/tests/drop.rs`
- `nepl-core/tests/recursive_type.rs`
- `nepl-core/tests/repro_recursive.rs`
- `nepl-core/tests/debug_loader.rs`
- `nepl-core/tests/neplg2.rs`

## 根拠

- `nepl-core/tests/move_check.rs` の `compile_move_test` は `Loader::load_inline` の戻り値から `loaded.module` だけを取り出し、`compile_module` を呼んでいた。
- `compile_module` は `compile_module_with_source_map(module, None, options)` へ委譲するため、import 済み stdlib file の capability が typecheck へ届かない。
- 同じ Loader + `compile_module` pattern が drop / recursive / debug loader 系 test にも残っていた。
- 失敗時は move/borrow/lifetime 本体の診断ではなく、`core/mem.nepl` の `memory.size` / `i32.load` / `memory.copy` などが `TypePureCallsImpureFunction` として先に報告された。

## 問題

Loader は configured stdlib の `core/mem.nepl` に raw memory boundary capability を付けているが、test harness が `SourceMap` を捨てると compiler はその事実を観測できない。これは検査本体の問題ではなく、Loader を使う Rust test が source-map aware compile API を使っていないことが根本原因だった。

## 影響

borrow / lifetime / move / drop の regression tests が、検査対象の失敗ではなく stdlib `core/mem` の raw body capability 欠落で全滅し、メモリ安全監視として機能しなくなる。

## 修正方針

Loader で source を読んだ test harness は `compile_module` ではなく `compile_module_with_source_map(loaded.module, Some(&loaded.source_map), options)` を使う。これにより imported stdlib module の capability を typecheck / effect check へ保持する。

## 対応結果

- `move_check.rs`、`drop.rs`、`recursive_type.rs`、`repro_recursive.rs`、`debug_loader.rs` の Loader-based compile call を `compile_module_with_source_map` へ変更した。
- `loaded.source_map` を渡すことで、configured stdlib の `core/mem` raw memory boundary capability が compiler pipeline に保持されるようにした。
- `move_check` 全51件が再び borrow / lifetime / move 本体の regression として実行できることを確認した。
- 2026-04-28 追記: `neplg2.rs` の LLVM IR harness 3件も Loader の `SourceMap` を落としていたため、`emit_ll_from_module_for_target_with_source_map(..., Some(&loaded.source_map))` へ変更した。
- これにより `stdlib/core/mem.nepl` の raw memory intrinsic が pure body と誤判定される D3025 失敗を、検査本体を緩めずに解消した。

## 検証

Run cargo test -p nepl-core --test move_check and keep the source capability behavior covered by effects tests.

## 実施した検証

- `cargo fmt --check`: pass
- `cargo test -p nepl-core --test move_check`: 51 passed
- `cargo test -p nepl-core --test drop`: 9 passed
- `cargo test -p nepl-core --test recursive_type`: 1 passed
- `cargo test -p nepl-core --test repro_recursive`: 1 passed
- `cargo test -p nepl-core --test debug_loader`: 1 passed
- `cargo check -p nepl-core --tests`: pass
- `cargo test -p nepl-core --test neplg2 llvm_match_i32_literal_lowers_to_switch -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 llvm_reference_scalar_addr_of_and_deref_lowers -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 llvm_reference_aggregate_addr_of_lowers -- --nocapture`: pass

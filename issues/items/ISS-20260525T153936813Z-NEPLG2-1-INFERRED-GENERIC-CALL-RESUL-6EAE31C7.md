---
id: ISS-20260525T153936813Z-NEPLG2-1-INFERRED-GENERIC-CALL-RESUL-6EAE31C7
title: "NEPLg2.1 inferred generic call result loses Resource IR initialization"
area: core
status: fixed
resolved: true
priority: P0
type: bug
created: 2026-05-25
updated: 2026-05-25
target: "nepl-core/src/typecheck/selected_call_apply.rs; nepl-core/src/resource/initialized*.rs; nepl-core/src/resource/collection_slot_summary_apply_return_path.rs; nepl-core/tests/functions.rs"
---

# ISS-20260525T153936813Z-NEPLG2-1-INFERRED-GENERIC-CALL-RESUL-6EAE31C7: NEPLg2.1 inferred generic call result loses Resource IR initialization

## 概要

NEPLg2.1 postfix-free overloaded generic calls can infer concrete type arguments but leave the HIR call result type as the original fresh result variable. The following Resource IR pass then sees collection-slot return-path alternatives with no concrete variant match and can merge an empty path state over an otherwise initialized call output.

## 対象

- `nepl-core/src/typecheck/selected_call_apply.rs; nepl-core/src/resource/initialized*.rs; nepl-core/src/resource/collection_slot_summary_apply_return_path.rs; nepl-core/tests/functions.rs`

## 根拠

- `function_neplg21_overloaded_generic_call_uses_ascribed_result_without_type_args` が、`and_then res0 positive_double` / `unwrap_ok res1` の postfix-free generic call で `resource.cell.uninit` になった。
- typecheck trace では selected callable の type args は `i32, str` に解決していたが、HIR call expression の戻り値型は `c_result` の fresh type variable のままだった。
- Resource IR summary apply 側では、callsite concrete variant 条件に合う return path が 0 件のとき、空の path-sensitive state が直線状態を上書きし、直前に初期化された call output を消していた。
- RawAddressView は call output に raw address alias proof を付与する操作だが、alias rekey の副作用で target value cell の初期化状態を落とす経路があった。

## 問題

NEPLg2.1 postfix-free overloaded generic calls can infer concrete type arguments but leave the HIR call result type as the original fresh result variable. The following Resource IR pass then sees collection-slot return-path alternatives with no concrete variant match and can merge an empty path state over an otherwise initialized call output.

## 影響

Valid NEPLg2.1 Result/Option-style generic call sites fail resource checking with an uninitialized return value, blocking corpus migration away from explicit generic postfixes.

## 修正方針

Substitute resolved inferred type arguments into the selected callable result before HIR assembly, treat an empty path-sensitive return-path candidate set as no refinement instead of unreachable state, and preserve value initialization when RawAddressView only attaches alias proof to an already produced call target.

## 検証

Run the focused NEPLg2.1 overloaded generic call regression, broader NEPLg2.1 function tests, Resource IR initialization tests, issue check, syntax migration check, and git diff check.

## 2026-05-25 修正結果

- `selected_call_apply.rs` で、明示 type args が無い generic call について constraint から解けた type args を declared result に代入し、HIR call expression の戻り値型を concrete result へ更新するようにした。
- `apply_collection_slot_return_paths` と direct call 側の path alternative 接続で、concrete variant 条件に合う return path が 0 件のときは path-sensitive refinement を作らず、通常の call output 初期化済み状態を維持するようにした。
- `ResourcePathAlternatives::from_states(Vec::new())` は既存の「全 path が棄却された」表現として残し、no-refinement の扱いは direct call return-path の呼び出し元で限定的に処理するようにした。
- `RawAddressView` は raw address alias proof を付与する操作として扱い、target が既に initialized だった場合は alias rekey 後も値 cell の初期化状態を保持するようにした。

検証:

- `cargo fmt -p nepl-core --check`: passed.
- `cargo test -p nepl-core --test functions function_neplg21_overloaded_generic_call_uses_ascribed_result_without_type_args -- --nocapture`: passed.
- `cargo test -p nepl-core --test functions neplg21 -- --nocapture`: 8/8 passed.
- `cargo test -p nepl-core --test typeannot neplg21 -- --nocapture`: 2/2 passed.
- `cargo test -p nepl-core --test resource_ir raw_address_view -- --nocapture`: 1/1 passed.
- `node nodesrc/neplg21_syntax_migrate.js --check`: would update 0 file(s).
- `node nodesrc/issues.js check --dir issues`: passed.
- `git diff --check`: passed. CRLF checkout warning のみ。

既知 baseline:

- `cargo test -p nepl-core --test resource_ir return_path -- --nocapture` は current branch と clean HEAD `457d8b32` の両方で `resource_ir_collection_slot_return_path_state_only_replay_does_not_duplicate_diagnostics` が同じ形で失敗する。今回差分固有の regression ではないため、`ISS-20260525T154712899Z-RESOURCE-IR-RETURN-PATH-REPLAY-UNIT--010D6DB7` として分離した。

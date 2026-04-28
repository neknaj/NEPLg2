---
id: ISS-20260428T123750146Z-RESOURCE-EFFECT-CHECKER-KEEPS-STALE--0C4747E0
title: "Resource effect checker keeps stale function aliases after assignment"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "nepl-core/src/resource/effect.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260428T123750146Z-RESOURCE-EFFECT-CHECKER-KEEPS-STALE--0C4747E0: Resource effect checker keeps stale function aliases after assignment

## 概要

The effect checker has its own FunctionAliasTable. Its copy_alias keeps the target's previous known callee when the source has no known function alias, so assigning an unknown callback over a known function value can leave stale alias state.

## 対象

- `nepl-core/src/resource/effect.rs`
- `nepl-core/tests/resource_ir.rs`

## 根拠

- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 5 は raw memory identity を safe surface から閉じ、unknown callback 境界でも保守的に扱う方針である。
- borrow / owner checker 側の `FunctionAliasTable` は `ISS-20260428T122920913Z-RESOURCE-FUNCTION-ALIAS-TABLE-KEEPS--6208EF78` で stale alias を消すようになったが、effect checker は別定義の `FunctionAliasTable` を持っていた。
- effect checker の `copy_indirect_call_return_identity` は callee に known alias がある場合、その known function summary だけを使い、alias が空のときだけ unknown callback fallback で raw identity 引数を output へ伝播する。
- known safe function alias を unknown callback で上書きした後も stale alias が残ると、unknown callback fallback が使われず、internal raw allocation の return escape が診断されない。

## 問題

The effect checker has its own FunctionAliasTable. Its copy_alias keeps the target's previous known callee when the source has no known function alias, so assigning an unknown callback over a known function value can leave stale alias state.

## 影響

Raw identity and pointer escape checks can apply summaries for the wrong known callee and skip the conservative unknown-callback fallback. A pure function can return an internal raw allocation through an unknown callback without a RawAddressEscape diagnostic.

## 修正方針

Make the effect checker function alias copy overwrite target alias state just like the Resource check alias table: copy known aliases when present and clear target aliases when absent. Add a regression for unknown callback raw identity escape after overwriting a known safe alias.

## 検証

- `cargo test -p nepl-core --test resource_ir -- --nocapture`
- `trunk build`
- `node nodesrc/issues.js check`
- `rustfmt --check nepl-core\src\resource\effect.rs nepl-core\tests\resource_ir.rs`
- `git diff --check`

## 2026-04-28 Stage 5 effect function alias clear 対応

effect checker 側の `FunctionAliasTable::copy_alias` も target alias state の上書き操作に変更した。source に known function alias がある場合は target に copy し、source に known alias がない場合は target alias entry を消す。

これにより、known safe function value を保持していた local に unknown callback を assign した後の `IndirectCall` は stale known summary ではなく unknown callback fallback を使う。raw identity 引数が戻り値へ返る可能性を保守的に output へ伝播し、pure function から internal raw allocation が返る経路を `RawAddressEscapeFromInternalAlloc` として検出できる。

`nepl-core/tests/resource_ir.rs` に、known safe function alias を unknown callback で上書きした後、raw allocation を indirect call 経由で return する経路が effect boundary diagnostic になる回帰を追加した。

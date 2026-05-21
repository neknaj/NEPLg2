---
id: ISS-20260521T214431160Z-GENERIC-DROP-BOUND-TYPE-VARIABLES-AR-F3533E34
title: "Generic Drop-bound type variables are treated as StateOnly by Resource drop requirements"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-21
updated: 2026-05-22
target: "nepl-core/src/resource/drop_requirement.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260521T214431160Z-GENERIC-DROP-BOUND-TYPE-VARIABLES-AR-F3533E34: Generic Drop-bound type variables are treated as StateOnly by Resource drop requirements

## 概要

`ResourceDropRequirement` は concrete な `impl Drop` target だけを drop code requirement として扱い、未束縛の `.T: Drop` 型変数を `StateOnly` として扱っていた。

generic collection cleanup が `.T: Drop` を受け取る段階で、このままだと要素ごとの actual loaded-value drop proof を要求せずに state-only cleanup として扱われる。

## 対象

- `nepl-core/src/resource/drop_requirement.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- 親 issue: [Non-Copy collection payload support needs compiler-issued owner and drop traversal](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md)
- 関連 doc: [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- 関連 doc: [stdlib collection/mem/string と静的検査の安全設計](../../doc/neplg2/stdlib_collection_mem_string_static_safety_design.md)
- 開発方針: https://zenn.dev/bem130/articles/1b352797de94e7

## 問題

`.T: Drop` は source typecheck の trait bound としては確定しているが、Resource IR の drop requirement derivation では concrete impl lookup だけを見ていた。そのため `.T` が未束縛のまま Resource IR に現れる generic helper では、drop requirement が `StateOnly` になる。

これは stdlib allowlist ではなく compiler が source の性質を証明する方針に反する。generic `.T: Drop` は「どの concrete type でも drop code が必要になり得る」ことを型境界が表しているため、Resource IR 側でも `WholeValue` drop obligation として扱う必要がある。

## 影響

Non-Copy collection support could accept generic drop traversal without proving that each loaded .T payload was dropped, violating the compiler-owned Resource IR proof model and memory safety policy.

## 修正方針

`ResourceDropRequirement` の derivation で、未束縛の `TypeKind::Var` が `drop_cap` を持つ場合は `WholeValue` として扱う。

これにより generic `.T: Drop` の scope drop / assignment overwrite / collection slot drop proof は、state-only cleanup ではなく actual drop elaboration を要求する。

## 検証

- `cargo test -p nepl-core --test resource_ir generic_drop_bound_type_param_requires_whole_value_drop_requirement`

## 対応

- `nepl-core/src/resource/drop_requirement.rs` で unbound `.T: Drop` を `WholeValue` drop requirement にした。
- `nepl-core/tests/resource_ir.rs` に generic `.T: Drop` parameter の auto-drop requirement が `StateOnly` へ退行しない回帰テストを追加した。

---
id: ISS-20260514T180856087Z-VEC-ROOT-FACADE-RE-EXPORTS-RAW-ELEME-93D13B29
title: "Vec root facade re-exports raw element helpers"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-14
updated: 2026-05-15
target: stdlib/alloc/collections/vec.nepl
---

# ISS-20260514T180856087Z-VEC-ROOT-FACADE-RE-EXPORTS-RAW-ELEME-93D13B29: Vec root facade re-exports raw element helpers

## 概要

The safe alloc/collections/vec facade still publicly merges vec/raw, so ordinary root imports expose vec_read_at and vec_write_at even though those helpers are unchecked raw MemPtr element load/store boundaries.

## 対象

- `stdlib/alloc/collections/vec.nepl`

## 根拠

- `stdlib/alloc/collections/vec.nepl` は root safe facade であるにもかかわらず、`pub #import "./vec/raw" as @merge` により unchecked raw helper を通常 import 面へ混ぜていた。
- `stdlib/alloc/collections/vec/raw/element.nepl` の `vec_read_at` / `vec_write_at` は `MemPtr<T>` と index を受け、範囲検査なしで raw `load<T>` / `store<T>` を行う実装境界である。
- Stage 6 では `Vec` public API と raw-backed implementation boundary を分ける方針であり、root facade が raw helper を再公開するとこの分離が弱くなる。

## 問題

The safe alloc/collections/vec facade still publicly merges vec/raw, so ordinary root imports expose vec_read_at and vec_write_at even though those helpers are unchecked raw MemPtr element load/store boundaries.

## 影響

The Stage 6 public API boundary remains wider than the safe Vec contract. Callers can discover and depend on raw helper names from the normal Vec facade, making the future OwnedBuffer and initialized-cell split harder and weakening reviewability of raw-backed operations.

## 修正方針

Stop re-exporting vec/raw from the root Vec facade. Keep alloc/collections/vec/raw as an explicit raw submodule for implementation and focused tests, update doctests to import it explicitly, and add source policy coverage that root Vec does not merge raw helpers.

## 検証

Run Vec source policy, raw element doctests, focused Vec tests, issues check, and diff check.

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- [stdlib collection / memory / string static safety design](../../doc/neplg2/stdlib_collection_mem_string_static_safety_design.md)

## 2026-05-15 Agent 1 解決

`alloc/collections/vec` root facade から `pub #import "./vec/raw" as @merge` を削除した。これにより通常の `#import "alloc/collections/vec" as *` では `vec_read_at` / `vec_write_at` が見えず、unchecked raw element helper は `#import "alloc/collections/vec/raw" as *` を明示した実装・focused test 境界だけで使う形になった。

`vec/raw/element.nepl` の doctest は root `Vec` API と raw submodule を分けて import するように更新した。source policy では root が `vec/raw` を再公開しないこと、raw helper 名を root に戻さないこと、`vec/raw` facade 自体は explicit raw submodule として `element` だけを re-export することを固定した。

検証:

- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/raw/element.nepl --no-tree -o tmp/agent1-vec-root-raw-facade-raw-element.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i stdlib/tests/vec.n.md --no-tree -o tmp/agent1-vec-root-raw-facade-vec-tests.json -j 1 --dist web/dist --assert-io`

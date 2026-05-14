---
id: ISS-20260514T200755109Z-KPFENWICK-AND-KPDSU-EXPOSE-RAW-I32-O-953345F8
title: "kpfenwick and kpdsu expose raw i32 owner handles"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-15
target: "stdlib/kp/kpfenwick.nepl, stdlib/kp/kpdsu.nepl, nodesrc/test_stdlib_kp_structures_owner_boundary.js"
---

# ISS-20260514T200755109Z-KPFENWICK-AND-KPDSU-EXPOSE-RAW-I32-O-953345F8: kpfenwick and kpdsu expose raw i32 owner handles

## 概要

kp/kpfenwick and kp/kpdsu expose allocation owners as public i32 handles and operate through raw load/store helpers. Ordinary callers can copy or reuse the handle, double-free it, skip ownership-preserving error recovery, and depend on raw storage identity. This contradicts Stage 6 static-check complexity reduction: MemPtr/raw address operations must remain internal boundaries, while public stdlib APIs should expose typed owners and compiler-checkable ownership flow.

## 対象

- `stdlib/kp/kpfenwick.nepl, stdlib/kp/kpdsu.nepl, nodesrc/test_stdlib_kp_structures_owner_boundary.js`

## 根拠

- `stdlib/kp/kpfenwick.nepl` は `core/mem` / `core/mem/allocator` / `core/mem/raw` を import し、`fenwick_new` が raw allocation handle を `i32` として返していた。
- `stdlib/kp/kpdsu.nepl` も同様に raw `i32` handle へ parent/size storage owner を詰め、`dsu_free` / `dsu_find` / `dsu_unite` が raw load/store を直接操作していた。
- どちらも `i32` が Copy scalar であるため、compiler の owner/free obligation 検査が public API 上の所有権を追跡できない。

## 問題

kp/kpfenwick and kp/kpdsu expose allocation owners as public i32 handles and operate through raw load/store helpers. Ordinary callers can copy or reuse the handle, double-free it, skip ownership-preserving error recovery, and depend on raw storage identity. This contradicts Stage 6 static-check complexity reduction: MemPtr/raw address operations must remain internal boundaries, while public stdlib APIs should expose typed owners and compiler-checkable ownership flow.

## 影響

Memory-safety and type-safety checks cannot prove the lifetime/free obligation of these KP structures from the source API because the owner is erased to i32. It also encourages more stdlib code to bypass Vec/collection owner contracts instead of relying on compiler-proved properties.

## 修正方針

Replace the raw implementations with thin facades over alloc/collections/fenwick and alloc/collections/disjoint_set. Public construction must return Result-owned structures, queries must borrow typed owners and return Result diagnostics, and updates must consume and return owners on success or typed owner-carrying errors on failure. Do not keep the old raw i32 compatibility APIs.

## 検証

Add a source-level regression test that rejects raw memory imports/helpers and raw i32 owner signatures in kpfenwick/kpdsu, then run focused doctests for both modules and the issue metadata check.

## 修正結果

- `kpfenwick` から raw memory import、`alloc_raw` / `dealloc_raw` / `load_i32` / `store_i32` を削除した。
- `fenwick_new` は `Result<Fenwick, Diag>`、`fenwick_add` は `Result<Fenwick, FenwickAddError>`、query は `&Fenwick` から `Result<i32, Diag>` を返す API に変更した。
- `kpfenwick` は `alloc/collections/fenwick` の root facade を丸ごと import しない。既存 Fenwick API の `add` 名が `core/math.add` と干渉するため、typed storage helper / mutation helper / query helper / diagnostic helper を直接使う構成にした。
- `kpdsu` から raw memory import と raw handle API を削除し、`alloc/collections/disjoint_set` の owner-consuming update / borrowed query API へ委譲した。
- 古い raw `i32` API との互換 alias は残していない。

## 回帰テスト

- `nodesrc/test_stdlib_kp_structures_owner_boundary.js` を追加した。
- このテストは `kpfenwick` / `kpdsu` の raw memory import、raw helper、raw `i32` owner signature の再導入を拒否し、typed owner / borrow / `Result` API になっていることを検査する。

## 検証結果

- `node nodesrc/test_stdlib_kp_structures_owner_boundary.js`
- `node nodesrc/tests.js -i stdlib/kp/kpfenwick.nepl --no-tree -o tmp/agent1-kpfenwick-owner-boundary-module.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i stdlib/kp/kpdsu.nepl --no-tree -o tmp/agent1-kpdsu-owner-boundary-module.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/issues.js check --dir issues`

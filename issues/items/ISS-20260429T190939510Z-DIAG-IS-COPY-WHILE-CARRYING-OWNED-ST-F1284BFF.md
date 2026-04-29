---
id: ISS-20260429T190939510Z-DIAG-IS-COPY-WHILE-CARRYING-OWNED-ST-F1284BFF
title: "Diag is Copy while carrying owned string fields and lacks a consumption contract"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-04-29
updated: 2026-04-30
target: "stdlib/alloc/diag/error.nepl, tests/stdlib/collections_diag.n.md, stdlib/alloc/collections/**"
---

# ISS-20260429T190939510Z-DIAG-IS-COPY-WHILE-CARRYING-OWNED-ST-F1284BFF: Diag is Copy while carrying owned string fields and lacks a consumption contract

## 概要

Strict Resource IR now reports a leak when a collection test matches Result::Err d and inspects diag_std_error_kind_str d: the local Diag keeps an owned message field alive. Diag currently implements Copy even though it contains str fields such as message, notes, help, and optional source. That makes diagnostic values look lightweight while still carrying ownership obligations in failure branches.

## 対象

- `stdlib/alloc/diag/error.nepl, tests/stdlib/collections_diag.n.md, stdlib/alloc/collections/**`

## 根拠

- 未記入

## 問題

Strict Resource IR now reports a leak when a collection test matches Result::Err d and inspects diag_std_error_kind_str d: the local Diag keeps an owned message field alive. Diag currently implements Copy even though it contains str fields such as message, notes, help, and optional source. That makes diagnostic values look lightweight while still carrying ownership obligations in failure branches.

## 影響

Collection and self-host tests cannot safely inspect rich Diag values returned from Err without either leaking strings or relying on Copy semantics that conflict with the owner model. Self-host diagnostics need a clear type-safe contract before diagnostic aggregation grows larger.

## 修正方針

Redesign Diag ownership: either make the low-level collection error path return a Copy-only StdErrorKind/lightweight code, or make Diag a non-Copy owned diagnostic with explicit borrowed accessors and a free/drop contract. Add fixtures that match Err(Diag), inspect the kind, and close the diagnostic owner without weakening Resource IR.

## 検証

Run node nodesrc/run_doctest.js -i tests/stdlib/collections_diag.n.md -n 1 --dist web/dist and the diag/error stdlib suites after the ownership contract is redesigned.

## 2026-04-30 修正完了

`Diag` が失敗値として軽量にコピーされる経路へ、`concat` で作った owned string を直接保持しないように整理した。

修正内容:

- `diag_add_note` / `diag_add_help` は `Diag` 内で note/help を連結済み text block にせず、1 件分の text fragment として保持するようにした。
- `alloc/diag/diag.nepl` の renderer 側で note の改行と `help: ` prefix を付けるようにし、表示文字列は従来と同じ順序に保った。
- `diag_out_of_memory` / `diag_empty_collection` / `diag_capacity_exceeded` / `diag_key_not_found` は operation 名を受け取らず、静的な標準メッセージを持つ `Diag` を返すようにした。
- bitset / adjacency_matrix / bloom_filter / counting_bloom_filter / disjoint_set / fenwick / segment_tree / sparse_set の collection-specific diagnostic helper も、値引数を受け取らず静的メッセージを返す形にした。
- `Diags` の by-value `diags_len` / `diags_has_errors` / `diags_to_string` は、観測後に `diags_free` で backing `Vec<Diag>` を閉じる契約にした。
- `nodesrc/test_stdlib_diag_error_no_unsafe_unwraps.js` を更新し、Diag helper が owned `concat` text block へ戻らないことと、by-value `Diags` helper が owner を閉じることを source policy で固定した。
- `stdlib/tests/error.n.md` は `diags_has_errors ds1` の by-value 経路で最後に `Diags` owner を閉じる regression に更新した。

検証:

- `node nodesrc/run_doctest.js -i tests/stdlib/collections_diag.n.md -n 1 --dist web/dist`: pass
- `node nodesrc/run_doctest.js -i tests/stdlib/collections_diag.n.md -n 2 --dist web/dist`: pass
- `node nodesrc/tests.js -i stdlib/tests/diag.n.md -i stdlib/tests/error.n.md --no-tree -o tmp/diag-owner-contract.json -j 1 --dist web/dist`: 5 passed
- `node nodesrc/tests.js -i stdlib/tests/diag.n.md -i stdlib/tests/error.n.md --no-tree -o tmp/diag-owner-contract-2.json -j 1 --dist web/dist`: 5 passed
- `node nodesrc/test_stdlib_diag_error_no_unsafe_unwraps.js`: pass

補足:

- `node nodesrc/tests.js -i tests/stdlib/collections_diag.n.md --no-tree -o tmp/collections-diag-owner-contract.json -j 1 --dist web/dist` は HashMap/HashSet の Diag 回帰 2 件は pass したが、Queue/RingBuffer の `pop` 実装が RawMemoryLoadCell gate で落ちる既存の raw-memory-backed collection 残件が 2 件残る。これは `ISS-20260429T155343006Z-COLLECTION-STORAGE-STATES-USE-NUMERI-E4B3A749` の残件として扱う。

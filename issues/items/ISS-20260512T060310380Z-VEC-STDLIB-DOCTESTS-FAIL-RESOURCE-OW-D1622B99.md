---
id: ISS-20260512T060310380Z-VEC-STDLIB-DOCTESTS-FAIL-RESOURCE-OW-D1622B99
title: "Vec stdlib doctests fail resource owner checks in query helpers"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-12
updated: 2026-05-12
target: "stdlib/alloc/collections/vec/query.nepl, stdlib/alloc/collections/vec/transform/prefix.nepl, stdlib/tests/vec.n.md"
---

# ISS-20260512T060310380Z-VEC-STDLIB-DOCTESTS-FAIL-RESOURCE-OW-D1622B99: Vec stdlib doctests fail resource owner checks in query helpers

## 概要

2026-05-12 の `trunk build` 後に Vec sort の広めの focused suite を実行すると、`stdlib/tests/vec.n.md` の doctest#2-#6 が `resource.owner.no_free_obligation` で失敗する。失敗している monomorphized function は `all`、`count`、`partition`、`vec_take_while_len_impl` であり、同じ run 内の merge sort test は通過しているため、`vec/sort/merge` 分割とは別の Vec query / transform helper と Resource IR の契約不一致として扱う。

## 対象

- `stdlib/alloc/collections/vec/query.nepl`
- `stdlib/alloc/collections/vec/transform/prefix.nepl`
- `stdlib/alloc/collections/vec/raw/prefix.nepl`
- `stdlib/alloc/collections/vec/transform/filter.nepl`
- `stdlib/tests/vec.n.md`

## 根拠

- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/sort/common.nepl -i stdlib/alloc/collections/vec/sort/simple.nepl -i stdlib/alloc/collections/vec/sort/quick.nepl -i stdlib/alloc/collections/vec/sort/heap.nepl -i stdlib/alloc/collections/vec/sort/merge.nepl -i stdlib/alloc/collections/vec/sort/merge/buffer.nepl -i stdlib/alloc/collections/vec/sort/merge/range.nepl -i stdlib/alloc/collections/vec/sort/merge/api.nepl -i stdlib/alloc/collections/vec/sort.nepl -i stdlib/alloc/collections/vec.nepl -i tests/stdlib/sort.n.md -i stdlib/tests/vec.n.md --no-tree -o tmp/vec-sort-merge-split-focused.json -j 4 --dist web/dist`: total=34, passed=29, failed=5。
- remote main の `3487e386` 取り込み後に同じ suite を `tmp/vec-sort-merge-split-focused-after-rebase.json` へ再実行しても total=34, passed=29, failed=5。
- 失敗は `stdlib/tests/vec.n.md::doctest#2` から `#6` に集中している。
- error は `all__ref_Vec...`、`count__ref_Vec...`、`partition__Vec...`、`vec_take_while_len_impl...` の `CallArgument` が `NoFreeObligation` だったという Resource IR 検査結果。
- 同じ変更後に `node nodesrc/tests.js -i ...merge... -i tests/stdlib/sort.n.md --no-tree -o tmp/vec-sort-merge-split-sort-focused.json -j 4 --dist web/dist` は total=25, passed=25 で通過している。

## 問題

Vec の query / transform / prefix helper が raw storage から読み出した `.T` を predicate や result 構築へ渡す境界で、Resource IR が free obligation を証明できていない。`all` / `count` のような observer、`partition` のような owner-producing transform、`take_while` / `drop_while` の prefix boundary helper が同じ静的検査 failure を起こしているため、単発の doctest 修正ではなく helper 境界の所有権設計を見直す必要がある。

## 影響

`stdlib/tests/vec.n.md` を post-change の広い回帰信号として使えない。さらに、メモリ安全を必達とする方針に対して、raw element read と predicate/helper 呼び出しの契約が曖昧なままになるため、Vec を利用する stdlib collection や selfhost 実装にも同型の問題を波及させる。

## 修正方針

- raw element read を callback に渡す API を、Copy 前提、借用前提、owner-moving のどれとして扱うか明確化する。
- `all` / `count` / `find` など observer helper は、読み出した値の lifetime と callback argument が Resource IR で証明できる形へ境界を再設計する。
- `partition` / `filter` / prefix helper は、出力 Vec へ move する値と predicate 判定に使う値を混同しないよう、必要なら read API や callback signature を分離する。
- `stdlib/tests/vec.n.md`、Vec source policy、broader Vec focused suite を回帰テストとして固定する。

## 検証

- `node nodesrc/tests.js -i stdlib/tests/vec.n.md --no-tree -o tmp/vec-stdlib-resource-owner-after.json -j 1 --dist web/dist`
- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`
- `node nodesrc/test_stdlib_vec_borrowed_observers.js`
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/sort/common.nepl -i stdlib/alloc/collections/vec/sort/simple.nepl -i stdlib/alloc/collections/vec/sort/quick.nepl -i stdlib/alloc/collections/vec/sort/heap.nepl -i stdlib/alloc/collections/vec/sort/merge.nepl -i stdlib/alloc/collections/vec/sort/merge/buffer.nepl -i stdlib/alloc/collections/vec/sort/merge/range.nepl -i stdlib/alloc/collections/vec/sort/merge/api.nepl -i stdlib/alloc/collections/vec/sort.nepl -i stdlib/alloc/collections/vec.nepl -i tests/stdlib/sort.n.md -i stdlib/tests/vec.n.md --no-tree -o tmp/vec-sort-merge-split-focused-after-vec-fix.json -j 4 --dist web/dist`

## 解決

- raw storage から直接読んだ `.T` を callback / output へ渡していた query / transform helper を、borrowed `Vec` と `get<T: Copy>` を使う設計へ戻した。
- `vec/raw/{aggregate,predicate,prefix}.nepl` は raw callback helper としての責務が不適切だったため削除し、`raw.nepl` は raw element access の facade に限定した。
- `map` / `filter` / `partition` / `take_while` / `drop_while` は `T: Copy` 境界で predicate scan と output copy を行う形に統一した。
- owner-consuming transform の source cleanup は `Vec` を field 分解して `vec_free_storage` へ渡すのではなく、`vec_cleanup::free` へ Vec owner を丸ごと渡す形にした。特に `partition` の右側 allocation failure path は `left0` と `v` を `free` で回収し、storage enum と data owner の対応を呼び出し側で分断しない。
- `partition` の成功 path は `VecPartition` を直接構築し、不要な `left_vec` / `right_vec` 中間束縛を置かない形にした。
- `nepl-core/tests/resource_ir.rs` に `Vec.partition` が named `VecPartition` 経由で 2 本の Vec owner を返しても intermediate storage owner を漏らさない回帰を追加した。

## 解決後の検証

- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_vec_borrowed_observers.js`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_vec_partition_returns_named_vec_owners -- --nocapture`: passed
- `node nodesrc/tests.js -i stdlib/tests/vec.n.md --no-tree -o tmp/vec-stdlib-resource-owner-after-direct-partition.json -j 1 --dist web/dist`: total=6, passed=6
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/sort/common.nepl -i stdlib/alloc/collections/vec/sort/simple.nepl -i stdlib/alloc/collections/vec/sort/quick.nepl -i stdlib/alloc/collections/vec/sort/heap.nepl -i stdlib/alloc/collections/vec/sort/merge.nepl -i stdlib/alloc/collections/vec/sort/merge/buffer.nepl -i stdlib/alloc/collections/vec/sort/merge/range.nepl -i stdlib/alloc/collections/vec/sort/merge/api.nepl -i stdlib/alloc/collections/vec/sort.nepl -i stdlib/alloc/collections/vec.nepl -i tests/stdlib/sort.n.md -i stdlib/tests/vec.n.md --no-tree -o tmp/vec-sort-merge-split-focused-after-vec-fix.json -j 4 --dist web/dist`: total=34, passed=34
- `cargo fmt --check -p nepl-core`: passed
- `cargo check -p nepl-core --tests`: passed
- `node nodesrc/issues.js check`: passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: passed

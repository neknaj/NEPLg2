---
id: ISS-20260506T162903318Z-SELFHOST-SOURCETEXT-LINE-MAP-VEC-OWN-2444558D
title: "selfhost SourceText line map Vec owner leaks after Vec raw boundary"
area: selfhost
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-07
target: "stdlib/neplg2/core/infra/text.nepl, stdlib/alloc/collections/vec.nepl"
---

# ISS-20260506T162903318Z-SELFHOST-SOURCETEXT-LINE-MAP-VEC-OWN-2444558D: selfhost SourceText line map Vec owner leaks after Vec raw boundary

## 概要

After alloc/collections/vec.nepl is granted the exact raw-memory boundary, focused selfhost doctests progress past effect.pure.calls_impure and expose resource.owner.maybe_leak in source_text_collect_line_starts plus a use_after_move at the initial Vec push in source_text_new. The line-start builder consumes Vec<i32> through v::push and replaces failed outputs with vec_empty, but the strict Resource IR still cannot prove the previous owner has been closed or transferred on all branches.

## 対象

- `stdlib/neplg2/core/infra/text.nepl, stdlib/alloc/collections/vec.nepl`

## 根拠

- `node nodesrc/tests.js -i stdlib/neplg2/core/infra/text.nepl -i stdlib/neplg2/core/resolve/name_resolver.nepl --no-tree -o tmp/vec-raw-boundary-selfhost.json -j 1` で total=3, passed=2, failed=1。
- `stdlib/neplg2/core/infra/text.nepl::doctest#1` は compile phase で `resource.owner.maybe_leak` を `source_text_collect_line_starts` に、`resource.owner.use_after_move` を `source_text_new` の initial `v::push` に報告した。
- 同じ実行で `stdlib/neplg2/core/resolve/name_resolver.nepl` の 2 doctest は run phase まで進み passed したため、Vec raw-memory boundary の effect blocker は解消済みで、SourceText 側の owner transfer が次の独立した blocker である。

## 問題

After alloc/collections/vec.nepl is granted the exact raw-memory boundary, focused selfhost doctests progress past effect.pure.calls_impure and expose resource.owner.maybe_leak in source_text_collect_line_starts plus a use_after_move at the initial Vec push in source_text_new. The line-start builder consumes Vec<i32> through v::push and replaces failed outputs with vec_empty, but the strict Resource IR still cannot prove the previous owner has been closed or transferred on all branches.

## 影響

Selfhost source text construction remains blocked under mandatory memory-safety checking even after the raw-memory boundary is declared. This prevents selfhost parser/diagnostic modules from being validated and risks hiding real Vec owner-transfer bugs if the checker is weakened.

## 修正方針

Redesign the line-start accumulation API so Vec owner transfer is statically visible: either use a dedicated builder/result type that returns or closes the consumed Vec on push failure, or refactor Vec push failure handling and Resource IR summaries so the consumed owner is provably released. Keep the Resource IR diagnostics strict; do not silence maybe_leak or use_after_move.

## 対応

- `SourceTextLineStartPushState` enum と `SourceTextLineStartPush` owner-carrying outcome を追加し、line start 追加の成功/失敗を bool や数値ではなく enum で表現した。
- `source_text_push_line_start` を追加し、`Vec::push` 成功時は追加済み Vec、失敗時は owner を持たない空 Vec を loop 側へ返す contract にした。
- `source_text_collect_line_starts` は outcome の enum を `match` し、成功/失敗のどちらでも loop accumulator `out` を返却された Vec owner で再初期化するようにした。
- push failure path では Err を返す前に replacement `out` を `v::free` で閉じ、Resource IR に owner cleanup を明示した。
- `source_text_new` の初期 line start table を `new + push(0)` から `filled<i32> 1 0` に変更し、初期化だけのために consuming push を通さないようにした。
- `nodesrc/test_selfhost_source_text_no_recursive_line_map.js` を強化し、loop 実装、owner-carrying outcome、失敗時 cleanup、初期 `filled` 構築を再発防止として固定した。

## 検証

Run focused selfhost doctests for stdlib/neplg2/core/infra/text.nepl and stdlib/neplg2/core/resolve/name_resolver.nepl after trunk build; require the SourceText doctest to compile and run without resource.owner.maybe_leak/use_after_move while the name_resolver doctests continue to pass.

- `node nodesrc/test_selfhost_source_text_no_recursive_line_map.js`: passed
- `node nodesrc/tests.js -i stdlib/neplg2/core/infra/text.nepl -i stdlib/neplg2/core/resolve/name_resolver.nepl --no-tree -o tmp/source-text-line-map-owner-local.json -j 1`: total=3, passed=3
- `node nodesrc/run_source_policy_regressions.js --warn-only`: SourceText policy は passed。既存 `nodesrc/test_stdlib_io_bytebuf_owner_boundary.js` warning は継続。
- `trunk build`: passed
- `node nodesrc/tests.js -i stdlib/neplg2/core/infra/text.nepl -i stdlib/neplg2/core/resolve/name_resolver.nepl --no-tree -o tmp/source-text-line-map-owner-after-trunk.json -j 1`: total=3, passed=3
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed

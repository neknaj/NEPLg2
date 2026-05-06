---
id: ISS-20260506T162903318Z-SELFHOST-SOURCETEXT-LINE-MAP-VEC-OWN-2444558D
title: "selfhost SourceText line map Vec owner leaks after Vec raw boundary"
area: selfhost
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-06
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

## 検証

Run focused selfhost doctests for stdlib/neplg2/core/infra/text.nepl and stdlib/neplg2/core/resolve/name_resolver.nepl after trunk build; require the SourceText doctest to compile and run without resource.owner.maybe_leak/use_after_move while the name_resolver doctests continue to pass.

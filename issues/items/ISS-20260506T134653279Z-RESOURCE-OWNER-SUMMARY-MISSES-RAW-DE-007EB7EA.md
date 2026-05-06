---
id: ISS-20260506T134653279Z-RESOURCE-OWNER-SUMMARY-MISSES-RAW-DE-007EB7EA
title: "Resource owner summary misses raw dealloc consumption through unwrap_ok"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-06
target: "nepl-core/src/resource/owner_*.rs, stdlib/core/result.nepl, tests/stdlib/kp.n.md"
source: doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-への移行
---

# ISS-20260506T134653279Z-RESOURCE-OWNER-SUMMARY-MISSES-RAW-DE-007EB7EA: Resource owner summary misses raw dealloc consumption through unwrap_ok

## 概要

tests/stdlib/kp.n.md doctest#7 allocates a raw buffer through unwrap_ok alloc and frees it through unwrap_ok dealloc, but Resource IR reports a resource.owner.leak for the data owner. The checked dealloc consumption is hidden behind Result unwrap helper flow instead of being visible to the caller summary.

## 対象

- `nepl-core/src/resource/owner_*.rs, stdlib/core/result.nepl, tests/stdlib/kp.n.md`

## 根拠

- `node nodesrc/tests.js -i tests/stdlib/kp.n.md -o output/kp_owner_read_scratch_after_fix.json --runner wasm --no-tree -j 1 --assert-io` の doctest#7 で再現した。
- 該当 doctest は `let data <i32> unwrap_ok alloc mul len 4;` で raw storage owner を取得し、最後に `unwrap_ok dealloc data mul len 4;` で checked cleanup を行う。
- Resource IR owner checker は `resource.owner.leak` として `Place { root: Local("data"), ... } still owns StorageId(0)` を報告する。
- private scratch ではなく doctest の user-visible raw cleanup 経路であるため、単純に `dealloc_raw` へ置き換えると checked API の summary 不備を隠す。
- `unwrap_ok` は `Result` payload を取り出す helper であり、`dealloc` の success branch が owner を消費する事実を caller summary に伝える必要がある。

## 問題

tests/stdlib/kp.n.md doctest#7 allocates a raw buffer through unwrap_ok alloc and frees it through unwrap_ok dealloc, but Resource IR reports a resource.owner.leak for the data owner. The checked dealloc consumption is hidden behind Result unwrap helper flow instead of being visible to the caller summary.

## 影響

Idiomatic checked cleanup using unwrap_ok dealloc fails static owner checking, while replacing it with unchecked raw dealloc would hide the summary completeness bug and weaken memory-safety evidence.

## 修正方針

Model Result unwrap helpers and checked dealloc success consumption in Resource IR owner summaries so ownership transfer is visible through unwrap_ok, or redesign the helper boundary so checked dealloc consumption remains statically explicit without raw public escape.

## 検証

Add a Resource IR regression for unwrap_ok dealloc consuming a raw owner and rerun tests/stdlib/kp.n.md doctest#7.

## 2026-05-06 string boundary 修正後の再確認

`ISS-20260506T135746003Z-STRING-ACCESS-SPLIT-LOSES-RAW-MEMORY-8C64A912` の修正後、`tests/stdlib/kp.n.md::doctest#7` は引き続き `resource.owner.leak` で停止した。

診断は `Place { root: Local("data"), projections: [], ty: TypeId(1) } still owns StorageId(0)` であり、`unwrap_ok dealloc data ...` の checked cleanup consumption が Resource IR owner summary に伝わっていないというこの issue の本体を再確認した。

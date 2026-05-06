---
id: ISS-20260506T134653279Z-RESOURCE-OWNER-SUMMARY-MISSES-RAW-DE-007EB7EA
title: "Resource owner summary misses raw dealloc consumption through unwrap_ok"
area: core
status: fixed
resolved: true
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

## 2026-05-06 修正

Resource IR owner summary に `resolved_parameter_variants` を追加し、`unwrap_ok` のように戻り値の reachable arm が `Result::Ok` だけへ確定する helper を、呼び出し元の `Result` variant 確定として表現するようにした。

根本原因は、`unwrap_ok` の Resource IR が `match` scrutinee として引数 `%r` そのものではなく `read %r -> tmp` を使うこと、さらに直後の `expr LocalRead` 注釈で既存の透明 alias 追跡が切れていたことだった。summary 収集では `Read` / `Move` / local initializer / assignment の透明な値 alias を追跡し、`LocalRead` などの注釈 op では alias を保持する。一方で call / construct / borrow / raw / match output は変換値として alias を切るため、任意の値変換を parameter variant と誤認しない。

`unwrap_ok dealloc data ...` では、`dealloc` が `Result::Ok` のときだけ raw owner を consume する pending effect を出し、`unwrap_ok` の summary が引数 0 の `Ok` 確定を呼び出し元で適用する。これにより checked cleanup API を `dealloc_raw` へ置き換えずに、Resource IR の owner discipline を保ったまま leak false positive を解消した。

検証:

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_resolves_unwrap_ok_raw_dealloc_consumption -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_ -- --nocapture`: 55 passed
- `node nodesrc/tests.js -i tests/stdlib/kp.n.md -o output/kp_after_unwrap_ok_dealloc_summary.json --runner wasm --no-tree -j 1 --assert-io`: total=7, passed=4, failed=1, errored=2。doctest#7 の `resource.owner.leak` は消滅し、残件は `ISS-20260430T012045983Z-RESOURCE-IR-CANNOT-SUMMARIZE-RETURNE-2FDA4B38` の doctest#3 dynamic range と `ISS-20260506T130138471Z-KP-STREAM-SCANNER-FLOAT-DOCTESTS-EXC-0D4A3BF8` の doctest#5/#6 timeout。

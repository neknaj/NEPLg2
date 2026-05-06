---
id: ISS-20260506T145720311Z-KP-PREFIX-SUM-DOCTEST-RELIES-ON-IMPL-5F1F3821
title: "KP prefix sum doctest relies on implicit dynamic buffer initialization"
area: test
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-06
target: "tests/stdlib/kp.n.md, nepl-core/tests/kp.rs"
---

# ISS-20260506T145720311Z-KP-PREFIX-SUM-DOCTEST-RELIES-ON-IMPL-5F1F3821: KP prefix sum doctest relies on implicit dynamic buffer initialization

## 概要

tests/stdlib/kp.n.md doctest#3 initializes only pref[0] and then relies on a loop/input convention to make later dynamic pref offsets initialized. Resource IR correctly rejects the later dynamic load because the source has no explicit range contract or full-buffer initialization.

## 対象

- `tests/stdlib/kp.n.md, nepl-core/tests/kp.rs`

## 根拠

- `ISS-20260430T012045983Z-RESOURCE-IR-CANNOT-SUMMARIZE-RETURNE-2FDA4B38` の調査中に、`tests/stdlib/kp.n.md::doctest#3` は `pref` の dynamic-offset load で `resource.cell.possibly_moved` を出していた。
- 同等の Rust regression `nepl-core/tests/kp.rs::kpread_to_kpwrite_prefixsum_i32` は `fill_i32 pref pref_len 0` で prefix buffer 全体を明示初期化しており、Resource IR initialized check を通過している。
- doctest 側は `store_i32 pref 0` だけで、以降の `pref[i]` 初期化を loop induction と入力制約に依存していた。現行 source には `l/r` の範囲 guard も range contract もないため、compiler がこれを暗黙に信じると dynamic offset safety を弱める。

## 問題

tests/stdlib/kp.n.md doctest#3 initializes only pref[0] and then relies on a loop/input convention to make later dynamic pref offsets initialized. Resource IR correctly rejects the later dynamic load because the source has no explicit range contract or full-buffer initialization.

## 影響

The KP doctest remains a compile blocker and can tempt an unsound compiler relaxation that treats arbitrary dynamic offsets as initialized.

## 修正方針

Align the doctest with the Rust KP regression by explicitly initializing the allocated prefix buffer with fill_i32 pref pref_len 0 before the prefix loop. Keep the broader guarded dynamic range summary issue open for a future typed proof model.

## 検証

Run tests/stdlib/kp.n.md and confirm doctest#3 no longer reports resource.cell.possibly_moved; remaining failures must be the existing float timeout issue.

## 2026-05-06 修正

`tests/stdlib/kp.n.md::doctest#3` の prefix buffer 初期化を `store_i32 pref 0` から `fill_i32 pref pref_len 0` へ変更した。これは Rust KP regression と同じ source discipline であり、compiler 側に「任意の dynamic offset を initialized とみなす」緩和を入れない。

`ISS-20260430T012045983Z-RESOURCE-IR-CANNOT-SUMMARIZE-RETURNE-2FDA4B38` の guarded dynamic range summary は引き続き open とする。将来の compiler 側対応は、明示 guard / typed range fact がある source だけを通す設計で進める。

検証:

- `node nodesrc/run_doctest.js -i tests/stdlib/kp.n.md -n 3 --dist web/dist`: passed, stdout は `6\n14\n15\n`
- `node nodesrc/tests.js -i tests/stdlib/kp.n.md -o output/kp_after_prefixsum_explicit_init.json --runner wasm --no-tree -j 1 --assert-io`: total=7, passed=4, failed=2, errored=1。doctest#3 は top issues から消滅。残件は `ISS-20260506T150445017Z-STRING-INTEGER-SPLIT-LOSES-RAW-MEMOR-36A59D71` の `from_u128_radix` boundary miss と `ISS-20260506T130138471Z-KP-STREAM-SCANNER-FLOAT-DOCTESTS-EXC-0D4A3BF8` の timeout。

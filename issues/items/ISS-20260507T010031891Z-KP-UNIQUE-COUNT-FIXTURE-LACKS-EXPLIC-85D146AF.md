---
id: ISS-20260507T010031891Z-KP-UNIQUE-COUNT-FIXTURE-LACKS-EXPLIC-85D146AF
title: "kp unique/count fixture lacks explicit raw range initialization proof"
area: TEST
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-07
updated: 2026-05-07
target: tests/stdlib/kp.n.md
---

# ISS-20260507T010031891Z-KP-UNIQUE-COUNT-FIXTURE-LACKS-EXPLIC-85D146AF: kp unique/count fixture lacks explicit raw range initialization proof

## 概要

tests/stdlib/kp.n.md::kpsearch_unique_and_count initializes a raw i32 buffer with fixed-offset stores, then reads it through a dynamic post-unique loop. Current Resource IR correctly refuses the later dynamic load because the fixture does not expose a typed initialized-range fact for the loop.

## 対象

- `tests/stdlib/kp.n.md`

## 根拠

- `tests/stdlib/kp.n.md::doctest#7` は `data` を `unwrap_ok alloc` で確保し、`store_i32 add data <known-offset> <value>` を 6 回行った後、`unique_sorted_i32` の戻り値 `new_len` を上限に `load_i32 add data (mul i 4)` で走査していた。
- Resource IR は exact offset store と、関数戻り値 `new_len` を介した dynamic post-unique loop の範囲関係をまだ dependent range summary として結び付けない。
- `ISS-20260430T012045983Z-RESOURCE-IR-CANNOT-SUMMARIZE-RETURNE-2FDA4B38` はこの親設計を追跡しているが、この fixture では runtime semantics 上、配列全体を Copy range として初期化してから exact value で上書きしても出力は変わらない。

## 問題

tests/stdlib/kp.n.md::kpsearch_unique_and_count initializes a raw i32 buffer with fixed-offset stores, then reads it through a dynamic post-unique loop. Current Resource IR correctly refuses the later dynamic load because the fixture does not expose a typed initialized-range fact for the loop.

## 影響

The KP focused suite remains red with resource.cell.uninit, and future work may be pushed toward weakening RawMemoryLoadCell instead of keeping initialized range proof explicit.

## 修正方針

Keep the parent dependent-range summary issue open, but update this fixture to establish the whole raw range with fill_i32 before overwriting exact elements. This preserves runtime semantics while giving Resource IR an explicit initialized Copy range.

## 検証

Run node nodesrc/tests.js -i tests/stdlib/kp.n.md --no-tree --dist web/dist -j 1 --assert-io and require the KP fixture suite to pass.

## 修正内容

- `kpsearch_unique_and_count` fixture で raw buffer 確保直後に `fill_i32 data len 0` を追加し、`data[0..len)` が initialized Copy range であることを source 上に明示した。
- 既存の fixed-offset `store_i32` はそのまま維持し、runtime の入力配列と期待 stdout は変更していない。
- Resource IR の `RawMemoryLoadCell` を緩める変更は行わず、dependent range summary 親 issue は open のまま残す。

## 検証結果

- `NEPL_TEST_CASE_TIMEOUT_MS=120000 node nodesrc/tests.js -i tests/stdlib/kp.n.md --no-tree --dist web/dist -o tmp/kp_agent1_after_unique_range_init.json -j 1 --assert-io`: total=7, passed=7, failed=0, errored=0
- `node nodesrc/tests.js -i tests/stdlib/kp.n.md --no-tree --dist web/dist -o tmp/kp_agent1_after_unique_range_init_default_timeout.json -j 1 --assert-io`: total=7, passed=7, failed=0, errored=0

## 関連 issue

- [ISS-20260430T012045983Z-RESOURCE-IR-CANNOT-SUMMARIZE-RETURNE-2FDA4B38](./ISS-20260430T012045983Z-RESOURCE-IR-CANNOT-SUMMARIZE-RETURNE-2FDA4B38.md)

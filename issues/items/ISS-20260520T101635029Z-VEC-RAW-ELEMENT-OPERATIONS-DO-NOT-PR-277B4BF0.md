---
id: ISS-20260520T101635029Z-VEC-RAW-ELEMENT-OPERATIONS-DO-NOT-PR-277B4BF0
title: "Vec raw element operations do not prove OwnedBuffer invariants before memory access"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-20
updated: 2026-05-20
target: "stdlib/alloc/collections/vec/**, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js"
---

# ISS-20260520T101635029Z-VEC-RAW-ELEMENT-OPERATIONS-DO-NOT-PR-277B4BF0: Vec raw element operations do not prove OwnedBuffer invariants before memory access

## 概要

Vec<T> の raw load/store 境界が len / initialized_len / cap / storage variant の整合性を明示的に検査せず、malformed owner aggregate が流れた場合に len > cap などから backing storage 範囲外 access を構成し得る。

## 対象

- `stdlib/alloc/collections/vec/**, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`

## 根拠

- `OwnedBuffer<T>` は `len` / `initialized_len` / `cap` / `storage` を持つが、修正前の `get` / `replace` / `push` / `pop` / transform / sort の raw load/store 境界は、これらの相関を 1 箇所で証明していなかった。
- `push` は `len == cap` だけで grow 分岐を決めるため、malformed metadata の `len > cap` が入ると non-grow path の raw store に進み得た。
- `sort` family は `len` から range を作って raw load/store traversal に入るため、`len <= cap` と storage variant の相関を先に確認する境界が必要だった。

## 問題

Vec<T> の raw load/store 境界が len / initialized_len / cap / storage variant の整合性を明示的に検査せず、malformed owner aggregate が流れた場合に len > cap などから backing storage 範囲外 access を構成し得る。

## 影響

collection Drop traversal 完成前の Copy-only Vec でも、Resource IR が raw memory span を証明する前提が source 側 invariant に依存してしまい、静的検査大規模修正の Stage D の安全境界が弱くなる。

## 修正方針

OwnedBuffer の current Copy-only invariant を enum/match と i32 条件で確認する helper を追加し、Vec の raw element load/store、transform、sort、push/pop/replace が invariant を満たす場合だけ raw memory operation へ進むようにする。

## 検証

Vec source policy と focused doctest で、raw access boundary が invariant helper を通ること、既存 Vec doctest が通ること、issues index が整合することを確認する。

## 2026-05-20 Agent 1 修正

`stdlib/alloc/collections/vec/invariant.nepl` を追加し、current Copy-only `Vec` の raw element access invariant を `vec_buffer_current_copy_invariant<T>` / `vec_current_copy_invariant<T>` に集約した。

この helper は `0 <= len == initialized_len <= cap`、`VecStorage::Empty` なら `len == 0 && cap == 0`、`VecStorage::Owned(_)` なら `cap > 0` を `match` で確認する。これにより raw memory boundary は `len` や `cap` の個別読み取りではなく、`OwnedBuffer<T>` 全体の相関を検査してから raw pointer view / load / store へ進む。

修正対象:

- `get` / `replace` は malformed invariant の場合に raw load/store を行わない。
- `push` は grow / raw store の前に invariant を検査し、失敗時は owner を `VecPushError` に戻す。
- `pop` / `drop_last` は raw load や initialized prefix 更新の前に invariant を検査し、不正時は元 owner を保持した結果を返す。
- `map` / `filter` / `partition` / `take_while` / `drop_while` は入力 invariant が不正な場合、出力 buffer 構築や raw copy/write に進まず owner-bearing error を返す。
- quick / heap / merge / simple sort family は各 public raw traversal entry で invariant を検査してから raw data view を導出する。

回帰テスト:

- `nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` に invariant helper と raw access boundary の source policy を追加した。sort family は各 public entry ごとに invariant 判定が raw data view / load / store / scratch allocation より前にあることを検査する。
- `vec/invariant.nepl` に public constructor 由来の valid Vec が invariant を満たす doctest を追加した。

focused verification:

- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_collection_cleanup_contract.js`: passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/invariant.nepl --no-tree --dist web/dist -o tmp/agent1-vec-invariant.json -j 1 --assert-io`: 1/1 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec.nepl --no-tree --dist web/dist -o tmp/agent1-vec-root-invariant.json -j 1 --assert-io`: 3/3 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/mutation/push.nepl --no-tree --dist web/dist -o tmp/agent1-vec-push-invariant.json -j 1 --assert-io`: 3/3 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/mutation/pop.nepl --no-tree --dist web/dist -o tmp/agent1-vec-pop-invariant.json -j 1 --assert-io`: 3/3 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/mutation/replace.nepl --no-tree --dist web/dist -o tmp/agent1-vec-replace-invariant.json -j 1 --assert-io`: 1/1 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/query/get.nepl --no-tree --dist web/dist -o tmp/agent1-vec-get-invariant.json -j 1 --assert-io`: 1/1 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/transform/map.nepl --no-tree --dist web/dist -o tmp/agent1-vec-map-invariant.json -j 1 --assert-io`: 1/1 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/transform/filter.nepl --no-tree --dist web/dist -o tmp/agent1-vec-filter-invariant.json -j 1 --assert-io`: 7/7 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/transform/prefix.nepl --no-tree --dist web/dist -o tmp/agent1-vec-prefix-invariant.json -j 1 --assert-io`: 2/2 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/sort.nepl --no-tree --dist web/dist -o tmp/agent1-vec-sort-facade-invariant-2.json -j 1 --assert-io`: 3/3 passed

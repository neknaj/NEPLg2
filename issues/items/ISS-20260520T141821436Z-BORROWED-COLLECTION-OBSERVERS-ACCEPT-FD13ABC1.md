---
id: ISS-20260520T141821436Z-BORROWED-COLLECTION-OBSERVERS-ACCEPT-FD13ABC1
title: "Borrowed collection observers accept non-Copy payload owners"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-20
updated: 2026-05-20
target: "stdlib/alloc/collections/**, nodesrc/test_stdlib_collection_cleanup_contract.js, tests/stdlib/collection_cleanup_contract.n.md"
parent: "ISS-20260425T000000Z-RV-STDLIB-004-91534828"
stage: "static-check-complexity-reduction Stage 6 / collection cleanup Copy-only boundary"
---

# ISS-20260520T141821436Z-BORROWED-COLLECTION-OBSERVERS-ACCEPT-FD13ABC1: Borrowed collection observers accept non-Copy payload owners

## 概要

collection の借用 observer は owner を移動しないため一見安全に見えるが、現行の `Vec<T>` / map / set / queue family は non-Copy payload の drop traversal、moved slot、compiler-issued owner token がまだ完成していない。その段階で `&Collection<NonCopy>` の scalar observer を公開すると、non-Copy collection owner を作れる surface と組み合わさり、未完成の所有権モデルを「読めるだけ」の API からも正当化してしまう。

今回の修正では、collection owner aggregate の借用 observer を関数型から横断検出し、payload generic に `Copy` bound を要求する source policy と compile-fail 回帰テストを追加した。

## 対象

- `stdlib/alloc/collections/vec/access/header.nepl`
- `stdlib/alloc/collections/vec/invariant.nepl`
- `stdlib/alloc/collections/vec/transform/filter/partition/view.nepl`
- `stdlib/alloc/collections/{stack,queue,deque,ringbuffer,list}/**`
- `stdlib/alloc/collections/{binary_heap,btreemap,btreeset,hashmap,hashset,bloom_filter,counting_bloom_filter}/**`
- `nodesrc/test_stdlib_collection_cleanup_contract.js`
- `tests/stdlib/collection_cleanup_contract.n.md`

## 根拠

- `Vec::len<T>(&Vec<T>) -> i32`、`cap`、`is_empty`、`VecCopyInvariant` observer、partition len observer が `.T: Copy` を要求していなかった。
- Stack / Queue / Deque / RingBuffer / BinaryHeap / List / BTreeMap / BTreeSet / HashMap / HashSet / BloomFilter 系の `len` / `cap` / `is_empty` も同様に、borrowed owner aggregate から scalar だけを返すため policy 対象から漏れていた。
- 既存 source policy は cleanup / owner-returning error accessor / pop accessor / owner-producing API / borrowed storage view に寄っており、`&Collection<T> -> i32|bool|proof` の observer surface を構造的に見ていなかった。
- 検査拡張後、BTreeMap storage/search helper も owner aggregate storage view を借用しつつ `.K` / `.V` の片側だけに `Copy` がないことが露出した。

## 問題

現行の collection 安全化は、non-Copy payload を本格対応する前の Copy-only 境界を置いている。ところが scalar observer が non-Copy payload collection を受け入れると、次の問題が残る。

- collection owner を作る API と observer を組み合わせることで、non-Copy payload collection が部分的に有効な API surface として見えてしまう。
- observer は要素を直接 move/drop しないが、内部 storage shape や invariant proof と結びついた owner aggregate を受け取るため、drop traversal 未完成の型を「安全に借用可能」と誤認させる。
- policy が関数名や個別 module ではなく関数型の構造で監視していなかったため、同種の observer 追加で再発する。

## 影響

- non-Copy payload collection の未完成な所有権モデルが stdlib API から露出し、後続の `OwnedBuffer<T>` / `InitializedCell` / Resource IR 接続時に互換のない設計債務になる。
- static check の正確性検証で、Copy-only で閉じているはずの collection surface に抜け穴が残る。
- doctest や user code が non-Copy collection を借用 observer に渡せてしまうと、将来の drop traversal 導入時に破壊的な再設計が必要になる。

## 修正方針

- `&Collection<...> -> i32|bool|VecCopyInvariant|Option<T>` の borrowed observer surface を source policy で構造検出する。
- collection owner aggregate に含まれる payload generic は、non-Copy drop traversal が完成するまで `Copy` bound を必須にする。
- Vec invariant / partition observer / map storage/search helper のような proof/helper layer も同じ境界で扱う。
- 回帰テストは `CleanupPayload` を使った compile-fail とし、特定関数の allowlist ではなく structural policy の expected inspected set で監視する。

## 修正内容

- Vec / Stack / Queue / Deque / RingBuffer / BinaryHeap / List / BTreeMap / BTreeSet / HashMap / HashSet / BloomFilter / CountingBloomFilter の borrowed scalar observer に `Copy` bound を追加した。
- `vec_buffer_current_copy_invariant<T>` / `vec_current_copy_invariant<T>` と filter partition len observer も `T: Copy` に揃えた。
- `BTreeMapStorage<K,V>` の key/value slot helper と storage search helper に、借用 storage view の両 payload 側 `Copy` bound を追加した。
- `nodesrc/test_stdlib_collection_cleanup_contract.js` に borrowed owner observer policy を追加し、対象 surface の inspected set を固定した。
- `tests/stdlib/collection_cleanup_contract.n.md` に non-Copy payload を observer / invariant / partition / map len へ渡す compile-fail regression を追加した。
- 検証中に既存 collection observer doctest の古い呼び出し書式が focused run を失敗させる別問題を確認したため、`ISS-20260520T142914786Z-COLLECTION-OBSERVER-DOCTESTS-STILL-U-19B433D2` として分離した。

## 検証

- `node nodesrc/test_stdlib_collection_cleanup_contract.js`: passed
- `node nodesrc/tests.js -i tests/stdlib/collection_cleanup_contract.n.md --no-tree --dist web/dist -o tmp/agent1-borrowed-collection-observer-contract-rerun.json -j 4 --assert-io`: 53/53 passed
- `node nodesrc/run_source_policy_regressions.js`: passed
- `node nodesrc/test_stdlib_documentation_contract.js`: passed (`declarationNoDoctest=1032`)
- `node nodesrc/issues.js check --dir issues`: passed
- `git diff --check`: passed

Focused observer doctest run for the modified collection API files still has legacy doctest failures unrelated to this boundary (`eq len ...` / hashset rehash capacity helper). It is tracked separately by `ISS-20260520T142914786Z-COLLECTION-OBSERVER-DOCTESTS-STILL-U-19B433D2`.

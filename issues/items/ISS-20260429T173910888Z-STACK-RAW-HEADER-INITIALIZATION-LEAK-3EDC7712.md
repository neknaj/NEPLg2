---
id: ISS-20260429T173910888Z-STACK-RAW-HEADER-INITIALIZATION-LEAK-3EDC7712
title: "Stack raw header initialization leaks helper pointer owners"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-30
target: "stdlib/alloc/collections/stack.nepl, nodesrc/test_stdlib_stack_no_unsafe_unwraps.js, tests/compiler/overload.n.md, tests/compiler/sizeof.n.md"
---

# ISS-20260429T173910888Z-STACK-RAW-HEADER-INITIALIZATION-LEAK-3EDC7712: Stack raw header initialization leaks helper pointer owners

## 概要

Stack::new / push kept the data buffer owner only as a raw address inside the header. Resource IR could track the header allocation, but `free` reconstructed the data pointer from the header as a non-owning view and failed with `NoFreeObligation`; the earlier helper-pointer initialization also encouraged false owner moves around header cells.

## 対象

- `stdlib/alloc/collections/stack.nepl, nodesrc/test_stdlib_stack_no_unsafe_unwraps.js, tests/compiler/overload.n.md, tests/compiler/sizeof.n.md`

## 根拠

- `tests/compiler/overload.n.md` の Stack fixture が、修正前は `Stack::new` / `Stack::push` / `Stack::free` 周辺の ResourceIR owner obligation で失敗していた。
- `Stack` の旧 layout は `hdr` しか field に持たず、`data` owner を header 内 raw address へ隠していたため、静的検査がデータ領域の最終解放責任を構造体 field として追えなかった。
- header offset helper (`len_ptr` / `cap_ptr` / `data_ptr_ptr`) は、owner ではない派生 pointer local を作り、初期化境界を不必要に複雑にしていた。

## 問題

Stack は実行時 metadata として header `[len, cap, data_ptr]` を使う一方で、ResourceIR が見る構造体 field は header owner だけだった。このため data allocation owner が header 内 raw address に押し込まれ、`free` / `push` / failure cleanup で「誰が data を所有しているか」を検査できない設計になっていた。

## 影響

Strict owner checking leaves overload Stack fixtures failing and forces either weakening ResourceIR or accepting untracked collection storage. Self-host collections need owner transfer and cleanup to be visible as typed fields so memory safety checks remain precise.

## 修正方針

Keep the runtime header layout, but make `Stack<.T>` carry both `hdr <MemPtr<u8>>` and `data <MemPtr<.T>>`. Use direct raw header stores for metadata, transfer the `data` owner through `new` / `push` / `clear`, and deallocate both owners in `free`. On `push` realloc failure, release the consumed storage before returning `Err`. Add source policy coverage for this owner contract and update compiler fixtures to observe with `len_ref` before `free`.

## 検証

Run the Stack source policy, focused overload/sizeof fixtures, compiler suite, issues check, and diff check.

## 修正内容

- `Stack<.T>` に `data <MemPtr<.T>>` field を追加し、データ領域の owner を header raw cell ではなく構造体 field として ResourceIR に見せる設計へ変更した。
- `new` は header offset 用の `MemPtr` helper local を作らず、`header_addr` へ `len/cap/data_addr` を直接 store し、`Stack<.T> header data` を返すようにした。
- `push` は `hdr` と `data` の owner field を明示的に取り出し、grow / non-grow の両 path で返却 Stack に owner を一度だけ移すようにした。
- `push` の realloc 失敗 path は、入力 Stack が消費済みになるため `data` と `hdr` を `dealloc_raw` してから `Err` を返すようにした。
- `push_ref` は借用中に owner field を差し替えられないため、容量不足時は再確保せず `Err` を返す仕様へ明確化した。
- `clear` / `free` は `hdr` と `data` の両 field を移動して、更新後 Stack または解放へ渡す形にした。
- `nodesrc/test_stdlib_stack_no_unsafe_unwraps.js` に、Stack が header/data owner を持つこと、`new` / `push` の owner transfer と failure cleanup を固定する source policy を追加した。
- `tests/compiler/overload.n.md` は Stack / Vec を `len_ref` で観測してから `free` する fixture に修正した。
- `tests/compiler/sizeof.n.md` は `Stack<i32>` が `hdr` + `data` の 2 pointer field になったため期待 size を 8 に更新した。

## 検証

- `trunk build`: passed
- `node nodesrc/test_stdlib_stack_no_unsafe_unwraps.js`: passed
- `node nodesrc/tests.js -i tests/compiler/sizeof.n.md --no-tree -o tmp/stack-owner-contract-sizeof-after.json -j 1 --dist web/dist`: total=9, passed=9
- `node nodesrc/tests.js -i tests/compiler/overload.n.md --no-tree -o tmp/stack-owner-contract-overload-final.json -j 1 --dist web/dist`: total=45, passed=44, failed=1。残りは tuple field owner obligation の既存 core 側問題で、Stack 由来の failure は解消。
- `node nodesrc/tests.js -i tests/compiler --no-tree -o tmp/compiler-after-stack-owner-contract-final.json -j 4 --dist web/dist`: total=649, passed=648, failed=1。残りは同じ `overload.n.md::doctest#10` の tuple field owner obligation。

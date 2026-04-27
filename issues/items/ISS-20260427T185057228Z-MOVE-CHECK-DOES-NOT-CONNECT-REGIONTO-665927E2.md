---
id: ISS-20260427T185057228Z-MOVE-CHECK-DOES-NOT-CONNECT-REGIONTO-665927E2
title: "move_check does not connect RegionToken dealloc to raw place state"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-27
updated: 2026-04-28
target: "nepl-core/src/passes/move_check.rs, tests/compiler/move_effect.n.md"
---

# ISS-20260427T185057228Z-MOVE-CHECK-DOES-NOT-CONNECT-REGIONTO-665927E2: move_check does not connect RegionToken dealloc to raw place state

## 概要

RegionToken values can carry the same raw storage provenance as MemPtr values, but move_check does not track RegionToken aliases or classify dealloc_region as a deallocation of the underlying raw place. A live non-Copy value can be stored through the MemPtr and then freed through RegionToken without D3100.

## 対象

- `nepl-core/src/passes/move_check.rs, tests/compiler/move_effect.n.md`

## 根拠

- `stdlib/core/mem.nepl` の `region_new<T>` は `MemPtr<T>` と size から `RegionToken<T>` を作る。
- `move_check` の raw place alias は i32 raw address と `MemPtr<T>` を主対象にしており、`RegionToken<T>` が指す raw place を call site state に保持していなかった。
- 修正前の `tmp/region-dealloc-drops-noncopy.nepl` では `region_new p size_of<T>` の後に `store<T> mem_ptr_addr p ...` し、`dealloc_region token` しても compiler が成功していた。

## 問題

RegionToken values can carry the same raw storage provenance as MemPtr values, but move_check does not track RegionToken aliases or classify dealloc_region as a deallocation of the underlying raw place. A live non-Copy value can be stored through the MemPtr and then freed through RegionToken without D3100.

## 影響

This leaves a memory-safety hole after the raw dealloc fix: region-based cleanup can still discard initialized payloads and bypass future Resource IR drop obligations.

## 修正方針

Track RegionToken raw-place aliases from region_new/RegionToken construction and copies, resolve region_ptr/region_size projections, classify dealloc_region calls, and run the same live non-Copy dealloc check used for raw and MemPtr dealloc.

## 検証

Add compile_fail regression for store through MemPtr followed by dealloc_region of the corresponding RegionToken, plus a passing regression after load consumes the payload.

## 対応結果

- `RegionToken<T>` を raw place alias の対象に追加し、`region_new` call、`RegionToken` struct construct、token copy から underlying raw place を引き継ぐようにした。
- `region_ptr token` と `get token "ptr"` が返す `MemPtr<T>` を、token の raw place に正規化するようにした。
- `dealloc_region<T> token` を raw dealloc event として分類し、raw/MemPtr dealloc と同じ live non-Copy dealloc 検査へ通すようにした。
- `tests/compiler/move_effect.n.md` に RegionToken dealloc の compile_fail と、`load<T>` 後に storage-only dealloc できる正常系を追加した。

## 実施した検証

- `cargo fmt --check`: pass
- `cargo check -p nepl-core`: pass
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/region-token-raw-dealloc-node.json -j 1`: `total=49`, `passed=49`
- 修正前再現ファイル `tmp/region-dealloc-drops-noncopy.nepl` は修正後 `D3100` で拒否されることを確認した。

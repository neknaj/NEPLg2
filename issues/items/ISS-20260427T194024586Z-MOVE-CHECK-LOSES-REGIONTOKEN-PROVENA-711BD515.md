---
id: ISS-20260427T194024586Z-MOVE-CHECK-LOSES-REGIONTOKEN-PROVENA-711BD515
title: "move_check loses RegionToken provenance through region_ptr_at Ok binding"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-27
updated: 2026-04-28
target: "nepl-core/src/passes/move_check.rs, tests/compiler/move_effect.n.md"
---

# ISS-20260427T194024586Z-MOVE-CHECK-LOSES-REGIONTOKEN-PROVENA-711BD515: move_check loses RegionToken provenance through region_ptr_at Ok binding

## 概要

region_ptr_at token off の Result::Ok payload を match で bind した MemPtr が、元の RegionToken / base MemPtr の raw place alias に接続されない。実行時に同じ storage を指す q から non-Copy owner を二重 load できる。

## 対象

- `nepl-core/src/passes/move_check.rs, tests/compiler/move_effect.n.md`

## 根拠

- `move_check` の `HirExprKind::Match` 処理は、arm の `bind_local` に borrow binding だけを保持し、raw address alias は設定していなかった。
- `region_ptr_at<T,U> token off` は `Result::Ok (mem_ptr_wrap (base_raw + off))` を返すが、Ok payload の `MemPtr<U>` を match bind した時点で `RegionToken` 由来の raw place provenance が失われていた。
- 修正前再現 `tmp/region-ptr-at-result-alias-double-load.nepl` では、`region_ptr_at token 0` の Ok payload `q` と base `p` から同じ `LocalToken` を二重 `load` しても compiler が exit 0 で受理した。

## 問題

region_ptr_at token off の Result::Ok payload を match で bind した MemPtr が、元の RegionToken / base MemPtr の raw place alias に接続されない。実行時に同じ storage を指す q から non-Copy owner を二重 load できる。

## 影響

RegionToken の bounds-checked pointer projection を使う safe-looking path で compiler の raw ownership state を迂回でき、self-host / collection storage の non-Copy payload を浅く複製できる可能性がある。

## 修正方針

match binding 時に、region_ptr_at の Result::Ok payload MemPtr を元 token の raw place + offset として正規化し、literal / non-literal offset の双方で existing raw place と overlap させる。

## 対応結果

- `move_check` の match binding で raw alias を設定できるようにした。
- `region_ptr_at` の `Result::Ok` arm payload は、元 `RegionToken` の raw place + offset として正規化するようにした。
- offset が literal の場合は known raw place、non-literal の場合は `base+?` の unknown-offset raw place として扱う。
- 直接 `EnumConstruct` を match する場合も、payload が raw alias を持つ値なら bind local に引き継ぐようにした。

## 検証

region_ptr_at Ok binding 経由の二重 non-Copy load / live payload dealloc を D3100 で拒否する回帰テストを追加する。

2026-04-28 実施:

- `cargo fmt --check`: pass
- `cargo check -p nepl-core`: pass
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/region-ptr-at-alias-node.json -j 1`: `total=73`, `passed=73`
- 修正前再現 `tmp/region-ptr-at-result-alias-double-load.nepl` は修正後 `D3100` で拒否されることを確認した。

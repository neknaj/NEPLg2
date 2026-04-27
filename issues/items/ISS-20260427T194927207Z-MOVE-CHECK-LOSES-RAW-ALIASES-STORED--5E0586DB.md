---
id: ISS-20260427T194927207Z-MOVE-CHECK-LOSES-RAW-ALIASES-STORED--5E0586DB
title: "move_check loses raw aliases stored in enum payload variables"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-27
updated: 2026-04-28
target: "nepl-core/src/passes/move_check.rs, tests/compiler/move_effect.n.md"
---

# ISS-20260427T194927207Z-MOVE-CHECK-LOSES-RAW-ALIASES-STORED--5E0586DB: move_check loses raw aliases stored in enum payload variables

## 概要

Result::Ok p のように MemPtr / raw address alias を enum payload に入れて変数へ束縛すると、match res の payload bind で alias が復元されない。直前の region_ptr_at direct match 修正だけでは、Result 変数を挟む経路が untracked のまま残る。

## 対象

- `nepl-core/src/passes/move_check.rs, tests/compiler/move_effect.n.md`

## 根拠

- `move_check` は raw alias を `i32` / `MemPtr` / `RegionToken` の変数にだけ保持し、`Result::Ok p` のような enum payload 内の raw alias を保存していなかった。
- 直前の `region_ptr_at` direct match 対応では scrutinee expression から Ok payload alias を計算したが、`let res = Result::Ok p; match res` のように enum 変数を挟むと復元できなかった。
- 修正前再現 `tmp/result-memptr-variable-alias-double-load.nepl` では、`Result::Ok p` を `res` に入れてから `match res` で得た `q` と `p` から同じ `LocalToken` を二重 `load` しても compiler が exit 0 で受理した。

## 問題

Result::Ok p のように MemPtr / raw address alias を enum payload に入れて変数へ束縛すると、match res の payload bind で alias が復元されない。直前の region_ptr_at direct match 修正だけでは、Result 変数を挟む経路が untracked のまま残る。

## 影響

Result<MemPtr<T>,E> や将来の self-host outcome のような enum wrapper を経由して、同じ raw storage から non-Copy owner を二重に作れる。compiler の raw ownership state が enum payload を越えられず、メモリ安全検査が不健全になる。

## 修正方針

MoveCheckContext に enum payload raw alias stack を追加し、let/set で enum payload alias を保存し、match bind 時に variant payload alias を bind local へ引き継ぐ。branch merge では全 continuing branch で一致する payload alias だけ保持する。

## 対応結果

- `MoveCheckContext` に `enum_payload_raw_alias_stacks` を追加し、変数 scope / snapshot / restore / branch merge に含めた。
- `let` / `set` で enum payload が raw alias を持つ場合、variant 名ごとに alias を保存するようにした。
- `region_ptr_at` の `Result::Ok` も、変数に束縛される場合は Ok payload alias として保存するようにした。
- `match` の payload bind では、scrutinee 変数に保存された enum payload alias を bind local に引き継ぐようにした。
- branch merge では全 continuing branch の同じ stack index で payload alias map が一致する場合だけ保持し、不一致なら空にする。

## 検証

Result::Ok MemPtr variable match 経由の二重 non-Copy load と、branch merge 後の alias 引き継ぎを D3100 回帰テストで確認する。

2026-04-28 実施:

- `cargo fmt --check`: pass
- `cargo check -p nepl-core`: pass
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/enum-payload-raw-alias-node.json -j 1`: `total=75`, `passed=75`
- 修正前再現 `tmp/result-memptr-variable-alias-double-load.nepl` は修正後 `D3100` で拒否されることを確認した。

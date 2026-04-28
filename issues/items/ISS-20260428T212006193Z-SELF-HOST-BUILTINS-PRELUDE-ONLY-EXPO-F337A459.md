---
id: ISS-20260428T212006193Z-SELF-HOST-BUILTINS-PRELUDE-ONLY-EXPO-F337A459
title: "self-host builtins prelude only exposes marker API"
area: selfhost
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-04-28
updated: 2026-04-28
target: stdlib/neplg2/core/builtins/prelude.nepl
---

# ISS-20260428T212006193Z-SELF-HOST-BUILTINS-PRELUDE-ONLY-EXPO-F337A459: self-host builtins prelude only exposes marker API

## 概要

stdlib/neplg2/core/builtins/prelude.nepl is still a marker module, so primitive type names and Rust builtin function metadata remain implicit hardcoded knowledge outside the self-host pipeline.

## 対象

- `stdlib/neplg2/core/builtins/prelude.nepl`

## 根拠

- `stdlib/neplg2/core/builtins/prelude.nepl` は `selfhost_prelude_stage0` だけを返す marker module で、後続 stage が参照できる builtin / primitive registry を持っていなかった。
- `nepl-core/src/builtins.rs` は Rust 側で `alloc` / `dealloc` / `realloc` を builtin として登録しているが、self-host 側には同じ名前、kind、arity、引数型、戻り値型、effect を取得する API がなかった。
- Rust loader の default prelude path は `std/prelude_base` だが、self-host 側でその値を共有する API もなかった。

## 問題

stdlib/neplg2/core/builtins/prelude.nepl is still a marker module, so primitive type names and Rust builtin function metadata remain implicit hardcoded knowledge outside the self-host pipeline.

## 影響

The resolver and checker cannot seed builtin functions such as alloc/dealloc/realloc or primitive names consistently, and later self-host stages would duplicate raw strings and ad hoc type tables.

## 修正方針

`SelfhostBuiltinKind`、`SelfhostBuiltinFunction`、`SelfhostPrimitiveTypeName` を追加し、primitive type canonical name と builtin function metadata を型付き registry として参照できるようにしました。

`alloc` / `dealloc` / `realloc` は現行 Rust `builtins()` と同じ signature metadata として固定し、`MemLoadI32` / `MemStoreI32` は Rust 側 enum には存在するが registry には未登録であることを index 範囲外 lookup の doctest で確認しています。

registry lookup はまず index API として固定しました。name lookup は `hash32` + `str_eq` で実装する予定でしたが、remote main の `RawMemoryLoadCell` gate 有効化後に `alloc/string` の `len(str)` 自体が D3100 になることを確認したため、`ISS-20260428T213732897Z-RAWMEMORYLOADCELL-GATE-REJECTS-INITI-6041ACF2` として分離し、この issue では string 依存を入れない typed registry までを完了範囲にしました。

実装中に、self-host type layer が Rust 側 primitive surface と完全には一致していないことも確認したため、`ISS-20260428T212439976Z-SELF-HOST-TYPE-KIND-LACKS-RUST-PRIMI-43D4589C` として分離しました。

また、remote main 取り込み後に `std/test` の raw backing-store scan が `RawMemoryLoadCell` gate で落ちることを確認したため、`ISS-20260428T213253278Z-STD-TEST-AGGREGATE-HELPERS-RAW-LOAD--F9E9112A` として分離しました。

## 検証

- `node nodesrc\tests.js -i stdlib\neplg2\core\builtins\prelude.nepl --no-tree -o tmp\selfhost-builtins-registry.json -j 1`: total=1 passed=1
- `trunk build`: pass
- `node nodesrc\tests.js -i stdlib\neplg2\core\builtins\prelude.nepl --no-tree -o tmp\selfhost-builtins-registry-after-rebase.json -j 1`: total=1 passed=1
- `node nodesrc\tests.js -i stdlib\neplg2 --no-tree -o tmp\selfhost-builtins-registry-neplg2-after-rebase.json -j 2`: total=32 passed=12 failed=20（`ISS-20260428T213732897Z...` / `ISS-20260428T213253278Z...` に分離）
- `node nodesrc\issues.js index`: total=323 open=12 resolved=311
- `node nodesrc\issues.js check`: pass, files=323
- `git diff --check`: pass

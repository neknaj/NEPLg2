---
id: ISS-20260513T062000411Z-RAW-MEMORY-BOUNDARY-CAPABILITY-REMAI-7C67C7C9
title: "Raw memory boundary capability remains on facade-only modules"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-13
updated: 2026-05-15
target: "nepl-core/src/loader.rs, nodesrc/test_stdlib_string_facade_boundary.js, nodesrc/test_stdlib_core_mem_boundary.js"
---

# ISS-20260513T062000411Z-RAW-MEMORY-BOUNDARY-CAPABILITY-REMAI-7C67C7C9: Raw memory boundary capability remains on facade-only modules

## 概要

Loader の raw-memory-boundary table が core/mem/types.nepl や alloc/string facade modules のような raw implementation body を持たない source にも capability を付与している。これは Stage 6 の移行中許可を必要以上に広げ、stdlib path 列挙に依存した境界を残す。

## 対象

- `nepl-core/src/loader.rs, nodesrc/test_stdlib_string_facade_boundary.js, nodesrc/test_stdlib_core_mem_boundary.js`

## 根拠

- `nepl-core/src/loader.rs` の `RAW_MEMORY_BOUNDARY_STDLIB_PATHS` に `core/mem/types.nepl`、`alloc/string.nepl`、`alloc/string/float.nepl`、`alloc/string/integer.nepl` が残っていた。
- これらは現行 source では public layout / facade / doctest を持つだけで、raw implementation body や checked wrapper の call site を所有しない。
- `nodesrc/test_stdlib_string_facade_boundary.js` は一部 facade capability を要求しており、source policy 自体が境界縮小の妨げになっていた。

## 問題

Loader の raw-memory-boundary table が core/mem/types.nepl や alloc/string facade modules のような raw implementation body を持たない source にも capability を付与している。これは Stage 6 の移行中許可を必要以上に広げ、stdlib path 列挙に依存した境界を残す。

## 影響

facade/type-only module に capability が残ると、将来そこへ関数本体が戻った場合に raw constructor や unsafe memory call が file 単位で許可され、source code に基づく最小境界の検査が弱くなる。

## 修正方針

raw implementation body や checked wrapper が存在しない facade/type-only module から raw-memory-boundary capability を外し、source policy を capability 不在を監視する方向へ更新する。

## 修正内容

- `RAW_MEMORY_BOUNDARY_STDLIB_PATHS` から `core/mem/types.nepl`、`alloc/string.nepl`、`alloc/string/float.nepl`、`alloc/string/integer.nepl` を削除した。
- core/mem policy は `types.nepl` に raw-memory-boundary capability が戻らないことを監視するようにした。
- alloc/string policy は root / integer facade / float facade に raw-memory-boundary capability が戻らないことを監視するようにした。
- loader effects regression は core/mem facade と alloc/string facade の raw body を `effect.pure.calls_impure` として拒否し、実装 boundary module は引き続き許可されることを確認する形へ更新した。

## 検証

- `cargo test -p nepl-core loader_ -- --nocapture`
- `node nodesrc/test_stdlib_string_facade_boundary.js`
- `node nodesrc/test_stdlib_core_mem_boundary.js`
- `trunk build`
- `node nodesrc/issues.js check`

## 2026-05-15 Agent 1 StringBuilder wrapper boundary 追記

`ISS-20260514T153830277Z-STRING-APPEND-BOUNDARY-POLICY-STILL--F431B18E` で、`nodesrc/test_stdlib_string_facade_boundary.js` が `stdlib/alloc/string/builder/append.nepl` に raw boundary evidence を要求していた stale policy を修正した。

同じ監査で `build.nepl` / `reserve.nepl` / `types.nepl` も direct raw operation を持たない `StringBuilder` wrapper 層であることを確認した。これらは現在、`ByteBuilder` / `ByteBuf` 境界へ storage mutation を委譲している。direct raw operation を持たない wrapper に raw evidence を要求すると、Stage 6 の source-based raw capability proof を必要以上に広げるため、これらの file は「raw evidence を持たない」側の監視対象に移した。

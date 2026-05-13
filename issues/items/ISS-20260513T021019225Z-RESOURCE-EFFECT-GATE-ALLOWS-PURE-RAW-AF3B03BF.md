---
id: ISS-20260513T021019225Z-RESOURCE-EFFECT-GATE-ALLOWS-PURE-RAW-AF3B03BF
title: "Resource effect gate allows pure raw dealloc without unsafe memory diagnostic"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-13
updated: 2026-05-13
target: "nepl-core/src/resource/effect_check.rs, tests/compiler/move_effect.n.md"
---

# ISS-20260513T021019225Z-RESOURCE-EFFECT-GATE-ALLOWS-PURE-RAW-AF3B03BF: Resource effect gate allows pure raw dealloc without unsafe memory diagnostic

## 概要

Resource IR effect boundary checks UnsafeMemory in pure functions and raw allocation identity escape, but InternalAlloc operations such as dealloc_raw, realloc_raw, mem_size, and mem_grow do not emit a diagnostic when used in a pure function and their raw identity does not escape. A pure function can therefore mutate allocator or memory state without being rejected.

## 対象

- `nepl-core/src/resource/effect_check.rs, tests/compiler/move_effect.n.md`

## 根拠

- `nepl-core/src/resource/effect_check.rs` は `EffectOp::UnsafeMemory` を pure 関数内で拒否していたが、`EffectOp::InternalAlloc` は count だけ記録していた。
- `alloc_raw` は raw identity が返却・格納などで public surface へ漏れる場合に `resource.raw.identity_escape` で拒否される。一方で `dealloc_raw p 4`、`realloc_raw p 4 8`、`mem_size`、`mem_grow` は返却 identity がなくても allocator / linear memory state を変更または観測できる。
- Stage 5 の internal effect fold は「compiler-owned internal allocation が外へ漏れない場合」だけ pure fold 可能であり、caller-provided raw address の dealloc / realloc や memory size observation は同じ扱いにできない。

## 問題

Resource IR effect boundary checks UnsafeMemory in pure functions and raw allocation identity escape, but InternalAlloc operations such as dealloc_raw, realloc_raw, mem_size, and mem_grow do not emit a diagnostic when used in a pure function and their raw identity does not escape. A pure function can therefore mutate allocator or memory state without being rejected.

## 影響

Pure functions can hide raw allocator mutation behind a pure surface, weakening Stage 5 internal-effect folding and making memory safety/effect safety depend on whether a raw identity is returned.

## 修正方針

Treat non-alloc InternalAlloc raw memory operations as pure-boundary violations in ResourceEffectBoundaryEngine while preserving alloc_raw internal allocation folding when the allocation identity does not escape.

## 検証

Add compile_fail regressions for pure dealloc_raw and pure realloc_raw and run the focused compiler doctest plus cargo check.

## 2026-05-13 修正

`ResourceEffectBoundaryEngine` で `InternalAlloc` の pure fold 条件を明示した。`RawMemoryOp::Alloc` は既存の raw identity escape 検査に委ね、identity が漏れない internal allocation として扱う。一方で `Dealloc` / `Realloc` / `MemorySize` / `MemoryGrow` は pure 関数内では `UnsafeMemoryInPureFunction` diagnostic に送る。

この分岐は `RawMemoryOp` の `match` で列挙し、将来 raw operation が増えた場合に compiler の網羅性検査が効く形にした。`load` / `store` / bulk copy / fill は従来通り `UnsafeMemory` 側の branch で扱うため、診断責務は混在させない。

回帰テストとして `tests/compiler/move_effect.n.md` に次を追加した。

- pure `dealloc_raw` が `effect.pure.calls_impure` で拒否される。
- pure `realloc_raw` が `effect.pure.calls_impure` で拒否される。
- pure `mem_size` が `effect.pure.calls_impure` で拒否される。

検証:

- `cargo fmt -p nepl-core --check`: passed
- `cargo check -p nepl-core --tests`: passed
- `trunk build`: passed
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/agent1-pure-raw-internal-alloc-move-effect.json -j 1 --dist web/dist`: 113/113 passed
- `node nodesrc/issues.js check --dir issues`: passed

関連ドキュメント:

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)

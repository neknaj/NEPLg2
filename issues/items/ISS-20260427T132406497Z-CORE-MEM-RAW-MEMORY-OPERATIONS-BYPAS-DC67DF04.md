---
id: ISS-20260427T132406497Z-CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS-DC67DF04
title: "core/mem raw memory operations bypass effect and ownership checks"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "nepl-core/src/effects.rs, nepl-core/src/typecheck.rs, nepl-core/src/passes/move_check.rs, nepl-core/src/passes/drop_insertion.rs, stdlib/core/mem.nepl, tests/compiler/move_effect.n.md"
---

# ISS-20260427T132406497Z-CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS-DC67DF04: core/mem raw memory operations bypass effect and ownership checks

## 概要

`stdlib/core/mem.nepl` は `alloc_raw` / `dealloc_raw` / `realloc_raw` / `load` / `store` を pure function signature として公開している。一方、`nepl-core` の effect 判定は既知 WASI call だけを impure とするため、raw memory 操作が pure 文脈から観測可能なまま呼べる。

## 対象

- `nepl-core/src/effects.rs, nepl-core/src/typecheck.rs, nepl-core/src/passes/move_check.rs, nepl-core/src/passes/drop_insertion.rs, stdlib/core/mem.nepl, tests/compiler/move_effect.n.md`

## 根拠

- `nepl-core/src/ast.rs` の `Effect` は `Pure` / `Impure` の 2 値だけで、`InternalAlloc` や `UnsafeMemory` の内部効果を表現できない。
- `nepl-core/src/effects.rs` の `intrinsic_effect` は既知 WASI marker だけを `Impure` とし、それ以外の intrinsic を `Pure` とする。
- `stdlib/core/mem.nepl` の `alloc_raw` / `dealloc_raw` / `realloc_raw` / `load<T>` / `store<T>` は `*` なしの pure signature で公開されている。
- `nepl-core/src/passes/move_check.rs` と `nepl-core/src/passes/drop_insertion.rs` は intrinsic `load` / `store` を field move などの局所 pattern として扱うが、任意 raw address が所有 place かどうかは追跡しない。
- `doc/compare/memory_model.md` は Phase 0 で `alloc/dealloc/realloc/load/store` を `Effect::Pure` から `Effect::InternalAlloc` へ移す計画を明記しているが、実装側 issue としては未分離だった。

## 問題

`move_check` と `drop_insertion` は intrinsic `load` / `store` を field move などの局所 pattern として special-case しているが、任意の `MemPtr` / raw address がどの owning place に属するかは追跡しない。そのため、raw memory から non-Copy 値を浅く読み出す経路や、pure 関数内で raw address identity を観測しながら allocate/free する経路を、effect / ownership 検査が正しく表現できない。

## 影響

pure source code が observable raw address を allocate / free / load / store でき、non-Copy 値を owned place 外の raw memory から浅く複製できる。self-host compiler の AST / diagnostic / buffer が owning value を増やすほど、effect、borrow、type safety の前提が崩れる。

## 修正方針

`InternalAlloc` / `UnsafeMemory` のような内部 memory effect を導入し、raw identity が観測できない場合だけ surface `Pure` へ畳み込む。raw `load` / `store` / `alloc` / `dealloc` は unsafe 層または compiler-owned boundary に閉じ込める。Resource IR では memory token / place を表現し、non-Copy raw load は unrestricted copy ではなく owning place からの move として扱う。

## 検証

raw identity が観測可能な public raw memory operation を pure function から呼ぶ compile_fail を追加する。同じ raw place から non-Copy 値を繰り返し `load` する case も、将来の明示 unsafe escape がない限り拒否する ownership test を追加する。`MemPtr` safe overload の正常系は別途維持する。

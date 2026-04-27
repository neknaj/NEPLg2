---
id: ISS-20260427T152947135Z-RAW-BODY-MEMORY-INSTRUCTIONS-BYPASS--162A8C00
title: "raw body memory instructions bypass pure effect validation"
area: core
status: open
resolved: false
priority: P0
type: bug
created: 2026-04-27
updated: 2026-04-28
target: "nepl-core/src/effects.rs, nepl-core/src/typecheck.rs, tests/compiler/raw_body_precheck.n.md"
---

# ISS-20260427T152947135Z-RAW-BODY-MEMORY-INSTRUCTIONS-BYPASS--162A8C00: raw body memory instructions bypass pure effect validation

## 概要

pure function の raw body 検査は raw body 内の `call` だけを見ており、`i32.load` / `i32.store` / `memory.grow` / LLVM `load` / `store` のような memory instruction を effect として分類していない。そのため、source 上は pure な関数から raw memory を直接読み書きする raw body を受理できる。

## 対象

- `nepl-core/src/effects.rs, nepl-core/src/typecheck.rs, tests/compiler/raw_body_precheck.n.md`

## 根拠

- `nepl-core/src/ast.rs:11` の `Effect` は `Pure` / `Impure` の 2 値だけで、compiler 内部 memory effect を表現できない。
- `nepl-core/src/effects.rs:58` の `intrinsic_effect` は既知 WASI marker だけを `Impure` とし、それ以外を `Pure` とする。
- `nepl-core/src/effects.rs:66` の `raw_body_direct_callees` は raw body の直接 `call` だけを抽出する。
- `nepl-core/src/typecheck.rs:2491` の `validate_raw_body_effect` は pure raw body で `raw_body_direct_callees` の callee だけを確認する。
- `nepl-core/src/typecheck.rs:2506` の `raw_callee_is_impure` は callee 名に対する effect 判定で止まり、raw memory instruction 自体を診断対象にしない。
- `tests/compiler/move_effect.n.md:3` には pure から `alloc_raw` / `store_i32` / `load_i32` / `dealloc_raw` を呼べる正常系があり、現行仕様が raw memory を pure 表面に露出している。

## 問題

raw body は compiler が型付き HIR の外側へ落とす escape hatch なので、ここで memory instruction を effect model に通さないと、`core/mem.nepl` 側で API を絞っても raw body 経由で同じ穴が残る。現在は `call` だけを検査しているため、pure raw body が直接 memory を mutate しても `TypePureCallsImpureFunction` に届かない。

## 影響

pure 関数が hidden mutable state を持てるため、effect / borrow / lifetime / ownership の前提が崩れる。特に allocator、raw cell、diagnostic buffer、self-host AST storage のような compiler 基盤で raw body を使うと、型検査済みの pure code から observable memory mutation や aliasing を作れる。

## 修正方針

raw body を構文または IR として解析し、memory instruction を `InternalAlloc` / `UnsafeMemory` 相当の内部 effect に分類する。まず `Effect` に内部 effect を追加し、surface へは `InternalAlloc -> Pure` のように畳み込む。pure raw body では memory instruction を原則拒否し、compiler-owned helper だけを明示的な内部 ABI として許可する。

## 検証

pure raw body に `i32.store` / LLVM `store` / `memory.grow` 相当を含めた compile_fail を追加する。既存の raw body call 検査が維持されること、compiler-owned allocator helper を許可する場合はその許可が public raw body へ漏れないことを回帰テストで固定する。

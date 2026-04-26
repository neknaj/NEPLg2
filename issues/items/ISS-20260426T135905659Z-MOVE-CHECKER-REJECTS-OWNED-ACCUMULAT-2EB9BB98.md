---
id: ISS-20260426T135905659Z-MOVE-CHECKER-REJECTS-OWNED-ACCUMULAT-2EB9BB98
title: "move checker rejects owned accumulator updated after fallible consuming loop call"
area: core
status: open
resolved: false
priority: P2
type: bug
created: 2026-04-26
updated: 2026-04-26
target: "nepl-core/src/passes/move_check.rs, nepl-core/src/typecheck.rs, stdlib/alloc/io.nepl"
source: "ByteBuilder implementation while handling ISS-20260426T010005Z-BYTEBUF-BUILDER-WASM-D63A7C11"
---

# ISS-20260426T135905659Z-MOVE-CHECKER-REJECTS-OWNED-ACCUMULAT-2EB9BB98: move checker rejects owned accumulator updated after fallible consuming loop call

## 概要

Owned accumulator values such as ByteBuilder cannot be updated through a fallible consuming call inside a loop without triggering D3065/D3054, even when the loop stops on Err and returns the last successful value.

## 対象

- `nepl-core/src/passes/move_check.rs, nepl-core/src/typecheck.rs, stdlib/alloc/io.nepl`

## 根拠

During ByteBuilder implementation, a loop in byte_builder_push_leb_u32 that repeatedly called byte_builder_push_u8 current out failed with D3065 potentially moved value current, and a doctest helper that accumulated ByteBuilder in a loop failed the same way for cur.

## 問題

The move checker marks the loop-carried owned accumulator as potentially moved because one iteration may pass it to a consuming function before the result branch reassigns the accumulator. It does not model the control-flow invariant that Err exits the loop and Ok always installs the replacement value before the next iteration or final return.

## 影響

Stdlib and self-host compiler code must avoid natural owned-builder accumulator loops and use recursion or hand-unrolled code. That hides the real ownership-flow limitation, makes binary emitters harder to write, and can force awkward workarounds in ByteBuilder, parser, and codegen code.

## 修正方針

Teach move/borrow analysis to track loop-carried ownership state across Result-match branches: after a consuming call, the old value is unavailable, but an Ok branch that assigns the replacement and an Err branch that exits should make the variable available on loop continuation and final success paths. Add CFG-level tests covering owned accumulator loops, break-on-Err, and use-after-Err rejection.

## 検証

Add core tests where an owned non-Copy value is repeatedly replaced in a while loop through Result::Ok and returned after the loop; add negative tests where an Err path can continue without replacement and must still be rejected.

---
id: ISS-20260604T102900000Z-HIR-FUNCTION-VALUE-REACHABILITY-SHOUL-6C0E9B12
title: "HIR function-value reachability should report precise unknown causes and merge branch candidates"
area: core
status: open
resolved: false
priority: P2
type: performance
created: 2026-06-04
updated: 2026-06-04
target: "nepl-core/src/compiler.rs; nepl-core/src/hir.rs; nepl-core/src/typecheck/indirect_apply.rs"
---

# ISS-20260604T102900000Z-HIR-FUNCTION-VALUE-REACHABILITY-SHOUL-6C0E9B12: HIR function-value reachability should report precise unknown causes and merge branch candidates

## 概要

`resource_reachable_prune` は、entry から到達しない stdlib 関数を Resource IR 静的検査の対象から外す。
この pruning は RPN cold base の Resource 検査時間を抑える重要な前段である。

2026-06-04 の RPN 高速化では、`CallIndirect` を一律 unknown call graph にする挙動を改め、
`@fn` と、非 mutable 引数へ直接渡された既知 function value だけは HIR reachability で追跡できるようにした。
ただし、branch / match で候補集合が分岐する場合や、function value がより複雑な式から戻る場合は、
まだ保守的に unknown へ落ちる。

## 対象

- `nepl-core/src/compiler.rs`
- `nepl-core/src/hir.rs`
- `nepl-core/src/typecheck/indirect_apply.rs`

## 根拠

- `stdlib/std/stdio/print_i32` から共通 formatter へ `@print_byte` を渡す試行で、
  formatter 内部の `emit` 呼び出しが `CallIndirect { callee: Var("emit") }` になり、
  旧実装では `resource_reachable_prune_functions=219 kept=219 reason=unknown_call_graph` へ落ちた。
- 同じ run では `resource_initialized_collection_slot_summaries` が約 `2480ms` まで膨らみ、
  cache hit 前提ではない cold base の探索範囲が大きく悪化した。
- 現行修正では非 mutable 関数引数への既知 function value 伝播を扱えるが、
  branch / match 後の候補集合合流、closure 的な戻り値、unknown 理由の分解までは扱っていない。

## 問題

`unknown_call_graph` が一種類だけだと、次のどれが原因で全体保守に落ちたのかを計測から判断しにくい。

- raw LLVM / raw wasm body による未知 call
- ambiguous mangled function reference
- `CallIndirect` callee が未知の変数
- branch / match で候補集合が合流できない function value
- function value が戻り値や mutable place へ escape した経路

さらに、候補集合を安全に合流できる場合でも unknown へ落ちると、
stdlib が高階関数を使うだけで未使用関数まで Resource 検査へ戻る。

## 修正方針

- `ReachableFunctionSet` に unknown reason を持たせ、stage timing で理由別に出す。
- HIR reachability の function-value environment を候補集合として扱い、branch / match の両枝が既知集合を返す場合は union する。
- mutable place、return、public storage、raw pointer、unknown callee へ escape した場合は、その変数だけ unknown とし、必要なときだけ全体保守へ落とす。
- direct `@fn`、immutable `let f @fn`、非 mutable parameter 伝播は現在の fast path を維持する。
- 候補集合が大きくなりすぎる場合は、閾値超過を明示 reason として conservative all にする。

## 検証

- `cargo test -p nepl-core resource_reachability --lib -- --nocapture`
- RPN cold base: `NEPL_DISABLE_CHECK_CACHE=1 NEPL_COMPILE_STAGE_TIMING=1 target\release\nepl-cli.exe --check -i examples\rpn.nepl`
- known direct `@fn`、known parameter callback、unknown variable、branch candidate union、mutable escape の unit test を追加する。
- unknown reason が `unknown_call_graph` だけでなく、原因別に出ることを stage timing で確認する。

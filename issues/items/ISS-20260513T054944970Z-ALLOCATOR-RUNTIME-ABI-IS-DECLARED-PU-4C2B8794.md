---
id: ISS-20260513T054944970Z-ALLOCATOR-RUNTIME-ABI-IS-DECLARED-PU-4C2B8794
title: "Allocator runtime raw identity returns need static summaries"
area: core
status: fixed
resolved: true
priority: P0
type: bug
created: 2026-05-13
updated: 2026-05-13
target: nepl-core/src/resource
---

# ISS-20260513T054944970Z-ALLOCATOR-RUNTIME-ABI-IS-DECLARED-PU-4C2B8794: Allocator runtime raw identity returns need static summaries

## 概要

alloc_raw / realloc_raw / __nepl_rt_alloc / __nepl_rt_realloc のような raw boundary 実装は、内部 allocator raw identity を構築して返す必要がある。一方で、その戻り値が一般の pure 関数から外へ出る場合は resource.raw.identity_escape として検出されなければならない。ABI を単純に impure 化すると内部 allocation を raw identity として保持する既存の設計意図を壊すため、raw boundary 実装の許可と、呼び出し先へ伝播する raw identity summary を分離する必要があった。

## 対象

- `nepl-core/src/resource/effect_summary.rs`
- `nepl-core/src/resource/effect_check.rs`
- `nepl-core/src/compiler.rs`
- `nepl-core/tests/resource_ir.rs`

## 根拠

- tests/compiler/move_effect.n.md が allocator.nepl 起点の resource.raw.identity_escape で先に停止し、MemPtr / raw cell の本来の検査へ進めなかった。
- raw boundary file capability は raw memory 命令の実装境界を示すものであり、境界外の pure API まで raw identity escape を許す根拠にはならない。
- allocator の raw pointer 返却は stdlib 関数名の allowlist ではなく、Resource IR の関数本体から「内部 allocation identity を返すか」を summary として計算できる。

## 問題

従来の ResourceEffectBoundary summary は「戻り値が引数の raw identity を返すか」だけを表現しており、「戻り値が関数内部で生成した allocation identity か」を呼び出し元へ伝播できなかった。そのため raw boundary 実装自身を許可すると、呼び出し元での raw identity escape を見逃す危険があり、逆に boundary 実装を許可しないと allocator stdlib 自身がコンパイルできなかった。

## 影響

Resource IR の effect boundary 検査が allocator 実装境界と一般利用側の区別を静的に扱えず、raw boundary の正当な実装を拒否するか、一般 pure 関数から内部 raw identity を逃がす不安全な実装を見逃すかのどちらかになっていた。

## 修正方針

Resource IR の関数本体から、戻り値が内部 allocation identity かどうかを `RawIdentityReturnSummary::returns_internal_alloc` として固定点計算する。呼び出し側は direct call / indirect call の戻り値にその identity を伝播し、境界外の pure 関数がそれを返す場合は引き続き resource.raw.identity_escape を報告する。raw-memory-boundary capability は、raw boundary 実装ファイル内の boundary diagnostic だけを許可し、一般ソースからの escape には適用しない。

## 修正内容

- raw identity return summary に `returns_internal_alloc` を追加した。
- summary 計算時に Resource IR 本体を走査し、`RawMemoryOp::Alloc` やそれを返す関数呼び出しから戻る内部 allocation identity を固定点で伝播するようにした。
- direct call / indirect call の effect check で、callee summary が内部 allocation identity を返す場合に call output を raw identity として扱うようにした。
- raw-memory-boundary capability を持つ source file では、raw boundary 実装中の `RawAddressEscapeFromInternalAlloc` を許可する一方、呼び出し元での escape は summary で検出する設計にした。
- `resource_ir_effect_check_propagates_internal_alloc_return_summary` を追加し、内部 allocation identity を返す pure 関数を呼び出した側でも escape が報告されることを固定した。

## 検証

- `cargo test -p nepl-core resource_effect_gate_allows_raw_identity_escape_inside_raw_boundary -- --nocapture`
- `cargo test -p nepl-core resource_ir_effect_check_propagates_internal_alloc_return_summary -- --nocapture`
- `cargo test -p nepl-core resource_ir_effect_check_reports_raw_alloc_return_escape -- --nocapture`
- `trunk build`
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/agent1-allocator-effect-abi-move-effect.json -j 1 --dist web/dist`

move_effect doctest は 99/113 pass まで進み、allocator.nepl 起点の resource.raw.identity_escape は解消した。残り 14 件は MemPtr / aggregate / Result / function return を経由した non-Copy raw load の二重 move を resource.cell.moved として検出できない別問題であり、`ISS-20260513T060220120Z-RESOURCE-CELL-ALIASES-MISS-MEMPTR-RA-DA8C864C` として切り出した。

---
id: ISS-20260428T105903144Z-RESOURCE-EFFECT-GATE-LOSES-RAW-SLOT--830E4162
title: "Resource effect gate loses raw slot pointer aliases returned by helpers"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "nepl-core/src/resource/effect.rs, tests/compiler/move_effect.n.md, nepl-core/tests/resource_ir.rs"
---

# ISS-20260428T105903144Z-RESOURCE-EFFECT-GATE-LOSES-RAW-SLOT--830E4162: Resource effect gate loses raw slot pointer aliases returned by helpers

## 概要

RawPointerAliasTable tracks local copies of raw slot pointers, but ResourceOp::Call and IndirectCall do not summarize parameter-to-return pointer aliases. A helper such as id_ptr(p): p can return a caller-provided slot pointer, after which store_i32 alias alloc_raw_value and load_i32 original_slot use the same address while the payload table sees different keys.

## 対象

- `nepl-core/src/resource/effect.rs, tests/compiler/move_effect.n.md, nepl-core/tests/resource_ir.rs`

## 根拠

- `doc/neplg2/static_check_complexity_reduction_plan.md` の Stage 5 は helper / callback 境界でも raw identity escape を途切れさせないことを求めている。
- `ISS-20260428T105154736Z-RESOURCE-EFFECT-GATE-KEYS-RAW-SLOT-P-9A800C94` で raw pointer alias table を追加したが、alias propagation は local copy / branch / match に限られていた。
- `fn slot_id(p): p` のような pure helper は raw memory を触らないため、callee 側だけでは問題に見えない。しかし caller がその戻り値を raw slot key として使うと、元の slot と同じ pointer value であることを payload table が失う。
- raw identity return summary と同様、raw pointer alias も parameter-to-return summary として関数境界を越える必要がある。

## 問題

RawPointerAliasTable tracks local copies of raw slot pointers, but ResourceOp::Call and IndirectCall do not summarize parameter-to-return pointer aliases. A helper such as id_ptr(p): p can return a caller-provided slot pointer, after which store_i32 alias alloc_raw_value and load_i32 original_slot use the same address while the payload table sees different keys.

## 影響

Pure helper abstractions can hide raw slot identity laundering behind pointer-returning helper functions. This leaves Stage 5 public escape diagnostics dependent on inlining-like local shapes instead of function boundaries.

## 修正方針

Add raw pointer parameter-to-return summaries alongside raw identity summaries, propagate them through direct and known indirect calls, and conservatively alias unknown indirect call outputs with their arguments. Keep raw allocation identity tracking separate from pointer alias tracking.

## 検証

cargo test -p nepl-core --test resource_ir -- --nocapture; trunk build; node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/stage5-raw-slot-pointer-return-summary.json -j 1; node nodesrc/issues.js check

## 2026-04-28 Stage 5 raw pointer return summary 対応

Resource IR effect boundary checker に raw pointer parameter-to-return summary を追加した。summary は direct call と known function value の indirect call で、戻り値がどの引数 pointer alias に由来し得るかを表す。unknown indirect call は callback が任意の引数 pointer を返す可能性を保守的に扱い、output を全引数と alias させる。

この summary は `RawPointerAliasTable` だけに作用し、`RawIdentityTable` の internal allocation identity とは分離している。これにより、pointer-returning helper から得た slot alias に internal allocation identity を store して、元 slot から load / return する経路も D3025 になる。

`tests/compiler/move_effect.n.md` に direct helper と function value 経由の returned raw slot pointer laundering 回帰を追加し、`nepl-core/tests/resource_ir.rs` に direct call summary の Resource IR 単体回帰を追加した。

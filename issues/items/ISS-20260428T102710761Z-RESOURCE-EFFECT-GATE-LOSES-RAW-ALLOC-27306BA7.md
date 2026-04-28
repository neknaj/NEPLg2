---
id: ISS-20260428T102710761Z-RESOURCE-EFFECT-GATE-LOSES-RAW-ALLOC-27306BA7
title: "Resource effect gate loses raw allocation identity through higher-order helper callbacks"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "nepl-core/src/resource/effect.rs, tests/compiler/move_effect.n.md, nepl-core/tests/resource_ir.rs"
---

# ISS-20260428T102710761Z-RESOURCE-EFFECT-GATE-LOSES-RAW-ALLOC-27306BA7: Resource effect gate loses raw allocation identity through higher-order helper callbacks

## 概要

Stage 5 raw identity escape detection handles direct calls and known function values, but a higher-order helper such as apply(p, f): f p has an unknown callback inside its own Resource IR summary. When caller passes alloc_raw identity and @raw_id, the helper summary does not mark its return as parameter-derived, so D3025 is missed.

## 対象

- `nepl-core/src/resource/effect.rs, tests/compiler/move_effect.n.md, nepl-core/tests/resource_ir.rs`

## 根拠

- `doc/neplg2/static_check_complexity_reduction_plan.md` は Stage 5 で unknown callback を保守的に扱い、raw identity が public surface へ漏れない場合だけ `InternalAlloc` を `Pure` へ fold する方針を示している。
- `nepl-core/src/resource/effect.rs` は direct call と known function value call には parameter-to-return raw identity summary を適用している。
- しかし `fn apply(p, f): f p` のような higher-order helper 自身を summary 化するとき、callee は function-typed parameter であり known function value alias を持たない。そのため helper の戻り値が `p` 由来になり得ることを summary へ記録できなかった。
- caller 側で `apply p @raw_id` のように concrete callback を渡していても、direct call の対象は `apply` なので、`apply` summary が空だと D3025 へつながらない。

## 問題

Stage 5 raw identity escape detection handles direct calls and known function values, but a higher-order helper such as apply(p, f): f p has an unknown callback inside its own Resource IR summary. When caller passes alloc_raw identity and @raw_id, the helper summary does not mark its return as parameter-derived, so D3025 is missed.

## 影響

Self-host and stdlib helper abstractions can hide raw allocation identity behind callback parameters, leaving a public pure-surface escape route even after direct call and known function value fixes.

## 修正方針

Treat unknown indirect callback returns conservatively in raw identity return summaries: if an indirect call receives a tracked raw identity argument and returns a value, its output may carry that identity. Add focused Resource IR and compiler regressions for apply p @raw_id.

## 検証

cargo test -p nepl-core --test resource_ir -- --nocapture; trunk build; node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/stage5-higher-order-raw-identity-summary.json -j 1; node nodesrc/issues.js check

## 2026-04-28 Stage 5 higher-order callback raw identity summary 対応

Resource IR effect boundary checker で、callee が known function value alias を持たない `ResourceOp::IndirectCall` を保守的に扱うようにした。indirect call に tracked raw identity 引数が渡される場合、その callback は同じ identity を返す可能性があるため、call output へ identity を伝播する。

この挙動は direct user function の raw identity return summary 計算にも効く。`fn apply(p, f): f p` は「戻り値が p identity に由来し得る」summary を持ち、caller 側で `alloc_raw` 由来 p を `apply p @raw_id` に渡すと、`apply` の direct call output が D3025 の対象になる。

`tests/compiler/move_effect.n.md` に higher-order helper 経由の compile_fail を追加し、`nepl-core/tests/resource_ir.rs` に Resource IR checker 単体の回帰を追加した。

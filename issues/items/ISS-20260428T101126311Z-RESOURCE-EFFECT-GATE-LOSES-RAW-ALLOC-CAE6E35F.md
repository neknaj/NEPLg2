---
id: ISS-20260428T101126311Z-RESOURCE-EFFECT-GATE-LOSES-RAW-ALLOC-CAE6E35F
title: "Resource effect gate loses raw allocation identity through pure helper calls"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "nepl-core/src/resource/effect.rs, tests/compiler/move_effect.n.md, nepl-core/tests/resource_ir.rs"
---

# ISS-20260428T101126311Z-RESOURCE-EFFECT-GATE-LOSES-RAW-ALLOC-CAE6E35F: Resource effect gate loses raw allocation identity through pure helper calls

## 概要

Stage 5 raw identity escape detection tracks local copies and aggregate construction, but ResourceOp::Call does not know whether a pure helper returns one of its raw address parameters. A caller can allocate raw memory, pass the address through an identity-like helper, and return the helper result without RawAddressEscapeFromInternalAlloc.

## 対象

- `nepl-core/src/resource/effect.rs, tests/compiler/move_effect.n.md, nepl-core/tests/resource_ir.rs`

## 根拠

- `doc/neplg2/static_check_complexity_reduction_plan.md` の Stage 5 は、`InternalAlloc` を `Pure` へ fold できる条件を raw identity が public surface へ漏れない場合に限定している。
- `nepl-core/src/resource/effect.rs` の `RawIdentityTable` は local copy、aggregate construction、branch / match value の伝播を扱っていたが、`ResourceOp::Call` は無視していた。
- `fn raw_id <(i32)->i32> (p): p` のような pure helper は raw address を新しく作らないため callee 側では問題に見えないが、caller が `alloc_raw` 由来 address を渡すと helper return が同じ raw identity を運ぶ。
- 関数境界の parameter-to-return identity summary がないと、Stage 5 public escape diagnostics は call で raw identity を途切れさせる。

## 問題

Stage 5 raw identity escape detection tracks local copies and aggregate construction, but ResourceOp::Call does not know whether a pure helper returns one of its raw address parameters. A caller can allocate raw memory, pass the address through an identity-like helper, and return the helper result without RawAddressEscapeFromInternalAlloc.

## 影響

Internal allocation can still be hidden behind a pure helper function boundary and escape the public surface. This weakens the compiler-side guarantee that InternalAlloc may fold to Pure only when raw identity stays internal.

## 修正方針

Add a Resource IR raw identity return summary for direct user functions, propagate parameter-derived identity through ResourceOp::Call, and add compiler/regression tests for helper-mediated allocation escape.

## 検証

cargo test -p nepl-core --test resource_ir -- --nocapture; trunk build; node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/stage5-call-raw-identity-summary.json -j 1; node nodesrc/issues.js check

## 2026-04-28 Stage 5 direct call raw identity summary 対応

Resource IR effect boundary checker に、direct user function の raw identity return summary を追加した。summary は「戻り値がどの引数 identity に由来し得るか」を関数ごとに固定点で計算し、caller 側の `ResourceOp::Call` で該当引数が internal allocation identity を持つ場合に call output へ identity を伝播する。

この summary は raw allocation 自体を parameter summary へ混ぜない。`alloc_raw` や `realloc` 由来の identity は caller の通常検査でだけ mark し、callee summary では parameter-derived identity だけを追跡する。これにより `load_i32 p` のような raw pointer を読む helper を、単に引数を返す helperと誤判定しない。

`tests/compiler/move_effect.n.md` に `raw_id` helper 経由で `alloc_raw` address を返す compile_fail を追加し、`nepl-core/tests/resource_ir.rs` に Resource IR checker 単体の direct call propagation 回帰を追加した。

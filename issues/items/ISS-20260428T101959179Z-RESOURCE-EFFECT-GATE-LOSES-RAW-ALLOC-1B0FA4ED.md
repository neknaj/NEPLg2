---
id: ISS-20260428T101959179Z-RESOURCE-EFFECT-GATE-LOSES-RAW-ALLOC-1B0FA4ED
title: "Resource effect gate loses raw allocation identity through function value calls"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "nepl-core/src/resource/effect.rs, tests/compiler/move_effect.n.md, nepl-core/tests/resource_ir.rs"
---

# ISS-20260428T101959179Z-RESOURCE-EFFECT-GATE-LOSES-RAW-ALLOC-1B0FA4ED: Resource effect gate loses raw allocation identity through function value calls

## 概要

Stage 5 raw identity escape detection now propagates direct user call summaries, but known function values such as let f @raw_id are lowered to ResourceOp::IndirectCall. The effect boundary checker does not track FunctionValue aliases, so alloc_raw identity can pass through f p and escape a pure function.

## 対象

- `nepl-core/src/resource/effect.rs, tests/compiler/move_effect.n.md, nepl-core/tests/resource_ir.rs`

## 根拠

- `doc/neplg2/static_check_complexity_reduction_plan.md` は Stage 5 の完了条件として、internal allocation の raw identity が public surface へ漏れない場合だけ surface `Pure` へ fold できることを求めている。
- `nepl-core/src/resource/effect.rs` は direct `ResourceOp::Call` に parameter-to-return raw identity summary を適用するようになったが、`ResourceOp::FunctionValue` と `ResourceOp::IndirectCall` は無視していた。
- `let f @raw_id; f p` は lowering 後に known function value alias を持つ indirect call になるため、direct call summary だけでは `alloc_raw` 由来 identity が途切れる。
- self-host 実装では higher-order helper や callback を使う可能性があるため、known function value の境界で検査が落ちると raw pointer discipline が public API へ漏れる。

## 問題

Stage 5 raw identity escape detection now propagates direct user call summaries, but known function values such as let f @raw_id are lowered to ResourceOp::IndirectCall. The effect boundary checker does not track FunctionValue aliases, so alloc_raw identity can pass through f p and escape a pure function.

## 影響

A raw allocation identity can still be hidden behind a first-class function value and leave the internal allocation boundary. This keeps a public-surface escape route open for higher-order helpers used by self-host code.

## 修正方針

Track known function value aliases in the Resource IR effect boundary checker, merge them through local copies and branches, and apply the existing raw identity return summary to known ResourceOp::IndirectCall targets.

## 検証

cargo test -p nepl-core --test resource_ir -- --nocapture; trunk build; node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/stage5-indirect-raw-identity-summary.json -j 1; node nodesrc/issues.js check

## 2026-04-28 Stage 5 known function value raw identity summary 対応

Resource IR effect boundary checker に known function value alias table を追加した。`ResourceOp::FunctionValue` で生成された関数名を place に紐づけ、`DeclareLocal` / `Read` / `Move` / `Assign` と branch / loop / match merge で alias を保持する。

`ResourceOp::IndirectCall` では、callee place が known function value alias を持つ場合に既存の parameter-to-return raw identity summary を適用する。これにより `let f @raw_id; f p` のような indirect call でも、`p` が `alloc_raw` 由来 identity なら call output へ identity が伝播し、pure function return で D3025 になる。

未知 callback は今回の修正では保守的に全部拒否する方向へは広げていない。known function value の false negative を塞ぐことをこの issue の範囲とし、function-typed parameter など caller で特定できない callback の扱いは別 issue で Resource IR summary / effect set 設計として扱う。

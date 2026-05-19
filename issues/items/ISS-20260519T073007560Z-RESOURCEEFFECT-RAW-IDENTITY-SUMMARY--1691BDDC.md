---
id: ISS-20260519T073007560Z-RESOURCEEFFECT-RAW-IDENTITY-SUMMARY--1691BDDC
title: "ResourceEffect raw identity summary replay drops TypeCtx"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-19
updated: 2026-05-19
target: "nepl-core/src/resource/effect_summary_identity.rs; nepl-core/src/resource/effect_summary_identity_replay_tests.rs; nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260519T073007560Z-RESOURCEEFFECT-RAW-IDENTITY-SUMMARY--1691BDDC: ResourceEffect raw identity summary replay drops TypeCtx

## 概要

raw identity return summary replay は `TypeCtx` を受け取る API を持っていたが、内部の `ResourceEffectBoundaryEngine` に `types: None` を渡していた。そのため summary replay 中の `Move` が Copy 型を通常 move として扱い、typed suffix / copy proof が効かない経路が残っていた。

## 対象

- `nepl-core/src/resource/effect_summary_identity.rs`
- `nepl-core/src/resource/effect_summary_identity_replay_tests.rs`
- `nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `function_returned_identity_projections_with_engine` は引数として `types: Option<&TypeCtx>` を受け取るが、旧実装では `ResourceEffectBoundaryEngine { types: None, ... }` を構築していた。
- `ResourceEffectBoundaryEngine::handle_move` は `TypeCtx` がある場合に Copy 型の move を identity invalidation から除外する。summary replay で `TypeCtx` を落とすと、実体の function check と summary replay の Copy semantics がずれる。
- `examples/nm.nepl` の stage timing 調査中にこの不整合を発見したが、修正後も full compile timeout は残ったため、performance 残件は `ISS-20260519T074504799Z-NM-FULL-COMPILE-STILL-EXCEEDS-CI-BUD-5653B487` に分離した。

## 問題

raw identity summary は caller 側で callee return identity を再生する証明 artifact である。ここで `TypeCtx` を落とすと、Copy 型、typed projection suffix、owner carrier 分類が本体検査と同じ条件で評価されず、summary を通った経路だけが異なる静的検査結果を持ち得る。

## 影響

Resource IR の summary replay が型証明を完全に消費しないため、静的検査の正確性が落ちる。特に Copy move 後の identity propagation が summary 経由で壊れると、generic helper / function value / summary application を通る場合だけ raw identity や owner provenance の結果が変わる。

## 修正方針

raw identity summary の internal engine に呼び出し元から受け取った `TypeCtx` を渡し、実体の ResourceEffectBoundaryEngine と同じ typed proof で summary replay を行う。Copy 型の move 後も identity が維持されることを regression test で固定する。

## 検証

- `cargo test -p nepl-core resource::effect_summary_identity_replay_tests::raw_identity_summary_replay_uses_typectx_for_copy_moves -- --nocapture`: pass
- `node nodesrc/issues.js check`: pass
- `node nodesrc/run_source_policy_regressions.js --warn-only`: pass
- `git diff --check`: pass

## 対応結果

- `function_returned_identity_projections_with_engine` で `ResourceEffectBoundaryEngine` に `types` を渡すようにした。
- regression として、Copy trait が有効な `i32` parameter を `Move` した後に raw identity summary call へ渡しても、summary replay が Copy move で source identity を消さないことを確認する unit test を追加した。
- `examples/nm.nepl` の full compile timeout はこの修正だけでは解消しなかったため、stage timing を保持したまま `ISS-20260519T074504799Z-NM-FULL-COMPILE-STILL-EXCEEDS-CI-BUD-5653B487` へ分離した。

## 関連

- Stage: `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 6
- remaining performance: `ISS-20260519T074504799Z-NM-FULL-COMPILE-STILL-EXCEEDS-CI-BUD-5653B487`

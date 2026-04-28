---
id: ISS-20260428T125920330Z-RESOURCE-EFFECT-CHECKER-LOSES-FUNCTI-6F8443ED
title: "Resource effect checker loses function aliases stored in aggregate fields"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "nepl-core/src/resource/effect.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260428T125920330Z-RESOURCE-EFFECT-CHECKER-LOSES-FUNCTI-6F8443ED: Resource effect checker loses function aliases stored in aggregate fields

## 概要

ResourceEffectBoundaryEngine tracks function aliases across locals, reads, assignments, branches, loops, and matches, but ResourceOp::Construct does not propagate a function value alias from an input to the corresponding aggregate field projection.

## 対象

- `nepl-core/src/resource/effect.rs`
- `nepl-core/tests/resource_ir.rs`
- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 5: effect model の拡張

## 根拠

- `ResourceEffectBoundaryEngine` は `FunctionAliasTable` を使い、`DeclareLocal` / `Read` / `Move` / `Assign` / branch / loop / match で known function alias を追跡している。
- 一方で `ResourceOp::Construct` は raw identity の aggregate root 集約だけを行い、function value alias を output field projection へ伝播していなかった。
- `IndirectCall` は callee に known alias がない場合、unknown callback fallback として raw identity 引数を戻り値へ保守的に伝播するため、known callback の情報喪失が false positive になる。

## 問題

ResourceEffectBoundaryEngine tracks function aliases across locals, reads, assignments, branches, loops, and matches, but ResourceOp::Construct does not propagate a function value alias from an input to the corresponding aggregate field projection.

## 影響

Indirect calls through struct fields, tuple fields, or enum payloads can be treated as unknown callbacks. The unknown callback fallback then propagates raw identities conservatively and can report RawAddressEscapeFromInternalAlloc for known callbacks that do not return the raw argument.

## 修正方針

Propagate function aliases during aggregate construction using the same deterministic field projection mapping as Resource IR owner construction, and add a regression where a known field-stored callback ignores a raw argument.

## 修正内容

- `ResourceOp::Construct` で input の known function alias を output の struct / tuple / enum payload field projection へ伝播するようにした。
- effect checker 内に aggregate kind と input index から deterministic な field place を作る helper を追加し、borrow/owner 側と同じ projection 表現に揃えた。
- known field-stored callback の summary を使えるようにし、raw 引数を返さない callback まで unknown callback fallback で raw escape 扱いしないようにした。

## 検証

- `resource_ir_effect_check_uses_known_function_alias_stored_in_aggregate_field` を追加した。
- 修正前は `RawAddressEscapeFromInternalAlloc { function: "main" }` が出て失敗することを確認した。
- 修正後に targeted regression は成功済み。最終確認として `resource_ir` 全体、`trunk build`、issue check、rustfmt check、diff check を実行する。

---
id: ISS-20260521T094024431Z-COLLECTION-SLOT-RETURN-SUMMARY-LOSES-5E121C4F
title: "Collection slot return summary loses path correlation for indirect calls"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-21
updated: 2026-05-21
target: "nepl-core/src/resource/collection_slot_summary_return_*.rs"
---

# ISS-20260521T094024431Z-COLLECTION-SLOT-RETURN-SUMMARY-LOSES-5E121C4F: Collection slot return summary loses path correlation for indirect calls

## 概要

collection slot return summary の逆追跡が branch/match 後の ResourceCheckState alternatives を使わず merged state だけで indirect call callee alias と collection slot state を読むため、実行不能な callee/state の組み合わせから return transfer / return slot を作る可能性がある。

## 対象

- `nepl-core/src/resource/collection_slot_summary_return_*.rs`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) Stage 6 は、collection slot state と owner transfer を stdlib module allowlist ではなく Resource IR の generic proof boundary で扱う方針を定めている。
- [ISS-20260521T082640712Z-COLLECTION-SLOT-INDIRECT-CALL-SUMMAR-6AB52846](./ISS-20260521T082640712Z-COLLECTION-SLOT-INDIRECT-CALL-SUMMAR-6AB52846.md) で main checker 側の path-correlated indirect call replay は修正済みだったが、return summary builder の逆追跡は同じ `ResourceCheckState` alternatives を保持していなかった。
- 本 issue は [ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md) の compiler-core 側残件であり、non-Copy collection payload support の owner-preserving return / fallible update に直結する。

## 問題

collection slot return summary の逆追跡が branch/match 後の ResourceCheckState alternatives を使わず merged state だけで indirect call callee alias と collection slot state を読むため、実行不能な callee/state の組み合わせから return transfer / return slot を作る可能性がある。

## 影響

non-Copy collection payload の initialized slot state が function boundary を跨ぐときに、実際には存在しない live slot を caller へ伝播したり、逆に path correlation を前提にした安全性判断を壊す。stdlib allowlist ではなく Resource IR summary の generic proof correctness の問題。

## 修正方針

return summary collection を ResourceCheckState alternatives aware にし、branch/match/indirect call 後も callee alias、raw alias、collection slot state を同じ feasible path から読む。

## 検証

focused Resource IR regression, cargo check -p nepl-core, cargo fmt --check, nodesrc/issues.js check

## 対応

2026-05-21 に修正した。

根本原因は、collection slot function summary が callee side-effect `ops` と `return_transfers` / `return_slots` を別々の flat list として保持し、caller replay 時に `ops` を path merge した後で return transfer を一括適用していたことだった。branch の then path だけで caller argument slot を initialize し、else path だけで identity callee を返す場合、then 側の live slot と else 側の return transfer が cross join され、実行不能な `MaybeInitialized` return slot が作られた。

修正後は `CollectionSlotLifecycleReturnPath` を追加し、return summary を path ごとの `ops`、`return_transfers`、`return_slots` として保持する。summary builder は branch / match / indirect call の return producer 逆追跡で `ReturnPathBuildState` を使い、callee alias、raw alias、collection slot state、return transfer を同じ feasible path から導出する。caller replay では通常の side effect は従来どおり summary ops として merge しつつ、call output の slot state だけは initial caller state から return path ごとに再生して merge するため、別 path の side effect と return transfer を混ぜない。

subagent review で、branch / match の `never` return arm と checked state の個数ずれ、return output 計算用 replay の diagnostics 重複が同じ種類の設計穴として見つかった。追加修正では control path 専用の `collection_slot_summary_return_path_control.rs` を分離し、branch / match の feasible arm ごとに selected ResourceOp を本体 checker と同じ semantics で評価する。これにより unreachable / `never` arm の summary ops は return path に入らない。さらに return output state の計算は `apply_collection_slot_lifecycle_summary_ops_state_only` を使い、callee side-effect diagnostics を本体 replay と output-state replay で二重に出さない。

これは stdlib 関数名や collection module 名の allowlist ではなく、Resource IR の `Branch` / `Match` / `IndirectCall`、`FunctionAliasTable`、`CollectionSlotStateTable` から導く generic proof 修正である。実装は return path model / value producer tracing / call summary composition / path state replay / slot translation / apply output merge に分割し、`nodesrc/test_resource_checker_responsibility.js` の監視対象へ追加した。

追加 regression:

- `resource_ir_collection_slot_return_summary_does_not_cross_join_indirect_callee_and_slot_path`
- `resource_ir_collection_slot_return_summary_skips_never_branch_path_effects`
- `resource_ir_collection_slot_return_summary_skips_never_match_arm_path_effects`
- `resource_ir_collection_slot_return_path_state_only_replay_does_not_duplicate_diagnostics`

検証:

- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_return_summary_does_not_cross_join_indirect_callee_and_slot_path -- --test-threads=1`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_return_summary_skips_never_branch_path_effects -- --test-threads=1`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_return_summary_skips_never_match_arm_path_effects -- --test-threads=1`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_return_path_state_only_replay_does_not_duplicate_diagnostics -- --test-threads=1`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_call_summary_ -- --test-threads=1`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_indirect_return_summary_preserves_path_correlation -- --test-threads=1`: pass
- `cargo check -p nepl-core`: pass
- `cargo fmt --check`: pass
- `node nodesrc/test_resource_checker_responsibility.js`: pass
- `node nodesrc/issues.js check --dir issues`: pass
- `git diff --check`: pass

---
id: ISS-20260505T132758518Z-RESOURCEIR-INITIALIZED-SUMMARIES-KEE-A65C9148
title: "ResourceIR initialized summaries keep unbounded exact storage-offset projections"
area: core
status: fixed
resolved: true
priority: P1
type: performance
created: 2026-05-05
updated: 2026-05-05
target: "nepl-core/src/resource/initialized_alias_flow.rs,nepl-core/src/resource/initialized_summary_build.rs,nepl-core/src/resource/initialized_projection_domain.rs"
---

# ISS-20260505T132758518Z-RESOURCEIR-INITIALIZED-SUMMARIES-KEE-A65C9148: ResourceIR initialized summaries keep unbounded exact storage-offset projections

## 概要

selfhost CLI driver doctest#2 の Resource IR initialized summary が raw pointer arithmetic の exact StorageOffset chain を増やし続け、alias/initialization/destruction summary の fixed point が巨大化して wasm emit が 180 秒以上完了しない。

## 対象

- `nepl-core/src/resource/initialized_alias_flow.rs,nepl-core/src/resource/initialized_summary_build.rs,nepl-core/src/resource/initialized_projection_domain.rs`

## 根拠

- 2026-05-05 の計測では、`tmp\selfhost_cli_driver_doctest2_latest.nepl` の wasm emit が `check_resource_initialized_moves` 内で進まなかった。
- raw alias summary のサンプルでは、`StorageOffset(Exact(1))` が返り値 projection に繰り返し追加され、iteration ごとに同じ pointer arithmetic を別 fact として保持していた。
- `lex_next__str_i32_i32_i32__SelfhostToken__pure` などの selfhost lexer path で alias summary が肥大化し、初期化/destruction summary にも同じ projection 増殖が伝播していた。
- exact offset を厳密に保持することは必要だが、summary fixed point の抽象domainとしては、同一形状で offset だけが増える列を finite widening へ落とさなければ収束性を保証できない。

## 問題

selfhost CLI driver doctest#2 の Resource IR initialized summary が raw pointer arithmetic の exact StorageOffset chain を増やし続け、alias/initialization/destruction summary の fixed point が巨大化して wasm emit が 180 秒以上完了しない。

## 影響

selfhost parser/driver のように MemPtr/string/Vec helper を多用する graph で静的検査の計算量が発散し、codegen timeout の根本原因が隠れる。

## 修正方針

StorageOffset projection を正規化し、小さい fixed-layout offset は exact のまま保持しつつ、同一形状の exact offset fact が閾値を超える場合だけ Dynamic summary を追加する有限domainにする。positive initialization facts は exact precision を維持し、alias / destruction / move のような保守的な effect summary で widening を使う。

## 検証

selfhost CLI driver doctest#2 extracted source が 240 秒 timeout ではなく約 105 秒で次の `resource.raw.unsafe_memory_boundary` 診断に到達することを確認する。widening domain の回帰は alias summary unit test と ResourceIR summary / exact-dynamic offset の focused tests で固定する。

## 対応結果

- `initialized_projection_domain.rs` を追加し、連続する `StorageOffset` を 1 つへ正規化する共通処理を置いた。
- exact + exact は overflow しない限り exact 和へ畳み込み、dynamic が混ざる場合や overflow 時は `ResourceOffset::Dynamic` へ落とす。
- alias summary の `RawCellAddressReturnAlias` では、parameter/return projection を正規化し、同一 parameter / type / projection shape の exact offset fact を一定数まで exact に保持し、その後は dynamic summary を追加して以降の増殖を止めるようにした。
- initialization summary の positive facts は exact を保持する。これは returned header pointer / external aggregate field などの既存 ResourceIR 回帰で、exact offset の初期化証明が必要なためである。
- initialization summary の param destruction / param move facts は同じ threshold widening domain を使い、destructive effect summary が unbounded exact offset 列にならないようにした。
- fixed point 更新順序は既存の全関数一括iteration semantics を維持し、今回の修正範囲を projection domain の有限化に絞った。

## 関連 issue

- `ISS-20260505T081814569Z-SELFHOST-CLI-DRIVER-DOCTEST-CODEGEN--052EB57C`: この issue の調査中に分離した親 performance issue。
- `ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84`: timeout 解消後に露出した stdlib raw-memory boundary migration 残件。

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)

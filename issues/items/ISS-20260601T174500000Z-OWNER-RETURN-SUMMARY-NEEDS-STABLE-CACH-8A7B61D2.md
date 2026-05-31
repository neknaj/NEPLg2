---
id: ISS-20260601T174500000Z-OWNER-RETURN-SUMMARY-NEEDS-STABLE-CACH-8A7B61D2
title: "Owner return summary needs stable value cache"
area: core
status: investigating
resolved: false
priority: P1
type: performance
created: 2026-06-01
updated: 2026-06-01
target: "nepl-core/src/resource/owner_summary.rs; nepl-core/src/resource/resource_summary_value_cache"
---

# ISS-20260601T174500000Z-OWNER-RETURN-SUMMARY-NEEDS-STABLE-CACH-8A7B61D2: Owner return summary needs stable value cache

## 概要

RPN same-session edit では、owner obligation の function check は diagnostic-free pass cache で skip できるようになった。しかし `resource_static_owner_obligations` はまだ秒単位で残り、支配項は `compute_owner_return_summaries` の全関数固定点計算に移った。owner return summary を stable mirror value として保存し、変更関数と依存 closure だけを再計算する必要がある。

## 根拠

- 2026-06-01 の `tmp/rpn_owner_obligation_cache_probe_final_20260601.json` では、RPN string literal edit が base `compile_ms=10801`、edit `compile_ms=3006`。
- edit delta は `resource_owner_obligation_function_checks=0`、`resource_owner_obligation_function_check_ops=0`、`resource_summary_value_owner_obligation_check_replay_hit_functions=288` であり、owner checker 本体は replay できている。
- それでも edit の `resource_static_owner_obligations=1534.075ms` が残った。これは cached pass の前に `compute_owner_return_summaries` が全関数に対して走るためである。

## 問題

`OwnerReturnSummary` は `TypeId`、`PlaceProjection`、owner extent、variant condition、host memory span、storage origin marker などを含む。これを session-local 値のまま `CompilerSession` に保存すると、別 compile の `TypeCtx` と `ResourceFunction` へ誤って再利用される危険がある。

したがって、単純な in-memory `Vec<OwnerReturnSummary>` cache は不可である。`ResourceSummaryValueCache` に保存する場合は、既存の raw-init / i32 scalar と同じく stable type key、function identity、function body hash、dependency closure hash、source capability policy hash、type boundary hash を持つ stable mirror entry に変換する必要がある。

## 修正方針

- `OwnerReturnSummary` の stable mirror value を設計し、`TypeId` と session-local place 表現を stable type key / parameter projection へ変換する。
- owner return summary 用の dependency closure kind を追加し、callee の owner summary に依存する caller が stale hit しないようにする。
- stable mirror に変換できない field は no-store / fail-closed とし、bypass reason counter を分ける。
- 初期実装では diagnostic replay ではなく summary replay だけを対象にし、owner obligation pass cache と組み合わせて `resource_static_owner_obligations` の edit 固定費を削る。
- `.neplproof` 永続 artifact へ進められる key 形状を維持し、session-local `TypeId` / `Span` / `OwnerStateEntry` を直接保存しない。

## 検証

- focused unit test で、unchanged owner return summary が replay され、function body edit と callee body edit では miss することを確認する。
- RPN same-session edit 測定で、`resource_static_owner_obligations` の edit stage が owner return summary cache なしの約 1.4 秒から明確に下がることを確認する。
- `node nodesrc/issues.js check --dir issues`、`git diff --check` を通す。

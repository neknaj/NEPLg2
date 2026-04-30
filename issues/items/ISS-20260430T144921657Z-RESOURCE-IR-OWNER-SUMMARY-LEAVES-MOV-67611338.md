---
id: ISS-20260430T144921657Z-RESOURCE-IR-OWNER-SUMMARY-LEAVES-MOV-67611338
title: "Resource IR owner summary leaves moved Result::Ok payload alias after unwrap pipeline"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-30
updated: 2026-04-30
target: "nepl-core/src/resource/owner_variant.rs, nepl-core/src/resource/owner_return.rs, nepl-core/src/resource/owner_flow.rs, nepl-core/src/resource/owner_transfer.rs, nepl-core/tests/resource_ir.rs, tests/compiler/overload.n.md"
---

# ISS-20260430T144921657Z-RESOURCE-IR-OWNER-SUMMARY-LEAVES-MOV-67611338: Resource IR owner summary leaves moved Result::Ok payload alias after unwrap pipeline

## 概要

tests/compiler/overload.n.md doctest#20 fails with resource.owner.use_after_move when a typed block pipelines Stack::new through unwrap_ok, push, and unwrap_ok. The final Stack initializer is resolved back to a moved Result::Ok payload projection, including duplicated owner projections.

## 対象

- `nepl-core/src/resource/owner_variant.rs, nepl-core/src/resource/owner_return.rs, nepl-core/tests/resource_ir.rs, tests/compiler/overload.n.md`

## 根拠

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_preserves_unwrap_push_pipeline_owner -- --nocapture` で、`Stack::new |> unwrap_ok |> push |> unwrap_ok` の owner が `tmp5.Ok.field2.field2.field0` の `Moved` alias へ戻り、`DeclareInitializer` の `resource.owner.use_after_move` として拒否されることを確認した。
- `push` の `OwnerReturnSummary` は `Result::Ok` payload projection を返却する一方で、元 `Stack` の同じ owner projection を `consumed_parameter_sources` にも持つ。従来の消費処理は raw alias 全体を retire するため、返却先 `Result::Ok` payload まで `Moved` にしていた。
- `unwrap_ok` は `parameter_sources=[EnumPayload(Ok)]` を使うため、Ok payload の live/maybe owner が残っていないと出力 `Stack` に owner を移せない。

## 問題

tests/compiler/overload.n.md doctest#20 fails with resource.owner.use_after_move when a typed block pipelines Stack::new through unwrap_ok, push, and unwrap_ok. The final Stack initializer is resolved back to a moved Result::Ok payload projection, including duplicated owner projections.

## 影響

Valid ownership transfer through Result-returning collection helpers is rejected. Weakening owner diagnostics would hide real use-after-move bugs, so Result payload owner summaries must transfer and retire aliases precisely.

## 修正方針

Resource IR owner use-after-move 診断は緩めない。関数 summary 適用時に、返却 projection と消費 projection が同じ raw owner alias group に含まれる場合は、戻り値側の projection を保護対象として扱い、元引数側だけを消費済みにする。

同時に、`unwrap_ok` のように variant payload を通常 call argument として読む summary でも、呼び出し前に pending variant owner return を materialize してから parameter source transfer を行う。これにより `match` だけでなく通常 call でも `Result::Ok` payload の owner が正規の出力 place へ移る。

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_preserves_unwrap_push_pipeline_owner -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_moves_result_payload_field_owner_to_match_bind -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_safe_realloc_variant_return_preserves_err_owner -- --nocapture`: passed
- `cargo check -p nepl-core`: passed
- `cargo fmt --check`: passed
- `trunk build`: passed
- `node nodesrc/run_doctest.js -i tests/compiler/overload.n.md -n 20 --dist web/dist`: passed
- `node nodesrc/tests.js -i tests/compiler/overload.n.md --no-tree -o tmp/overload-owner-pipeline-agent1.json -j 1 --dist web/dist`: total=45, passed=43, failed=2。対象の `doctest#20` は passed。残る `doctest#10` / `doctest#19` は `ISS-20260430T154405890Z-RESOURCE-IR-TUPLE-OWNER-PROJECTIONS--CCF76754` と `ISS-20260430T154415527Z-TYPED-BLOCK-CONTEXT-LOSES-VEC-LEN-RE-46F57311` に分離した。

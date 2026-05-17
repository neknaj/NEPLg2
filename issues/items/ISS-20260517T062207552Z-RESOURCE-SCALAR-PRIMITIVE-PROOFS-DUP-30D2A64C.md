---
id: ISS-20260517T062207552Z-RESOURCE-SCALAR-PRIMITIVE-PROOFS-DUP-30D2A64C
title: "Resource scalar primitive proofs duplicate call spellings"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/resource/scalar_primitive.rs, nepl-core/src/resource/address_projection.rs, nepl-core/src/resource/i32_call_facts.rs, nepl-core/src/resource/lower_condition.rs"
---

# ISS-20260517T062207552Z-RESOURCE-SCALAR-PRIMITIVE-PROOFS-DUP-30D2A64C: Resource scalar primitive proofs duplicate call spellings

## 概要

Resource IR consumers classify add/sub/mul and comparison/logical primitives by direct helper-name strings in several proof paths. Address projection coverage, raw-address offset propagation, i32 constant facts, and condition facts can drift from each other because each consumer owns its own spelling table.

## 対象

- `nepl-core/src/resource/scalar_primitive.rs, nepl-core/src/resource/address_projection.rs, nepl-core/src/resource/i32_call_facts.rs, nepl-core/src/resource/lower_condition.rs`

## 根拠

- `coverage_hir_projection.rs` は reference address projection を `Some("add" | "sub")`、reference field projection を `Some("add")` で判定していた。
- `lower_aggregate.rs` は `helper_base_name(name) != "add"` で reference field projection lowering を分岐していた。
- `i32_call_facts.rs` は `add` / `sub` / `mul` の constant / scale / difference fact を独自の call-target base-name classifier で分類していた。
- `lower_condition.rs` は `or` / `and` / `eq` / `ne` / `lt` / `le` / `gt` / `ge` の条件 fact を直接文字列 match していた。

## 問題

Resource IR consumers classify add/sub/mul and comparison/logical primitives by direct helper-name strings in several proof paths. Address projection coverage, raw-address offset propagation, i32 constant facts, and condition facts can drift from each other because each consumer owns its own spelling table.

## 影響

Static-check proof correctness depends on duplicated string branches instead of one typed primitive domain. A primitive spelling or semantic change can update one Resource IR proof path while leaving another stale, weakening memory-safety and initialized-range diagnostics.

## 修正方針

Introduce a Resource IR scalar primitive classifier with typed arithmetic, comparison, and boolean operators. Make address projection, raw-address offset propagation, i32 call facts, and condition fact lowering consume that classifier through exhaustive matches, and add source policy coverage against local string classifiers.

## 対応内容

- `resource/scalar_primitive.rs` を追加し、`I32ArithmeticPrimitive` / `I32ComparisonPrimitive` / `BooleanPrimitive` を Resource IR の scalar primitive domain として定義した。
- `AddressProjectionPrimitive` は `I32ArithmeticPrimitive` から導出するようにし、reference address / field projection の `add` / `sub` spelling を Resource IR consumer 側で重複管理しない形にした。
- raw address offset propagation、transparent raw address return projection、i32 constant / scale / difference fact、condition fact lowering は scalar primitive enum を `match` して処理するようにした。
- `resource_ir` の condition fact regression に `core/math` import を追加し、`lt` が未解決になる stale fixture を修正した。
- `nodesrc/test_resource_checker_responsibility.js` に scalar primitive module、Resource IR consumer の enum 使用、旧 local string classifier 再導入禁止を追加した。

## 検証

- `cargo fmt -p nepl-core --check`: pass
- `cargo check -p nepl-core`: pass
- `cargo test -p nepl-core i32_call_facts --lib -- --nocapture`: 3/3 passed
- `cargo test -p nepl-core --test resource_ir resource_ir_lowering_preserves_nonzero_i32_relation_condition_fact -- --exact --nocapture`: 1/1 passed
- `cargo test -p nepl-core --test resource_ir resource_ir_lowering_preserves_loop_i32_relation_condition_fact -- --exact --nocapture`: 1/1 passed
- `cargo test -p nepl-core --test resource_ir resource_ir_lowering_preserves_symbolic_mem_ptr_add_offset -- --exact --nocapture`: 1/1 passed
- `node nodesrc/test_resource_checker_responsibility.js`: pass
- `node nodesrc/test_static_check_boundary_responsibility.js`: pass
- `node nodesrc/issues.js check --dir issues`: pass
- `git diff --check`: pass（CRLF warning のみ）

---
id: ISS-20260517T000913110Z-SOURCE-CAPABILITY-RAW-PROOF-TREATS-I-23B11FA9
title: "source capability raw proof treats impl method names as raw helper evidence"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/source_capability/scope.rs, nepl-core/src/source_capability/raw_evidence_gate.rs, nepl-core/src/loader.rs"
---

# ISS-20260517T000913110Z-SOURCE-CAPABILITY-RAW-PROOF-TREATS-I-23B11FA9: source capability raw proof treats impl method names as raw helper evidence

## 概要

Source capability proof distinguishes local values and top-level callables, but impl method names are not represented in the module scope. A compiler-owned stdlib module can therefore contain an impl method whose name matches a raw helper such as load_i32, and a call-head use of that method name can be classified as raw memory operation evidence. This is an over-grant: raw authority should be proven from raw helper calls/raw bodies/top-level raw wrapper definitions, not from arbitrary method names.

## 対象

- `nepl-core/src/source_capability/scope.rs, nepl-core/src/source_capability/raw_evidence_gate.rs, nepl-core/src/loader.rs`

## 根拠

- `SourceCapabilityScope` は top-level function / alias と local value を区別していたが、`impl` method 名を module scope に登録していなかった。
- `load_i32` など raw helper 名と同名の `impl` method が compiler-owned source にあると、method body 内の call-head symbol が shadowed symbol として扱われず、raw memory operation evidence になり得た。
- raw helper wrapper の自己証明は top-level raw helper function の body evidence に限定すべきであり、任意の method 名から raw authority を付与してはいけない。

## 問題

Source capability proof distinguishes local values and top-level callables, but impl method names are not represented in the module scope. A compiler-owned stdlib module can therefore contain an impl method whose name matches a raw helper such as load_i32, and a call-head use of that method name can be classified as raw memory operation evidence. This is an over-grant: raw authority should be proven from raw helper calls/raw bodies/top-level raw wrapper definitions, not from arbitrary method names.

## 影響

RawMemoryOperationBoundary can be granted from source spelling instead of a real raw primitive proof, weakening the static check boundary and making errors in the checker harder to catch statically.

## 修正方針

Add a distinct source capability binding kind for impl methods or otherwise bind impl method names as non-raw evidence in SourceCapabilityScope. Keep same-name top-level raw helper wrappers valid, but reject impl method name evidence. Add a loader regression proving an impl method named load_i32 does not grant RawMemoryOp::Load.

## 検証

cargo test -p nepl-core loader::tests::raw_memory_boundary_ignores_impl_method_raw_helper_names -- --exact --nocapture; cargo test -p nepl-core loader::tests::raw_memory_boundary_accepts_same_name_raw_helper_wrapper_evidence -- --exact --nocapture; node nodesrc/test_static_check_boundary_responsibility.js; node nodesrc/issues.js check --dir issues

## 2026-05-17 Agent 1 修正

`SourceCapabilityBindingKind` を `source_capability/binding.rs` に分離し、`TopLevelCallable` / `ImplMethod` / `LocalValue` の enum で source proof 用の名前束縛を表現した。

修正内容:

- `SourceCapabilityScope::from_module` が `impl` method 名を `ImplMethod` として登録するようにした。
- `ImplMethod` は raw helper self-proof を許可しない。一方で同名 top-level raw helper wrapper は `TopLevelCallable` として維持し、body に raw operation evidence がある場合だけ従来通り許可する。
- `bind_symbol_kind` は `LocalValue` と `TopLevelCallable` を優先し、`ImplMethod` は既存の top-level function / alias を上書きしない。
- `loader::tests::raw_memory_boundary_ignores_impl_method_raw_helper_names` を追加し、`impl i32: fn load_i32` が `RawMemoryOp::Load` capability を付与しないことを固定した。
- `nodesrc/test_static_check_boundary_responsibility.js` に `source_capability/binding.rs` を監視対象として追加し、binding kind の enum 管理と行数上限を policy 化した。

検証:

- `cargo test -p nepl-core loader::tests::raw_memory_boundary_ignores_impl_method_raw_helper_names -- --exact --nocapture`: pass
- `cargo test -p nepl-core loader::tests::raw_memory_boundary_accepts_same_name_raw_helper_wrapper_evidence -- --exact --nocapture`: pass
- `cargo fmt -p nepl-core --check`: pass
- `node nodesrc/test_static_check_boundary_responsibility.js`: pass

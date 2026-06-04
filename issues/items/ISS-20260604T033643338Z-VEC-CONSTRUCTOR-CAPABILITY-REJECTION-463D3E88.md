---
id: ISS-20260604T033643338Z-VEC-CONSTRUCTOR-CAPABILITY-REJECTION-463D3E88
title: "Vec constructor capability rejection doctests lost PlainPayload coverage"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-06-04
updated: 2026-06-04
target: stdlib/alloc/collections/vec/storage/api.nepl, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js
---

# ISS-20260604T033643338Z-VEC-CONSTRUCTOR-CAPABILITY-REJECTION-463D3E88: Vec constructor capability rejection doctests lost PlainPayload coverage

## 概要

`node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` reports that Vec allocation constructors must still reject payloads with neither Copy nor Drop capability in doctests. The `new<PlainPayload>` and `with_capacity<PlainPayload>` compile-fail coverage is present and passes, but the source policy expected the obsolete diagnostic code `type.overload.no_match` instead of the current `type.trait_bound.unsatisfied` contract. This conflicts with the Zenn policy because the static check should verify the real trait capability rejection rather than a stale diagnostic spelling.

## 対象

- `stdlib/alloc/collections/vec/storage/api.nepl`
- `nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`

## 根拠

- `stdlib/alloc/collections/vec/storage/api.nepl` already has `compile_fail` doctests for `new<PlainPayload>` and `with_capacity<PlainPayload>`.
- Focused doctest execution for `stdlib/alloc/collections/vec/storage/api.nepl` passes 9/9 with `diag_codes: type.trait_bound.unsatisfied`.
- The overload candidates exist (`.T: Copy` and `.T: Drop`), but `PlainPayload` satisfies neither trait bound. Therefore the precise diagnostic is a trait-bound rejection, not missing overload.

## 問題

The source policy was stale. It required `type.overload.no_match`, so correct `PlainPayload` compile-fail blocks using `type.trait_bound.unsatisfied` were treated as missing coverage. This made the regression test less accurate and could push the docs toward an incorrect diagnostic contract.

## 影響

Collection APIs may regress toward accepting payload types whose lifetime cannot be statically cleaned up, reopening non-Copy owner/drop holes that previous Resource IR work tried to close.

## 修正方針

Keep the stdlib constructor bounds and doctest diagnostic contract unchanged. Update the source policy to inspect `compile_fail` blocks and require `PlainPayload` rejection for both `new` and `with_capacity` through `type.trait_bound.unsatisfied`.

## 検証

- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: pass
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/storage/api.nepl --no-tree -o tmp/agent2-vec-storage-api-after2.json -j 1 --dist web/dist --assert-io`: total=9, passed=9, failed=0
- `node nodesrc/test_stdlib_collection_cleanup_contract.js`: pass
- `node nodesrc/tests.js -i tests/stdlib/collection_cleanup_contract.n.md --no-tree -o tmp/agent2-vec-collection-cleanup-contract.json -j 1 --dist web/dist --assert-io`: total=54, passed=54, failed=0
- `node nodesrc/run_source_policy_regressions.js --warn-only`: `nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` pass。既存 warning は 12 件から 11 件へ減少
- `node nodesrc/issues.js index --dir issues && node nodesrc/issues.js check --dir issues`: pass
- `git diff --check`: pass
- `trunk build`: pass
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/agent2-vec-constructor-playground-editor.json`: 13/13 pass

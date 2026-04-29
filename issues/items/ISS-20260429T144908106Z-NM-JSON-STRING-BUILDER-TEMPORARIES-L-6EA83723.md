---
id: ISS-20260429T144908106Z-NM-JSON-STRING-BUILDER-TEMPORARIES-L-6EA83723
title: "Resource owner summary treats scalar bool returns as free obligation owners in nm JSON"
area: core
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "nepl-core/src/resource/owner_summary_leaf.rs, nepl-core/tests/resource_ir.rs, stdlib/nm/parser.nepl, stdlib/alloc/string.nepl, examples/nm.nepl"
---

# ISS-20260429T144908106Z-NM-JSON-STRING-BUILDER-TEMPORARIES-L-6EA83723: Resource owner summary treats scalar bool returns as free obligation owners in nm JSON

## 概要

After cliarg out-pointer fixes, `examples/nm.nepl` failed `nm-compile` with `resource.raw.ownership_violation` in `document_to_json` and `sb_build_result`. Initial review found a real `sb_build_result` failure-path leak, but the remaining `document_to_json` diagnostics were a core false positive: Resource IR owner return summaries treated scalar `bool` parameters and return values as if they could carry free obligations.

## 対象

- `nepl-core/src/resource/owner_summary_leaf.rs`
- `nepl-core/tests/resource_ir.rs`
- `stdlib/nm/parser.nepl`
- `stdlib/alloc/string.nepl`
- `examples/nm.nepl`

## 根拠

- `cargo run -p nepl-cli -- --target wasi --profile debug --input examples/nm.nepl --output target/ci-nm` reported four `OwnerMaybeLeaked` diagnostics in `document_to_json` after the cliarg issue was fixed.
- A temporary Resource IR dump showed the reported `Temporary(ResourceId(...))` values corresponded to `nm_json_needs_comma` `bool` call outputs used as branch conditions, not owned strings.
- `owner_summary_leaf.rs` classified `Bool`, `U8`, `F32`, `Char`, and function values as owner leaves. That made helpers returning scalar parameters produce owner-return summaries and leak false free obligations in callers.
- `sb_build_result` also had a real failure-path issue: after allocating `out_region`, `mem_copy<u8>` failure returned `Err` without deallocating the output region.

## 問題

Resource IR owner summaries were too broad for non-owning scalar types. Treating `bool` as an owner leaf caused ordinary control-flow predicates to be propagated as possible free obligations. Separately, `sb_build_result` did not release its allocated output region on copy failure.

## 影響

`examples/nm.nepl` could not compile under the strict Resource IR owner gate, blocking nm/self-host validation. The scalar false positive also risked making any helper that returns `bool`/`char`/`u8`/`f32` look like an ownership transfer.

## 修正方針

Keep the owner gate strict, but classify owner leaves more accurately. Non-owning scalar values must not create free-obligation summaries. Preserve the transitional `i32` raw-pointer owner behavior for existing raw-memory summaries. Fix the real `sb_build_result` failure-path leak and reduce NM JSON temporary string construction by appending escaped content directly into builders.

## 検証

## 修正内容

- `owner_summary_leaf.rs` で `Bool` / `U8` / `F32` / `Char` / function value を owner leaf から外した。`i32` は現行 raw pointer 互換のため、この issue では維持した。
- `resource_ir_owner_summary_does_not_treat_bool_parameters_as_owners` を追加し、`bool` parameter を返す helper が caller 側へ owner obligation を作らないことを固定した。
- `sb_build_result` の `mem_copy<u8>` 失敗 path で `out_region` を `dealloc_region<u8>` するようにした。
- `sb_append_byte(_result)` と `json_escape_into` / `json_escape_builder_into` / `nm_inline_to_json_into` を追加し、NM JSON 出力で不要な中間 `str` を作らず builder へ直接追加できる境界を作った。
- `str_slice_trim_suffix_cr` を追加し、`str_trim_suffix_cr str_slice ...` の不要な中間行文字列を避けた。
- CI source policy の `json_escape` match 検査を、現在の責務分割に合わせて `json_escape_byte_into` の match 検査へ移した。公開 `json_escape` / `json_escape_into` / `json_escape_mem_into` は builder/byte-range helper へ委譲することも同じ policy で固定した。
- CI source policy の `nm_inline_to_json` match 検査も、現在の責務分割に合わせて `nm_inline_to_json_into` の match 検査へ移した。公開 wrapper は `sb_build nm_inline_to_json_into string_builder_new s` を通ることを固定した。

## 検証

- `cargo run -p nepl-cli -- --target wasi --profile debug --input examples/nm.nepl --output target/ci-nm`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_ -- --nocapture`: 33 passed
- `cargo check -p nepl-core --tests`: passed
- `trunk build`: passed
- `node nodesrc/tests.js -i stdlib/nm/parser.nepl --no-tree -o tmp/nm-parser-owner-boundary.json -j 1 --dist web/dist`: total=3, passed=3
- `node nodesrc/test_stdlib_match_decision_trees.js`: passed
- `node nodesrc/test_stdlib_nm_parser_no_inline_unwraps.js`: passed
- `node nodesrc/tests.js -i stdlib/alloc/string.nepl --no-tree -o tmp/string-owner-boundary.json -j 1 --dist web/dist`: unrelated existing failures remain in checks owner leak / doctest type mismatch; this run is not used as this issue's pass condition.

---
id: ISS-20260505T154332456Z-ALLOC-IO-AND-STD-TEXT-SAFE-WRAPPERS--FA2B4CA6
title: "alloc/io and std/text safe wrappers lack raw operation boundary"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-05
updated: 2026-05-06
target: "nepl-core/src/loader.rs, stdlib/alloc/io.nepl, stdlib/std/text.nepl, tests/stdlib/selfhost_cli_file_io.n.md, tests/stdlib/text_utf8.n.md"
---

# ISS-20260505T154332456Z-ALLOC-IO-AND-STD-TEXT-SAFE-WRAPPERS--FA2B4CA6: alloc/io and std/text safe wrappers lack raw operation boundary

## 概要

alloc/io and std/text implement safe byte buffer and UTF-8 helpers using raw memory operations, but SourceCapabilities did not grant those compiler-owned stdlib files the operation-only raw memory boundary.

## 対象

- `nepl-core/src/loader.rs, stdlib/alloc/io.nepl, stdlib/std/text.nepl, tests/stdlib/selfhost_cli_file_io.n.md, tests/stdlib/text_utf8.n.md`

## 根拠

- `node nodesrc/tests.js -i tests/stdlib/selfhost_cli_file_io.n.md --no-tree -o tmp/selfhost_cli_file_io_stdout_contract.json -j1 --dist web/dist` が total=4, failed=4 になり、全件が compile phase の `resource.raw.unsafe_memory_boundary` で止まっていた。
- `node nodesrc/tests.js -i tests/stdlib/text_utf8.n.md --no-tree -o tmp/text_utf8_stdout_contract.json -j1 --dist web/dist` が total=9, passed=2, failed=7 になり、失敗の主因は同じく `resource.raw.unsafe_memory_boundary` だった。
- 代表診断は `stdlib/alloc/io.nepl:202` の `io_bytebuf_from_str_result` が `mem_copy` を呼ぶ経路と、`stdlib/std/text.nepl:64` の `text_utf8_byte_at` が `load_u8` を呼ぶ経路だった。
- 既存の operation-only raw memory boundary は `stdlib/alloc/string.nepl` と `stdlib/alloc/collections/vec.nepl` だけを対象にしており、同じ safe-wrapper 実装である `alloc/io` と `std/text` が漏れていた。

## 問題

alloc/io and std/text implement safe byte buffer and UTF-8 helpers using raw memory operations, but SourceCapabilities did not grant those compiler-owned stdlib files the operation-only raw memory boundary.

## 影響

Focused doctests that exercise selfhost file I/O and UTF-8 conversion fail at compile phase with resource.raw.unsafe_memory_boundary even though the public API does not expose raw address escape.

## 修正方針

Register alloc/io.nepl and std/text.nepl as operation-only raw memory boundary modules, preserving raw address escape, raw cell, and owner obligation diagnostics.

## 検証

Run loader capability regression, Resource effect boundary regression, trunk build, and focused selfhost_cli_file_io/text_utf8 doctests.

## 2026-05-06 対応

- `nepl-core/src/loader.rs` の raw-memory-backed stdlib module 判定を `StdlibRawMemoryOperationsModule` enum に分離し、対象を `AllocString`、`AllocVec`、`AllocIo`、`StdText` として明示した。
- `stdlib/alloc/io.nepl` と `stdlib/std/text.nepl` へ `raw_memory_operations` のみを許可し、`raw_address_escape` は引き続き許可しない。
- full raw memory boundary ではないため、raw address escape、raw cell gate、owner obligation gate は抑制されない。

## 2026-05-06 検証結果

- `cargo fmt --check -p nepl-core`: pass。
- `cargo test -p nepl-core loader::tests::source_capabilities_split_stdlib_raw_memory_files -- --nocapture`: pass。
- `cargo test -p nepl-core compiler::tests::resource_effect_gate_splits_raw_operation_and_identity_escape_capabilities -- --nocapture`: pass。
- `trunk build --release`: pass。
- `node nodesrc/tests.js -i tests/stdlib/selfhost_cli_file_io.n.md --no-tree -o tmp/selfhost_cli_file_io_raw_boundary_after.json -j1 --dist web/dist`: total=4, passed=4。
- `node nodesrc/tests.js -i tests/stdlib/text_utf8.n.md --no-tree -o tmp/text_utf8_raw_boundary_after.json -j1 --dist web/dist`: total=9, passed=9。
- `node nodesrc/tests.js -i tests/stdlib/string.n.md --no-tree -o tmp/string_after_io_text_boundary.json -j1 --dist web/dist`: total=17, passed=17。

---
id: ISS-20260506T152038161Z-STRING-BUILDER-SPLIT-LOSES-RAW-MEMOR-637613A4
title: "String builder split loses raw-memory boundary capability"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-06
target: "nepl-core/src/loader.rs, stdlib/alloc/string/builder.nepl, tests/stdlib/kp.n.md"
---

# ISS-20260506T152038161Z-STRING-BUILDER-SPLIT-LOSES-RAW-MEMOR-637613A4: String builder split loses raw-memory boundary capability

## 概要

StringBuilder was split into stdlib/alloc/string/builder.nepl and directly performs raw byte stores and mem_copy while constructing owned string buffers, but the loader exact raw-memory-boundary path table does not include that internal module. KP doctest#1 now fails with effect.pure.calls_impure in sb_append_byte_result, sb_append_result, and sb_build_result.

## 対象

- `nepl-core/src/loader.rs, stdlib/alloc/string/builder.nepl, tests/stdlib/kp.n.md`

## 根拠

- `stdlib/alloc/string/builder.nepl` は `sb_append_result` で `mem_copy<u8>`、`sb_append_byte_result` で `store_u8`、`sb_build_result` で `mem_copy<u8>` を直接呼び出す。
- `node nodesrc/tests.js -i tests/stdlib/kp.n.md -o output/kp_after_string_integer_boundary.json --runner wasm --no-tree -j 1 --assert-io` の partial JSON で、`from_u128_radix` の boundary miss は消えた一方、doctest#1 が `alloc/string/builder.nepl` の `effect.pure.calls_impure` で compile fail になった。
- `alloc/string/builder.nepl` は owned byte buffer を構築する内部 module であり、`alloc/string/access.nepl` や `alloc/string/integer.nepl` と同じく Stage 6 移行まで exact raw-memory-boundary capability で扱う必要がある。

## 問題

StringBuilder was split into stdlib/alloc/string/builder.nepl and directly performs raw byte stores and mem_copy while constructing owned string buffers, but the loader exact raw-memory-boundary path table does not include that internal module. KP doctest#1 now fails with effect.pure.calls_impure in sb_append_byte_result, sb_append_result, and sb_build_result.

## 影響

Any stdlib path that builds strings through StringBuilder can fail compilation behind a Stage5 boundary configuration miss, hiding the remaining Resource IR and runtime issues.

## 修正方針

Audit stdlib/alloc/string/builder.nepl as an internal raw-memory-backed string construction module, add only that configured exact stdlib path to the loader boundary table, and add regression coverage so future builder splits must be deliberate.

## 検証

Run loader raw-memory-boundary regressions and tests/stdlib/kp.n.md focused doctests to confirm StringBuilder no longer fails with effect.pure.calls_impure.

## 2026-05-06 修正

`stdlib/alloc/string/builder.nepl` を監査し、`sb_append_result` / `sb_append_char_result` / `sb_append_ascii_result` / `sb_append_byte_result` / `sb_build_result` が owned byte buffer に対して `store_u8` または `mem_copy` を直接行うことを確認した。StringBuilder は Stage 6 の `OwnedRegion` / owner-token API 移行が完了するまでは internal raw-memory-backed string construction boundary であり、safe public source へ任意の raw memory capability を与えるものではない。

`nepl-core/src/loader.rs` の configured stdlib exact path table に `alloc/string/builder.nepl` を追加し、`nepl-core/tests/effects.rs` の loader regression に同じ path を追加した。configured `stdlib_root` の canonical path と完全一致した file だけを許可するため、同名 suffix や user source には raw-memory-boundary capability は渡らない。

検証:

- `cargo test -p nepl-core --test effects loader_marks_configured_stdlib_byte_and_scanner_boundaries_as_raw_memory_boundary -- --nocapture`: passed
- `cargo check -p nepl-core --tests`: passed
- `cargo fmt --check`: passed
- `node nodesrc/test_stdlib_builder_owner_boundary.js`: passed
- `node nodesrc/test_stdlib_string_integer_boundary.js`: passed
- `node nodesrc/test_stdlib_string_float_boundary.js`: passed
- `node nodesrc/issues.js check`: passed
- `trunk build`: passed
- `node nodesrc/run_doctest.js -i tests/stdlib/kp.n.md -n 1 --dist web/dist`: passed, stdout `10\n20\n30\n`
- `node nodesrc/run_doctest.js -i tests/stdlib/kp.n.md -n 7 --dist web/dist`: passed, stdout `2 3\n1 2 5\n`

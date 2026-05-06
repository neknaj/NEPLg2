---
id: ISS-20260506T155747740Z-VEC-RAW-MEMORY-COLLECTION-LACKS-EXAC-3F7CA727
title: "Vec raw-memory collection lacks exact loader effect boundary"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-07
target: "nepl-core/src/loader.rs, stdlib/alloc/collections/vec.nepl, stdlib/neplg2/core/infra/text.nepl, stdlib/neplg2/core/resolve/name_resolver.nepl"
---

# ISS-20260506T155747740Z-VEC-RAW-MEMORY-COLLECTION-LACKS-EXAC-3F7CA727: Vec raw-memory collection lacks exact loader effect boundary

## 概要

Focused selfhost doctests that import Vec now fail in compile phase with effect.pure.calls_impure because alloc/collections/vec.nepl uses load/store raw memory operations but the loader exact raw-memory boundary table does not grant that configured stdlib path.

## 対象

- `nepl-core/src/loader.rs, stdlib/alloc/collections/vec.nepl, stdlib/neplg2/core/infra/text.nepl, stdlib/neplg2/core/resolve/name_resolver.nepl`

## 根拠

- `stdlib/alloc/collections/vec.nepl` は `alloc_ptr` / `realloc_ptr` / `store<.T>` / `load<.T>` / `dealloc_raw` を使う raw-memory-backed collection module である。
- 修正前の focused selfhost doctest は `effect.pure.calls_impure` で止まり、`push__Vec...` の `store` と `get__ref_Vec...` の `load` が pure function 内の unsafe memory operation として報告されていた。
- `nepl-core/src/loader.rs` の configured exact raw-memory boundary table には `alloc/collections/vec.nepl` がなく、同じ raw-memory-backed stdlib owner である `core/mem.nepl`、`alloc/string/*`、`alloc/io.nepl` だけが許可されていた。

## 問題

Focused selfhost doctests that import Vec now fail in compile phase with effect.pure.calls_impure because alloc/collections/vec.nepl uses load/store raw memory operations but the loader exact raw-memory boundary table does not grant that configured stdlib path.

## 影響

Selfhost modules using Vec cannot be validated under mandatory static checks; the failure masks later owner/type diagnostics and encourages weakening the effect checker instead of declaring the stdlib raw-memory boundary explicitly.

## 修正方針

Audit Vec as an internal raw-memory-backed collection module, add only the configured stdlib exact path if this remains the approved Stage 6 design, and add Rust/source-policy regressions so future collection splits update the table deliberately.

## 対応

- `nepl-core/src/loader.rs` の `RAW_MEMORY_BOUNDARY_STDLIB_PATHS` に exact path `alloc/collections/vec.nepl` を追加した。
- `nepl-core/tests/effects.rs` の configured stdlib raw-memory boundary regression に `alloc/collections/vec` の loader case を追加した。
- `nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` に loader raw-memory boundary source-policy を追加し、Vec の raw-memory owner module と loader 許可表がずれた場合に検出できるようにした。
- 追加後の focused selfhost doctest で Vec raw `store/load` の effect failure は解消し、次の別問題として SourceText line map の Vec owner leak が露出したため、`ISS-20260506T162903318Z-SELFHOST-SOURCETEXT-LINE-MAP-VEC-OWN-2444558D` として切り分けた。

## 検証

Run cargo effect tests, trunk build, and focused selfhost doctests for source_text/name_resolver after the Vec boundary decision.

- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: passed
- `cargo test -p nepl-core --test effects loader_marks_configured_stdlib_byte_and_scanner_boundaries_as_raw_memory_boundary -- --nocapture`: passed
- `cargo test -p nepl-core --test effects`: 27 passed
- `trunk build`: passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: Vec boundary policy は passed。既存 `nodesrc/test_stdlib_io_bytebuf_owner_boundary.js` warning は継続。
- `node nodesrc/tests.js -i stdlib/neplg2/core/infra/text.nepl -i stdlib/neplg2/core/resolve/name_resolver.nepl --no-tree -o tmp/vec-raw-boundary-selfhost.json -j 1`: total=3, passed=2, failed=1。修正前の Vec raw `store/load` effect failure は解消し、残件は新規 issue 化した SourceText owner leak。
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed

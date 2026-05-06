---
id: ISS-20260506T135746003Z-STRING-ACCESS-SPLIT-LOSES-RAW-MEMORY-8C64A912
title: "String access and scanner splits lose raw-memory boundary capability"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-06
target: "nepl-core/src/loader.rs, stdlib/alloc/string/access.nepl, stdlib/alloc/string/scanner.nepl, tests/stdlib/kp.n.md"
source: doc/neplg2/static_check_complexity_reduction_plan.md#stage-5-effect-model-の拡張
---

# ISS-20260506T135746003Z-STRING-ACCESS-SPLIT-LOSES-RAW-MEMORY-8C64A912: String access and scanner splits lose raw-memory boundary capability

## 概要

After remote main split alloc/string/access.nepl and alloc/string/scanner.nepl out of alloc/string.nepl, pure functions len, string_byte_at_unchecked, scanner_str_byte_len, and scanner_string_byte_at_unchecked use load_i32/load_u8 but the loader raw-memory-boundary exact path list still grants the capability only to older string modules. wasm doctests now fail before fs/stdio owner verification with effect.pure.calls_impure for len__str__i32__pure.

## 対象

- `nepl-core/src/loader.rs, stdlib/alloc/string/access.nepl, stdlib/alloc/string/scanner.nepl, tests/stdlib/kp.n.md`

## 根拠

- remote main の `232715b8` / `7b6afed3` を取り込んだ後、`trunk build` は通過した。
- その後の `node nodesrc/tests.js -i stdlib/std/fs/read.nepl -i stdlib/std/fs/raw.nepl -i stdlib/std/fs/fd.nepl -i stdlib/std/stdio/read.nepl -i stdlib/std/stdio/read/buffer.nepl -o output/read_owner_raw_cleanup_targeted.json --runner wasm --no-tree -j 1 --assert-io` は、8 件中 7 件が `effect.pure.calls_impure` で compile failure になった。
- 失敗箇所は `/stdlib/alloc/string/access.nepl:33` の `load_i32 string_addr s` と `/stdlib/alloc/string/access.nepl:109` の `load_u8 ...`。
- `alloc/string/access.nepl` は `232715b8` 系の string responsibility split で root から分離されたが、`RAW_MEMORY_BOUNDARY_STDLIB_PATHS` は `alloc/string.nepl`、`alloc/string/storage.nepl`、`alloc/string/utf8.nepl` までしか追従していない。
- `c413b776` 系では `alloc/string/scanner.nepl` も分離され、`scanner_str_byte_len` / `scanner_string_byte_at_unchecked` が同じ内部 string layout read を持つ。scanner module も exact raw-memory-boundary capability の対象にしないと、次の doctest 群で同じ `effect.pure.calls_impure` が再発する。

## 問題

After remote main split alloc/string/access.nepl and alloc/string/scanner.nepl out of alloc/string.nepl, pure functions len, string_byte_at_unchecked, scanner_str_byte_len, and scanner_string_byte_at_unchecked use load_i32/load_u8 but the loader raw-memory-boundary exact path list still grants the capability only to older string modules. wasm doctests now fail before fs/stdio owner verification with effect.pure.calls_impure for len__str__i32__pure.

## 影響

Any stdlib or tutorial path that imports alloc/string can fail compile after the split. The failure hides downstream Resource IR owner/range diagnostics and prevents kp/stdout doctests from being a useful regression signal.

## 修正方針

Audit the new string access and scanner module boundaries. If they remain internal raw-memory-backed str layout modules during Stage 6 migration, add only those exact configured stdlib paths to the loader capability table and add a regression so future string module splits update the table deliberately.

## 検証

Run cargo test for loader raw-memory-boundary regressions, trunk build, and rerun focused string/kp doctests that previously failed with len__str effect.pure.calls_impure.

## 2026-05-06 修正

`alloc/string/access.nepl` と `alloc/string/scanner.nepl` を configured stdlib の exact raw-memory-boundary path に追加した。stdlib 全体や arbitrary suffix path は許可せず、loader が configured `stdlib_root` から計算した canonical path だけを許可する既存方針を維持する。

回帰として `loader_marks_configured_stdlib_byte_and_scanner_boundaries_as_raw_memory_boundary` に `alloc/string/access` と `alloc/string/scanner` を追加し、次回の string module split で raw-memory-backed helper が増えた場合に loader capability table の更新漏れが再発しないようにした。

検証:

- `node nodesrc/test_stdlib_string_access_boundary.js`: passed
- `node nodesrc/test_stdlib_byte_scanner_helpers_boundary.js`: passed
- `cargo test -p nepl-core --test effects loader_ -- --nocapture`: 5 passed
- `cargo fmt --check`: passed
- `cargo check -p nepl-core --tests`: passed
- `node nodesrc/issues.js check`: passed
- `trunk build`: passed
- `node nodesrc/tests.js -i stdlib/std/fs/read.nepl -i stdlib/std/fs/raw.nepl -i stdlib/std/fs/fd.nepl -i stdlib/std/stdio/read.nepl -i stdlib/std/stdio/read/buffer.nepl -o output/read_owner_raw_cleanup_after_string_boundary.json --runner wasm --no-tree -j 1 --assert-io`: total=8, passed=8
- `node nodesrc/tests.js -i tests/stdlib/kp.n.md -o output/kp_after_string_boundary.json --runner wasm --no-tree -j 1 --assert-io`: `len__str` / `string_byte_at_unchecked` / scanner boundary の `effect.pure.calls_impure` は消滅。残件は `ISS-20260430T012045983Z-RESOURCE-IR-CANNOT-SUMMARIZE-RETURNE-2FDA4B38`、`ISS-20260506T134653279Z-RESOURCE-OWNER-SUMMARY-MISSES-RAW-DE-007EB7EA`、`ISS-20260506T130138471Z-KP-STREAM-SCANNER-FLOAT-DOCTESTS-EXC-0D4A3BF8`。

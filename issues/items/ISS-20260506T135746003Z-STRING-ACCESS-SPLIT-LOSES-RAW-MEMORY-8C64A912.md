---
id: ISS-20260506T135746003Z-STRING-ACCESS-SPLIT-LOSES-RAW-MEMORY-8C64A912
title: "String access split loses raw-memory boundary capability"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-06
target: "nepl-core/src/loader.rs, stdlib/alloc/string/access.nepl, tests/stdlib/kp.n.md"
source: doc/neplg2/static_check_complexity_reduction_plan.md#stage-5-effect-model-の拡張
---

# ISS-20260506T135746003Z-STRING-ACCESS-SPLIT-LOSES-RAW-MEMORY-8C64A912: String access split loses raw-memory boundary capability

## 概要

After remote main split alloc/string/access.nepl out of alloc/string.nepl, pure functions len and string_byte_at_unchecked use load_i32/load_u8 but the loader raw-memory-boundary exact path list still grants the capability only to older string modules. wasm doctests now fail before fs/stdio owner verification with effect.pure.calls_impure for len__str__i32__pure.

## 対象

- `nepl-core/src/loader.rs, stdlib/alloc/string/access.nepl, tests/stdlib/kp.n.md`

## 根拠

- remote main の `232715b8` / `7b6afed3` を取り込んだ後、`trunk build` は通過した。
- その後の `node nodesrc/tests.js -i stdlib/std/fs/read.nepl -i stdlib/std/fs/raw.nepl -i stdlib/std/fs/fd.nepl -i stdlib/std/stdio/read.nepl -i stdlib/std/stdio/read/buffer.nepl -o output/read_owner_raw_cleanup_targeted.json --runner wasm --no-tree -j 1 --assert-io` は、8 件中 7 件が `effect.pure.calls_impure` で compile failure になった。
- 失敗箇所は `/stdlib/alloc/string/access.nepl:33` の `load_i32 string_addr s` と `/stdlib/alloc/string/access.nepl:109` の `load_u8 ...`。
- `alloc/string/access.nepl` は `232715b8` 系の string responsibility split で root から分離されたが、`RAW_MEMORY_BOUNDARY_STDLIB_PATHS` は `alloc/string.nepl`、`alloc/string/storage.nepl`、`alloc/string/utf8.nepl` までしか追従していない。

## 問題

After remote main split alloc/string/access.nepl out of alloc/string.nepl, pure functions len and string_byte_at_unchecked use load_i32/load_u8 but the loader raw-memory-boundary exact path list still grants the capability only to older string modules. wasm doctests now fail before fs/stdio owner verification with effect.pure.calls_impure for len__str__i32__pure.

## 影響

Any stdlib or tutorial path that imports alloc/string can fail compile after the split. The failure hides downstream Resource IR owner/range diagnostics and prevents kp/stdout doctests from being a useful regression signal.

## 修正方針

Audit the new string access module boundary. If it remains an internal raw-memory-backed str layout module during Stage 6 migration, add only that exact configured stdlib path to the loader capability table and add a regression so future string module splits update the table deliberately.

## 検証

Run cargo test for loader raw-memory-boundary regressions, trunk build, and rerun focused string/kp doctests that previously failed with len__str effect.pure.calls_impure.

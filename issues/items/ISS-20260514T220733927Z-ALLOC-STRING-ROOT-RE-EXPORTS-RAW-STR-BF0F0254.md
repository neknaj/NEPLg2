---
id: ISS-20260514T220733927Z-ALLOC-STRING-ROOT-RE-EXPORTS-RAW-STR-BF0F0254
title: "alloc/string root re-exports raw string storage helpers"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-14
updated: 2026-05-15
target: "stdlib/alloc/string.nepl, stdlib/alloc/string/storage.nepl, stdlib/alloc/string/utf8.nepl, nodesrc/test_stdlib_string_storage_boundary.js, nodesrc/test_stdlib_string_utf8_boundary.js"
---

# ISS-20260514T220733927Z-ALLOC-STRING-ROOT-RE-EXPORTS-RAW-STR-BF0F0254: alloc/string root re-exports raw string storage helpers

## 概要

alloc/string is the ordinary safe string facade, but it currently re-exports alloc/string/storage and alloc/string/utf8 with pub wildcard imports. This makes MemPtr-based string storage and UTF-8 memory helpers visible to normal consumers that import alloc/string.

## 対象

- `stdlib/alloc/string.nepl, stdlib/alloc/string/storage.nepl, stdlib/alloc/string/utf8.nepl, nodesrc/test_stdlib_string_storage_boundary.js, nodesrc/test_stdlib_string_utf8_boundary.js`

## 根拠

- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 6 は、raw-memory-backed stdlib implementation が safe public discipline へ raw identity / owner token を漏らさないことを完了条件にしている。
- `alloc/string` root は通常利用者が `#import "alloc/string" as *` で使う safe facade だが、`pub #import "./string/storage" as *` と `pub #import "./string/utf8" as *` により `string_data_ptr` / `string_from_mem_unchecked_result` / `string_utf8_validate_mem` などの `MemPtr`-based helper を root 経由で公開していた。
- `core/mem` と `Vec` facade は既に raw implementation module を明示 import 境界へ閉じる方向へ移行済みであり、`alloc/string` root だけ raw helper を再公開し続けると Stage 6 の public/raw facade split が一貫しない。

## 問題

alloc/string is the ordinary safe string facade, but it currently re-exports alloc/string/storage and alloc/string/utf8 with pub wildcard imports. This makes MemPtr-based string storage and UTF-8 memory helpers visible to normal consumers that import alloc/string.

## 影響

Safe string users can reach raw storage identity helpers without explicitly importing the raw boundary modules. That weakens the Stage 6 separation where MemPtr is a non-owning view and raw-memory-backed implementation details must not be pushed into ordinary public APIs.

## 修正方針

Stop re-exporting raw storage and UTF-8 memory helpers from alloc/string. Keep those helpers available only through explicit alloc/string/storage and alloc/string/utf8 imports, and migrate stdlib implementation modules that genuinely operate at raw OS/storage boundaries to import those modules directly.

## 検証

Add source policy regressions that forbid alloc/string from re-exporting raw helper modules and require explicit raw-boundary module ownership. Run focused string boundary policies, affected stdlib doctests, and issue validation.

## 解決内容

- `alloc/string` root から `./string/storage` と `./string/utf8` の public wildcard re-export を削除した。
- root doc comment に、raw `MemPtr` / storage helper は `alloc/string/storage` と `alloc/string/utf8` を明示 import する boundary module だけで使う方針を記述した。
- `std/fs` / `std/stdio` / `std/env/cliarg` / `std/streamio` のうち、OS boundary や storage conversion として raw string helper を本当に使う実装は、`alloc/string/storage` / `alloc/string/utf8` を明示 import する形へ移した。
- `stdlib/tests/string.n.md` と `tests/stdlib/stdio_result_stderr.n.md` の raw helper 利用箇所も explicit raw boundary import に移した。
- `string_utf8_mem_result` doctest は raw address `store_u8` / `dealloc_raw` ではなく、checked `MemPtr` store と `dealloc_ptr` で invalid UTF-8 byte を構成するよう更新した。
- `nodesrc/test_stdlib_string_utf8_boundary.js` と `nodesrc/test_stdlib_string_storage_boundary.js` を、root が raw helper module を再公開しないことを監視する policy に反転した。

## 検証結果

- `node nodesrc/test_stdlib_string_utf8_boundary.js`: passed
- `node nodesrc/test_stdlib_string_storage_boundary.js`: passed
- `node nodesrc/tests.js -i stdlib/std/stdio/write/text.nepl -i stdlib/std/stdio/print.nepl --no-tree -o tmp/agent1-string-raw-facade-stdio-modules.json -j 1 --dist web/dist --assert-io`: 4/4 passed
- `node nodesrc/tests.js -i stdlib/std/fs/fd.nepl -i stdlib/std/fs/path/entry.nepl --no-tree -o tmp/agent1-string-raw-facade-fs-modules.json -j 1 --dist web/dist --assert-io`: 2/2 passed
- `node nodesrc/tests.js -i stdlib/std/env/cliarg/cstr.nepl -i stdlib/std/env/cliarg/raw.nepl --no-tree -o tmp/agent1-string-raw-facade-cliarg-modules.json -j 1 --dist web/dist --assert-io`: 2/2 passed
- `node nodesrc/tests.js -i tests/stdlib/streamio.n.md --no-tree -o tmp/agent1-string-raw-facade-streamio.json -j 1 --dist web/dist --assert-io`: 15/15 passed
- `node nodesrc/tests.js -i stdlib/tests/string.n.md --no-tree -o tmp/agent1-string-raw-facade-stdlib-string-after.json -j 1 --dist web/dist --assert-io`: 7/9 passed。今回変更した raw UTF-8 memory case は pass。残る 2 件は stale import assumptions として `ISS-20260514T221807506Z-STDLIB-STRING-DOCTESTS-RETAIN-STALE--CC9D6303` に分離した。

## 関連

- 親: `ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84`
- 計画: `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 6

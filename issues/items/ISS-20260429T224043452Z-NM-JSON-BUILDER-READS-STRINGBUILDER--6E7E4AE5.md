---
id: ISS-20260429T224043452Z-NM-JSON-BUILDER-READS-STRINGBUILDER--6E7E4AE5
title: "nm JSON builder reads StringBuilder data as raw MemPtr after storage became Option"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-30
target: "stdlib/nm/parser.nepl, examples/nm.nepl"
---

# ISS-20260429T224043452Z-NM-JSON-BUILDER-READS-STRINGBUILDER--6E7E4AE5: nm JSON builder reads StringBuilder data as raw MemPtr after storage became Option

## 概要

GitHub Actions run `25137357734` の `nm-compile` は `examples/nm.nepl` の compile で失敗した。原因は `stdlib/nm/parser.nepl::json_escape_builder_into` が `StringBuilder.data` を raw `MemPtr<u8>` として `json_escape_mem_into` に渡していたことだった。

## 対象

- `stdlib/nm/parser.nepl, examples/nm.nepl`

## 根拠

- `nm-compile` log では `stdlib/nm/parser.nepl:265` の `json_escape_mem_into sb get src "data" get src "len"` に対して `type.overload.no_match` が出ていた。
- `StringBuilder.data` は現在 `Option<MemPtr<u8>>` であり、storage owner boundary は `get_ref` で `Option` を borrow して `Some` / `None` を分岐する形に統一されている。
- `examples/nm.nepl` は CI の NM compile smoke であり、stdlib/nm の JSON writer が現在の `StringBuilder` contract に追従していないと self-host / docs まわりの検証が止まる。

## 問題

`json_escape_builder_into` が、旧 `StringBuilder.data <MemPtr<u8>>` 時代の呼び出し形のまま残っていた。`Option<MemPtr<u8>>` を raw pointer として扱うため overload が解けず、戻り値推論も `StringBuilder` から外れていた。

## 影響

examples/nm.nepl cannot compile, blocking NM/self-host documentation validation. Treating StringBuilder.data as a raw MemPtr also violates the current owner-boundary design that requires borrowing the Option storage and freeing the source builder exactly once.

## 修正方針

Update json_escape_builder_into to match the borrowed StringBuilder.data Option, escape only the Some data path, handle empty/invalid None storage without moving the field, and then consume src through string_builder_free.

## 検証

- `cargo run -p nepl-cli -- --target wasi --profile debug --input examples/nm.nepl --output target/agent1-ci-nm`: pass
- `node nodesrc/test_stdlib_nm_parser_no_inline_unwraps.js`: pass
- `node nodesrc/test_stdlib_nm_parser_doc_no_boilerplate.js`: pass
- `node nodesrc/test_stdlib_string_no_unsafe_unwraps.js`: pass
- `node nodesrc/tests.js -i stdlib/nm/parser.nepl --no-tree -o tmp/agent1-nm-parser-option-storage.json -j 1 --dist web/dist`: total=3, passed=3
- `node nodesrc/tests.js -i tests/stdlib/nm.n.md --no-tree -o tmp/agent1-tests-stdlib-nm-option-storage.json -j 1 --dist web/dist`: total=5, passed=5

## 対応結果

`json_escape_builder_into` は `*get_ref &src "data"` で `Option<MemPtr<u8>>` を borrow し、`Option::Some data` のときだけ `json_escape_mem_into` へ渡す形にした。追加後は source builder を `string_builder_free src` で消費する。`Option::None` は source builder を解放したうえで destination builder をそのまま返し、古い raw field access を残さない。

---
id: ISS-20260428T045151813Z-STRINGBUILDER-VEC-STR-STORAGE-D3100--B8C4A9D1
title: "StringBuilder Vec<str> storage fails under strict move checking"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "stdlib/alloc/string.nepl, stdlib/nm/parser.nepl, stdlib/nm/html_gen.nepl, stdlib/tests/string.n.md, tests/stdlib/nm.n.md"
---

# ISS-20260428T045151813Z-STRINGBUILDER-VEC-STR-STORAGE-D3100--B8C4A9D1: StringBuilder Vec<str> storage fails under strict move checking

## 概要

`StringBuilder` が `Vec<str>` に文字列片を保持しているため、最新の strict move checking では `sb_append` / `sb_build` を含む parser・HTML・JSON 生成経路が `D3100` で停止する。

## 対象

- `stdlib/alloc/string.nepl, stdlib/nm/parser.nepl, stdlib/nm/html_gen.nepl, stdlib/tests/string.n.md, tests/stdlib/nm.n.md`

## 根拠

- `node nodesrc/tests.js -i stdlib/nm/parser.nepl -i stdlib/nm/html_gen.nepl -i tests/stdlib/nm.n.md --no-tree -o tmp/nm-raw-detour-focused-1.json -j 1` で 10 件中 8 件が compile fail。
- 代表エラーは `D3100 overwriting raw memory place containing non-Copy value: $memptr:grown_data+?` と `D3100 reallocating/deallocating raw memory place containing non-Copy value: $memptr:v_data+?`。
- `parse_inlines` / `json_escape` / `render_nodes` などの call site は `StringBuilder` を通常どおり使っているだけで、根本原因は builder 内部が `Vec<str>` の raw storage に non-Copy `str` payload を保持している点にある。

## 問題

`StringBuilder` は「最後にまとめて copy するために `str` 片を蓄積する」設計だが、`Vec<str>` は raw storage 上に non-Copy 値を置く。strict move checker が raw storage の live non-Copy payload を追跡するようになると、grow/realloc、overwrite、finish/free がすべて所有権違反として扱われる。

## 影響

nm parser/html generator、JSON serializer、diagnostic text、self-host の source/report builder が clean に検証できない。D3100 を弱めると non-Copy payload の shallow copy / overwrite を再び許すため、builder 側を byte storage owner へ移行する必要がある。

## 修正方針

`StringBuilder` を `Vec<str>` 片リストではなく growable `u8` buffer owner として再実装する。`sb_append_result` は入力 `str` の byte 列を builder buffer へ copy し、`sb_build_result` は buffer を `str` レイアウトへ移すか複製して確定する。公開 API 名は互換維持しつつ、内部 raw storage に non-Copy `str` を置かない。修正後は string builder tests と nm focused tests を実行し、source policy で `StringBuilder` の `Vec<str>` 回帰を防ぐ。

## 検証

- `node nodesrc/tests.js -i stdlib/alloc/string.nepl -i stdlib/tests/string.n.md -i tests/stdlib/string.n.md --no-tree -o tmp/string-builder-byte-storage.json -j 1`
- `node nodesrc/tests.js -i stdlib/nm/parser.nepl -i stdlib/nm/html_gen.nepl -i tests/stdlib/nm.n.md --no-tree -o tmp/nm-after-string-builder-byte-storage.json -j 1`
- `node nodesrc/issues.js check`

## 対応結果

- `StringBuilder` の内部表現を `Vec<str>` から `MemPtr<u8>` / `len` / `cap` の owned byte buffer に変更した。
- `sb_append_result` は入力 `str` の byte 列を append 時に builder buffer へ copy し、raw storage に `str` owner/view を保存しない実装にした。
- `sb_build_result` は builder buffer から新しい `str` 領域へ copy してから builder buffer を解放するようにし、公開 API の戻り値互換を保った。
- `nodesrc/test_stdlib_string_no_unsafe_unwraps.js` に `StringBuilder` が `Vec<str>` へ戻らない source policy を追加した。

## 実施した検証

- `trunk build`: pass
- `node nodesrc/test_stdlib_string_no_unsafe_unwraps.js`: pass
- `node nodesrc/tests.js -i stdlib/alloc/string.nepl -i stdlib/tests/string.n.md -i tests/stdlib/string.n.md --no-tree -o tmp/string-builder-byte-storage-after-second-rebase.json -j 1`: `total=32`, `passed=32`
- `node nodesrc/issues.js check`: pass

## 残件

`StringBuilder` 由来の `Vec<str>` raw storage は解消したが、nm parser/html_gen 全体の D3100 はこの issue では閉じない。`Vec<Inline>` / `Vec<Node>` の generic raw storage と `ParaRes` の non-Copy field decomposition は `ISS-20260428T003718356Z-NM-PARSER-AND-HTML-GEN-RAW-MEMORY-DE-99175378` の継続対象として扱い、D3100 を緩めずに AST container / owned aggregate decomposition の設計で解消する。

## 2026-04-28 CI 再発

`main` の CI run `25035206074`（`fix(stdlib): make string builder byte-backed`）で `nodesrc/test_stdlib_string_no_unsafe_unwraps.js` が失敗したため、この issue を再オープンする。Source policy regressions job は `stdlib/alloc/string.nepl must document StringBuilder ownership contract` で失敗しており、byte-backed 実装自体の focused tests ではなく、公開コメントが新しい ownership contract を説明しているかの policy で落ちている。

修正側では `StringBuilder` の内部が `Vec<str>` に戻っていないことに加え、`StringBuilder` / append / build / free 周辺の日本語 nm comment に、builder が owned byte buffer を保持し、append 時に入力 `str` bytes を copy し、build/free 後に builder を再利用しないことを明記する必要がある。

## 2026-04-28 再確認

現在の `stdlib/alloc/string.nepl` には、`StringBuilder` が owned byte buffer を保持すること、append 時に入力 `str` bytes を copy して `str` owner を保持しないこと、build/free 後に builder を再利用しないことを説明する nm comment が入っている。

`nodesrc/test_stdlib_string_no_unsafe_unwraps.js` はこの ownership contract と `Vec<str>` storage 禁止を検査しており、最新 main 同期後に pass した。string focused tests と nm focused tests も pass しているため、CI 再発として開き直したこの issue は解消済みとして閉じる。

## 再確認した検証

- `node nodesrc/test_stdlib_string_no_unsafe_unwraps.js`: pass
- `node nodesrc/tests.js -i stdlib/alloc/string.nepl -i stdlib/tests/string.n.md -i tests/stdlib/string.n.md --no-tree -o tmp/string-builder-policy-reclose.json -j 1`: `total=32`, `passed=32`
- `node nodesrc/tests.js -i stdlib/nm/parser.nepl -i stdlib/nm/html_gen.nepl -i tests/stdlib/nm.n.md --no-tree -o tmp/nm-after-stringbuilder-policy-reclose.json -j 1`: `total=10`, `passed=10`

## 2026-04-28 doc policy expectation 再発

`main` の CI run `25038139819`（`refactor(core): add resource ir skeleton`）でも Source policy regressions が `stdlib/alloc/string.nepl must document StringBuilder ownership contract` で失敗した。今回の原因は `stdlib/alloc/string.nepl` のコメント不足ではなく、`nodesrc/test_stdlib_string_doc_no_boilerplate.js` が byte-backed 化前の「`StringBuilder` が str 片を保持する」文言を要求していたことである。

現在の設計では `StringBuilder` は `str` owner を保持せず、owned `u8` byte buffer に copy して最後に `str` を新規確保する。このため source policy は旧 `Vec<str>` 的な説明を要求してはならない。policy の required phrase を byte buffer 追加、non-Copy owner、raw storage は `u8` のみ、という現行 ownership contract に合わせて更新した。

---
id: ISS-20260507T114726130Z-NM-JSON-ESCAPE-MEM-INTO-IS-REJECTED--0F849F57
title: "nm json_escape_mem_into is rejected as pure raw memory load"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-07
updated: 2026-05-07
target: "stdlib/nm/json_escape.nepl, tests/stdlib/nm.n.md"
---

# ISS-20260507T114726130Z-NM-JSON-ESCAPE-MEM-INTO-IS-REJECTED--0F849F57: nm json_escape_mem_into is rejected as pure raw memory load

## 概要

`tests/stdlib/nm.n.md::doctest#1` が current main で `effect.pure.calls_impure` になる。原因は `stdlib/nm/json_escape.nepl` の `json_escape_mem_into` が public pure helper のまま `MemPtr<u8>` から raw `load` していること。

## 対象

- `stdlib/nm/json_escape.nepl, tests/stdlib/nm.n.md`

## 根拠

- `node nodesrc/tests.js -i tests/stdlib/nm.n.md --no-tree -o tmp/nm-json-escape-pure-raw-current-issue.json -j 1 --dist web/dist` で `total=5, passed=4, failed=1`。
- 失敗内容は `tests\stdlib\nm.n.md::doctest#1` の compile error: `error[effect.pure.calls_impure]: pure function 'json_escape_mem_into__StringBuilder_MemPtr_T_u8_i32__StringBuilder__pure' uses unsafe memory operation 'load'`。
- `stdlib/nm/json_escape.nepl` は `json_escape_mem_into <(StringBuilder,MemPtr<u8>,i32)->StringBuilder>` を public API として持ち、byte loop 内で `load_u8` を使って JSON escape を行う。静的 effect checker の現在の方針では、この raw memory load は pure surface へそのまま置けない。
- 親設計問題は `ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84` の raw-memory-backed stdlib API 移行に属する。ただし、この issue は `nm` suite の具体的な赤を追跡するために分離する。

## 問題

`json_escape_mem_into` は `MemPtr<u8>` / byte length を受ける低レベル helper だが、`pub fn` かつ pure signature のまま公開されている。結果として、nm の JSON escape 実装は静的検査上「pure 関数から unsafe memory load を呼ぶ」形になり、`tests/stdlib/nm.n.md` の doctest が 1 件失敗する。

## 影響

`nm` suite が current main で完全には通らない。さらに、public pure helper が raw memory backed 実装を直接露出しているため、型安全・メモリ安全を必達とする現在の静的検査方針と矛盾する。

## 修正方針

静的検査を緩めずに `nm` JSON escape の byte traversal を再設計する。

- raw `MemPtr` traversal は compiler/stdlib が認める明示的な raw-memory boundary へ閉じる。
- 可能なら `str` / `StringBuilder` 側に effect-safe な byte iteration / append API を用意し、`nm` は raw pointer を受け取らない public pure API へ寄せる。
- `json_escape_mem_into` を残す場合は public pure API ではなく internal/unsafe boundary として扱い、safe wrapper との責務を分ける。
- `effect.pure.calls_impure` を抑制するために checker を弱める修正は禁止する。

## 検証

- `node nodesrc/tests.js -i tests/stdlib/nm.n.md --no-tree -o tmp/nm-json-escape-pure-raw-after.json -j 1 --dist web/dist`: total=5, passed=5 になること。
- public pure `nm` helper が unmanaged raw memory を直接 `load` しないことを source policy で固定すること。
- 既存の `nodesrc/test_stdlib_nm_json_escape_boundary.js` と `nodesrc/test_stdlib_match_decision_trees.js` を、新しい責務境界に合わせて更新すること。

## 2026-05-07 修正

`json_escape_mem_into` を削除し、`nm/json_escape` の public API から `MemPtr<u8>` を受ける raw traversal helper を外した。`json_escape_into` は `str` の byte length と `string_byte_at_unchecked` を使う safe string access boundary に一本化し、各 byte を従来どおり `json_escape_byte_into` の `match` へ渡す。

`json_escape_builder_into` は source `StringBuilder` を `sb_build` で `str` に確定してから `json_escape_into` に渡す。これにより、source builder の storage borrow / raw pointer traversal は `nm` module から消え、raw memory load は既存の `alloc/string/access` boundary へ閉じる。

検証:

- `node nodesrc/test_stdlib_nm_json_escape_boundary.js`: passed
- `node nodesrc/test_stdlib_match_decision_trees.js`: passed
- `node nodesrc/tests.js -i tests/stdlib/nm.n.md --no-tree -o tmp/nm-json-escape-pure-raw-after.json -j 1 --dist web/dist`: total=5, passed=5
- `node nodesrc/tests.js -i stdlib/nm/json_escape.nepl --no-tree -o tmp/nm-json-escape-module-after.json -j 1 --dist web/dist`: total=1, passed=1
- `node nodesrc/tests.js -i stdlib/nm/parser.nepl -i stdlib/nm/parser/json_inline.nepl -i tests/stdlib/nm.n.md --no-tree -o tmp/nm-json-escape-parser-after.json -j 1 --dist web/dist`: total=9, passed=9
- `node nodesrc/run_source_policy_regressions.js --warn-only`: nm/json_escape 関連は passed。既知 open issue `ISS-20260507T112048241Z-RESOURCE-AGGREGATE-PROJECTION-MODULE-595EC35D` の `lower_aggregate_projection.rs has 204 lines; responsibility split limit is 180` warning は継続。

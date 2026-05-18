---
id: ISS-20260518T121529150Z-STRING-BYTE-INDEX-CHECKED-OR-UNREACH-4C77E237
title: "string byte index checked-or-unreachable helper keeps unsafe trap surface"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "stdlib/alloc/string/byte_index.nepl, nodesrc/test_stdlib_no_unsafe_helpers.js, nodesrc/test_stdlib_string_access_boundary.js"
---

# ISS-20260518T121529150Z-STRING-BYTE-INDEX-CHECKED-OR-UNREACH-4C77E237: string byte index checked-or-unreachable helper keeps unsafe trap surface

## 概要

`stdlib/alloc/string/byte_index.nepl` exposed a public infallible byte reader that accepted an arbitrary `i32` and trapped with `#intrinsic "unreachable"` when the index proof failed. The helper did check before the raw read, but its public API still let hot paths depend on a trap-based precondition instead of threading typed evidence or a checked result.

2026-05-18 に修正済み。public surface は `checked_string_byte_at(str, i32) -> Option<i32>`、`string_byte_eq`、`string_bytes_eq`、`string_bytes_cmp` に置き換え、raw layout read は private `StringByteIndex` witness を要求する `string_byte_at_checked` に閉じた。

関連計画: [NEPLg2 静的検査の複雑化解消計画 Stage 6](../../doc/neplg2/static_check_complexity_reduction_plan.md)

## 対象

- `stdlib/alloc/string/byte_index.nepl`
- `stdlib/alloc/string/search/*.nepl`
- `stdlib/alloc/string/{scanner,char_offsets,integer/parse,float/parse}.nepl`
- `stdlib/neplg2/**`
- `stdlib/nm/**`
- `stdlib/std/**`
- `nodesrc/test_stdlib_no_unsafe_helpers.js`
- `nodesrc/test_stdlib_string_access_boundary.js`

## 根拠

- Stage 6 の raw-memory-backed public surface 方針では、`0 <= idx < len(s)` の証明を caller convention や trap に置かず、型または checked result として表現する必要がある。
- `string_byte_at_checked_or_unreachable` は witness 生成後に raw read していたため memory corruption へは直結しないが、public infallible API として「失敗時は trap」という設計を残し、source policy の no-unsafe-helper 方針と矛盾していた。
- hot path caller は範囲証明済みの loop 内でも、証明失敗を `Option` / sentinel / checked compare API で扱えるため、public trap helper を残す必要はなかった。

## 問題

`alloc/string/byte_index` の witness 化は raw read を private witness に閉じたが、移行用 public helper が任意 `i32` を受け、precondition 違反を trap に変換していた。これにより caller は `Option<StringByteIndex>` や checked byte result を match せず、失敗経路を型で表現しないまま高速 loop を書けた。

## 影響

静的検査上の危険は、raw read そのものではなく、範囲証明の失敗が型・値の分岐として現れないことだった。public trap helper が残ると、将来の caller が証明を局所的に捨てても source policy では「checked helper 経由」と見えてしまい、Stage 6 の「検査プログラム自体の誤りを静的に見つけやすくする」方針を弱める。

## 修正方針

1. public `string_byte_at_checked_or_unreachable` を削除する。
2. public checked API は `Option<i32>` を返す byte read と、byte equality / byte comparison helper に限定する。
3. 呼び出し側は `match checked_string_byte_at` または checked compare helper を使い、失敗経路を明示する。
4. parser / lexer などの sentinel loop は private adapter で `Option::None` を `-1` などの既存 failure sentinel に変換し、trap へ戻さない。
5. source policy と memory safety doctest で public trap helper と unsafe helper 名の再導入を拒否する。

## 対応内容

- `checked_string_byte_at`、`string_byte_eq`、`string_bytes_eq`、`string_bytes_cmp` を追加し、public byte access を checked result / checked comparison に統一した。
- stdlib / selfhost / nm / fs / streamio / hash / lexer / parser の call site を public trap helper から checked API へ移行した。
- scanner 系や parser 系の境界付き loop は private `*_byte_or_end` / `*_byte_or_invalid` adapter に寄せ、`Option::None` を通常の失敗値へ変換する形にした。
- policy tests を更新し、旧 helper 名、public `unreachable` helper、unchecked byte access の再導入を検出する。

## 検証

- `node nodesrc/test_stdlib_string_access_boundary.js`: passed
- `node nodesrc/test_stdlib_no_unsafe_helpers.js`: passed
- `node nodesrc/test_stdlib_byte_scanner_helpers_boundary.js`: passed
- `node nodesrc/test_stdlib_string_search_boundary.js`: passed
- `node nodesrc/test_stdlib_fs_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_streamio_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_streamio_writer_boundary.js`: passed
- `node nodesrc/test_stdlib_match_decision_trees.js`: passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: passed
- `node nodesrc/test_stdlib_documentation_contract.js`: passed
- `node nodesrc/tests.js -i stdlib\alloc\string\byte_index.nepl --no-tree -o tmp\agent1-string-byte-index-byte-index.json -j 1 --dist web\dist --assert-io`: total=5, passed=5
- `node nodesrc/tests.js -i stdlib\alloc\string\search -i stdlib\alloc\string\slice -i stdlib\alloc\string\integer\parse.nepl -i stdlib\alloc\string\float\parse.nepl --no-tree -o tmp\agent1-string-byte-index-string-focused.json -j 1 --dist web\dist --assert-io`: total=5, passed=5
- `node nodesrc/tests.js -i stdlib\nm\json_escape.nepl -i stdlib\nm\html_escape.nepl -i stdlib\nm\html_inline.nepl -i stdlib\nm\parser\scanner.nepl -i stdlib\nm\parser\json_inline.nepl --no-tree -o tmp\agent1-string-byte-index-nm-focused.json -j 1 --dist web\dist --assert-io`: total=5, passed=5
- `node nodesrc/tests.js -i stdlib\neplg2\core\syntax\lexer.nepl -i stdlib\neplg2\core\module\import_scan.nepl -i stdlib\neplg2\core\module\import_spec.nepl -i stdlib\neplg2\core\module\stdlib_map.nepl -i stdlib\neplg2\core\infra\text.nepl -i stdlib\neplg2\cli\args\emit.nepl --no-tree -o tmp\agent1-string-byte-index-selfhost-focused.json -j 1 --dist web\dist --assert-io`: total=5, passed=5
- `node nodesrc/tests.js -i stdlib\alloc\hash\hash32.nepl -i stdlib\core\traits\hash.nepl -i stdlib\core\traits\hash_key.nepl -i stdlib\tests\hash.n.md --no-tree -o tmp\agent1-string-byte-index-hash-focused.json -j 1 --dist web\dist --assert-io`: total=2, passed=2
- `node nodesrc/tests.js -i tests\stdlib\memory_safety.n.md --no-tree -o tmp\agent1-string-byte-index-memory-safety.json -j 1 --dist web\dist --assert-io`: total=60, passed=60

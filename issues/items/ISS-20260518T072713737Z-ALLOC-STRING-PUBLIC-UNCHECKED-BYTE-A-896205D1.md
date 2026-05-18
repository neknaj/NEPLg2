---
id: ISS-20260518T072713737Z-ALLOC-STRING-PUBLIC-UNCHECKED-BYTE-A-896205D1
title: "alloc string public unchecked byte access needs safe boundary redesign"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-18
updated: 2026-05-18
target: "stdlib/alloc/string/access.nepl, stdlib/alloc/string/byte_index.nepl, stdlib/**, tests/stdlib/memory_safety.n.md"
---

# ISS-20260518T072713737Z-ALLOC-STRING-PUBLIC-UNCHECKED-BYTE-A-896205D1: alloc string public unchecked byte access needs safe boundary redesign

## 概要

`alloc/string/access` exposed `string_byte_at_unchecked` through the root `alloc/string` facade. Many stdlib and selfhost modules call byte reads after local bounds reasoning, but ordinary callers could also direct-call the same raw layout reader with arbitrary indices because the proof obligation was only documented, not represented in the type system or Resource IR API boundary.

2026-05-18 の最終修正で、過渡 module `alloc/string/unchecked_access` を廃止し、`alloc/string/byte_index` に private `StringByteIndex` witness を導入した。raw string layout read は `string_byte_at_checked(str, StringByteIndex)` に閉じ、witness は `checked_string_byte_index(str, i32) -> Option<StringByteIndex>` が `0 <= idx < len(s)` を確認した場合だけ生成する。constructor と raw address projection は private なので、ordinary caller は任意の `i32` を raw reader へ渡せない。

## 対象

- `stdlib/alloc/string/access.nepl`
- `stdlib/alloc/string/byte_index.nepl`
- `stdlib/alloc/string.nepl`
- `stdlib/**`
- `tests/stdlib/memory_safety.n.md`

## 根拠

- root `alloc/string` facade は `./string/access` を再公開するため、`access.nepl` 内の public unchecked reader は通常 import から到達可能だった。
- `string_byte_at_unchecked` は `[len:i32][bytes...]` の byte payload を直接読むため、`0 <= idx < len(s)` が守られない場合に `str` layout 外を読む。
- Stage 6 の方針では、raw memory backed public API は caller discipline ではなく source / type / Resource IR proof で境界を証明する必要がある。
- `nodesrc/test_stdlib_string_access_boundary.js` が root re-export、`access.nepl` public unchecked reader、`byte_index.nepl` public unchecked reader、public witness constructor、checked factory を経由しない raw read の再導入を拒否する。

## 問題

旧実装では `alloc/string/access` が unchecked raw byte reader を public に持ち、root `alloc/string` facade からも見えていた。これは `byte_at` のような checked API と unchecked raw layout API が同じ public surface に並ぶ設計であり、通常 caller と compiler-owned stdlib implementation boundary を分けられなかった。

一次修正では unchecked reader を明示 `unchecked_access` module に隔離したが、direct import による任意 index 呼び出しはまだ可能だった。最終修正ではこの module 自体を削除し、範囲確認済み index を private witness として型に乗せる設計へ置き換えた。

## 影響

Root `alloc/string` と `alloc/string/access` の利用者は unchecked reader に到達しない。通常の byte access は `Option<i32>` を返す `byte_at` を使う。

後続の `ISS-20260518T121529150Z-STRING-BYTE-INDEX-CHECKED-OR-UNREACH-4C77E237` で、移行用の trap-based public helper も削除した。stdlib / selfhost の高速 scanner / parser / hashing loop は、`checked_string_byte_at`、checked byte comparison helper、または module-private sentinel adapter を使い、範囲証明失敗を `Option::None` や既存の失敗値として明示する。

## 修正方針

1. `alloc/string/access` は safe public API だけを持つ。`byte_at` は `len` による範囲確認後に private raw helper を呼ぶ。
2. Root `alloc/string` facade は unchecked byte reader を再公開しない。
3. `alloc/string/unchecked_access` は削除し、public unchecked raw reader を残さない。
4. `StringByteIndex` は private witness とし、checked factory だけが constructor を使う。
5. `string_byte_at_checked` は witness を要求し、public API からは任意 `i32` で直接呼べない。
6. stdlib / selfhost call site は `alloc/string/byte_index` の checked helper へ移行し、source policy と memory safety doctest で退行を拒否する。
7. follow-up issue で public trap helper を削除し、hot path も checked result / checked compare / private sentinel adapter へ移行する。

## 検証

- `nodesrc/test_stdlib_string_access_boundary.js` で root `alloc/string` の unchecked re-export、`access.nepl` / `byte_index.nepl` の public unchecked reader、public witness constructor、checked factory を迂回した raw read の崩れを検出する。
- `tests/stdlib/memory_safety.n.md` に root `alloc/string` import と direct `alloc/string/access` import から `string_byte_at_unchecked` を呼べない compile-fail regression を追加する。
- `tests/stdlib/memory_safety.n.md` に `alloc/string/byte_index` 直 import から任意 `i32` を `string_byte_at_checked` へ渡せない compile-fail regression と、`StringByteIndex` constructor が見えない compile-fail regression を追加する。
- `alloc/string/access.nepl` / `alloc/string/byte_index.nepl` の focused doctest と `tests/stdlib/memory_safety.n.md` を実行する。

2026-05-18 一次修正の focused verification:

- `node nodesrc/test_stdlib_string_access_boundary.js`: passed
- `node nodesrc/test_stdlib_string_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_byte_scanner_helpers_boundary.js`: passed
- `trunk build`: passed
- `node nodesrc/tests.js -i tests\stdlib\memory_safety.n.md --no-tree -o tmp\agent1-string-unchecked-access-boundary-memory-safety.json -j 1 --dist web\dist --assert-io`: total=49, passed=49
- `node nodesrc/tests.js -i stdlib\alloc\string\access.nepl -i stdlib\alloc\string\unchecked_access.nepl --no-tree -o tmp\agent1-string-unchecked-access-boundary-docs.json -j 1 --dist web\dist --assert-io`: total=1, passed=1
- `node nodesrc/tests.js -i stdlib\tests\string.n.md --no-tree -o tmp\agent1-string-unchecked-access-boundary-string-tests.json -j 1 --dist web\dist --assert-io`: total=9, passed=9

2026-05-18 witness 化の focused verification:

- `node nodesrc/test_stdlib_string_access_boundary.js`: passed
- `node nodesrc/test_stdlib_byte_scanner_helpers_boundary.js`: passed
- `node nodesrc/test_stdlib_string_search_boundary.js`: passed
- `node nodesrc/test_stdlib_fs_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_streamio_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_streamio_writer_boundary.js`: passed
- `node nodesrc/test_stdlib_match_decision_trees.js`: passed
- `node nodesrc/tests.js -i stdlib\alloc\string\access.nepl -i stdlib\alloc\string\byte_index.nepl --no-tree -o tmp\agent1-string-byte-index-proof-docs.json -j 1 --dist web\dist --assert-io`: total=2, passed=2
- `node nodesrc/tests.js -i stdlib\alloc\hash\hash32.nepl -i stdlib\core\traits\hash.nepl -i stdlib\core\traits\hash_key.nepl --no-tree -o tmp\agent1-string-byte-index-proof-hash-modules.json -j 1 --dist web\dist --assert-io`: total=1, passed=1
- `node nodesrc/tests.js -i tests\stdlib\memory_safety.n.md --no-tree -o tmp\agent1-string-byte-index-proof-memory-safety.json -j 1 --dist web\dist --assert-io`: total=52, passed=52
- `node nodesrc/tests.js -i stdlib\tests\string.n.md --no-tree -o tmp\agent1-string-byte-index-proof-string-tests.json -j 1 --dist web\dist --assert-io`: total=9, passed=9

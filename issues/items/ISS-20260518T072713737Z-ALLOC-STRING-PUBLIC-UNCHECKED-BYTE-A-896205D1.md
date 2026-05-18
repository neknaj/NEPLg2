---
id: ISS-20260518T072713737Z-ALLOC-STRING-PUBLIC-UNCHECKED-BYTE-A-896205D1
title: "alloc string public unchecked byte access needs safe boundary redesign"
area: stdlib
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-18
updated: 2026-05-18
target: "stdlib/alloc/string/access.nepl, stdlib/**, tests/stdlib/memory_safety.n.md"
---

# ISS-20260518T072713737Z-ALLOC-STRING-PUBLIC-UNCHECKED-BYTE-A-896205D1: alloc string public unchecked byte access needs safe boundary redesign

## 概要

`alloc/string/access` exposed `string_byte_at_unchecked` through the root `alloc/string` facade. Many stdlib and selfhost modules call unchecked byte reads after local bounds reasoning, but ordinary callers could also direct-call the same raw layout reader with arbitrary indices because the proof obligation was only documented, not represented in the type system or Resource IR API boundary.

2026-05-18 の一次修正で、root `alloc/string` と `alloc/string/access` の公開面から `string_byte_at_unchecked` を外した。範囲確認済みの通常 API は `byte_at` に集約し、残る unchecked reader は明示的な `alloc/string/unchecked_access` module へ移した。この module は stdlib / selfhost 内部の範囲証明済み hot path を段階移行するための過渡境界であり、最終設計では bounded-index proof / checked byte scanner / compiler-owned private boundary のいずれかへ移す。

## 対象

- `stdlib/alloc/string/access.nepl`
- `stdlib/alloc/string/unchecked_access.nepl`
- `stdlib/alloc/string.nepl`
- `stdlib/**`
- `tests/stdlib/memory_safety.n.md`

## 根拠

- root `alloc/string` facade は `./string/access` を再公開するため、`access.nepl` 内の public unchecked reader は通常 import から到達可能だった。
- `string_byte_at_unchecked` は `[len:i32][bytes...]` の byte payload を直接読むため、`0 <= idx < len(s)` が守られない場合に `str` layout 外を読む。
- Stage 6 の方針では、raw memory backed public API は caller discipline ではなく source / type / Resource IR proof で境界を証明する必要がある。
- 2026-05-18 の一次修正後、`nodesrc/test_stdlib_string_access_boundary.js` が root re-export と `access.nepl` public unchecked reader の再導入を拒否する。

## 問題

旧実装では `alloc/string/access` が unchecked raw byte reader を public に持ち、root `alloc/string` facade からも見えていた。これは `byte_at` のような checked API と unchecked raw layout API が同じ public surface に並ぶ設計であり、通常 caller と compiler-owned stdlib implementation boundary を分けられない。

一次修正後も問題は完全には解決していない。`alloc/string/unchecked_access` を明示 import すれば unchecked reader はまだ public に呼べるため、現段階の修正は「safe facade からの accidental exposure を閉じる」ものであり、「bounds proof を型・Resource IR artifact として要求する」最終設計ではない。

## 影響

Root `alloc/string` と `alloc/string/access` の利用者は unchecked reader に到達しなくなったため、通常の byte access は `Option` を返す `byte_at` 経路へ戻った。これにより accidental public API としての危険は減った。

ただし `alloc/string/unchecked_access` はまだ public module であり、precondition を型で持たない。stdlib / selfhost の高速 scanner / parser / hashing loop を一括して checked API へ移すまでは必要な過渡境界だが、Stage 6 完了条件としては bounded byte index 証明、range scanner API、または compiler-owned private boundary へ移行する必要がある。

## 修正方針

1. `alloc/string/access` は safe public API だけを持つ。`byte_at` は `len` による範囲確認後に private raw helper を呼ぶ。
2. Root `alloc/string` facade は unchecked byte reader を再公開しない。
3. 既存 stdlib / selfhost の範囲証明済み hot path は、一次修正では明示的な `alloc/string/unchecked_access` import へ移し、safe facade との混同をなくす。
4. 最終修正では、unchecked reader を public raw function として残さない。選択肢は次のいずれかにする。
   - `BoundedByteIndex` のような proof/witness を checked bounds operation から作り、unchecked raw read はその witness を要求する。
   - scanner/range API へ call site を寄せ、caller が arbitrary index を渡せない形にする。
   - compiler-owned stdlib private boundary に閉じ、ordinary import から到達できないようにする。
5. この issue は 1-3 の一次修正後も open のまま維持し、4 の proof/witness 化または private boundary 化が完了した時点で fixed にする。

## 検証

- `nodesrc/test_stdlib_string_access_boundary.js` で root `alloc/string` の unchecked re-export、`access.nepl` の public unchecked reader、private raw helper の崩れを検出する。
- `tests/stdlib/memory_safety.n.md` に root `alloc/string` import と direct `alloc/string/access` import から `string_byte_at_unchecked` を呼べない compile-fail regression を追加する。
- `alloc/string/access.nepl` / `alloc/string/unchecked_access.nepl` の focused doctest と `tests/stdlib/memory_safety.n.md` を実行する。
- 最終修正時には、`alloc/string/unchecked_access` 直 import による arbitrary index 呼び出しも拒否される regression を追加する。

2026-05-18 一次修正の focused verification:

- `node nodesrc/test_stdlib_string_access_boundary.js`: passed
- `node nodesrc/test_stdlib_string_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_byte_scanner_helpers_boundary.js`: passed
- `trunk build`: passed
- `node nodesrc/tests.js -i tests\stdlib\memory_safety.n.md --no-tree -o tmp\agent1-string-unchecked-access-boundary-memory-safety.json -j 1 --dist web\dist --assert-io`: total=49, passed=49
- `node nodesrc/tests.js -i stdlib\alloc\string\access.nepl -i stdlib\alloc\string\unchecked_access.nepl --no-tree -o tmp\agent1-string-unchecked-access-boundary-docs.json -j 1 --dist web\dist --assert-io`: total=1, passed=1
- `node nodesrc/tests.js -i stdlib\tests\string.n.md --no-tree -o tmp\agent1-string-unchecked-access-boundary-string-tests.json -j 1 --dist web\dist --assert-io`: total=9, passed=9

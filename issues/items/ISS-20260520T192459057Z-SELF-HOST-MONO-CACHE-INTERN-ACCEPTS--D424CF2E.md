---
id: ISS-20260520T192459057Z-SELF-HOST-MONO-CACHE-INTERN-ACCEPTS--D424CF2E
title: "Self-host mono cache intern accepts invalid keys as records"
area: selfhost
status: fixed
resolved: true
priority: P2
type: bug
created: 2026-05-20
updated: 2026-05-21
target: "stdlib/neplg2/core/mono/mono.nepl, tests/stdlib/neplg2_mono.n.md, nodesrc/test_selfhost_mono_instance_absence.js"
---

# ISS-20260520T192459057Z-SELF-HOST-MONO-CACHE-INTERN-ACCEPTS--D424CF2E: Self-host mono cache intern accepts invalid keys as records

## 概要

`SelfhostMonoInstanceCache` の `intern` は、まだ登録されていない `SelfhostMonoInstanceKey` をそのまま typed record table へ保存していた。そのため、負の def id や負の type-argument range を持つ invalid key が assigned instance として cache に入る可能性があった。さらに `Err` は `StdErrorKind` だけだったため、invalid-key rejection と storage failure が owner-returning result 境界で区別できなかった。

## 対象

- `stdlib/neplg2/core/mono/mono.nepl, tests/stdlib/neplg2_mono.n.md, nodesrc/test_selfhost_mono_instance_absence.js`

## 根拠

- [ISS-20260425T000000Z-RV-STDLIB-008-F4BCB5DD](./ISS-20260425T000000Z-RV-STDLIB-008-F4BCB5DD.md) は self-host compiler 実装の親 issue であり、S4 monomorphization foundation を段階的に進めている。
- [ISS-20260520T185255970Z-SELF-HOST-MONO-NEEDS-TYPED-INSTANCE--71C76C96](./ISS-20260520T185255970Z-SELF-HOST-MONO-NEEDS-TYPED-INSTANCE--71C76C96.md) で cache storage を `Vec<SelfhostMonoInstanceRecord>` に移し、key/id の並列配列や seed identity を禁止した。
- 開発方針上、invalid state を文字列や数値 sentinel で後段へ流すのではなく、enum payload と match で静的に分岐できる error boundary に閉じる必要がある。

## 問題

`SelfhostMonoInstanceKey` には `selfhost_mono_instance_key_is_valid` が存在するが、cache intern はそれを storage mutation 前に呼んでいなかった。したがって、`SelfhostMonoDefId(-1, ...)` や負の type-arg range を持つ key でも lookup miss 後に record として追加される。これは cache を typed identity boundary とする設計に反し、後続 pass に invalid-key 再検査を漏らす。

## 影響

後続の trait impl lookup、HIR clone、mangle、codegen が invalid instance record を改めて検査する必要を持つ。これは mono cache が「valid key だけを intern 済み instance に対応させる」境界として働かない状態であり、self-host 実装が進むほど ad hoc な防御分岐が増える。

## 修正方針

次を実装した。

- `SelfhostMonoInstanceCacheInternError` を追加し、`InvalidKey(SelfhostMonoInstanceKey)` と `Storage(StdErrorKind)` を typed enum payload として分離した。
- `selfhost_mono_instance_cache_intern` が lookup / push の前に `selfhost_mono_instance_key_is_valid` を検査するようにした。
- invalid key の場合は cache owner を解放し、`InvalidKey(key)` を返すようにした。
- storage failure は `Storage(error)` で包み、`StdErrorKind` だけに collapse しないようにした。
- invalid key fixture と source policy を追加し、文字列 sentinel / 数値 sentinel の再導入を監視した。

## 検証

- `node nodesrc/test_selfhost_mono_instance_absence.js`
- `node nodesrc/tests.js -i stdlib/neplg2/core/mono/mono.nepl --no-tree --dist web/dist -o tmp/selfhost-mono-invalid-key-module.json -j 1 --assert-io`
- `node nodesrc/tests.js -i tests/stdlib/neplg2_mono.n.md --no-tree --dist web/dist -o tmp/selfhost-mono-invalid-key-fixture.json -j 1 --assert-io`
- `node nodesrc/issues.js check --dir issues`
- `git diff --check`

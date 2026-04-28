---
id: ISS-20260428T163153838Z-SELF-HOST-IMPORT-ITEM-CLASSIFIER-STI-A3A4A6E8
title: "self-host import item classifier still carries enum wildcard workaround"
area: selfhost
status: fixed
resolved: true
priority: P3
type: architecture
created: 2026-04-28
updated: 2026-04-28
target: "stdlib/neplg2/core/module/import_spec.nepl, tests/stdlib/neplg2_import_spec.n.md"
---

# ISS-20260428T163153838Z-SELF-HOST-IMPORT-ITEM-CLASSIFIER-STI-A3A4A6E8: self-host import item classifier still carries enum wildcard workaround

## 概要

core 側で enum match wildcard support が修正済みになったが、`selfhost_module_item_kind_is_import_directive` は compiler bug 回避のために non-import `SelfhostModuleItemKind` をすべて列挙したままだった。

## 対象

- `stdlib/neplg2/core/module/import_spec.nepl, tests/stdlib/neplg2_import_spec.n.md`

## 根拠

- `ISS-20260428T141727754Z-ENUM-MATCH-WILDCARD-ARM-IS-REJECTED--B1684C75` は `1166ee3 fix(core): support enum match wildcard arms` で fixed / resolved になっている。
- `stdlib/neplg2/core/module/import_spec.nepl` の `selfhost_module_item_kind_is_import_directive` は `ImportDirective` だけを true にし、それ以外を false にする単純な分類にもかかわらず、全 non-import variant を明示列挙していた。

## 問題

enum match wildcard support が core で修正済みになったあとも、`selfhost_module_item_kind_is_import_directive` は全 non-import `SelfhostModuleItemKind` variant を列挙していた。これは既に不要な compiler bug workaround であり、分類の意図を読みにくくしていた。

## 影響

self-host module code に古い workaround が残り、non-import item は default false という意図が見えにくくなる。今後 non-import variant を追加したときに、この helper の保守が不要に増える。

## 修正方針

non-import variant list を enum match wildcard arm に置き換え、import directive の抽出回帰で `ImportDirective` は収集され、non-import item は無視されることを確認する。

## 対応

- `selfhost_module_item_kind_is_import_directive` を `ImportDirective: true` / `_: false` の match に戻した。
- コメントも「全 variant を列挙する」説明から、non-import variant を default false として扱う説明へ更新した。

## 検証

- `node nodesrc/tests.js -i stdlib/neplg2/core/module/import_spec.nepl --no-tree -o tmp/neplg2-import-spec-wildcard-doctest.json -j 1`: total=1 passed=1
- `node nodesrc/tests.js -i tests/stdlib/neplg2_import_spec.n.md --no-tree -o tmp/neplg2-import-spec-wildcard-focused.json -j 1`: total=3 passed=3
- `trunk build`: pass
- `node nodesrc/tests.js -i stdlib/neplg2/core/module/import_spec.nepl --no-tree -o tmp/neplg2-import-spec-wildcard-doctest-after-build.json -j 1`: total=1 passed=1
- `node nodesrc/tests.js -i tests/stdlib/neplg2_import_spec.n.md --no-tree -o tmp/neplg2-import-spec-wildcard-focused-after-build.json -j 1`: total=3 passed=3

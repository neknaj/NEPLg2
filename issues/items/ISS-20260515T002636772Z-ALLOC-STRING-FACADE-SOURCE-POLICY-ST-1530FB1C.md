---
id: ISS-20260515T002636772Z-ALLOC-STRING-FACADE-SOURCE-POLICY-ST-1530FB1C
title: "alloc/string facade source policy still expects raw helper re-exports"
area: test
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-15
updated: 2026-05-15
target: "nodesrc/test_stdlib_string_facade_boundary.js, stdlib/alloc/string.nepl, stdlib/alloc/string/storage.nepl, stdlib/alloc/string/utf8.nepl"
---

# ISS-20260515T002636772Z-ALLOC-STRING-FACADE-SOURCE-POLICY-ST-1530FB1C: alloc/string facade source policy still expects raw helper re-exports

## 概要

nodesrc/test_stdlib_string_facade_boundary.js still requires alloc/string root to public re-export string/utf8 and string/storage, even though Stage 6 fixed the root facade to hide raw MemPtr/storage helpers behind explicit submodule imports.

## 対象

- `nodesrc/test_stdlib_string_facade_boundary.js, stdlib/alloc/string.nepl`

## 根拠

- `ISS-20260514T220733927Z-ALLOC-STRING-ROOT-RE-EXPORTS-RAW-STR-BF0F0254` で、`alloc/string` root は通常利用者向け safe facade とし、`string/storage` / `string/utf8` の raw `MemPtr` helper を明示 submodule import 境界へ閉じた。
- しかし `nodesrc/test_stdlib_string_facade_boundary.js` はまだ `pub #import "./string/utf8" as *` と `pub #import "./string/storage" as *` を root に要求していた。
- この policy のままだと Stage 6 の public/raw facade split を維持する検査ではなく、逆に raw helper 再公開を要求する検査になる。

## 問題

nodesrc/test_stdlib_string_facade_boundary.js still requires alloc/string root to public re-export string/utf8 and string/storage, even though Stage 6 fixed the root facade to hide raw MemPtr/storage helpers behind explicit submodule imports.

## 影響

The source policy warning hides real Stage 6 regressions and could pressure future changes to re-open raw string storage helpers through the ordinary safe facade.

## 修正方針

Update the policy to classify access/builder/search/slice/split/integer/float/concat/builder_ext/find as root safe re-exports, and assert string/utf8 and string/storage are not public root re-exports while their implementation modules remain directly importable by raw-boundary code.

## 検証

Run node nodesrc/test_stdlib_string_facade_boundary.js, node nodesrc/run_source_policy_regressions.js --warn-only, node nodesrc/issues.js check --dir issues, and git diff --check.

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画 Stage 6](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-6-stdlib--self-host-buffer-%E7%A7%BB%E8%A1%8C)

## 2026-05-15 Agent 1 解決

`nodesrc/test_stdlib_string_facade_boundary.js` を Stage 6 の現在設計に合わせた。`alloc/string` root が公開すべき safe API submodule は `access` / `builder` / `search` / `slice` / `split` / `integer` / `float` / `concat` / `builder_ext` / `find` として明示し、`storage` / `utf8` は root から public re-export されないことを検査する。

同時に `alloc/string/storage.nepl` と `alloc/string/utf8.nepl` が explicit raw-boundary import 用 module として存在し、source-level raw memory boundary evidence を持つことを確認対象へ入れた。これにより、safe facade の公開面と raw implementation 境界を同じ policy で監視できる。

検証:

- `node nodesrc/test_stdlib_string_facade_boundary.js`
- `node nodesrc/run_source_policy_regressions.js --warn-only` (`test_stdlib_string_facade_boundary.js` は pass。残警告は sort/merge policy、documentation contract、kpgraph policy の 3 件)
- `node nodesrc/issues.js check --dir issues`
- `git diff --check`

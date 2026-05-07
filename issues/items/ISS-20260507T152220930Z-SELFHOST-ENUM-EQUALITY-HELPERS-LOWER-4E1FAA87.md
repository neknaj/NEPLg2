---
id: ISS-20260507T152220930Z-SELFHOST-ENUM-EQUALITY-HELPERS-LOWER-4E1FAA87
title: "Selfhost enum equality helpers lower variants to numeric tags"
area: selfhost
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-07
updated: 2026-05-08
target: "stdlib/neplg2/core/ty/ty.nepl, stdlib/neplg2/core/hir/hir.nepl, stdlib/neplg2/core/builtins/prelude.nepl, stdlib/neplg2/core/resolve/name_resolver.nepl, nodesrc/test_selfhost_model_no_numeric_kind_tags.js"
---

# ISS-20260507T152220930Z-SELFHOST-ENUM-EQUALITY-HELPERS-LOWER-4E1FAA87: Selfhost enum equality helpers lower variants to numeric tags

## 概要

Selfhost TypeKind, HirExprKind, BuiltinKind, and DefKind equality helpers compare enum variants by converting them to i32 tag values. That makes ordinary numeric values the internal authority and prevents match exhaustiveness from guarding future variant additions.

## 対象

- `stdlib/neplg2/core/ty/ty.nepl, stdlib/neplg2/core/hir/hir.nepl, stdlib/neplg2/core/builtins/prelude.nepl, stdlib/neplg2/core/resolve/name_resolver.nepl, nodesrc/test_selfhost_model_no_numeric_kind_tags.js`

## 根拠

- 親 issue [ISS-20260507T150754473Z-SELFHOST-TYPE-HIR-AND-BUILTIN-MODELS-8EBC822D](./ISS-20260507T150754473Z-SELFHOST-TYPE-HIR-AND-BUILTIN-MODELS-8EBC822D.md) は、selfhost typed IR model が numeric tag / sentinel に依存していることを指摘している。
- `selfhost_type_kind_tag`、`selfhost_hir_expr_kind_tag`、`selfhost_builtin_kind_tag`、`selfhost_def_kind_tag` が enum variant を i32 へ落としていた。
- equality helper はそれらの tag を `eq` で比較しており、variant を直接 match していなかった。

## 問題

Selfhost TypeKind, HirExprKind, BuiltinKind, and DefKind equality helpers compare enum variants by converting them to i32 tag values. That makes ordinary numeric values the internal authority and prevents match exhaustiveness from guarding future variant additions.

## 影響

Selfhost resolve/type/HIR/builtin model changes can add or reorder enum variants without forcing equality logic to inspect each variant directly. This weakens the enum-first static-check policy and keeps the larger typed absence issue partially hidden behind numeric sentinels.

## 修正方針

Replace numeric tag conversion helpers with direct nested match equality helpers and add a source policy rejecting selfhost *_kind_tag helpers and uses.

## 検証

Run the new source policy, selfhost focused policy tests, issues check, and related doctests.

## 対応結果

2026-05-08 に selfhost model の enum equality helper を数値 tag 変換から直接 match へ移行した。

- `selfhost_type_kind_tag` / `selfhost_hir_expr_kind_tag` / `selfhost_builtin_kind_tag` / `selfhost_def_kind_tag` を削除。
- `selfhost_type_kind_eq` / `selfhost_hir_expr_kind_eq` / `selfhost_builtin_kind_eq` / `selfhost_def_kind_eq` は、左辺・右辺の enum variant を直接 match して比較する。
- `nodesrc/test_selfhost_model_no_numeric_kind_tags.js` を追加し、`*_kind_tag` helper と numeric tag 比較の再導入を拒否する。
- `nodesrc/run_source_policy_regressions.js` へ policy を接続した。

親 issue の invalid ID / empty range sentinel / builtin placeholder argument などは残件として継続する。この issue は enum-to-i32 tag equality の根本修正に限定して resolved とする。

検証:

- `node nodesrc/test_selfhost_model_no_numeric_kind_tags.js`
- `node nodesrc/tests.js -i stdlib/neplg2/core/ty/ty.nepl -i stdlib/neplg2/core/hir/hir.nepl -i stdlib/neplg2/core/builtins/prelude.nepl -i stdlib/neplg2/core/resolve/name_resolver.nepl --no-tree -o tmp/agent1-selfhost-enum-kind-eq.json -j 1 --dist web/dist`
- `node nodesrc/issues.js check`
- `git diff --check`
- `node nodesrc/run_source_policy_regressions.js --warn-only`

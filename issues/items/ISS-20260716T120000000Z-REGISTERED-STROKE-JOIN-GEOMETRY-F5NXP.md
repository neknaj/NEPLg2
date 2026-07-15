---
id: ISS-20260716T120000000Z-REGISTERED-STROKE-JOIN-GEOMETRY-F5NXP
title: "Registered stroke closure lacks scan-ready join geometry"
area: GUI_FONT
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-07-16
updated: 2026-07-16
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour.nepl
---

# 概要

F5nxo completed owner は registered glyph の closure topology、directed endpoint、join policy、miter limit、compact side-edge storage を保持するが、coverage scan が消費できる bevel / miter / round geometry をまだ生成しない。

# 実装境界

- F5nxo completed owner だけを direct authority とし、F5kz/F5lc owner、paint、glyph lookup、F5nxm/F5nxn drain を再構築しない。
- F5lc と registered 経路は owner に依存しない共通の数値 projection helper を使う。
- stroke width は side-edge offset normal evidence から読み、from/to の positive/equal invariant を再検査する。
- bevel は closure chord、line-only miter は交点と width/miter limit、line-only round は source center と二分 chord を保持する。quadratic を含む miter/round は evidence 付き bevel にする。
- exact-capacity geometry Vec、budget 0 不変、1 budget 1 push、terminal probe 非消費、push 後 commit、owner-bearing recovery を維持する。

# 完了条件

- bevel、miter、parallel/limit clip、round、quadratic bevel、Left/Right、failure recovery、exact completion を runtime fixture で検査する。
- F5lc focused regression、source policy、normal compile、trunk build、CLI JSON、GUI contract を通す。
- 差分 review と全体整合 review 後に一度だけ main へ統合する。

# 後続

registered coverage writer/scan、packed stroke mask、paint composition、glyph raster/runtime bridge、native/Web GUI 表示は別 phase とし、本 issue の完了を全体完成とは扱わない。

# 検証結果

- owner-neutral policy matrix 1/1、paint-bound production registered chain 1/1、F5lcを含むglyf module 2477/2477、registered owner module 52/52を通過した。
- Web GUI font contract、normal compile test-only isolation、source-policy runner、issues check、`git diff --check`、`trunk build`、trunk後Playground editor CLI JSON 13/13を通過した。
- 差分reviewのtest-only隔離とpayload不足を修正し、再reviewでBlocker/Majorなしを確認した。

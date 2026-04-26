---
id: ISS-20260426T021004000Z-IMPORT-VISIBILITY-CLONE-6F92C1A0
title: "typecheck import visibility expansion clones the whole map each iteration"
area: core
status: open
resolved: false
priority: P2
type: performance
created: 2026-04-26
updated: 2026-04-26
target: nepl-core/src/typecheck.rs
source: doc/neplg2/pre_selfhost_performance_audit_20260426.md
---

# ISS-20260426T021004000Z-IMPORT-VISIBILITY-CLONE-6F92C1A0: typecheck import visibility expansion clones the whole map each iteration

## 概要

`expand_unqualified_import_visibility` は transitive import visibility を閉包化するたびに `out.clone()` で map 全体を snapshot している。
module 数と import edge 数が増えると、clone 量と比較量が増えやすい。

## 根拠

- `nepl-core/src/typecheck.rs:9181` に `expand_unqualified_import_visibility` がある。
- `typecheck.rs:9183` は loop ごとに `let snapshot = out.clone();` を実行する。
- snapshot 内の `source_file -> middle_file -> target_file` を走査し、変更がなくなるまで繰り返す。

## 問題

self-host compiler は stdlib と compiler source tree の import graph を扱うため、module 数が現在より増える。
全体 clone による閉包計算は、小さな project では目立たなくても、stdlib 分割後の module graph で memory allocation と copy のノイズになり得る。

## 影響

`RV-STDLIB-009` の stdlib 分割を進めるほど import visibility map が増え、typecheck 前処理のコストが上がる可能性がある。
性能問題が module 分割の副作用として見え、self-host source tree の分割方針を誤らせる。

## 修正方針

Floyd-Warshall 風の全体 clone loop ではなく、import graph の adjacency を使った worklist / BFS で source ごとに可視性を伝播する。
`All` と `Selected` の merge 規則を保ったまま、変更があった edge だけを queue に戻す。

## 検証

- chain import、diamond import、selected + glob import の既存 semantics regression。
- module 数と edge 数を増やした synthetic import graph fixture で clone 回数または実行時間を測る。

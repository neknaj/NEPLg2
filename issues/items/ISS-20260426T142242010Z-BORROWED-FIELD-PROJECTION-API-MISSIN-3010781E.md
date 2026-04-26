---
id: ISS-20260426T142242010Z-BORROWED-FIELD-PROJECTION-API-MISSIN-3010781E
title: "borrowed field projection API missing for repeated aggregate reads"
area: core
status: open
resolved: false
priority: P2
type: architecture
created: 2026-04-26
updated: 2026-04-26
target: "stdlib/core/field.nepl; nepl-core move/borrow checker"
---

# ISS-20260426T142242010Z-BORROWED-FIELD-PROJECTION-API-MISSIN-3010781E: borrowed field projection API missing for repeated aggregate reads

## 概要

self-host CLI args tests を追加した時、`Result::Ok opts` で得た `SelfhostCliOptions` から `get opts "output"`、`get opts "input"`、`get opts "check"` のように複数 field を読む自然なコードが D3053 `use of moved value: opts` で拒否された。

## 対象

- `stdlib/core/field.nepl; nepl-core move/borrow checker`

## 根拠

- `core/field.get` は by-value API で、field を 1 回読むだけでも aggregate owner を消費する。
- 現状の回避策は `alloc_raw` に aggregate を `store` し、各 field 読み取りのたびに `load<SelfhostCliOptions>` してから `get` する形であり、高レベルの self-host compiler code に raw memory detour が漏れる。
- `&T` から field を読む borrowed projection API がないため、単に複数 field を観察したいだけの処理も所有権移動として表現するしかない。

## 問題

borrowed aggregate から field を読む public API / intrinsic がない。by-value `get` と raw memory reload の二択になるため、move checker の制約を避ける目的で `core/mem` を使う不自然な書き方が増える。

## 影響

Self-host parser, AST, options, and diagnostic structs will need many repeated field reads. Without borrowed field projection, high-level compiler code is pushed toward core/mem workarounds, making ownership intent unclear and making borrow checker limitations look like stdlib style.

## 修正方針

Design and implement borrowed field projection, for example a get_ref-style API or field projection intrinsic over &T. The implementation must distinguish Copy scalar reads, borrowed field references, and owned field moves so non-Copy fields are not silently duplicated. Update stdlib tests away from raw memory detours once the API exists.

## 検証

Add compiler/stdlib tests where a struct with multiple fields is borrowed and several fields are read without moving the owner. Add compile_fail coverage that by-value extraction of a non-Copy owned field still moves it, and that borrowed field references cannot outlive the owner.

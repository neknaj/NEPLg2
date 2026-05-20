---
id: ISS-20260520T185255970Z-SELF-HOST-MONO-NEEDS-TYPED-INSTANCE--71C76C96
title: "self-host mono needs typed instance cache storage"
area: selfhost
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-05-20
updated: 2026-05-21
target: "stdlib/neplg2/core/mono/mono.nepl, tests/stdlib/neplg2_mono.n.md, nodesrc/test_selfhost_mono_instance_absence.js, stdlib/neplg2/README.md"
---

# ISS-20260520T185255970Z-SELF-HOST-MONO-NEEDS-TYPED-INSTANCE--71C76C96: self-host mono needs typed instance cache storage

## 概要

Self-host mono has typed instance keys and records, but the cache storage boundary still needs an owner-carrying typed table that interns by full key equality and returns Option for absence.

## 対象

- `stdlib/neplg2/core/mono/mono.nepl, tests/stdlib/neplg2_mono.n.md, nodesrc/test_selfhost_mono_instance_absence.js, stdlib/neplg2/README.md`

## 根拠

- `doc/neplg2/self_host_plan.md` と `stdlib/neplg2/README.md` は S4 mono foundation で instance cache と name mangling を分離する方針を示している。
- [ISS-20260520T164036020Z-SELF-HOST-MONO-LACKS-TYPED-INSTANCE--93C7721D](./ISS-20260520T164036020Z-SELF-HOST-MONO-LACKS-TYPED-INSTANCE--93C7721D.md) で typed record は追加済みだが、cache owner はまだ無かった。
- cache storage を parallel `Vec<Key>` / `Vec<Id>` や seed-only lookup にすると、型で key/id 対応を表せず、静的検査で退行を捕まえにくい。

## 問題

Self-host mono has typed instance keys and records, but the cache storage boundary still needs an owner-carrying typed table that interns by full key equality and returns Option for absence.

## 影響

Without a typed cache owner, monomorphization would drift toward parallel key/id arrays, seed identity shortcuts, or invalid-id sentinels, weakening the static-checkable self-host design.

## 修正方針

Add SelfhostMonoInstanceCache with typed Vec<SelfhostMonoInstanceRecord> storage, owner-returning intern result, full-key lookup, record_at, and source policy coverage.

## 対応

- `SelfhostMonoInstanceCache` を追加し、`Vec<SelfhostMonoInstanceRecord>` を cache owner として保持するようにした。
- `SelfhostMonoInstanceCacheInternResult` を追加し、intern 後の cache owner と assigned `SelfhostMonoInstanceId` を同時に返すようにした。
- lookup / intern は `selfhost_mono_instance_record_matches_key` による full key equality を使い、`selfhost_mono_instance_key_seed` を identity として扱わない。
- absence は引き続き `Option<SelfhostMonoInstanceId>` で表し、invalid id sentinel は導入しない。
- focused doctest と source policy を更新し、parallel key/id Vec、seed identity shortcut、invalid id sentinel への退行を検出するようにした。
- `stdlib/neplg2/README.md` の S4 mono foundation 説明を cache storage 実装済みに更新した。

## 検証

node nodesrc/test_selfhost_mono_instance_absence.js; node nodesrc/tests.js -i stdlib/neplg2/core/mono/mono.nepl --no-tree -o tmp/selfhost-mono-module-cache.json -j 1 --assert-io; node nodesrc/tests.js -i tests/stdlib/neplg2_mono.n.md --no-tree -o tmp/selfhost-mono-cache-fixture.json -j 1 --assert-io

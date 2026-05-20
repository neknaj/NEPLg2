---
id: ISS-20260520T164036020Z-SELF-HOST-MONO-LACKS-TYPED-INSTANCE--93C7721D
title: "self-host mono lacks typed instance record model"
area: selfhost
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-05-20
updated: 2026-05-20
target: "stdlib/neplg2/core/mono/mono.nepl, tests/stdlib/neplg2_mono.n.md, nodesrc/test_selfhost_mono_instance_absence.js"
---

# ISS-20260520T164036020Z-SELF-HOST-MONO-LACKS-TYPED-INSTANCE--93C7721D: self-host mono lacks typed instance record model

## 概要

The self-host mono stage has typed instance keys and assigned IDs, but cache entries still lack a typed record tying a key to the assigned instance identity. The focused mono doctest also still references the removed invalid-ID API, so the regression gate no longer matches the Option-based absence model.

## 対象

- `stdlib/neplg2/core/mono/mono.nepl, tests/stdlib/neplg2_mono.n.md, nodesrc/test_selfhost_mono_instance_absence.js`

## 根拠

- `doc/neplg2/self_host_plan.md` S4 は `mono/` で instance cache と name mangling を分離するとしている。
- `stdlib/neplg2/core/mono/mono.nepl` は `SelfhostMonoInstanceKey` と `SelfhostMonoInstanceId` を持っていたが、cache entry として key / id を束ねる typed record が無かった。
- `tests/stdlib/neplg2_mono.n.md` の invalid id fixture は削除済みの `selfhost_mono_instance_id_invalid` / `selfhost_mono_instance_id_is_valid` を参照しており、`nodesrc/test_selfhost_mono_instance_absence.js` の Option absence policy と矛盾していた。

## 問題

The self-host mono stage has typed instance keys and assigned IDs, but cache entries still lack a typed record tying a key to the assigned instance identity. The focused mono doctest also still references the removed invalid-ID API, so the regression gate no longer matches the Option-based absence model.

## 影響

Future mono cache work would have to pass loose key/id pairs between lookup, name mangling, and codegen, and the stale doctest can fail for the wrong reason instead of protecting the typed absence design.

## 修正方針

Add a Copy SelfhostMonoInstanceRecord model with constructor/accessors/key-match helper, exercise it in stage0 and focused doctests, and extend the mono source policy to keep the record key/id typed while preserving Option-based absence.

## 修正

- `SelfhostMonoInstanceRecord` を追加し、lookup key と assigned `SelfhostMonoInstanceId` を typed cache-entry value として保持するようにした。
- `selfhost_mono_instance_record_new`、key/id accessor、`selfhost_mono_instance_record_matches_key` を追加した。record lookup は seed ではなく `selfhost_mono_instance_key_eq` による full key equality を使う。
- `selfhost_mono_stage0` と `tests/stdlib/neplg2_mono.n.md` に record fixture を追加した。
- stale invalid-id doctest を `Option<SelfhostMonoInstanceId>` の pending / assigned check に更新した。
- `nodesrc/test_selfhost_mono_instance_absence.js` に record model と seed identity 不使用の source policy を追加した。
- `stdlib/neplg2/README.md` の S4 mono foundation 説明を更新した。

## 検証

- `node nodesrc/test_selfhost_mono_instance_absence.js`: passed
- `node nodesrc/tests.js -i stdlib/neplg2/core/mono/mono.nepl --no-tree --dist web/dist -o tmp/agent2-selfhost-mono-module.json -j 1 --assert-io`: total=1, passed=1
- `node nodesrc/tests.js -i tests/stdlib/neplg2_mono.n.md --no-tree --dist web/dist -o tmp/agent2-selfhost-mono-fixture.json -j 1 --assert-io`: total=3, passed=3

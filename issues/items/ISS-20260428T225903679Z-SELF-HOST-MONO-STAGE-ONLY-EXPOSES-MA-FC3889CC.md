---
id: ISS-20260428T225903679Z-SELF-HOST-MONO-STAGE-ONLY-EXPOSES-MA-FC3889CC
title: "self-host mono stage only exposes marker API and lacks instance key model"
area: selfhost
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-04-28
updated: 2026-04-28
target: "stdlib/neplg2/core/mono/mono.nepl, tests/stdlib/neplg2_mono.n.md"
---

# ISS-20260428T225903679Z-SELF-HOST-MONO-STAGE-ONLY-EXPOSES-MA-FC3889CC: self-host mono stage only exposes marker API and lacks instance key model

## 概要

stdlib/neplg2/core/mono/mono.nepl is still a Stage 0 marker, so later monomorphize work has no typed representation for a generic function instance key or deterministic symbol identity.

## 対象

- `stdlib/neplg2/core/mono/mono.nepl, tests/stdlib/neplg2_mono.n.md`

## 根拠

- `doc/neplg2/self_host_plan.md` の S4 は `mono/` で instance cache と name mangling を分離するとしている。
- `stdlib/neplg2/core/mono/mono.nepl` は現在 `selfhost_mono_stage0` だけを返す 26 行の marker module で、generic instance を識別する typed key を持たない。
- `doc/neplg2/self_host_execution_plan.md` でも S4 commit 単位として `selfhost/s4-mono-instance` が予定されている。

## 問題

stdlib/neplg2/core/mono/mono.nepl is still a Stage 0 marker, so later monomorphize work has no typed representation for a generic function instance key or deterministic symbol identity.

## 影響

S4 monomorphize work would otherwise spread ad hoc module/function/type-argument tuples and mangling seeds across lowering, cache, and codegen, making parity with the Rust compiler harder to test.

## 修正方針

Add a small Copy instance key model with module/function/type-argument range fields, equality, validity checks, and a deterministic mangle seed helper. Keep cache storage and trait impl lookup for later issues.

## 修正内容

- `SelfhostMonoDefId`、`SelfhostMonoTypeArgRange`、`SelfhostMonoInstanceKey`、`SelfhostMonoInstanceId` を追加し、generic instance identity を typed value として表せるようにした。
- key / range / id の constructor、validity check、key equality を追加した。
- `selfhost_mono_instance_key_seed` を追加し、name mangling / cache bucket に使える deterministic non-crypto seed を key から作れるようにした。
- `selfhost_mono_stage0` を marker return ではなく key identity / seed / id の smoke check に更新した。
- `tests/stdlib/neplg2_mono.n.md` と `stdlib/neplg2/README.md` の検証コマンドを追加した。

## 検証

- `node nodesrc/tests.js -i stdlib\neplg2\core\mono\mono.nepl --no-tree -o tmp\selfhost-mono-instance-key-mono-2.json -j 1`: total=1, passed=1
- `node nodesrc/tests.js -i tests\stdlib\neplg2_mono.n.md --no-tree -o tmp\selfhost-mono-instance-key-tests.json -j 1`: total=2, passed=2
- `node nodesrc/tests.js -i stdlib\neplg2 --no-tree -o tmp\selfhost-mono-instance-key-neplg2.json -j 1`: total=32, passed=19, failed=13。mono doctest は pass。失敗は既存の Vec element provenance と、別途 issue 化する `selfhost_cli_parse_args` / `selfhost_cli_parse_argv` の `VecDataLen` field move D3100。
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass

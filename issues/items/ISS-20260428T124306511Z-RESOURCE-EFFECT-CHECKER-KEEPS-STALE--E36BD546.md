---
id: ISS-20260428T124306511Z-RESOURCE-EFFECT-CHECKER-KEEPS-STALE--E36BD546
title: "Resource effect checker keeps stale raw pointer aliases after assignment"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "nepl-core/src/resource/effect.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260428T124306511Z-RESOURCE-EFFECT-CHECKER-KEEPS-STALE--E36BD546: Resource effect checker keeps stale raw pointer aliases after assignment

## 概要

RawPointerAliasTable::copy_alias unions the target into the source alias group without removing the target from its previous group, and RawIdentityTable::copy_identity similarly leaves stale target identity state. RawMemoryIdentityTable can therefore keep a reassigned pointer variable inside an old memory-slot identity group.

## 対象

- `nepl-core/src/resource/effect.rs`
- `nepl-core/tests/resource_ir.rs`

## 根拠

- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 5 は raw identity / pointer provenance を safe surface から閉じる計画であり、raw memory slot に保存された identity を pointer alias state と一貫して追跡する必要がある。
- `RawPointerAliasTable::copy_alias` は target を旧 alias group から外さずに source group と union していたため、pointer variable の再代入後も旧 storage alias と新 storage alias が同じ group に残った。
- `RawMemoryIdentityTable` は pointer alias group 単位で slot payload identity を保持する。target place が旧 memory identity group に残ると、再代入後の `store` が旧 storage の identity state まで clear できる。
- `RawIdentityTable::copy_identity` も source が identity を持たない場合に target の古い identity state を消さず、raw value identity の上書き状態と pointer alias の上書き状態が一致していなかった。

## 問題

RawPointerAliasTable::copy_alias unions the target into the source alias group without removing the target from its previous group, and RawIdentityTable::copy_identity similarly leaves stale target identity state. RawMemoryIdentityTable can therefore keep a reassigned pointer variable inside an old memory-slot identity group.

## 影響

A store through a reassigned pointer can clear raw identity state for the previous storage, so a later load from the original pointer may fail to report internal raw allocation escape. This weakens Stage 5 raw identity escape checking.

## 修正方針

Make raw identity and pointer alias copy operations overwrite the target state. Remove the target place from raw-memory identity groups before pointer alias overwrite so memory-slot identity stays attached to the old storage aliases, not to the reassigned variable.

## 検証

- `cargo test -p nepl-core --test resource_ir -- --nocapture`
- `trunk build`
- `node nodesrc/issues.js check`
- `rustfmt --check nepl-core\src\resource\effect.rs nepl-core\tests\resource_ir.rs`
- `git diff --check`

## 2026-04-28 Stage 5 raw pointer alias overwrite 対応

`RawIdentityTable::copy_identity` と `RawPointerAliasTable::copy_alias` を target state の上書き操作にした。copy 前に target place を既存 group から外し、source 側が identity / alias を持つ場合だけ target を新しい group に加える。

pointer alias の上書き前には `RawMemoryIdentityTable::remove_place` で target place を memory identity group からも外す。これにより pointer variable の再代入後、旧 storage に保存された raw identity は旧 storage aliases に残り、新しい pointer への `store` で誤って clear されない。

aggregate construction は複数 input のうち raw identity を持つものを aggregate output に集約する必要があるため、identity copy の overwrite 化とは別に `merge_identity` を使うようにした。

`nepl-core/tests/resource_ir.rs` に、ptr A に raw identity を store し、local pointer `p` を ptr A から ptr B へ再代入して `p` 経由で non-identity を store しても、ptr A から load した値の raw address escape が検出される回帰を追加した。

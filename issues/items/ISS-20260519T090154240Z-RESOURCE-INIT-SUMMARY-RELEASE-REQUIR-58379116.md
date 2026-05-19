---
id: ISS-20260519T090154240Z-RESOURCE-INIT-SUMMARY-RELEASE-REQUIR-58379116
title: "Resource init summary release requirements seed plain string views as raw-address carriers"
area: core
status: fixed
resolved: true
priority: P1
type: performance
created: 2026-05-19
updated: 2026-05-19
target: "nepl-core/src/resource/initialized_summary_*"
---

# ISS-20260519T090154240Z-RESOURCE-INIT-SUMMARY-RELEASE-REQUIR-58379116: Resource init summary release requirements seed plain string views as raw-address carriers

## 概要

Resource init summary の release requirement collection が全 parameter を raw-address alias 起点として扱い、str や str/i32 だけを持つ plain aggregate から作った non-owning raw view まで caller 側 release requirement として要約していた。NM parser/html のような大きい immutable string view 処理で、BulkSource requirement が意味なく増え、raw_init summary stage が過大になっていた。

## 対象

- `nepl-core/src/resource/initialized_summary_*`

## 根拠

- 直前の `examples/nm.nepl` probe では、variant summary pruning 後も `resource_initialized_raw_init_summaries=76306ms`、`resource_initialized_moves=95503ms` で止まっていた。
- release requirement collection は `ResourceCheckEngine` の raw-address view simulation により、普通の initialized parameter cell から作った view まで parameter alias とみなしていた。
- `str` / plain aggregate は型上 non-copy raw cell を保持できないため、caller 側の release requirement にする必要がない。一方、top-level raw i32 address と compiler memory identity で証明済みの `MemPtr` / `RegionToken` carrier は requirement を残す必要がある。
- 修正後 probe では `resource_initialized_raw_init_summaries=43194ms`、`resource_initialized_moves=65689ms` まで改善した。

## 問題

Resource init summary の release requirement collection が全 parameter を raw-address alias 起点として扱い、str や str/i32 だけを持つ plain aggregate から作った non-owning raw view まで caller 側 release requirement として要約していた。NM parser/html のような大きい immutable string view 処理で、BulkSource requirement が意味なく増え、raw_init summary stage が過大になっていた。

## 影響

examples/nm.nepl の full compile が CI budget を超え、静的検査の regression / deploy確認を不安定にする。型上 non-copy raw cell を保持できない source view まで release 証明対象にするため、summary graph の計算量も増える。

## 修正方針

stdlib/nm 名や関数名の allowlist ではなく、TypeCtx と Resource IR の型分類で summary input が raw address carrier になり得るかを判定する。str/plain aggregate は release requirement の param seed から外し、top-level i32 raw address と登録済み MemPtr/RegionToken carrier は維持する。

## 検証

cargo test -p nepl-core initialized_summary_ -- --nocapture; cargo fmt -p nepl-core -- --check; node nodesrc/test_resource_checker_responsibility.js; examples/nm.nepl stage timing comparison

## 関連

- parent performance issue: `ISS-20260519T074504799Z-NM-FULL-COMPILE-STILL-EXCEEDS-CI-BUD-5653B487`
- Stage: `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 6

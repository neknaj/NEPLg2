---
id: ISS-20260528T110220373Z-RESOURCE-SUMMARY-CACHE-NEEDS-QUALIFI-08D1AA04
title: "Resource summary cache needs qualified nominal stable type identity"
area: core
status: open
resolved: false
priority: P1
type: performance
created: 2026-05-28
updated: 2026-05-28
target: "nepl-core/src/resource/resource_summary_value_cache/*; nepl-core/src/types.rs"
---

# ISS-20260528T110220373Z-RESOURCE-SUMMARY-CACHE-NEEDS-QUALIFI-08D1AA04: Resource summary cache needs qualified nominal stable type identity

## 概要

Resource summary value cache は、現時点で `Named` / `Struct` / `Enum` の stable type key を拒否している。型名だけでは別 module / 別定義の同名型へ stale hit し得るためである。RPN の raw-init param facts 測定では `raw_init_param_facts_stores=0` のまま bypass が増えており、subagent review でも nominal 型を多く含む stdlib summary が candidate 化できない可能性が高いと確認した。

## 対象

- `nepl-core/src/resource/resource_summary_value_cache/*; nepl-core/src/types.rs`

## 根拠

- release WASM RPN same-session code-edit 測定で `raw_init_param_facts_stores=0`、`raw_init_param_facts_bypasses=225` を確認した。
- `ResourceSummaryStableTypeKey` は qualified definition identity がない nominal type を安全側に倒して拒否する。
- raw-init param facts cache は `RawCellInitializationParamCell` / simple `RawCellReleaseParamRequirement` の足場を持つが、nominal payload / storage carrier を含む stdlib summary では stable key 化できない候補が多い。

## 問題

Resource summary value cache が nominal type を stable key として表現できないため、型を含む summary value を長寿命 `CompilerSession` cache に保存できない。unqualified type name だけで保存すると、別 module の同名型や public surface edit 後の別定義へ誤って hit する危険がある。

## 影響

安全側に倒すことで stale hit は避けられるが、stdlib-heavy program では Resource summary value reuse が効きにくい。RPN のような workload では comment-only edit は compiled-output cache で 10ms 未満になっても、実コード edit は raw initialization summary / function check で秒単位の full compile に戻る。

## 修正方針

長寿命 cache key に使える qualified nominal type identity を導入する。identity は `TypeId` や `Span` ではなく、module path / canonical definition identity / public surface invalidation に由来する値にする。`ResourceSummaryStableTypeKey` はこの identity を使って nominal type を stable 表現へ落とし、hit 時には現在 compile の `TypeCtx` へ再投影できる場合だけ store/hit を許可する。

## 検証

- `cargo test -p nepl-core resource_summary_value -- --nocapture`
- public type edit / dependency public surface edit / stdlib overlay で stale hit しない regression
- RPN same-session code-edit 測定で nominal stdlib summary の Resource summary store / hit が非 0 になること

---
id: ISS-20260512T201359246Z-RESOURCE-CHECKER-RESPONSIBILITY-POLI-E382D3AB
title: "Resource checker responsibility policy misses newer Resource IR modules"
area: core
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-05-12
updated: 2026-05-12
target: "nodesrc/test_resource_checker_responsibility.js; nepl-core/src/resource"
---

# ISS-20260512T201359246Z-RESOURCE-CHECKER-RESPONSIBILITY-POLI-E382D3AB: Resource checker responsibility policy misses newer Resource IR modules

## 概要

Resource IR の responsibility source policy は主要 module の存在と行数上限を監視しているが、後続の Stage 4/5 作業で追加された storage/source/owner/trait helper module が一覧から漏れている。未監視 module は責務再集中や line limit 超過を検出できず、memory safety authority の保守性を落とす。

## 対象

- `nodesrc/test_resource_checker_responsibility.js; nepl-core/src/resource`

## 根拠

- `nepl-core/src/resource` には `condition_fact.rs`、`drop_elaboration_hir_bridge.rs`、`function_alias.rs`、`lower_raw_address_source.rs`、`owner_summary_raw_transfer.rs`、`owner_transfer.rs`、`trait_identity.rs` など Stage 4/5 で追加された module が存在する。
- `nodesrc/test_resource_checker_responsibility.js` は主要 module の存在確認と line limit を持つが、上記 module の一部を `maxLines` に含めていなかった。
- 未監視 module は `resource/mod.rs` に追加されても policy が落ちないため、Resource IR の responsibility split が今後の変更で形骸化し得る。

## 問題

Resource IR の responsibility source policy は主要 module の存在と行数上限を監視しているが、後続の Stage 4/5 作業で追加された storage/source/owner/trait helper module が一覧から漏れている。未監視 module は責務再集中や line limit 超過を検出できず、memory safety authority の保守性を落とす。

## 影響

Resource IR の owner/provenance/initialized/effect 境界が module 分割されたあと、追加 module が policy に入らないまま肥大化すると、旧 HIR special-case と同じように検査責務が局所的に積み上がる。型安全・メモリ安全検査の根拠が散逸し、静的検査大規模修正の完了条件を継続的に保証できない。

## 修正方針

resource 配下の Rust module を責務 policy の必須存在・mod 宣言・line limit に追加し、今後追加 module が監視対象から漏れないようにする。特に storage origin、raw address source、owner transfer、trait identity、type pattern、raw realloc、function alias など Stage 4/5 の memory safety helper を対象に含める。

## 対応記録

- `nodesrc/test_resource_checker_responsibility.js` に未監視だった Resource IR helper module の line limit を追加した。
- policy が future module drift を見逃さないよう、`nepl-core/src/resource/*.rs` の全 module が line limit を持つことを検査するようにした。
- `resource/mod.rs` 直下の module だけでなく、`#[path = "..."]` で親 module から読み込む test module も宣言済みとして扱う検査を追加した。
- これにより、今後 Resource IR の memory safety helper を追加した場合は、責務上限を同時に設計しなければ source policy が失敗する。

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)

## 検証

node nodesrc/test_resource_checker_responsibility.js; node nodesrc/run_source_policy_regressions.js --warn-only; node nodesrc/issues.js check --dir issues

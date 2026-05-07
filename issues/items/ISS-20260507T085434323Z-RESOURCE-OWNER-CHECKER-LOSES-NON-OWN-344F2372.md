---
id: ISS-20260507T085434323Z-RESOURCE-OWNER-CHECKER-LOSES-NON-OWN-344F2372
title: "Resource owner checker loses non-owning raw address views through aggregate payloads"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-07
updated: 2026-05-07
target: "nepl-core/src/resource/owner_raw_view.rs, nepl-core/src/resource/owner_raw_view_table.rs, nepl-core/src/resource/owner_check.rs, nepl-core/src/resource/owner_control.rs, nepl-core/src/resource/owner_flow.rs, nepl-core/src/resource/owner_return_apply.rs, nepl-core/src/resource/owner_return_view.rs, nepl-core/src/resource/owner_summary.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260507T085434323Z-RESOURCE-OWNER-CHECKER-LOSES-NON-OWN-344F2372: Resource owner checker loses non-owning raw address views through aggregate payloads

## 概要

RawAddressViewKind::NonOwningProjection is preserved for direct str_addr / borrowed region_ptr outputs, but Construct/branch/match owner flow does not carry the raw view marker or its storage origin into aggregate and enum payload places. A non-owning raw address view can therefore be wrapped in Result::Ok or another aggregate, matched back out, and used as if it were an untracked raw owner.

## 対象

- `nepl-core/src/resource/owner_raw_view.rs, nepl-core/src/resource/owner_raw_view_table.rs, nepl-core/src/resource/owner_check.rs, nepl-core/src/resource/owner_control.rs, nepl-core/src/resource/owner_flow.rs, nepl-core/src/resource/owner_return_apply.rs, nepl-core/src/resource/owner_return_view.rs, nepl-core/src/resource/owner_summary.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- `str_addr` 由来の `RawAddressViewKind::NonOwningProjection` は direct local では `dealloc_raw` / `realloc_raw` の owner として拒否される。
- しかし `Result::Ok addr` の payload に格納してから `match` bind すると、caller 側の payload place / bind local / read temporary へ non-owning raw view fact が伝播せず、raw owner が存在しない値を未追跡値として扱っていた。
- Resource IR owner summary 自体は payload projection に non-owning raw view marker を生成できるため、原因は summary 生成ではなく value-preserving owner-flow への non-owning fact copy 不足だった。

## 問題

RawAddressViewKind::NonOwningProjection is preserved for direct str_addr / borrowed region_ptr outputs, but Construct/branch/match owner flow does not carry the raw view marker or its storage origin into aggregate and enum payload places. A non-owning raw address view can therefore be wrapped in Result::Ok or another aggregate, matched back out, and used as if it were an untracked raw owner.

## 影響

Safe code can route str_addr-derived or borrowed pointer projections through enum/aggregate payloads and attempt dealloc/realloc without an OwnerUnavailable diagnostic. This violates the MemPtr = non-owning pointer / OwnedRegion = free obligation owner split required by the static check complexity reduction plan.

## 修正方針

Propagate non-owning raw address view state through aggregate construction, branch/match result moves, match payload binding, call return summaries, and local read copies as typed raw-view state, while keeping owner transfer blocked for the view itself. Add Resource IR owner regressions for Result payload wrapping of str_addr-derived raw views.

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_rejects_dealloc_through_result_wrapped_str_addr_view -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir str_addr -- --nocapture`: passed
- `cargo fmt --check -p nepl-core`: passed
- `cargo check -p nepl-core --tests`: passed
- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed

## 対応結果

`RawAddressViewTable` に通常 raw address view と non-owning raw address view の区別を持たせ、aggregate / branch / match / call return summary では non-owning fact だけを伝播するようにした。`OwnerState::NoFreeObligation` は enum payload の汎用 owner marker として残し、non-owning pointer authority には使わない。これにより `str_addr` / borrowed projection を `Result::Ok` などの aggregate payload に包んでも、payload bind 後の read temporary が owner として解放されることはなく、`OwnerUnavailable` で拒否される。

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)

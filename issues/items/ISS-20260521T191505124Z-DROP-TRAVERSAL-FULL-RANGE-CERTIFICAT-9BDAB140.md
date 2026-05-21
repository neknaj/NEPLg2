---
id: ISS-20260521T191505124Z-DROP-TRAVERSAL-FULL-RANGE-CERTIFICAT-9BDAB140
title: "Drop traversal full-range certificate outlives Resource IR state changes"
area: compiler
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-21
updated: 2026-05-22
target: "nepl-core/src/resource/collection_slot_summary_build_*.rs, nepl-core/src/resource/collection_slot_summary_model.rs"
---

# ISS-20260521T191505124Z-DROP-TRAVERSAL-FULL-RANGE-CERTIFICAT-9BDAB140: Drop traversal full-range certificate outlives Resource IR state changes

## 概要

CollectionSlotDropTraversal の full-range summary certificate が loop から生成された後、storage / initialized_count / slot state / alias state の変更で失効せず、後続 traversal で storage/count/type だけが一致すれば再利用され得る。

## 対象

- `nepl-core/src/resource/collection_slot_summary_build_*.rs, nepl-core/src/resource/collection_slot_summary_model.rs`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) Stage 6 は、collection slot lifecycle / drop traversal を stdlib module allowlist ではなく Resource IR の generic proof boundary で扱う方針を定めている。
- [ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md) は non-Copy collection payload support の親 issue であり、full initialized range cleanup を汎用 Resource IR proof として閉じる必要がある。
- `collect_summary_ops_from_ops` は loop から `CollectionSlotInitializedRangeDropTraversalCertificate` を生成して `CollectionSlotSummaryBuildState.drop_traversal_range_certificates` に蓄積していた。
- `collect_summary_drop_traversal_op` は後続 `CollectionSlotDropTraversal` の `storage / initialized_count / expected_ty` が一致すれば certificate を再利用していた。
- 生成後から traversal までに `storage`、`initialized_count`、storage 配下の slot lifecycle、または alias/scalar state に関係する operation が入った場合、loop が証明した range と traversal 時点の state は同一とは限らない。

## 問題

CollectionSlotDropTraversal の full-range summary certificate が loop から生成された後、storage / initialized_count / slot state / alias state の変更で失効せず、後続 traversal で storage/count/type だけが一致すれば再利用され得る。

## 影響

loop が証明した範囲と実際の CollectionSlotDropTraversal の時点の collection state がずれ、helper summary が caller 側の live non-Copy slot を drop 済みとして replay する可能性がある。これは generic Resource IR proof の健全性に関わり、stdlib allowlist ではなく証明寿命の設計で閉じる必要がある。

## 修正方針

full-range certificate を単なる蓄積 Vec ではなく Resource IR state に結び付いた typed evidence として扱い、loop 生成後から traversal までに関係する storage / initialized_count / slot state / alias/scalar state が変わる operation で失効させる。ResourceOp の enum を match で網羅的に分類し、新しい op 追加時に検査漏れが検出される構造にする。

## 対応

- `collection_slot_summary_build_range_lifetime.rs` を追加し、`DropTraversalRangeCertificateEffect::{Preserves, Invalidates}` と `ResourceOp` の exhaustive match で、full-range certificate が後続 operation を越えて有効でいられるかを判定するようにした。
- `CollectionSlotSummaryBuildState` に certificate retention boundary を置き、summary build は各 operation の本体検査後に既存 certificate を失効判定し、その後で同じ loop から得た新しい certificate を post-loop state に結び付けて登録する。
- `Expr` などの無関係な temporary 生成は certificate を維持する一方、`storage` / `initialized_count` への assignment、storage 配下の `CollectionSlotLifecycle`、storage に触れる raw memory operation / call / relocate / traversal は certificate を失効させる。
- `collection_slot_summary_build_range_lifetime_tests.rs` を追加し、storage/count/slot state の post-loop 変更では `ForallInitializedRange` summary が生成されず、無関係な scalar temporary では生成されることを固定した。
- `nodesrc/test_resource_checker_responsibility.js` に新規 Resource IR module を追加し、責務監視を緩めずに分割後の file line limit を維持した。

## 検証

loop 直後の traversal は ForallInitializedRange を生成すること、loop と traversal の間で storage または initialized_count または slot state が変わる場合は ForallInitializedRange を生成しないこと、既存 collection_slot_summary_build_range_certificate tests と cargo check が通ること。

実行した検証:

- `cargo test -p nepl-core --lib collection_slot_summary_loop_induction -- --test-threads=1`
- `cargo test -p nepl-core --lib collection_slot_summary_loop_certificate -- --test-threads=1`
- `cargo test -p nepl-core --lib collection_slot_summary_build -- --test-threads=1`
- `cargo test -p nepl-core --lib collection_slot_summary_forall_replay -- --test-threads=1`
- `cargo check -p nepl-core`
- `cargo fmt --check -p nepl-core`
- `node nodesrc/test_resource_checker_responsibility.js`
- `node nodesrc/issues.js check --dir issues`
- `git diff --check`

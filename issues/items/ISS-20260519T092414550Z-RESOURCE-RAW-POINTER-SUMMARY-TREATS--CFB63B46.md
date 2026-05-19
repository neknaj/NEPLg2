---
id: ISS-20260519T092414550Z-RESOURCE-RAW-POINTER-SUMMARY-TREATS--CFB63B46
title: "Resource raw pointer summary treats enum owner storage as raw carriers"
area: core
status: fixed
resolved: true
priority: P1
type: performance
created: 2026-05-19
updated: 2026-05-19
target: "nepl-core/src/resource/effect_return_owner_type.rs; nepl-core/src/resource/raw_pointer_type.rs; nepl-core/src/resource/raw_pointer_owner_carrier_tests.rs; nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260519T092414550Z-RESOURCE-RAW-POINTER-SUMMARY-TREATS--CFB63B46: Resource raw pointer summary treats enum owner storage as raw carriers

## 概要

Resource IR raw pointer summary carrier detection recognized owner-backed structs but did not recurse enum variant payloads when deciding whether a type is an owner carrier. Enum-backed storage such as ByteBuilderStorage::Owned(RegionToken) was therefore treated as a raw pointer carrier only because it also carried i32 metadata through StringBuilder/ByteBuilder, causing resource_effect_boundaries to spend minutes summarizing ordinary owner-backed builders.

## 対象

- `nepl-core/src/resource/effect_return_owner_type.rs; nepl-core/src/resource/raw_pointer_type.rs; nepl-core/src/resource/raw_pointer_owner_carrier_tests.rs; nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `examples/nm.nepl` の stage timing probe で `resource_effect_boundaries` が 140 秒以上戻らず、詳細 instrumentation では `nm_inline_to_json_into__StringBuilder_str__StringBuilder__pure` の raw pointer return summary で停止していた。
- `StringBuilder` / `ByteBuilder` は `ByteBuilderStorage::Owned(RegionToken)` と `len` / `cap` metadata を持つ。旧 `raw_identity_type_is_structural_owner_carrier` は struct payload だけを辿り enum payload を辿らなかったため、owner-backed storage enum を raw-address carrier と誤分類した。
- この誤分類により、普通の builder metadata `i32` が raw pointer summary の public carrier として広がり、Stage 6 の summary dependency graph が不要に膨らんでいた。
- 修正後 probe では `resource_effect_boundaries=786ms`、`resource_raw_pointer_summary_recomputations=238 summaries=27`、`resource_raw_identity_summary_recomputations=238 summaries=31` まで改善した。

## 問題

Resource IR raw pointer summary carrier detection recognized owner-backed structs but did not recurse enum variant payloads when deciding whether a type is an owner carrier. Enum-backed storage such as ByteBuilderStorage::Owned(RegionToken) was therefore treated as a raw pointer carrier only because it also carried i32 metadata through StringBuilder/ByteBuilder, causing resource_effect_boundaries to spend minutes summarizing ordinary owner-backed builders.

## 影響

examples/nm.nepl exceeded the CI compile budget in Stage 6 resource_effect_boundaries, and performance diagnosis was obscured by a non-generic false carrier classification. Leaving this unchecked would also keep owner-backed enum storage semantically different from owner-backed struct storage.

## 修正方針

Make structural owner-carrier detection recurse through enum payloads and applied enum payloads using TypeCtx shape proof, then keep raw pointer summary carrier predicates from treating such owner-backed storage metadata as raw address carriers. Add a regression test with an owner-token payload enum inside a builder struct.

## 検証

Run raw pointer type unit tests, focused Resource checker responsibility policy, issue check, source policy regressions, git diff check, and an examples/nm.nepl stage-timing probe.

## 解決内容

`raw_identity_type_is_structural_owner_carrier` を struct 専用の探索から、enum payload / applied enum payload も含む `raw_identity_type_contains_opaque_owner` へ統一した。これにより `RegionToken` を payload に持つ storage enum と、それを field に持つ builder struct は structural owner carrier として扱われ、`type_can_carry_raw_pointer_alias_summary` から除外される。

この修正は `StringBuilder` や `ByteBuilder` の名前を列挙しない。TypeCtx に登録済みの owner token と型構造から owner carrier 性を証明するため、他の enum-backed owner storage にも同じ判定が適用される。

回帰テストとして、`ByteBuilderStorage::Owned(RegionToken)` と `ByteBuilder { storage, len, cap }` を合成し、storage enum と builder struct の両方が raw pointer summary carrier にならないことを固定した。

## 検証結果

- `cargo test -p nepl-core raw_pointer_type -- --nocapture`: pass。
- `NEPL_COMPILE_STAGE_TIMING=1 cargo run -p nepl-cli -- --target wasi --profile debug --input examples/nm.nepl --output tmp\nm-stage6-effect-probe3.wasm`: `resource_effect_boundaries=786ms` まで改善し、次の owner obligation 診断へ到達。
- `cargo test -p nepl-core raw_pointer_owner_carrier -- --nocapture`: pass。
- `cargo fmt -p nepl-core -- --check`: pass。
- `node nodesrc/test_resource_checker_responsibility.js`: pass。
- `node nodesrc/issues.js check`: pass。
- `node nodesrc/run_source_policy_regressions.js --warn-only`: pass。
- `git diff --check`: pass。

## 関連

- parent performance issue: `ISS-20260519T074504799Z-NM-FULL-COMPILE-STILL-EXCEEDS-CI-BUD-5653B487`
- newly exposed follow-up: `ISS-20260519T092458711Z-CLIARG-ARGS-GET-DEPENDENT-OWNER-PROO-3D72A977`
- Stage: `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 6

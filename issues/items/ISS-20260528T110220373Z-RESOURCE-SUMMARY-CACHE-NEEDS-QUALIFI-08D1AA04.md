---
id: ISS-20260528T110220373Z-RESOURCE-SUMMARY-CACHE-NEEDS-QUALIFI-08D1AA04
title: "Resource summary cache needs qualified nominal stable type identity"
area: core
status: verified
resolved: true
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

## 2026-05-28 実装チェックポイント

- `TypeCtx` に `NominalStableTypeIdentity` を追加し、`Struct` / `Enum` 登録時に `SourceMap` の source path、定義 kind/name/arity、field / variant / type parameter fingerprint を保存する。
- `TypeId` / `Span` は stable identity へ含めない。rollback 時は nominal identity map も checkpoint に合わせて戻す。
- `ResourceSummaryStableTypeKey` は identity 付き `Struct` / `Enum` / resolved `Named` だけを nominal key として受け入れ、identity のない unresolved `Named` は従来どおり拒否する。
- backend scalar の `u32` / `i64` / `u64` / `f64` は compiler-owned scalar として `Named` から stable key 化する。
- `function body hash` と `generic type argument hash` の namespace version を更新し、identity 付き nominal type を keyable candidate にできるようにした。
- subagent review で指摘された name-only stale hit を避けるため、単なる `Struct.name` / `Enum.name` は長寿命 cache key に使わない。
- `ResourceSummaryTypeReprojection` は function signature から到達する nested nominal type も登録する。これにより、summary value の surface には直接出ない field / payload 型が replay 時だけ解決不能になる経路を塞いだ。
- raw-init dependency closure hash を key に追加し、依存先 function body / source policy / type boundary が変わる場合は dependency-bearing caller summary も miss するようにした。
- `resource_function_body_hash` は `StorageId` の数値を直接 key に出さず、function body 内の出現順 ordinal へ正規化する。
- raw-init cache の bypass reason を `incomplete_leaf` / `dependency` / `missing_source_policy` / `unstable_key` / `unstable_entry` / `reprojection` に分け、RPN 残件を issue 分解できるようにした。

## 2026-05-28 verified

release Web artifact の RPN same-session code-edit 測定で、初回 `raw_init_param_facts_stores=2`、2 回目 `raw_init_param_facts_hits=2` / `resource_summary_value_replay_hits=2` を確認した。初期値は `stores=0` / `hits=0` だったため、本 issue の完了条件である nominal stdlib summary の store / hit 非 0 化は満たした。

残る性能課題は別根本原因として分離した。

- `ISS-20260528T123956177Z-RESOURCE-SUMMARY-RAW-INIT-DEPENDENCY-70A9D8F6`: dependency closure 内の key 化不能関数により `raw_init_param_facts_unstable_key_bypasses=176` が残る。
- `ISS-20260528T123956163Z-RESOURCE-SUMMARY-RAW-INIT-CACHE-NEED-245DC1A5`: byte-range / variant / return facts を含む summary が `incomplete_leaf=37` として no-store になる。
- `ISS-20260528T123956303Z-RESOURCE-SUMMARY-TYPE-REPROJECTION-N-78929A8E`: generic nominal 型の instantiated mapping 不足が疑われる `reprojection=10` が残る。

## 検証

- `cargo test -p nepl-core resource_summary_value -- --nocapture`
- public type edit / dependency public surface edit / stdlib overlay で stale hit しない regression
- RPN same-session code-edit 測定で nominal stdlib summary の Resource summary store / hit が非 0 になること

現 checkpoint の focused verification:

- `cargo test -p nepl-core stable_type_key -- --nocapture`
- `cargo test -p nepl-core type_boundary -- --nocapture`
- `cargo test -p nepl-core candidate_key -- --nocapture`
- `cargo test -p nepl-core stable_drop_traversal_forall_value_reprojects_nominal_expected_type -- --nocapture`
- `cargo test -p nepl-core --test typectx_checkpoint -- --nocapture`
- `cargo test -p nepl-core --test check_pipeline typecheck_records_nominal_stable_identity_from_source_map -- --nocapture`

verified:

- `trunk build --release`
- RPN same-session code-edit: `tmp/rpn_dependency_closure_code_edit_session_20260528.json`

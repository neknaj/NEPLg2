---
id: ISS-20260517T120443246Z-RAW-BODY-BACKEND-INTRINSIC-CLASSIFIE-4A602A92
title: "Raw body backend intrinsic classifier accepts arbitrary llvm namespace callees"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/effects.rs, nepl-core/src/typecheck/effect_check.rs, nepl-core/tests/effects.rs, nodesrc/test_static_check_boundary_responsibility.js"
---

# ISS-20260517T120443246Z-RAW-BODY-BACKEND-INTRINSIC-CLASSIFIE-4A602A92: Raw body backend intrinsic classifier accepts arbitrary llvm namespace callees

## 概要

raw body direct callee classification treats any LLVM callee whose name starts with llvm. as a BackendIntrinsic, and typecheck then treats BackendIntrinsic calls as pure in raw bodies. This grants pure raw body authority to unknown or effectful LLVM intrinsic names instead of requiring a typed semantic classifier.

## 対象

- `nepl-core/src/effects.rs, nepl-core/src/typecheck/effect_check.rs, nepl-core/tests/effects.rs, nodesrc/test_static_check_boundary_responsibility.js`

## 根拠

- `nepl-core/src/effects.rs` の raw body direct callee 分類は、LLVM backend であれば `callee.starts_with("llvm.")` を満たす任意の callee を `RawBodyDirectCallee::BackendIntrinsic` としていた。
- `nepl-core/src/typecheck/effect_check.rs` は `BackendIntrinsic` を無条件で pure raw body 内に許可していた。
- `raw_callee_is_impure` は宣言済み impure / known impure intrinsic だけを拒否し、unknown callee を pure 相当として扱っていたため、`llvm.trap` のような effectful または unsupported intrinsic が source proof なしに通り得た。

## 問題

raw body direct callee classification treats any LLVM callee whose name starts with llvm. as a BackendIntrinsic, and typecheck then treats BackendIntrinsic calls as pure in raw bodies. This grants pure raw body authority to unknown or effectful LLVM intrinsic names instead of requiring a typed semantic classifier.

## 影響

Effect safety for pure raw LLVM bodies depends on an unbounded namespace prefix. A misspelled or effectful llvm.* callee can bypass raw_callee_is_impure and be accepted as a pure backend intrinsic, weakening Stage 6 raw body proof exactness and making checker mistakes hard to catch statically.

## 修正方針

Introduce a typed raw-body backend intrinsic enum with explicit semantic categories. Classify LLVM intrinsic callees through that enum using exact boundary parsing, have RawBodyDirectCallee carry the typed intrinsic, and make pure raw-body checking match on the intrinsic effect instead of accepting every BackendIntrinsic blindly. Add regressions and source policy against llvm. prefix allowance.

## 関連計画

- [静的検査の不必要な複雑化の解消についての大規模な修正の仕様と実装計画 Stage 6](../../doc/neplg2/static_check_complexity_reduction_plan.md)

## 対応内容

- `RawBodyBackendIntrinsic` と `LlvmRawBodyIntrinsic` を追加し、LLVM raw-body backend intrinsic の分類を enum-owned semantic classifier に移した。
- `RawBodyDirectCallee::BackendIntrinsic` は backend 名ではなく typed intrinsic を持つようにし、consumer が `match` で effect / memory operation を確認できる形にした。
- `callee.starts_with("llvm.")` による任意 namespace prefix 許可を削除した。
- pure raw body の ordinary direct callee は「宣言済み pure」と証明できる場合だけ許可するようにし、unknown callee を default pure とする `raw_callee_is_impure` gate を削除した。
- `llvm.sqrt.f32` は known pure backend intrinsic として許可し、`llvm.trap` は unknown/unsupported intrinsic として拒否する回帰テストを追加した。
- static-check responsibility policy に typed backend intrinsic classifier、memory operation gate、unknown callee pure-proof requirement、旧 prefix/default-pure gate 禁止を追加した。

## 検証

- `cargo fmt -p nepl-core --check`: pass
- `cargo check -p nepl-core`: pass
- `cargo test -p nepl-core --test effects raw_body -- --nocapture`: pass
- `cargo test -p nepl-core --test effects pure_llvm_raw_call_to_unknown_llvm_intrinsic_is_rejected -- --exact --nocapture`: pass
- `cargo test -p nepl-core --test effects pure_llvm_raw_call_to_known_pure_backend_intrinsic_is_allowed -- --exact --nocapture`: pass
- `cargo test -p nepl-core --test effects pure_llvm_raw_call_to_declared_pure_substring_name_is_allowed -- --exact --nocapture`: pass
- `cargo test -p nepl-core --test effects pure_llvm_raw_call_to_declared_impure_extern_is_rejected -- --exact --nocapture`: pass
- `node nodesrc/test_static_check_boundary_responsibility.js`: pass
- `node nodesrc/issues.js check --dir issues`: pass

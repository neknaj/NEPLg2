---
id: ISS-20260531T060756264Z-MEMO-CALL-PHASE1-NEEDS-COMPILER-KNOW-2DB7C53C
title: "memo_call phase1 needs compiler-known primitive boundary"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-31
updated: 2026-05-31
target: "nepl-core/src/typecheck; nepl-core/src/resource; stdlib"
---

# ISS-20260531T060756264Z-MEMO-CALL-PHASE1-NEEDS-COMPILER-KNOW-2DB7C53C: memo_call phase1 needs compiler-known primitive boundary

## 概要

memo_call を pure public API として提供するには、現状の関数値 string identity と Pure/Impure 二値だけでは private cache の非 escape と高階関数境界を表現できない。

## 対象

- `nepl-core/src/typecheck; nepl-core/src/resource; stdlib`

## 根拠

- `doc/neplg2/private_effect_memoization_purity_design.md` の Phase 1 方針に従う。
- `memo_call` は純粋関数の結果を private cache に保存する設計だが、cache が外部観測不能であることを示すまでは通常の `Pure` と同一視しない。
- 高階関数境界は typed function identity と明示 `@` function value に限定し、通常の部分適用や暗黙 coercion の副産物として扱わない。

## 問題

memo_call を pure public API として提供するには、現状の関数値 string identity と Pure/Impure 二値だけでは private cache の非 escape と高階関数境界を表現できない。

## 影響

primitive 境界を固定しないまま memo_call を通常ライブラリとして実装すると、impure/capturing/unresolved generic function や observable cache identity を pure と誤認する危険がある。

## 修正方針

Phase 1 は compiler-known primitive とし、memo_call @pure_named_func だけを受け入れる。typed function identity、MemoKey/MemoValue の保守的構造制約、private cache SourceCapability、sealed backend representation を依存条件として明示する。

## 2026-05-31 checkpoint

- `stdlib/core/memo.nepl` の解決済み `memo_call` 定義だけを compiler-known primitive として検出する typecheck 入口を追加した。
- overload 選択より前に `memo_call @func` を専用検査へ入れ、impure function value と暗黙 function-value coercion を memo 専用診断で拒否する。
- `MemoKey` / `MemoValue` Phase 1 predicate は `unit`、primitive scalar、`Copy` が証明された Drop なしの構造値に限定し、`str`、reference、raw pointer / owner token、function value、未解決型を拒否する。
- `HirExprKind::MemoizedFunctionValue` により typed HIR 上の memoization 境界を残した。
- `memo_call @func arg` の即時適用は、backend private cache representation が入るまで memoization 境界を消さないために拒否する。
- user code の同名 `memo_call` は通常関数として扱い、compiler-known primitive fast path へ入れない regression を追加した。

この checkpoint では backend private cache はまだ生成していない。現時点の codegen は `MemoizedFunctionValue` を通常の named function value と同じ可観測結果へ lowering する。private cache SourceCapability、Resource IR `PrivateCache` effect、sealed backend representation はこの issue の残件として継続する。

## 検証

memo_call @pure_named_func が pure に通り、impure function、capturing function、generic unresolved function、reference/raw pointer key/value、cache stats/clear/ref exposure が拒否される regression matrix を追加する。

現 checkpoint では `cargo check -p nepl-core`、`cargo test -p nepl-core function_memo_call --test functions -- --nocapture`、`cargo test -p nepl-core --test functions -- --nocapture`、`cargo test -p nepl-core --test typeannot -- --nocapture`、`cargo test -p nepl-core --tests --no-run`、`cargo check --manifest-path nepl-web\Cargo.toml`、`trunk build --release`、`node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-memo-call-phase1-20260531.json`、`node nodesrc/test_run_test_compiler_session.js`、`node nodesrc/issues.js check --dir issues`、`git diff --check` により、accepted pure named function value、dedicated HIR boundary、implicit function argument rejection、impure function rejection、`str` key/value rejection、non-Copy struct rejection、generic function value rejection、immediate application rejection、local same-name function fallback を確認した。

追加 checkpoint では `stdlib/core/traits/memo.nepl` の `MemoKey` / `MemoValue` を public signature と compiler-known primitive gate の両方に接続した。gate 側は trait definition の stdlib source identity も確認する。`cargo test -p nepl-core function_memo_call --test functions -- --nocapture` により、structural Copy aggregate acceptance、unit key/value acceptance、Copy struct without `MemoKey` rejection、Copy + `MemoKey` struct without `MemoValue` rejection、`f32` key rejection、`f32` field を持つ structural key rejection、function value key/value rejection、reference key rejection、`MemPtr i32` key/value rejection、`RegionToken i32` key/value rejection を確認した。

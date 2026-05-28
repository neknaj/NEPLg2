---
id: ISS-20260525T233735956Z-NEPLG2-1-NESTED-PRODUCER-GENERIC-CAL-B1C7C74C
title: "NEPLg2.1 nested producer generic call does not use outer expected parameter type"
area: core
status: verified
resolved: true
priority: P0
type: bug
created: 2026-05-25
updated: 2026-05-28
target: "nepl-core/src/typecheck/**; stdlib/tests/hashset.n.md; stdlib/tests/hashset_str.n.md"
---

# ISS-20260525T233735956Z-NEPLG2-1-NESTED-PRODUCER-GENERIC-CAL-B1C7C74C: NEPLg2.1 nested producer generic call does not use outer expected parameter type

## 概要

HashSet doctests on the current NEPLg2.1 branch fail at postfix-free calls such as must_hs new DefaultHash32 and must_hss new DefaultHash32 even when the outer helper parameter type fixes Result HashSet K DefaultHash32 Diag. Restoring explicit helper postfixes does not change the failure, and clean HEAD has the same type.overload.no_match diagnostics.

## 対象

- `nepl-core/src/typecheck/**; stdlib/tests/hashset.n.md; stdlib/tests/hashset_str.n.md`

## 根拠

- `node nodesrc/tests.js -i stdlib/tests/hashset.n.md -i stdlib/tests/hashset_str.n.md --no-tree -o tmp/neplg21-hashset-helper-postfix-current.json -j 1 --dist web/dist --assert-io` は、`must_hs new DefaultHash32` / `must_hss new DefaultHash32` で `type.overload.no_match` を出した。
- `unwrap_ok r` を明示 postfix 付きの `unwrap_ok<HashSet<...>, Diag> r` へ戻しても同じ 4 件が失敗した。
- `hashset_update_error_owner<...> e` も戻した clean HEAD 相当の比較でも同じ 4 件が失敗した。
- 失敗箇所は helper 本体ではなく、outer helper call の引数位置にある `new DefaultHash32` producer generic call である。
- subagent review でも、`unwrap_ok<HashSet<...>, Diag> r` と `hashset_update_error_owner<...> e` は helper 引数型 / `Err e` payload / `%HashSet ...` local annotation から型が決まるため、postfix-free 移行自体は妥当と確認した。

## 問題

HashSet doctests on the current NEPLg2.1 branch fail at postfix-free calls such as must_hs new DefaultHash32 and must_hss new DefaultHash32 even when the outer helper parameter type fixes Result HashSet K DefaultHash32 Diag. Restoring explicit helper postfixes does not change the failure, and clean HEAD has the same type.overload.no_match diagnostics.

## 影響

Semantic corpus migration cannot remove producer generic postfixes safely for nested calls where the only concrete type evidence is carried by an outer consumer/helper parameter. This blocks the NEPLg2.1 corpus migration and keeps hash collection coverage red.

## 修正方針

Propagate the selected outer callable argument expectation into nested prefix-call reduction so producer generic calls can solve their type parameters from the consumer parameter type without relying on explicit postfixes. Keep the fix in frontend/typecheck inference and do not weaken Resource IR checks or add stdlib-name allowlists.

## 修正内容

- `infer_expected_from_outer_consumer` / `infer_expected_from_outer_consumer_next_arg` が、未解決の外側 overload を単に飛ばさず、各候補の同じ引数位置から共通する期待型を作るようにした。
- 外側 overload の error payload など候補ごとに異なる部分は fresh type variable に落とし、`Result HashSet i32 DefaultHash32 ?E` のような下限情報として内側 producer へ渡す。
- overload の戻り値早期 filter は rollback-scoped unification を使い、宣言側 generic と外側期待型側の未解決変数が異なる位置にある場合を過剰に拒否しないようにした。
- `hashset_storage_states` / `hashset_storage_keys` 周辺に残っていた、今回の nested producer 修正を妨げる旧 postfix helper 呼び出しを、NEPLg2.1 の型推論に任せる形へ整理した。

## 検証

- `cargo test -p nepl-core --test functions function_neplg21_ -- --nocapture`
- `cargo check -p nepl-core`
- `cargo check --manifest-path nepl-web/Cargo.toml`
- `trunk build`
- `node nodesrc/tests.js -i stdlib/tests/hashset.n.md -i stdlib/tests/hashset_str.n.md --no-tree -o tmp/neplg21-hashset-nested-producer-20260528-after3.json -j 1 --dist web/dist --assert-io`

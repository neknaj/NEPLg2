# stdlib tests review

確認対象 commit: `c5f93163 fix(selfhost): split hir expr payloads`

## 確認対象

- `stdlib/tests/*.n.md`
- `tests/stdlib/**/*.n.md`
- `nodesrc/test_stdlib*.js`
- `nodesrc/run_source_policy_regressions.js`
- GitHub Actions `stdlib-test` / `nmd-doctest` / `tutorials-test`

## 現状

stdlib は `.n.md` doctest と source policy regression の両方を持つ。`nodesrc/test_stdlib*.js` は 80 個以上あり、unsafe unwrap、raw aggregate detour、module split、borrowed observer、ANSI boundary、string builder owner boundary、nm parser/htmlgen boundary などを監視している。

review の test 判定は local test ではなく GitHub Actions を根拠にする。`c5f93163` の latest run `25508600937` は in_progress であり、green 判定は保留である。直前の `b9e85f23` run `25507326678` では `build` / `compile-test` は success、`tutorials-test` / `nm-compile` は failure、その他は後続 push により cancelled になった。latest completed run で再判定する。

## 良い点

source policy がかなり具体的で、過去に修正した bad pattern の再導入を防いでいる。とくに string、Vec、HashMap/HashSet、BTree、stdio ANSI、NM、TUI、std/test の境界監視は有効である。

`std/test` は assertion/report/exit code を分ける方向へ進んでおり、`.n.md` でも stdout report と exit code を共通確認できる土台がある。

## 問題とリスク

open issue の通り、`.n.md` test 全体はまだ return value 中心の古い contract が残る。失敗時の詳細を stdout assertion report として確認し、exit code は可否だけにする運用へ移す必要がある。

source policy は強いが、file content pattern に依存するため、設計の意図と policy の同期が切れると false positive/false negative になる。policy 追加時は必ず doc/issue と対応づける必要がある。

local review では code変更前の確認目的以外に local test を根拠にしない。Actions の latest completed run を継続確認し、cancelled run は failure と同一視しない。

## 進捗状況

| 領域 | 状態 | 判定 |
|---|---|---|
| `stdlib/tests` | moduleごとの doctest。 | coverageは広い。 |
| `tests/stdlib` | integration/style/regression doctest。 | `.n.md` contract移行が必要。 |
| `nodesrc/test_stdlib*.js` | source policy多数。 | 再発防止に有効。 |
| GitHub Actions | latest `c5f93163` run `25508600937` in_progress。直前 run で `tutorials-test` / `nm-compile` failure を確認。 | 完了結果と failure log を後続checkpointで確認する。tutorials/VFS tree failure は issue 化済み。 |

## 推奨対応

- `.n.md` test は `std/test` の report API を標準化し、stdout detail + exit code success/failure に統一する。
- source policy は issue ID と doc に紐付け、古い設計を固定してしまわないよう定期レビューする。
- selfhost/Rust 共通テストは stdlib assert/report contract を前提に設計する。

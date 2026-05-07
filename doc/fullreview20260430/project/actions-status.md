# GitHub Actions 状況

確認対象 commit: `b350213c docs(review): add selfhost compiler review`

## 確認方法

レビュー用の test 状況は local test ではなく、`gh` で GitHub Actions から取得した結果を根拠にする。

実行した主なコマンド:

- `gh run list --limit 12 --json databaseId,headSha,headBranch,status,conclusion,createdAt,updatedAt,name,event,workflowName`
- `gh run view 25502095158 --json status,conclusion,name,headSha,headBranch,createdAt,updatedAt,jobs`
- `gh run list --limit 8 --json databaseId,status,conclusion,headSha,displayTitle,createdAt,updatedAt,url`

## 最新 run

| 項目 | 内容 |
|---|---|
| run id | `25505188662` |
| workflow | `CI (NEPL-g2)` |
| branch | `main` |
| head sha | `b350213c2e80afe81ba666a834d2e2e9ed9ad45b` |
| status | in_progress |
| conclusion | 未確定 |
| createdAt | `2026-05-07T15:23:35Z` |
| updatedAt | `2026-05-07T15:28:21Z` |
| active job | 未完了 run。job 詳細は次 checkpoint で必要に応じて取得する。 |

この run は `b350213c` 取り込み時点では完了していない。したがって、この時点では main の green 判定は未確定である。レビュー進行中に再度 `gh run view` を実行し、完了結果を本ファイルまたは最終 summary に反映する。

## 直近 run の傾向

| head | 状態 | 備考 |
|---|---|---|
| `b350213c` | in_progress | 最新 main。selfhost compiler review 追加。 |
| `31291b37` | cancelled | parser/backend responsibility policy 追加。 |
| `09ddb790` | cancelled | selfhost sentinel issue 更新。 |
| `e677a4e3` | cancelled | selfhost lexer enum coverage issue 追加。 |
| `08b58a2e` | cancelled | selfhost typed IR sentinel issue 追加。 |
| `c58dd6e3` | cancelled | public monomorphize API Result 化。 |
| `3742a1a7` | cancelled | CLI `--check` ResourceIR gate 共有修正。 |
| `d2ba8b8b` | cancelled | Rust compiler follow-up issue 追加。 |
| `cd44312f` | cancelled | ResourceIR `region_ptr_at` non-owning provenance 修正。 |
| `a97c5343` | cancelled | 次の main push により workflow concurrency で cancel。 |
| `e8a4e399` | cancelled | 次の main push により workflow concurrency で cancel。 |
| `9797bcbf` | cancelled | 次の main push により workflow concurrency で cancel。 |
| `545d2ab0` | cancelled | 次の main push により workflow concurrency で cancel。 |
| `97b07bad` | cancelled | 次の main push により workflow concurrency で cancel。 |
| `281646c7` | cancelled | 後続 push により最新判定対象から外れた。 |
| `104718bd` / `b31d4ac1` / `92ae1a7d` | cancelled | main への連続 push により cancel。 |
| `bf6f08df` 以前の複数 run | failure | 最新 main ではなく、過去の失敗。個別失敗原因は必要に応じて後続 review で確認する。 |

CI workflow は `concurrency.group: ci-${{ github.ref }}` と `cancel-in-progress: true` を持つため、main への連続 push 中は古い run が cancelled になる。review では cancelled run を failure と同一視せず、latest head の completed conclusion を基準にする。

## workflow 構成

`.github/workflows/ci.yml` で確認した job:

- `build`
- `compile-test`
- `rust-test`
- `nm-compile`
- `wasi-test`
- `nmd-doctest`
- `tutorials-test`
- `stdlib-test`
- `llvm-test`
- `llvm-dual-test`
- `pages-fast-bundle`
- `pages-fast-deploy`
- `pages-final-bundle`
- `pages-final-deploy`

test coverage は Rust compile/run、WASI doctest、`.n.md` doctest、tutorial doctest、stdlib doctest、LLVM backend、pages build/deploy まで広い。review では特に `wasi-test`、`nmd-doctest`、`stdlib-test`、`llvm-dual-test` の状態を重視する。

## 現時点の判定

- 最新 main の CI 結果は未確定。
- 連続 push による cancelled run が多いため、過去 run の conclusion だけで main の健全性を判断しない。
- 次 checkpoint 以降で `b350213c` 以降の run が完了していれば、成功/失敗 job と artifact 有無を追記する。

# GitHub Actions 状況

確認対象 commit: `545d2ab0 fix(resource): align region_ptr reference coverage`

## 確認方法

レビュー用の test 状況は local test ではなく、`gh` で GitHub Actions から取得した結果を根拠にする。

実行した主なコマンド:

- `gh run list --limit 12 --json databaseId,headSha,headBranch,status,conclusion,createdAt,updatedAt,name,event,workflowName`
- `gh run view 25502095158 --json status,conclusion,name,headSha,headBranch,createdAt,updatedAt,jobs`

## 最新 run

| 項目 | 内容 |
|---|---|
| run id | `25502095158` |
| workflow | `CI (NEPL-g2)` |
| branch | `main` |
| head sha | `545d2ab0f53bd9603bf65c937223c7590e29d13b` |
| status | in_progress |
| conclusion | 未確定 |
| createdAt | `2026-05-07T14:28:15Z` |
| updatedAt | `2026-05-07T14:29:27Z` |
| active job | `build` |

この run は project checkpoint 作成時点では完了していない。`build` job は `Shared bootstrap build` を実行中である。したがって、この時点では main の green 判定は未確定である。レビュー進行中に再度 `gh run view` を実行し、完了結果を本ファイルまたは最終 summary に反映する。

## 直近 run の傾向

| head | 状態 | 備考 |
|---|---|---|
| `545d2ab0` | in_progress | 最新 main。ResourceIR coverage 修正。 |
| `97b07bad` | cancelled | 次の main push により workflow concurrency で cancel。 |
| `281646c7` | in_progress/古い情報 | stdio debug split。latest run 確認時はまだ完了していなかったが、後続 push により最新判定対象から外れた。 |
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

- 最新 main の CI 結果は未確定。現在は `build` job 実行中。
- 連続 push による cancelled run が多いため、過去 run の conclusion だけで main の健全性を判断しない。
- 次 checkpoint 以降で `545d2ab0` の run が完了していれば、成功/失敗 job と artifact 有無を追記する。

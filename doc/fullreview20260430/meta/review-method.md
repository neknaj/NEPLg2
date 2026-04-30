# レビュー方法

対象 commit: `f108cebd`

## 目的

このレビューは、NEPLg2 の現状を「どこまで実装済みか」だけでなく、「今の設計が型安全・メモリ安全・selfhost に耐えるか」という観点で確認する。

レビュー中の remote main 更新は取り込む。取り込み後に判断が変わる章は更新し、対象 commit を更新する。最終再レビュー後の更新は今回レビューの範囲外とする。

## 判断基準

- 技術的負債を残さない。既存互換より設計の正しさを優先する。
- 暫定実装は許容しても、暫定の雑設計は許容しない。
- 型安全とメモリ安全は必達であり、検査が実際に compiler pipeline で強制される必要がある。
- 状態や診断は raw number / raw string ではなく enum を主表現にする。
- 有限分岐は `match` に寄せ、網羅性検査が効く構造にする。
- selfhost 実装は Rust 側の Resource IR / diagnostic code / static check 設計から後退しない。

## 調査入力

- `plan.md`: 言語の元仕様と構文方針。
- `README.md`: public-facing explanation と現行実装のずれ。
- `doc/neplg2/self_host_plan.md`: selfhost S0-S7 の正規計画。
- `doc/neplg2/static_check_soundness_review_20260430.md`: 静的検査の直近レビュー。
- `issues/index.md` と open issue: 現在の blocker。
- `note.n.md`: 他 agent の直近作業、検証結果、既存判断。
- GitHub Actions: review 対象 commit の CI 結果。review では local 実行結果ではなく `gh` で取得した Actions 結果を test 状況の根拠にする。
- ソース:
  - `nepl-core/src/**`
  - `stdlib/**`
  - `nodesrc/**`
  - `tests/**`
  - `tutorials/**`
  - `nepl-cli`, `nepl-language`, `nepl-lsp`, `nepl-web`, `web`

## 初期確認コマンド

```powershell
git pull --ff-only origin main
git checkout -b docs/fullreview-20260430
git log --oneline --decorate -n 20
Get-ChildItem nepl-core\src -Recurse -File
Get-ChildItem stdlib -Recurse -File
Get-Content issues\index.md -TotalCount 90
```

## レビュー手順

1. repository 全体の構成、巨大ファイル、open issue、直近 commit を確認する。
2. `doc/fullreview20260430/index.md` にレビュー目次を作成する。
3. `project/` で進捗、blocker、selfhost 開始可否を整理する。
4. `rust-compiler/` で Rust compiler pipeline を stage ごとに確認する。
5. `stdlib/` で core / alloc / std / nm / platform / collections を確認する。
6. `selfhost/` で S0-S7 の現物実装と plan を照合する。
7. `tools/` と `quality/` で CLI、nodesrc、tests、tutorial、web/editor を確認する。
8. `crosscutting/` で静的安全性、stdlib-selfhost readiness、diagnostics/tests/docs の横断判断をまとめる。
9. `summary/` と `meta/review-validity.md` で、レビュー内容そのものの妥当性を再確認する。

## 検証方針

review 文書に書く test 状況は GitHub Actions を `gh` で確認した結果を根拠にする。local test は、local で code 変更を行った commit 前確認のためにだけ使う。今回の docs-only review では、local runtime test の結果を review evidence として扱わない。

Actions 確認:

```powershell
gh run list --branch main --limit 10
gh run view <run-id> --json status,conclusion,headSha,headBranch,displayTitle,createdAt,updatedAt,jobs
gh run view <run-id> --job <job-id> --log-failed
```

docs-only commit 前確認:

```powershell
node nodesrc/issues.js check
git diff --check
```

実装変更 commit 前確認では、対象変更に応じて `cargo test`、`trunk build`、`node nodesrc/tests.js ...` などを追加する。ただし、その local 実行結果は「変更 commit の事前確認」であり、今回の総レビューにおける main の test 状況は Actions 結果で確認する。

## 報告方針

各レビュー単位で commit し、Discord へ次を含めて報告する。

- タイトル: `進捗確認及び総レビュー: 具体的な内容`
- 作成・更新した file
- 対象 commit
- 概要
- 検証結果
- 次に行うレビュー

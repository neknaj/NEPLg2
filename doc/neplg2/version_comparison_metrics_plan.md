# NEPLg2 git commit version comparison plan

作成日: 2026-05-20

関連 issue:

- [ISS-20260520T053104419Z-NEED-COMMIT-LEVEL-VERSION-COMPARISON-30FA1700](../../issues/items/ISS-20260520T053104419Z-NEED-COMMIT-LEVEL-VERSION-COMPARISON-30FA1700.md)

## 目的

NEPLg2 の開発では、静的検査の大規模修正、Resource IR、self-host compiler、stdlib doctest 整備が並行して進む。変更が増えるほど、ある commit で何が改善または悪化したかを、テスト通過率、コンパイル時間、実行時間、コード規模の観点で同じ条件から比較できる必要がある。

この文書は、git commit 単位で過去版と現行版を比較する仕組みの仕様を定める。

## 対象指標

比較対象は次の 4 系統に分ける。

- テスト通過率: `nodesrc/tests.js` の `summary.total` / `passed` / `failed` / `errored` / pass rate。
- コンパイル時間: doctest result の `timing.compile_ms` を count / sum / average / p50 / p95 / max で集計する。
- 実行時間: doctest result の `timing.run_ms` と `duration_ms` を同じ形式で集計する。
- コード規模: `repo_metrics.ts` の `byArea` / `byContentKind` / `byExtension` を読み、files / lines / source / doc_comment / document / test / testCases / bytes を集計する。

## 設計

`nodesrc/compare_git_versions.js` は、指定された `--rev` ごとに一時 `git worktree` を作る。比較処理は current checkout を移動せずに行うため、作業中の branch や未追跡 file に影響しない。

各 worktree では次を行う。

1. 必要なら `--build-cmd` を実行する。
2. current checkout の `repo_metrics.ts` を、対象 worktree を `--root` として実行する。
3. `-i` が指定され、`--metrics-only` でなければ、current checkout の `nodesrc/tests.js` を対象 worktree を cwd にして実行する。
4. JSON と Markdown の比較表を出力する。

`repo_metrics.ts` と comparison tool 自体は current checkout の実装を使う。これは、過去 commit に比較ツールが存在しない場合でも、同じ測定ロジックで過去版を測れるようにするためである。

## dist と build

コンパイラ性能を正確に比較するには、各 commit の compiler artifact を使う必要がある。その場合は `--build-cmd` と `--dist-rel` を使う。

例:

```powershell
node nodesrc/compare_git_versions.js `
  --rev HEAD~5 --rev HEAD `
  -i tests/compiler/overload.n.md `
  --build-cmd "trunk build" `
  --dist-rel web/dist `
  -o tmp/version_compare/overload.json `
  --markdown tmp/version_compare/overload.md
```

既にある dist を使って、テスト入力や stdlib source の変化だけを軽く見る場合は `--dist-current` を使う。ただしこの場合、compiler binary は全 commit で同じになるため、compiler implementation の速度比較としては扱わない。

```powershell
node nodesrc/compare_git_versions.js `
  --rev HEAD~1 --rev HEAD `
  -i tests/compiler/typeannot.n.md `
  --dist-current web/dist `
  --no-tree `
  -o tmp/version_compare/typeannot.json `
  --markdown tmp/version_compare/typeannot.md
```

コード規模だけを比較する場合は `--metrics-only` を使う。

```powershell
node nodesrc/compare_git_versions.js `
  --rev HEAD~10 --rev HEAD `
  --metrics-only `
  -o tmp/version_compare/metrics.json `
  --markdown tmp/version_compare/metrics.md
```

## 出力

JSON は `neplg2-git-version-comparison/v1` schema を持つ。各 revision には次を含む。

- `ref` と解決済み `commit`
- `tests`: pass rate と timing 集計
- `metrics`: repo metrics の totals / by_area / by_content_kind / by_extension
- `commands`: 実行した command の status と出力末尾
- `artifacts`: 対象 revision の中間 JSON を置いた directory

Markdown は Discord や issue へ貼るための要約表である。詳細な失敗内容や timing 分布を追う場合は JSON を正とする。

## 運用方針

- full comparison は重いので、通常は対象 input を絞る。
- CI 全体の評価には GitHub Actions の結果を使う。
- compiler performance を議論する場合は `--build-cmd` で commit ごとの compiler artifact を作る。
- stdlib doctest の通過率や repo metrics の傾向だけを見る場合は `--dist-current` または `--metrics-only` でよい。
- 比較結果の commit hash、入力、dist mode、build command は報告に必ず書く。

## 今後の拡張

- GitHub Actions run ID と commit comparison JSON を紐づける。
- 代表 benchmark input を doc に固定する。
- timing の外れ値を上位 N 件で Markdown に出す。
- `repo_metrics.ts` の totals を script 側にも正式 field として追加する。

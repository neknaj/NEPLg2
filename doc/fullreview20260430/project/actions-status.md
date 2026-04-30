# GitHub Actions 状況レビュー

対象 commit: `f108cebd`

確認 run: `25157230630`

確認方法:

```powershell
gh run view 25157230630 --json status,conclusion,headSha,headBranch,displayTitle,createdAt,updatedAt,jobs
gh run view 25157230630 --job <job-id> --log-failed
```

## 結論

`f108cebd` の main Actions は `failure` である。bootstrap build、source policy regressions、compile-test、LLVM doctest、Pages build/deploy は成功している。一方で、Rust unit/integration test、stdlib doctest、WASI doctest、`.n.md` doctest、tutorial doctest、dual backend verification が失敗している。

このため、現時点の project 状態は「compiler build と一部 gate は通るが、runtime / stdlib / doctest / dual backend の広域 regression は未解消」である。selfhost S1/S2 の限定的な整備は進められるが、S3 以降の型検査・Resource IR・codegen を完成扱いで積む状態ではない。

## 成功 job

| job | 結論 | 意味 |
|---|---|---|
| `build` | success | shared bootstrap build、source policy regressions、doc/tutorial/doc HTML build が成功 |
| `compile-test` | success | Rust compile tests と wasm32 compile tests が成功 |
| `llvm-test` | success | LLVM doctests via nodesrc runner が成功 |
| `pages-fast-bundle` / `pages-fast-deploy` | success | pending Pages artifact と deploy が成功 |
| `pages-final-bundle` / `pages-final-deploy` | success | test artifact を含む final Pages artifact と deploy が成功 |

`build` job の `Source policy regressions` step は成功している。この結果は、source policy aggregate が review 対象 main run では赤くないことを意味する。

## 失敗 job

| job | 結論 | 主な傾向 |
|---|---|---|
| `rust-test` | failure | WASI/fs/stdio 系 regression と drop 系 regression が `resource.cell.uninit` などで失敗 |
| `stdlib-test` | failure | stdlib doctest が広域に失敗し、selfhost CLI / module graph などで timeout が出ている |
| `wasi-test` | failure | NEPL-g2 doctest が失敗し、CLI multi-emit output は後続 step として skipped |
| `nmd-doctest` | failure | `tests/` 配下の `.n.md` doctest が 812 passed / 185 failed / 37 errored |
| `tutorials-test` | failure | tutorial doctest が失敗 |
| `nm-compile` | failure | NM compile tests が失敗 |
| `llvm-dual-test (tests)` | failure | tests dual backend verification が失敗 |
| `llvm-dual-test (stdlib)` | failure | stdlib dual backend verification が失敗 |

## 失敗内容の読み取り

`rust-test` の失敗では、WASI file descriptor / directory / path open 系 test と drop 系 test に `resource.cell.uninit` が出ている。これは Resource IR initialized-cell gate が std/fs/stdio/drop elaboration 周辺の未初期化 raw cell path を検出していることを示す。検査を弱めて通すのではなく、stdlib memory API と drop insertion / Resource IR summary の整合を取る必要がある。

`stdlib-test` と `nmd-doctest` の失敗は selfhost stdlib / `.n.md` runner policy / timeout 問題を含む。`.n.md` は Rust compiler と selfhost compiler の共通 test 基盤にする計画があるため、stdout report と exit code policy、stdlib assert 設計、timeout の原因切り分けが blocker である。

## review 運用上の注意

この review では test 状況を local 実行結果ではなく GitHub Actions 結果で確認する。local コマンドは docs 整合性確認や issue index check、または local code 変更 commit 前の確認としてだけ扱う。Actions と local 直接実行の結果が異なる場合、review の main 状態は Actions を正とし、local 直接実行で見つかった設計問題は別途 issue として扱う。

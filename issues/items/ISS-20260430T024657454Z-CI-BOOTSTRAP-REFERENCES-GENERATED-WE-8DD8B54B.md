---
id: ISS-20260430T024657454Z-CI-BOOTSTRAP-REFERENCES-GENERATED-WE-8DD8B54B
title: "CI bootstrap references generated web dist_ts before creation"
area: ci
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-30
updated: 2026-04-30
target: .github/actions/bootstrap-build/action.yml
---

# ISS-20260430T024657454Z-CI-BOOTSTRAP-REFERENCES-GENERATED-WE-8DD8B54B: CI bootstrap references generated web dist_ts before creation

## 概要

GitHub Actions run 25144577903 failed in Shared bootstrap build because Trunk canonicalized [watch].ignore path web/dist_ts before the TypeScript pre_build hook had created it in a clean checkout.

## 対象

- `.github/actions/bootstrap-build/action.yml`

## 根拠

- GitHub Actions run `25144577903` の `build / Shared bootstrap build` は `trunk build --release --public-url "/NEPLg2/"` で失敗した。
- 失敗ログは `error taking canonical path to [watch].ignore "web/dist_ts" in "/home/runner/work/NEPLg2/NEPLg2/Trunk.toml"` を示していた。
- `web/dist_ts` は TypeScript の `outDir` で生成される ignored directory であり、clean checkout には存在しない。
- `Trunk.toml` の `[watch].ignore` は Trunk の設定読込時に参照されるため、pre_build hook の `npm --prefix web run build:ts` が走る前に path が必要になる。

## 問題

GitHub Actions run 25144577903 failed in Shared bootstrap build because Trunk canonicalized [watch].ignore path web/dist_ts before the TypeScript pre_build hook had created it in a clean checkout.

## 影響

The build job stops before bootstrap-build artifact upload, so tutorial tests and final Pages deployment on latest main are skipped or fail from missing artifacts.

## 修正方針

Make the bootstrap action prepare generated web directories that are referenced by Trunk configuration before invoking trunk build, without committing generated TypeScript output.

## 修正内容

- `.github/actions/bootstrap-build/action.yml` の web asset 準備 step を `Prepare generated web directories for Trunk` に整理した。
- `web/examples` に加えて `web/dist_ts` も `trunk build` 前に作成し、Trunk の設定読込が clean checkout で失敗しないようにした。
- 生成された TypeScript 出力自体は引き続き git 管理せず、`npm --prefix web run build:ts` が本来の成果物を生成する。

## 検証

Run trunk build on a checkout where web/dist_ts and web/examples do not already exist, then confirm the GitHub Actions build job reaches artifact upload and Pages final deployment.

## 検証結果

- local clean-output bootstrap test: passed
  - `web/dist_ts` と `web/examples` を削除後、bootstrap step と同じ準備で両 directory が作成されることを確認した。
  - `trunk build --release --public-url "/NEPLg2/"`: passed
- `node nodesrc/tests.js -i tutorials/getting_started --no-tree -o tmp/tutorials-stdout-after-main.json -j 4`: 24 passed
- `node nodesrc/tests.js -i examples --no-tree -o tmp/examples-after-main.json -j 4`: 12 passed
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed
- GitHub Actions build job artifact upload: pending after push
- Pages final deployment: pending after push

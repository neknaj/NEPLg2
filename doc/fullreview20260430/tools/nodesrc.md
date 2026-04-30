# Tools Review: Node Test And Documentation Tooling

対象 commit: `f108cebd`

## 対象

- `nodesrc/tests.js`
- `nodesrc/run_test.js`
- `nodesrc/run_doctest.js`
- `nodesrc/run_source_policy_regressions.js`
- `nodesrc/issues.js`
- `nodesrc/parser.js` / `nodesrc/parser.ts`
- `nodesrc/html_gen.js` / `nodesrc/html_gen.ts`

## 概要

`nodesrc` は `.n.md` doctest、stdlib doctest、tutorial doctest、tree tests、HTML generation、issue index、source policy regression をまとめる実質的な project quality layer である。CI の build job では source policy を `--warn-only` で実行し、downstream artifact と test jobs が止まらないようにしている。

test runner は JSON artifact を出力し、今回 review でも Actions artifact の `summary` と `results` を根拠にした。これは local 実行結果ではなく、remote main の状態を確認する運用として妥当である。

## Actions 根拠

Actions run `25157230630` の artifact 集計:

- `nmd-tests`: `1034 total / 812 passed / 185 failed / 37 errored`
- `wasi-tests`: `1034 total / 812 passed / 185 failed / 37 errored`
- `tutorials-tests`: `44 total / 21 passed / 23 failed / 0 errored`
- `stdlib-tests`: `415 total / 232 passed / 173 failed / 10 errored`
- `llvm-dual-tests`: `1945 total / 1515 passed / 393 failed / 37 errored`
- `llvm-dual-stdlib`: `977 total / 607 passed / 360 failed / 10 errored`

主要 failure code は `resource.owner.maybe_leak`, `resource.owner.leak`, `resource.cell.uninit`, `resource.cell.possibly_moved` で、runner そのものより compiler/static-check/stdlib contract 側の failure が中心である。

## 良い点

- `nodesrc/tests.js` は runner mode (`wasm`, `llvm`, `all`) と tree suite を統合している。
- `run_test.js` は stdout/stderr/return/runtime metadata を JSON 化している。
- `issues.js` は issue file を正として index/check/new を持つ。
- source policy runner は多数の regression guard を 1 箇所に集約している。
- tutorial current-style guard が古い raw memory / unwrap 例の再導入を防いでいる。

## 問題

- `tests.js` は約 58KB で、scan / scheduling / worker / summary / tree integration が集中している。
- CI source policy が `--warn-only` なので、policy drift は downstream を止めない。これは artifact 確保には有効だが、方針違反を放置しやすい。
- `.n.md` の運用はまだ stdout report / exit code policy へ完全移行中で、`ISS-20260429T102425370Z...` が open。
- dual backend の failure は compile/static-check failure と runtime mismatch が混ざる。

## 必要な設計

- `tests.js` は scanner、scheduler、summary writer、tree runner adapter に分割する。
- source policy は CI では warn-only 継続でも、merge readiness では strict gate を別 job で要求する。
- `.n.md` は stdout assertion report と exit code を正規 contract にし、selfhost runner と Rust runner で共有する。
- artifact JSON の failure classification helper を公式化し、review/triage が手作業の one-off script に依存しないようにする。

## 進捗状況

- doctest runner: 実用中。
- issue tooling: 実用中。
- source policy runner: 実用中だが CI では warn-only。
- `.n.md` 共通運用: 設計中。
- runner 分割: 未着手。

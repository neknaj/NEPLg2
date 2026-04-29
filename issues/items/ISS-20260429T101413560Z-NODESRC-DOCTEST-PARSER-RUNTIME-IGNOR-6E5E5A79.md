---
id: ISS-20260429T101413560Z-NODESRC-DOCTEST-PARSER-RUNTIME-IGNOR-6E5E5A79
title: "nodesrc doctest parser runtime ignores diag_code metadata"
area: TEST
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "nodesrc/parser.js, nodesrc/parser.ts, nodesrc/tests.js, nodesrc/run_doctest.js"
---

# ISS-20260429T101413560Z-NODESRC-DOCTEST-PARSER-RUNTIME-IGNOR-6E5E5A79: nodesrc doctest parser runtime ignores diag_code metadata

## 概要

`.n.md` の `diag_code:` / `diag_codes:` は Rust 診断と selfhost 診断を共通に固定する主契約になるが、現在 Node 実行時が読み込む `nodesrc/parser.js` は古い `diag_id` / `diag_ids` 実装のままで、`diag_code:` を doctest case に渡していない。

## 対象

- `nodesrc/parser.js, nodesrc/parser.ts, nodesrc/tests.js, nodesrc/run_doctest.js`

## 根拠

- `nodesrc/parser.ts` は `diag_code` / `diag_codes` を parse して `diag_codes` を返す実装になっている。
- `nodesrc/parser.js` は `diag_id` / `diag_ids` を parse する古い生成物のままで、`diag_code:` 行を無視する。
- `nodesrc/tests.js` と `nodesrc/run_doctest.js` は `dt.diag_codes` を `expected_diag_codes` に写すため、実行時 parser が `diag_codes` を返さないと診断 code expectation が空になる。
- 再現コマンド:
  - `node -e "const {parseFile}=require('./nodesrc/parser'); const p=parseFile('tests/compiler/plan.n.md'); const x=p.doctests.find(d=>d.tags.includes('compile_fail')); console.log(JSON.stringify({tags:x.tags, diag_codes:x.diag_codes, diag_ids:x.diag_ids}, null, 2));"` が `diag_codes` なし、`diag_ids: []` を返す。

## 問題

`compile_fail` doctest に `diag_code:` を書いても runner が期待診断 code として検査していないため、診断 ID 再構築の回帰テストが「compile failed した」ことだけを確認し、意図した diagnostic code が出たことを確認できない。

## 影響

- Rust 側の diagnostic code-first 移行と selfhost 側の typed diagnostic code 移行を `.n.md` で共通検証する設計の前提が崩れる。
- 誤った診断 code、別 category の診断、code なし診断への退行が `.n.md` compile_fail で検出されない。
- `parser.ts` と `parser.js` の生成物 drift も再発しうるため、Node runner の source-of-truth が曖昧になる。

## 修正方針

`parser.ts` / `parser.js` の drift を解消し、Node runner が必ず `diag_code` / `diag_codes` を収集するようにする。あわせて `diag_code:` を持つ最小 fixture を追加し、`parseFile` の戻り値と `run_doctest` / `tests.js` の expectation 適用が有効であることを JS regression で固定する。今後の設計では `.n.md` metadata parser を Rust/selfhost 共通テスト manifest の単一入口にするため、生成物の更新手順も CI で検査する。

## 検証

- `node nodesrc/test_doctest_diag_code_metadata.js` のような source regression を追加し、`diag_code:` が `diag_codes` として収集されることを固定する。
- `node nodesrc/run_doctest.js -i <diag_code fixture> -n <case>` で、期待 code と異なる compile error が fail になることを確認する。
- `node nodesrc/tests.js -i <diag_code fixture> --no-tree -o tmp/doctest-diag-code-metadata.json -j 1` で aggregate runner でも同じ expectation が効くことを確認する。

## 2026-04-29 解決メモ

原因は 2 層あった。

1. `nodesrc/parser.ts` は `diag_code:` / `diag_codes:` を扱うが、runtime が読む generated `nodesrc/parser.js` が古い `diag_id:` 実装のままになる drift を検出していなかった。
2. focused runner の `nodesrc/run_doctest.js` は `compile_fail` の raw result が `ok: true` / `status: pass` になった場合、diagnostic code / span expectation を検査していなかった。
3. `nodesrc/run_doctest.js` の diagnostic span 抽出は ANSI escape sequence 付きの `--> file:line:col` 行を処理できず、aggregate runner と focused runner の結果がずれていた。

`nodesrc/parser.js` は `.gitignore` 対象の generated artifact であり、repository には tracked しない設計である。そのため、修正は source と CI regression に入れた。`npx tsc -p nodesrc/tsconfig.json` で生成される runtime parser が `diag_codes` を返すことを `nodesrc/test_doctest_diag_code_metadata.js` で固定した。

同じ regression で次を確認する。

- `.n.md` の `diag_code:` と `diag_codes:` が `diag_codes` 配列に入る。
- `.nepl` doc comment の `diag_code:` と `diag_codes:` も同じ形で入る。
- 旧 `diag_ids` property が doctest case に残らない。
- `run_doctest.js` が wrong `diag_code` を `compile_fail diagnostic code mismatch` として fail にする。
- `run_doctest.js` が ANSI 付き compile error から `diag_span:` を抽出できる。
- `nodesrc/tests.js` aggregate runner も wrong `diag_code` を fail にする。

`run_doctest.js` は `compile_fail` case では `compile_error` が存在する限り、raw result の `ok` に関係なく diagnostic code / span expectation を検査するようにした。span 抽出は aggregate runner と同じく ANSI escape sequence を除去してから `--> file:line:col` を読む。

CI の Source policy regressions に `node nodesrc/test_doctest_diag_code_metadata.js` を追加し、bootstrap build 後の generated parser と focused / aggregate runner の両方を検査する。

### 2026-04-29 検証

- `npx tsc -p nodesrc/tsconfig.json`: passed
- `node nodesrc/test_doctest_diag_code_metadata.js`: passed
- `node nodesrc/run_doctest.js -i tests/compiler/compile_fail_diag_location.n.md -n 1 --dist web/dist`: passed
- `node nodesrc/tests.js -i tests/compiler/compile_fail_diag_location.n.md --no-tree -o tmp/doctest-diag-code-existing.json -j 1 --dist web/dist`: total=3 passed=3
- `trunk build`: passed
- `git diff --check`: passed

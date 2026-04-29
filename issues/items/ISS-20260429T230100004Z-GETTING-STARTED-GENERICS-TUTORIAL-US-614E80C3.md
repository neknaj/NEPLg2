---
id: ISS-20260429T230100004Z-GETTING-STARTED-GENERICS-TUTORIAL-US-614E80C3
title: "getting_started generics tutorial uses unconstrained owner generic example"
area: examples
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-30
target: "tutorials/getting_started/18_generics.n.md, nodesrc/test_tutorial_getting_started_current_style.js, .github/workflows/ci.yml"
---

# ISS-20260429T230100004Z-GETTING-STARTED-GENERICS-TUTORIAL-US-614E80C3: getting_started generics tutorial uses unconstrained owner generic example

## 概要

GitHub Actions run `25137357734` の `tutorials-test` は `tutorials/getting_started/18_generics.n.md::doctest#1` で `resource.owner.maybe_leak` になった。入門の generics 例が `.T` の所有権境界を明示せず、`str` identity や unconstrained `Option<.T>` helper を quiet checks に渡していた。

## 対象

- `tutorials/getting_started/18_generics.n.md`
- `nodesrc/test_tutorial_getting_started_current_style.js`
- `.github/workflows/ci.yml`

## 根拠

- `gh api repos/neknaj/NEPLg2/actions/jobs/73679186492/logs` で、`tutorials/getting_started/18_generics.n.md::doctest#1` が `resource.owner.maybe_leak` により compile fail していることを確認した。
- local 再現でも、旧例の `identity "nepl"` / `or_default <.T>` を含む形では `main__unit__i32__imp` の temporary owner obligation が残った。
- getting_started は現在の静的検査と所有権モデルに合わせ、copy できる値、owner を持つ値、move/borrow/Clone 境界を明示して教える方針である。

## 問題

入門の generics 例が、`.T` を unconstrained のまま owner になりうる値へ適用する形だった。これは現在の Resource IR の static checking 方針と合わず、入門本文としても「どの値を copy してよいか」を曖昧にしていた。

## 影響

getting_started tutorial が CI で失敗する。さらに、読者に所有権境界を隠した generic helper の書き方を教えるため、後続の move / borrow / Clone 設計と矛盾する。

## 修正方針

generics 例を `.T: Copy` bound 付きにし、`i32` / `bool` / `Option<i32>` / `Result<i32,str>` のような Copy として扱える値だけを対象にする。owner を持つ値まで generic に扱う場合は、入門例ではなく move / borrow / Clone 境界を別章で明示する。旧例が戻らないよう source policy を CI に接続する。

## 検証

- `node nodesrc/tests.js -i tutorials/getting_started/18_generics.n.md --no-tree -o tmp/agent1-tutorial-18-generics-current-copy-style.json -j 1 --dist web/dist`: total=1, passed=1
- `node nodesrc/tests.js -i tutorials/getting_started --no-tree -o tmp/agent1-tutorial-getting-started-current-style.json -j 4 --dist web/dist`: total=24, passed=24
- `node nodesrc/test_tutorial_getting_started_current_style.js`: pass

## 対応結果

`18_generics.n.md` は `core/traits/copy` を import し、`identity <.T: Copy>` を使う例へ更新した。`Option<i32>` / `Result<i32,str>` は typed local に束縛してから `is_some` / `is_ok` で確認し、`str` identity や unconstrained `or_default <.T>` は削除した。`nodesrc/test_tutorial_getting_started_current_style.js` に Copy-bound generics の source policy を追加し、CI source policy regressions に接続した。

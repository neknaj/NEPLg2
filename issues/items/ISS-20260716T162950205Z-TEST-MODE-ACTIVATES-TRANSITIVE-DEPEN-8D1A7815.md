---
id: ISS-20260716T162950205Z-TEST-MODE-ACTIVATES-TRANSITIVE-DEPEN-8D1A7815
title: "test mode activates transitive dependency test declarations"
area: compiler
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-07-16
updated: 2026-07-17
target: nepl-core/src/loader.rs
---

# ISS-20260716T162950205Z-TEST-MODE-ACTIVATES-TRANSITIVE-DEPEN-8D1A7815: test mode activates transitive dependency test declarations

## 概要

Merged-module test mode activates every transitive dependency #test declaration, causing unrelated overload and trait declarations to collide when a focused adapter combines the font stroke fixture graph with the compositor RLE step graph.

## 対象

- `nepl-core/src/loader.rs`

## 根拠

- focused doctest が依存 graph 全体の `#test` overload と test impl を有効化し、F5nyh adapter が利用しない宣言まで名前解決・型検査へ混入した。
- root の通常 import だけを有効化すると、その adapter が明示的に利用する transitive test helper を失うため、direct/transitive の推測では契約を表現できない。

## 問題

Merged-module test mode activates every transitive dependency #test declaration, causing unrelated overload and trait declarations to collide when a focused adapter combines the font stroke fixture graph with the compositor RLE step graph.

## 影響

F5nyh and future cross-subsystem runtime fixtures cannot compile honestly even though normal production compile succeeds.

## 修正方針

`#import "..." as ... with tests` を test helper dependency の明示的な edge とする。test origin は root から始め、`with tests` import と include closure だけを推移的に辿る。通常 import と implicit prelude の test item は inactive に保つ。edge authority を module metadata、surface hash、loader session cache identity に保持し、通常 import semantics と `CompilerOptions.test_mode` API は変更しない。

## 検証

Parser/metadata/hash regressions、loader の opt-in/transitive/include/cycle/ordinary-inactive 回帰、active statement hoist exclusion、target gate tests、wasm と LLVM test-mode compilation、F5nyh focused runtime fixtures、normal-mode regression。

`4e5e4ff36`でexplicit `with tests` authority、artifact v13、cold/warm loader回帰を実装した。`cargo test -p nepl-core --lib` 883件、`cargo check --manifest-path nepl-web/Cargo.toml`、F5nyh/F5nyg focused runtime、normal compile isolation、trunk buildを通過した。

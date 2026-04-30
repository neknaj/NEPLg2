# Quality Review: Tutorials

対象 commit: `f108cebd`

## 対象

- `tutorials/getting_started/**`
- `nodesrc/test_tutorial_getting_started_current_style.js`

## 概要

getting_started tutorial は古い章から大きく作り直されており、`Option` / `Result` / `match` / `char` / string/text / collections / move/borrow / Drop / modules / generics / traits を順に扱う構成になっている。source policy では旧 raw memory、panic helper、unconstrained owner generic の再導入を禁止している。

## Actions 根拠

Actions run `25157230630` の `tutorials-test` は `44 total / 21 passed / 23 failed / 0 errored`。failure は主に `resource.owner.maybe_leak` で、最初の失敗は `01_hello_world.n.md` の `stdio_write_fd_mem_result` owner failure である。

したがって tutorial 本文の chapter 構成は現行化されているが、stdlib/std/stdio/string owner contract の failure により CI doctest は green ではない。

## 良い点

- `00_index.n.md` の章立ては現在の NEPLg2 方針に沿っている。
- `12_char_and_ascii.n.md` で char literal と string API 連携を扱う。
- `18_generics.n.md` は `.T: Copy` bound を明示する source policy がある。
- raw memory / unwrap helper を入門例から外す方針が明記されている。

## 問題

- Actions では tutorial doctest が半分近く失敗している。
- failure は tutorial の古さより stdlib owner failure が中心だが、読者向け docs としては「実行可能な入門」がまだ保証できない。
- tutorial は stdout report policy へ更新済みの箇所があるが、`.n.md` 全体の shared policy は open issue のまま。

## 必要な設計

- tutorial doctest failure を stdlib owner issue と tutorial 本文 issue に分類する。
- 章ごとに「本文として正しいが依存 stdlib failure で赤い」状態を追跡できるようにする。
- getting_started は Rust/selfhost shared tests の smoke subset として扱う。

## 進捗状況

- chapter rewrite: 完了済み。
- current-style source policy: あり。
- Actions tutorial doctest: failure。
- stdout report / exit code policy: 移行中。

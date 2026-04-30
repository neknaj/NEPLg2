# Tools Review: Web And Playground

対象 commit: `f108cebd`

## 対象

- `nepl-web/src/lib.rs`
- `nepl-web-playground/src/lib.rs`
- `web/src/**`
- `tests/playground_editor/**`

## 概要

web layer は wasm-bindgen compiler API、browser runtime、editor UI、terminal、workspace panel manager、language analysis provider を持つ。`nepl-web/src/lib.rs` は compile / analyze / WAT decoration / error mapping を提供し、`web/src` は TypeScript playground UI を構成する。

CI の Pages jobs は Actions run `25157230630` で success しているため、bundle/deploy 可能な状態は維持されている。ただし test job 全体は stdlib/static-check failure で赤く、playground が表示する compiler behavior もその影響を受ける。

## Actions 根拠

- `pages-fast-bundle`: success
- `deploy-pages`: success
- `build`: success。tutorial/doc HTML build と bootstrap artifact upload は成功。
- web/editor 専用の runtime UI test job は、この workflow では独立した必須 gate としては見えない。

## 良い点

- `web/src/editor-core` と `web/src/editor` に editor state / rendering / input handling が分離されている。
- `web/src/language/neplg2` が NEPLg2 provider を持つ。
- `web/src/workspace` が panel layout / drag-drop / panel manager に分かれている。
- playground editor fixtures が `tests/playground_editor` に JSON として存在する。
- clean checkout 用に `web/examples` / `web/dist_ts` 生成問題は過去 issue で対処済み。

## 問題

- `nepl-web/src/lib.rs` は約 126KB で、compiler API、analysis transformation、WAT/minify、diagnostic mapping が集中している。
- `web/src/workspace/panel-manager.ts` は約 48KB で、runtime map、layout persistence、drag/drop、terminal/editor/explorer orchestration が集中している。
- `web/dist` と `web/dist_ts` は生成物を含み、source と生成物の drift を継続監視する必要がある。
- Pages deploy は成功しても、compiler/stdout/stderr/runtime failure が多い状態では playground 上の examples 実行信頼性は限定的である。

## 必要な設計

- `nepl-web/src/lib.rs` は compiler facade、analysis facade、WAT utility、diagnostic conversion に分割する。
- panel manager は editor/terminal/explorer runtime adapter と layout persistence を分ける。
- `tests/playground_editor` を CI の明示 job として維持し、diagnostic code / hover / definition が Rust compiler changes に追従しているか確認する。
- generated artifact drift は source policy または build artifact comparison で検出する。

## 進捗状況

- web build/deploy: 通過。
- editor core: 実装中。
- playground workspace: 実装中だが巨大 file あり。
- playground editor fixtures: あり。
- generated artifact drift guard: 一部あり、継続監視が必要。

---
id: ISS-20260507T084050995Z-STD-TEST-FACADE-RE-EXPORTS-ARE-INVIS-F9A19F5A
title: "std/test facade re-exports are invisible to selective imports"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-07
updated: 2026-05-07
target: "nepl-core/src/resolve.rs, nepl-core/tests/resolve.rs, stdlib/std/test.nepl, nodesrc/test_stdlib_std_test_no_unsafe_unwraps.js, stdlib/tests/btreemap.n.md, stdlib/tests/btreeset.n.md"
---

# ISS-20260507T084050995Z-STD-TEST-FACADE-RE-EXPORTS-ARE-INVIS-F9A19F5A: std/test facade re-exports are invisible to selective imports

## 概要

`std/test` が `types` / `assertion` / `report` へ分割された後、`#import "std/test" as { checks_new, ... }` のような selective import が facade の re-export 先まで届かず、`checks_new` や `check_eq_i32` が未解決になっていた。

根本原因は `ImportResolution` が `as *` / `as @merge` の open edge だけを transitive 展開し、`as { ... }` で選択された名前を facade の公開 import 先へ合成していなかったことだった。

## 対象

- `nepl-core/src/resolve.rs`
- `nepl-core/tests/resolve.rs`
- `stdlib/std/test.nepl`
- `nodesrc/test_stdlib_std_test_no_unsafe_unwraps.js`
- `stdlib/tests/btreemap.n.md`
- `stdlib/tests/btreeset.n.md`

## 根拠

- `node nodesrc/tests.js -i stdlib/tests/btreemap.n.md -i stdlib/tests/btreeset.n.md --no-tree -o tmp/btree-selected-import-current.json -j 1 --dist web/dist`: total=10, passed=4, failed=6。
- 失敗はいずれも `std/test` selective import で選択した `checks_new`、`checks_push`、`check_eq_i32`、`checks_print_report`、`checks_exit_code` が未解決になるものだった。
- `std/test.nepl` の facade import を `@merge` に変更しても失敗は残ったため、stdlib facade 表記だけでなく compiler 側の import visibility closure が根本だった。

## 問題

flat loader は import 先 module の item を同一 `Module` へ取り込むため、typecheck 側では `ImportResolution` が元 file span と binding file を使って未修飾参照の可視性を制御している。

従来の `expand_unqualified_import_visibility` は `source -> facade` が `All` の場合だけ `facade -> child` を伝播していた。そのため `source -> facade` が `Selected({ visible_name -> exported_name })` の場合、`facade -> child` が `All` / `Merge` でも `source -> child` の selected edge が生成されず、facade 経由で選択した re-export symbol が不可視になっていた。

## 影響

BTreeMap/BTreeSet focused tests が collection の実挙動ではなく import surface の問題で失敗し、stdlib の pipe collection regression でも本来の RingBuffer owner/borrow 問題が name-resolution noise に隠れていた。

## 修正方針

`UnqualifiedImportVisibility` の transitive 合成を `All` 限定ではなく、`All` / `Selected` の組み合わせで行う。

- `All -> next` は従来どおり `next` を伝播する。
- `Selected -> All` は選択名をそのまま child binding file へ伝播する。
- `Selected -> Selected` は alias mapping を合成し、facade が `x as y` で re-export した symbol を利用側の `y as z` から正しい実体名へ解決する。
- 未選択 symbol は伝播しない。

あわせて `std/test` root は facade re-export であることを明示するため `pub #import ... as @merge` に揃え、source-policy regression もこの境界を固定する。

## 検証

- `cargo test -p nepl-core --test resolve import_resolution_expands_selective_facade_reexport`: passed
- `cargo test -p nepl-core --test resolve`: 18 passed
- `node nodesrc/test_stdlib_std_test_no_unsafe_unwraps.js`: passed
- `trunk build`: passed
- `node nodesrc/tests.js -i stdlib/tests/btreemap.n.md -i stdlib/tests/btreeset.n.md --no-tree -o tmp/std-test-selective-import-btree-after-rebase.json -j 1 --dist web/dist`: total=10, passed=10
- `node nodesrc/tests.js -i tests/stdlib/pipe_collections.n.md --no-tree -o tmp/pipe-collections-after-selective-import-rebase.json -j 1 --dist web/dist`: total=8, passed=7, failed=1。残る 1 件は `ISS-20260507T085551696Z-PIPE-COLLECTIONS-RINGBUFFER-DOCTEST--5893794D` として分離。

## 対応結果

`nepl-core/src/resolve.rs` に `compose_unqualified_import_visibility` を追加し、selected import が facade の re-export 先へ正しく伝播するようにした。`nepl-core/tests/resolve.rs` には `import_resolution_expands_selective_facade_reexport` を追加し、facade 経由の selected alias が実体 `DefId` へ解決されることを固定した。

`std/test.nepl` は `types` / `assertion` / `report` を `pub #import ... as @merge` で公開する facade に揃えた。

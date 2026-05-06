---
id: ISS-20260506T155806325Z-SELFHOST-LEXER-AND-IMPORT-SPEC-FIXTU-68BADCC8
title: "Selfhost lexer and import spec fixtures drift under strict static checks"
area: selfhost
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-07
target: "stdlib/neplg2/core/syntax/lexer.nepl, stdlib/neplg2/core/module/import_spec.nepl, tests/stdlib/neplg2_import_spec.n.md"
---

# ISS-20260506T155806325Z-SELFHOST-LEXER-AND-IMPORT-SPEC-FIXTU-68BADCC8: Selfhost lexer and import spec fixtures drift under strict static checks

## 概要

After direct string submodule imports, focused selfhost doctests no longer stop at undefined string facade names. They expose strict checker failures: lex_stack_drop_top returns a Vec constructor expression with stack/type mismatch, and import_spec doctests leak the SelfhostImportSpec path/alias str owners.

## 対象

- `stdlib/neplg2/core/syntax/lexer.nepl, stdlib/neplg2/core/module/import_spec.nepl, tests/stdlib/neplg2_import_spec.n.md`

## 根拠

- `node nodesrc/tests.js -i stdlib/neplg2/core/syntax/lexer.nepl -i stdlib/neplg2/core/module/import_spec.nepl -i tests/stdlib/neplg2_import_spec.n.md --no-tree -o tmp/selfhost-lexer-import-spec-before.json -j 1`: total=4, passed=0, failed=4。
- `lex_stack_drop_top` は Vec が `storage` field を持つ現行レイアウトへ更新された後も旧 4-field constructor を返しており、`type.stack.extra_values` と return type mismatch を起こしていた。
- import_spec の doctest と `tests/stdlib/neplg2_import_spec.n.md` は `SelfhostImportSpec` の `path` / `alias` owner を検査後に消費せず、strict ResourceIR で `resource.owner.maybe_leak` を報告していた。

## 問題

After direct string submodule imports, focused selfhost doctests no longer stop at undefined string facade names. They expose strict checker failures: lex_stack_drop_top returns a Vec constructor expression with stack/type mismatch, and import_spec doctests leak the SelfhostImportSpec path/alias str owners.

## 影響

Selfhost module graph and import-spec behavior cannot be validated reliably. These failures block selfhost progress independently of the Rust borrow-checker work and may hide parser/import regressions behind stale fixture ownership code.

## 修正方針

Fix the selfhost lexer Vec construction/stack discipline and update import_spec doctests or APIs so returned SelfhostImportSpec string owners are consumed or freed explicitly. Keep enum/match coverage and do not disable ResourceIR owner diagnostics.

## 対応

- `lex_stack_drop_top` を現行 `Vec<i32>` レイアウトに合わせ、`storage` を保持しつつ `data` owner を `field::get` で移動して返すようにした。
- `selfhost_import_spec_free` を追加し、parse で生成した `path` / `alias` の `str` owner を fixture や一時値の破棄時に明示的に閉じられるようにした。
- import_spec doctest と `tests/stdlib/neplg2_import_spec.n.md` の直接 parse fixture は、`field::get_ref` で Copy field を検査し、`field::get` で owner field を移動して assertion に渡す形へ更新した。
- module AST から `Vec<SelfhostImportSpec>` を取り出す経路は、owned aggregate を raw Vec storage に入れる設計問題が残るため、この issue では検査を弱めず `ISS-20260506T171738048Z-SELFHOST-MODULE-IMPORT-SPECS-STORES--9975F52D` として分離した。

## 検証

- `node nodesrc/test_selfhost_string_helpers_boundary.js`: passed
- `node nodesrc/tests.js -i stdlib/neplg2/core/syntax/lexer.nepl -i stdlib/neplg2/core/module/import_spec.nepl -i tests/stdlib/neplg2_import_spec.n.md --no-tree -o tmp/selfhost-lexer-import-spec-final-focused2.json -j 1`: total=4, passed=4, failed=0, errored=0
- `node nodesrc/run_source_policy_regressions.js --warn-only`: all source-policy regressions passed; warning 0
- `trunk build`: passed
- `origin/main` `4bc486af` 取り込み後の focused selfhost tests も total=4, passed=4。全 source-policy 再実行では selfhost/import-spec 関連は passed だが、remote main 由来の別件 `lower_raw_address.rs has 657 lines; responsibility split limit is 620` を検出したため `ISS-20260506T173138867Z-RESOURCE-RAW-ADDRESS-LOWERING-EXCEED-B64EB6D1` として分離。

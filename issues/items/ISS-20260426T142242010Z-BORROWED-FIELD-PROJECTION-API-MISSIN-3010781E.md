---
id: ISS-20260426T142242010Z-BORROWED-FIELD-PROJECTION-API-MISSIN-3010781E
title: "borrowed field projection API missing for repeated aggregate reads"
area: core
status: verified
resolved: true
priority: P2
type: architecture
created: 2026-04-26
updated: 2026-04-26
target: "nepl-core/src/typecheck.rs, nepl-core/src/codegen_wasm.rs, nepl-core/src/codegen_llvm.rs, nepl-core/src/wasm_shared.rs, nepl-core/src/passes/codegen_precheck.rs, stdlib/core/field.nepl, nepl-core/tests/move_check.rs, tests/compiler/move_check.n.md, tests/stdlib/selfhost_cliarg_parser.n.md, stdlib/neplg2/cli/args.nepl"
---

# ISS-20260426T142242010Z-BORROWED-FIELD-PROJECTION-API-MISSIN-3010781E: borrowed field projection API missing for repeated aggregate reads

## 概要

self-host CLI args tests を追加した時、`Result::Ok opts` で得た `SelfhostCliOptions` から `get opts "output"`、`get opts "input"`、`get opts "check"` のように複数 field を読む自然なコードが D3053 `use of moved value: opts` で拒否された。

## 対象

- `stdlib/core/field.nepl; nepl-core move/borrow checker`

## 根拠

- `core/field.get` は by-value API で、field を 1 回読むだけでも aggregate owner を消費する。
- 現状の回避策は `alloc_raw` に aggregate を `store` し、各 field 読み取りのたびに `load<SelfhostCliOptions>` してから `get` する形であり、高レベルの self-host compiler code に raw memory detour が漏れる。
- `&T` から field を読む borrowed projection API がないため、単に複数 field を観察したいだけの処理も所有権移動として表現するしかない。

## 問題

borrowed aggregate から field を読む public API / intrinsic がない。by-value `get` と raw memory reload の二択になるため、move checker の制約を避ける目的で `core/mem` を使う不自然な書き方が増える。

## 影響

Self-host parser, AST, options, and diagnostic structs will need many repeated field reads. Without borrowed field projection, high-level compiler code is pushed toward core/mem workarounds, making ownership intent unclear and making borrow checker limitations look like stdlib style.

## 修正方針

Design and implement borrowed field projection, for example a get_ref-style API or field projection intrinsic over &T. The implementation must distinguish Copy scalar reads, borrowed field references, and owned field moves so non-Copy fields are not silently duplicated. Update stdlib tests away from raw memory detours once the API exists.

## 検証

Add compiler/stdlib tests where a struct with multiple fields is borrowed and several fields are read without moving the owner. Add compile_fail coverage that by-value extraction of a non-Copy owned field still moves it, and that borrowed field references cannot outlive the owner.

## 解決内容

- `#intrinsic "get_field_ref"` を core intrinsic として追加し、`&T` と field selector から field storage address を求めて `&R` を返すようにした。
- `core/field.get_ref` を public API として追加し、by-value `get` と borrowed projection を分離した。`get_ref` 自体は field value を load/copy せず、所有者全体を共有借用する。
- typecheck の field accessor fast path に `get_field_ref` を追加し、string/index selector の aggregate layout 解決後に offset 0 は base reference、offset ありは base + offset の address expression として HIR へ下げるようにした。
- WASM / LLVM codegen と codegen precheck / wasm shared intrinsic list に `get_field_ref` を追加し、field reference を pointer arithmetic だけで lower するようにした。
- borrowed field reference の last-use 後は owner を move できる一方、reference が live な間の owner move と local owner からの field reference escape は拒否する回帰テストを追加した。
- self-host CLI args parser の doctest / regression test から `alloc_raw` / `store` / `load` による field 読み取り detour を外し、`get_ref &opts ...` を使う形に更新した。

## 検証結果

- `cargo fmt --all --check`: pass
- `cargo check --workspace`: pass
- `cargo test -p nepl-core --test move_check move_borrowed_field_projection -- --nocapture`: 3 passed
- `cargo test -p nepl-core --test move_check -- --nocapture`: 27 passed
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/move_check.n.md --no-tree -o tmp/move-check-field-ref-after-883d199.json -j 1`: `total=28`, `passed=28`, `failed=0`
- `node nodesrc/tests.js -i tests/stdlib/selfhost_cliarg_parser.n.md -i stdlib/neplg2/cli/args.nepl --no-tree -o tmp/selfhost-cliarg-field-ref-after-883d199.json -j 1`: `total=10`, `passed=10`, `failed=0`
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-field-ref-after-883d199.json`: 13/13 passed

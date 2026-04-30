# Rust compiler overview

対象 commit: `f108cebd`

## 概要

Rust compiler の中心は `nepl-core` である。現状は実用的な compiler pipeline と多くの regression を持つが、selfhost へ移植する設計としては、巨大 file と旧 HIR checker の残存をそのままコピーしてはいけない。

## 構成

- `lexer.rs`, `parser.rs`, `ast.rs`: NEPLg2 構文の token / AST。
- `loader.rs`, `module_graph.rs`, `source_map.rs`: import、source identity、stdlib resolution。
- `typecheck/*`, `types.rs`, `resolve.rs`: name/type/effect/trait/match/overload。
- `passes/move_check/*`, `passes/drop_insertion.rs`: 旧 move/borrow/drop 系の防壁。
- `resource/*`: Resource IR lowering と cell / owner / borrow / effect / coverage gate。
- `monomorphize.rs`, `hir.rs`, `layout.rs`: HIR instance 化と layout。
- `codegen_wasm.rs`, `codegen_llvm.rs`, `wasm_shared.rs`, `llvm_ir.rs`: backend。
- `compiler.rs`, `diagnostic_codes.rs`, `diagnostic.rs`: pipeline と diagnostic boundary。

## 巨大ファイル

| file | 概算行数 | 判定 |
|---|---:|---|
| `parser.rs` | 4037 | syntax layout と match / if / while / typeexpr が集中している。selfhost では parser submodule 分割が必要。 |
| `codegen_llvm.rs` | 4037 | LLVM backend と tests が大きい。backend diagnostic は改善済みだが分割余地が大きい。 |
| `codegen_wasm.rs` | 2478 | WASM backend の主要 logic が集中。selfhost backend では binary / section / layout / intrinsic を分けるべき。 |
| `types.rs` | 2033 | type arena と type operation が大きい。selfhost S3 では `ty/arena`, `ty/subst`, `ty/layout` に分けるべき。 |
| `typecheck/prefix_check.rs` | 1893 | prefix expression reduction の中心。多数の `unwrap` が stack invariant に依存するため、selfhost では invariant を型/Result で明示する。 |
| `compiler.rs` | 1613 | pipeline と Resource IR gate mapping が同居。diagnostic mapping は維持しつつ、stage orchestration と gate conversion の分割余地がある。 |

## 良い点

- `diagnostic_codes.rs` は stage ごとの enum と stable string boundary を持つ。
- `resource/check.rs` のような monolithic checker は削除され、source policy で再導入を防いでいる。
- Resource IR は lowering coverage、cell、borrow、effect、owner の順に compiler gate へ接続されている。
- `nodesrc/run_source_policy_regressions.js` に Resource checker responsibility policy が含まれ、責務再集中を監視する構成になっている。対象 Actions run では aggregate の `Source policy regressions` step は成功している。

## 残る問題

- `prepare_module_for_codegen_with_source_map` は `insert_drops` 後に monomorphize し、その後 `run_move_check` を実行する。drop elaboration が checked Resource IR に基づいていない。
- `run_move_check` は最初に旧 `passes::move_check::run` を authoritative に実行し、その後 Resource IR gate を実行する。最終設計としては Resource IR が単一 authority になるべきである。
- `owner_summary_variant_paths.rs` が 637 行規模になり、owner variant path logic が再び大きな責務を持ち始めている。対象 Actions run の source policy step は成功しているが、local 直接確認では responsibility split policy が赤くなるため、設計負債として再分割が必要である。
- parser / codegen / prefix_check は巨大で、selfhost へ同じ粒度で移植すると保守不能になる。
- backend は改善済みとはいえ、WASM / LLVM parity と diagnostic coverage を継続監視する必要がある。

## selfhost への示唆

selfhost compiler は現行 Rust compiler の挙動を参考にするが、ファイル構造は `doc/neplg2/self_host_plan.md` の分割を正とする。特に parser、typecheck、Resource IR、codegen は、Rust 側の巨大 file をそのまま移植せず、最初から module boundary を設計する。

---
id: ISS-20260426T213057843Z-LLVM-MONOMORPHIZE-LEAVES-GENERIC-HAS-8FDB0749
title: "LLVM monomorphize leaves generic Hasher trait calls unresolved"
area: core
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-26
updated: 2026-04-27
target: "nepl-core/src/typecheck.rs, nepl-core/src/types.rs, nepl-core/src/monomorphize.rs, nepl-core/src/compiler.rs, nepl-core/src/codegen_llvm.rs, nepl-core/src/diagnostic_ids.rs, nepl-cli/src/main.rs, tests/stdlib/traits_hash.n.md"
---

# ISS-20260426T213057843Z-LLVM-MONOMORPHIZE-LEAVES-GENERIC-HAS-8FDB0749: LLVM monomorphize leaves generic Hasher trait calls unresolved

## 概要

GitHub Actions run 24967172989 llvm-dual-tests and llvm-dual-stdlib panic in monomorphize.rs with unresolved trait call remained after monomorphize: Hasher<str>::hash32 for Option_T_i32 or Option_T_str self types.

## 対象

- `nepl-core/src/monomorphize.rs, nepl-core/src/codegen_llvm.rs, stdlib/alloc/collections/hashmap.nepl, tests/stdlib/traits_hash.n.md`

## 根拠

- `tests/stdlib/traits_hash.n.md` の LLVM compile-only 経路で `hash_with__H_K__i32__pure_str_Option_T_i32 :: Hasher<str>::hash32 [self=Option_T_i32]` が monomorphize 後に残り、内部 panic になっていた。
- LLVM emission は `SourceMap` を渡さずに typecheck / lowering を再実行していたため、`#import "core/field" as field` の qualified alias が解決されず、旧 fallback が `field::get` を bare `get` に落としていた。
- bare `get` の overload set には `HashMap get` が含まれるため、`HashMap` 内部の `field::get hm "hasher"` が誤って collection `get` 候補へ混入し、`.H` が `Option<.V>` や別 hasher 型に汚染されていた。
- 明示型引数を持つ generic 関数の置換 map は resolved TypeId 側だけをキーにしており、raw TypeId を参照する型構造では置換が抜ける経路があった。

## 問題

GitHub Actions run 24967172989 llvm-dual-tests and llvm-dual-stdlib panic in monomorphize.rs with unresolved trait call remained after monomorphize: Hasher<str>::hash32 for Option_T_i32 or Option_T_str self types.

## 影響

HashMap/HashSet and self-host symbol-table style code cannot be trusted under LLVM, and the compiler worker panics instead of emitting a diagnostic.

## 修正方針

Fix trait impl resolution/substitution for generic Hasher calls in LLVM-all monomorphization, and replace the residual panic path with a structured diagnostic if an unresolved trait call remains.

## 検証

Run llvm-all focused tests for stdlib/alloc/collections/hashmap.nepl and tests/stdlib/traits_hash.n.md without monomorphize panics.

## 解決

- CLI の LLVM IR emission から `SourceMap` を渡し、LLVM codegen 前の typecheck / lowering でも import alias と qualified name を保持するようにした。
- qualified call の overload 解決は qualified binding の候補に限定し、`field::get` を bare `get` に戻す旧 fallback は source-map 不在の legacy 経路だけに制限した。
- 明示型引数の置換 map と monomorphize の instantiation map に raw TypeId と resolved TypeId の両方を入れ、型変数束縛後も `.K` / `.H` の置換が抜けないようにした。
- 明示型引数が与えられた呼び出しでは、候補選択後の再推論が user-specified type args を上書きしないようにした。
- monomorphize 後に trait call が残った場合は panic ではなく `D4107` diagnostic として返すようにした。
- `nepl-core/tests/neplg2.rs` に、`HashMap<str, i32, DefaultHash32>` と `HashMap<ModKey, i32, ModHasher>` を同一 module に置いて hasher 型が交差汚染されないことを確認する回帰テストを追加した。

## 修正後検証

- `cargo fmt --all --check`: pass
- `cargo test -p nepl-core --test neplg2 llvm_hashmap_string_key_preserves_explicit_hasher_type_args -- --nocapture`: 1/1 passed
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/stdlib/traits_hash.n.md --runner llvm --llvm-all --llvm-compile-only --no-tree -o tmp/llvm-hasher-monomorphize-after-diagnostic.json -j 1`: total=6, passed=6
- `git diff --check`: pass（CRLF 変換警告のみ）

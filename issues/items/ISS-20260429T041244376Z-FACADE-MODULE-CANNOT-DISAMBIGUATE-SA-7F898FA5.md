---
id: ISS-20260429T041244376Z-FACADE-MODULE-CANNOT-DISAMBIGUATE-SA-7F898FA5
title: "facade module cannot disambiguate same-name imported implementation with alias-qualified call"
area: core
status: fixed
resolved: true
priority: P2
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "nepl-core/src/typecheck/*, nepl-core/tests/import_clause.rs, nepl-core/tests/resolve.rs"
---

# ISS-20260429T041244376Z-FACADE-MODULE-CANNOT-DISAMBIGUATE-SA-7F898FA5: facade module cannot disambiguate same-name imported implementation with alias-qualified call

## 概要

alloc/string.nepl が alloc/string/scanner.nepl を as scanner で import し、同じ public API 名の wrapper から scanner::str_* を呼ぶと、import された同名関数が shadow warning として扱われた後に scanner::str_* が D3001 undefined になる。

## 対象

- `nepl-core/src/typecheck/binding_rules.rs`
- `nepl-core/src/typecheck/block_check.rs`
- `nepl-core/src/typecheck/call_binding_lookup.rs`
- `nepl-core/src/typecheck/driver.rs`
- `nepl-core/src/typecheck/env.rs`
- `nepl-core/src/typecheck/function_check.rs`
- `nepl-core/src/typecheck/name_lookup.rs`
- `nepl-core/src/typecheck/signature.rs`
- `nepl-core/tests/import_clause.rs`
- `nepl-core/tests/resolve.rs`

## 根拠

- `Env::remove_duplicate_func` が同名・同シグネチャ関数を file/module 境界なしに削除していた。
- loader は import graph を flat module として typecheck へ渡すため、facade wrapper と実装 submodule の同名関数は同じ global callable scope に並ぶ。
- facade wrapper 登録時に実装 submodule 側の callable binding が削除され、`impls::scan` の qualified lookup が alias target file を持っていても候補 binding を見つけられなかった。
- 削除を回避しても、関数 symbol が `name + signature` だけで決まるため、同名同シグネチャの別 file 定義が `lookup_all_callables_by_symbol` 上で衝突し、HIR の call target と body type selection が不安定になる。
- `apply_function` 経路の callable lookup が raw `env.lookup_all_callables(name)` へ戻っており、alias-qualified / unqualified の import visibility と local-over-import shadowing を無視する経路が残っていた。

## 問題

alloc/string.nepl が alloc/string/scanner.nepl を as scanner で import し、同じ public API 名の wrapper から scanner::str_* を呼ぶと、import された同名関数が shadow warning として扱われた後に scanner::str_* が D3001 undefined になる。

## 影響

stdlib の巨大 file 分割で public facade が既存 API 名を保ったまま実装 submodule へ同名委譲できず、実装関数へ module-specific prefix を付ける不自然な回避が必要になる。

## 修正方針

同一 source file 内での duplicate function replacement と、別 file/module の同名 callable binding を分離する。同名 local wrapper が存在しても imported module namespace の pub item を `scanner::name` で解決できるようにする。さらに cross-file 同名同シグネチャが合法に共存する場合は、symbol と body type selection も定義単位で一意にする。

## 修正内容

- 同名同シグネチャ callable の重複削除を same file 内だけに限定し、alias-hidden import の実装 binding を local facade 登録時に消さないようにした。
- cross-file で同名同シグネチャ関数が共存する場合だけ、関数 symbol に定義 span 由来 suffix を付け、通常の単一定義関数の symbol は既存形式のまま維持した。
- 関数本体 typecheck と HIR symbol 決定は、名前だけでなく定義 span で hoist 済み binding を選ぶようにした。
- `apply_function` の非 symbol-resolved lookup を `lookup_qualified_bindings` / `lookup_all_unqualified_callables` に通し、alias-qualified call と unqualified call の import visibility を分離した。
- unqualified callable lookup は local 同シグネチャを imported 同シグネチャより優先しつつ、異なるシグネチャの imported overload は保持するようにした。
- `noshadow` の同シグネチャ検査も import visibility を見るようにし、`as *` で見える no_shadow は拒否し、`as alias` で隠れた import は local facade を邪魔しないようにした。

## 検証

- `cargo test -p nepl-core --test import_clause alias_qualified_call_survives_same_name_facade_wrapper -- --nocapture`: pass
- `cargo test -p nepl-core --test import_clause -- --nocapture`: pass
- `cargo test -p nepl-core --test resolve facade_wrapper_can_call_same_named_alias_member -- --exact`: pass
- `cargo test -p nepl-core --test resolve hir_user_call_keeps_local_def_id_when_open_import_is_shadowed -- --exact`: pass
- `cargo test -p nepl-core --test resolve`: 17 passed
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/shadowing.n.md --no-tree -o tmp/facade-alias-shadowing-rebased2.json -j 1`: total=27 passed=27
- `node nodesrc/tests.js -i tests/stdlib/neplg2_import_spec.n.md --no-tree -o tmp/facade-alias-import-spec-rebased2.json -j 1`: total=3 passed=3
- `node nodesrc/tests.js -i tests/compiler/list_dot_map.n.md --no-tree -o tmp/facade-alias-list-dot-map-rebased2.json -j 1`: total=4 passed=3 failed=1。失敗は `free__List_T_T__unit__pure_i32` の `resource.raw.ownership_violation` で、今回の alias/symbol 解決ではなく既存の raw-memory-backed collection / Resource IR 移行残件として扱う。

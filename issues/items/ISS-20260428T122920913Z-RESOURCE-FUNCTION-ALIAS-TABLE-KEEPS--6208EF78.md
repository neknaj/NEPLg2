---
id: ISS-20260428T122920913Z-RESOURCE-FUNCTION-ALIAS-TABLE-KEEPS--6208EF78
title: "Resource function alias table keeps stale aliases after assignment"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "nepl-core/src/resource/check.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260428T122920913Z-RESOURCE-FUNCTION-ALIAS-TABLE-KEEPS--6208EF78: Resource function alias table keeps stale aliases after assignment

## 概要

FunctionAliasTable::copy_alias copies known function aliases but does not clear the target when the source has no known alias. Assigning an unknown function value over a previously known function value can leave the old alias active.

## 対象

- `nepl-core/src/resource/check.rs`
- `nepl-core/tests/resource_ir.rs`

## 根拠

- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 は function / callback 境界でも borrow state と owner/free obligation を落とさない Resource IR 化を求めている。
- `FunctionAliasTable::copy_alias` は source に known function alias がある場合だけ target を更新し、source に alias がない場合は target の古い alias を残していた。
- borrow / owner checker の `IndirectCall` は callee に known alias がある場合、その summary を優先し、alias が空の場合だけ unknown callback fallback を使う。
- そのため known function value を保持していた変数へ unknown function value を assign すると、以後の indirect call が stale known callee として扱われ、unknown callback の保守的な borrow token / owner return propagation が抜ける。

## 問題

FunctionAliasTable::copy_alias copies known function aliases but does not clear the target when the source has no known alias. Assigning an unknown function value over a previously known function value can leave the old alias active.

## 影響

Borrow and owner summaries for indirect calls can be applied to the wrong callee. In particular, an unknown callback can be treated as a stale known function, skipping conservative unknown-callback borrow/owner propagation and weakening Stage 4 function-boundary checks.

## 修正方針

Make alias copy overwrite the target alias state: copy known aliases when present, otherwise clear the target alias. Add a regression where assigning an unknown function over a known alias still uses the unknown callback fallback for returned borrow tokens.

## 検証

- `cargo test -p nepl-core --test resource_ir -- --nocapture`
- `trunk build`
- `node nodesrc/issues.js check`
- `rustfmt --check nepl-core\src\resource\check.rs nepl-core\tests\resource_ir.rs`
- `git diff --check`

## 2026-04-28 Stage 4 function alias clear 対応

`FunctionAliasTable::copy_alias` を target alias state の上書き操作として扱うようにした。source に known function alias がある場合は従来通り target へ copy し、source に known alias がない場合は target の alias entry を消す。

これにより、known function value を保持していた local / temporary に unknown function value を assign / move / read / declare した後、indirect call は stale known summary ではなく unknown callback fallback を使う。borrow checker では戻り値型が一致する active borrow token を保守的に output へ伝播し、return escape を検出できる。

`nepl-core/tests/resource_ir.rs` に、known function alias を unknown function value で上書きした後の indirect call が borrow token return escape を検出する回帰を追加した。

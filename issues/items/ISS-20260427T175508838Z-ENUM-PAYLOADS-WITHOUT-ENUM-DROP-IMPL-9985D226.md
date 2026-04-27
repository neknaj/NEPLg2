---
id: ISS-20260427T175508838Z-ENUM-PAYLOADS-WITHOUT-ENUM-DROP-IMPL-9985D226
title: "enum payloads without enum Drop impl are not structurally dropped"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-27
updated: 2026-04-28
target: "nepl-core/src/passes/drop_insertion.rs, nepl-core/tests/drop.rs"
---

# ISS-20260427T175508838Z-ENUM-PAYLOADS-WITHOUT-ENUM-DROP-IMPL-9985D226: enum payloads without enum Drop impl are not structurally dropped

## 概要

Drop を持つ型が enum variant payload に入っている場合、enum 自体に Drop impl がないと scope end の structural drop が payload を破棄しない。struct field drop は実装済みだが、enum active payload は variant tag による条件付き drop が必要であり、aggregate_fields_with_offsets だけでは表現できない。

## 対象

- `nepl-core/src/passes/drop_insertion.rs, nepl-core/tests/drop.rs`

## 根拠

- `drop_insertion` の structural drop は `aggregate_fields_with_offsets` に依存しており、struct/tuple の field は追跡できるが enum の active variant payload は tag 分岐がないため対象外だった。
- Drop impl のない enum を scope end で破棄すると、payload が `Drop` を持つ型でも destructor が呼ばれないことを Rust harness の side-effect trace で確認した。
- generic `Result<Guard,str>` のような Apply enum では payload type parameter を実型へ写像しないと、`Guard` の Drop 必要性を判断できない。

## 問題

Drop を持つ型が enum variant payload に入っている場合、enum 自体に Drop impl がないと scope end の structural drop が payload を破棄しない。struct field drop は実装済みだが、enum active payload は variant tag による条件付き drop が必要であり、aggregate_fields_with_offsets だけでは表現できない。

## 影響

Option<T> / Result<T,E> / self-host AST enum などに owning payload を入れると、値を消費せず破棄する経路で payload leak が起きる。SelfhostOutcome の cleanup callback も Result cell を raw load して callback へ渡すまではよいが、compiler が enum payload の structural drop を持たない限り汎用 drop glue の土台が不足する。

## 修正方針

drop_insertion が Drop impl を持たない enum / Apply enum の各 variant payload を調べ、Drop を必要とする payload だけ match arm 内で drop する HIR を生成する。active variant だけを drop し、payload が move 済みの enum は既存 VarState に従って drop しない。

## 対応

- Drop impl のない enum / Apply enum について、全 variant の match arm を生成し、Drop が必要な payload を持つ active variant の arm だけ payload binding を作って drop するようにした。
- payload が Drop を必要としない variant は binding しないため、不要な payload copy と destructor 呼び出しを避ける。
- struct/tuple の structural drop 走査でも enum field を検出し、field を temp に move して同じ enum payload drop glue を適用するようにした。
- 既存 VarState に従うため、match などで enum 全体または payload が move 済みの場合は scope end の enum auto drop が発生せず、二重 drop しない。

## 検証

Drop side effect を持つ payload を Option/独自 enum の payload に入れ、enum を読み出さず scope exit したときに destructor が 1 回だけ走る regression を追加する。

- `cargo fmt --check`: pass
- `cargo test -p nepl-core --test drop enum_payload -- --nocapture`: `5 passed`
- `cargo test -p nepl-core --test drop -- --nocapture`: `16 passed`
- `cargo check -p nepl-core`: pass
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/drop.n.md --no-tree -o tmp/enum-payload-autodrop-node.json -j 1`: `total=4`, `passed=4`

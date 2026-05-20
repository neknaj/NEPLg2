---
id: ISS-20260520T050900767Z-SELF-HOST-PRELUDE-REGISTRY-REMAINS-A-653CFB79
title: "self-host prelude registry remains a flat implementation file"
area: selfhost
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-05-20
updated: 2026-05-20
target: "stdlib/neplg2/core/builtins/prelude.nepl; stdlib/neplg2/core/builtins/prelude/**; doc/neplg2/self_host_source_tree_layout_review_20260518.md"
---

# ISS-20260520T050900767Z-SELF-HOST-PRELUDE-REGISTRY-REMAINS-A-653CFB79: self-host prelude registry remains a flat implementation file

## 概要

The self-host builtins/prelude module still keeps builtin kind, arity-specific signature payloads, builtin registry entries, primitive type registry, and stage smoke checks in one flat file. This is already above the split budget and will grow again when resolver/checker/codegen consume builtin metadata.

## 対象

- `stdlib/neplg2/core/builtins/prelude.nepl; stdlib/neplg2/core/builtins/prelude/**; doc/neplg2/self_host_source_tree_layout_review_20260518.md`

## 根拠

- `core/builtins/prelude.nepl` は builtin kind、signature payload、builtin function registry、primitive type registry、default path、stage0 smoke を 1 ファイルに保持していた。
- `SelfhostBuiltinKind` と `SelfhostBuiltinSignature` は型安全と match 網羅性のための重要な model だが、registry 実装と同じ flat file に置かれ続けると numeric tag や fixed argument slot への退行を監視しにくい。
- `doc/neplg2/self_host_source_tree_layout_review_20260518.md` は root module を public facade に限定し、self-host compiler の初期段階から最終階層に実装を置く方針を定めている。

## 問題

The self-host builtins/prelude module still keeps builtin kind, arity-specific signature payloads, builtin registry entries, primitive type registry, and stage smoke checks in one flat file. This is already above the split budget and will grow again when resolver/checker/codegen consume builtin metadata.

## 影響

Leaving the registry flat makes builtin effect/type metadata harder to audit, encourages resolver and checker code to depend on one oversized module, and risks reintroducing numeric arity or raw string lookup shortcuts instead of typed enum/payload boundaries.

## 修正方針

Split prelude.nepl into an implementation-free facade plus responsibility-specific model, kind equality, signature accessors, builtin function registry, primitive type registry, and stage0 modules. Add source-policy regressions so the facade remains thin and existing signature payload checks read the combined split source.

## 修正内容

- `core/builtins/prelude.nepl` を doctest と public re-export だけを持つ implementation-free facade にした。
- `prelude/model.nepl` に builtin / primitive registry の Copy model を分離した。
- `prelude/kind.nepl` に `SelfhostBuiltinKind` の exhaustive equality を分離した。
- `prelude/signature.nepl` に signature constructor と arity / argument / result accessor を分離した。
- `prelude/function_registry.nepl` に `alloc` / `dealloc` / `realloc` の typed registry を分離した。
- `prelude/primitive_registry.nepl` と `prelude/path.nepl` に primitive registry と default path boundary を分離した。
- `prelude/stage0.nepl` に smoke check を分離し、必要な model import を明示した。
- `nodesrc/selfhost_prelude_sources.js` と `nodesrc/test_selfhost_prelude_split_contract.js` を追加し、既存の signature payload / numeric kind tag policy は split 後の合成 source を読むようにした。

## 検証

Run the prelude split contract, builtin signature payload policy, numeric kind tag policy, focused prelude doctest, issues check, and diff check.

- `node nodesrc/test_selfhost_prelude_split_contract.js`
- `node nodesrc/test_selfhost_builtin_signature_payload.js`
- `node nodesrc/test_selfhost_model_no_numeric_kind_tags.js`
- `node nodesrc/tests.js -i stdlib/neplg2/core/builtins/prelude.nepl --no-tree -o tmp/agent1-prelude-split-core.json -j 1 --dist web/dist --assert-io`: total=1, passed=1

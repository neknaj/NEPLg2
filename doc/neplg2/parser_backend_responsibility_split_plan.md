# NEPLg2 parser / backend responsibility split plan

作成日: 2026-05-08

## 目的

`typecheck` と Resource IR は責務分割と source policy の監視が入っている。一方で、parser、WASM backend、LLVM backend、monomorphize は巨大 file のまま残っており、新しい診断、target gate、layout lowering、trait specialization の修正が同じ場所へ積み増されやすい。

この文書は、静的検査の正確性を支える前段 parser と後段 backend を、無秩序な monolith に戻さないための責務境界と段階的な分割方針を定める。

関連 issue:

- [ISS-20260507T144627703Z-RUST-PARSER-AND-BACKEND-CODEGEN-LACK-11798587](../../issues/items/ISS-20260507T144627703Z-RUST-PARSER-AND-BACKEND-CODEGEN-LACK-11798587.md): parser / backend / monomorphize の responsibility source policy 欠落。
- [ISS-20260516T061424173Z-WASM-CODEGEN-RESPONSIBILITY-FREEZE-R-2705FB59](../../issues/items/ISS-20260516T061424173Z-WASM-CODEGEN-RESPONSIBILITY-FREEZE-R-2705FB59.md): WASM backend root file の responsibility freeze 回帰。
- [ISS-20260516T065711051Z-LLVM-CODEGEN-RESPONSIBILITY-FREEZE-R-0530B190](../../issues/items/ISS-20260516T065711051Z-LLVM-CODEGEN-RESPONSIBILITY-FREEZE-R-0530B190.md): LLVM backend root file の responsibility freeze 回帰。
- [ISS-20260429T040748194Z-RUST-COMPILER-DIAGNOSTICS-ARE-NOT-AL-1617747D](../../issues/items/ISS-20260429T040748194Z-RUST-COMPILER-DIAGNOSTICS-ARE-NOT-AL-1617747D.md): enum-first diagnostic と境界表示。
- [NEPLg2 静的検査の複雑化解消計画](./static_check_complexity_reduction_plan.md): Resource IR を authority にする大規模修正。

## 現状 baseline

2026-05-08 時点の source policy baseline は次の通り。

| file | lines | 責務 |
|---|---:|---|
| `nepl-core/src/parser.rs` | 4233 | token navigation、syntax recovery、declaration / expression / type expression parsing が集中している。 |
| `nepl-core/src/codegen_wasm.rs` | 2519 | WASM module assembly、instruction emission、runtime helper lowering が残る。string data layout と aggregate field selector は分離済み。 |
| `nepl-core/src/codegen_llvm.rs` | 4184 | LLVM IR text lowering、raw LLVM body handling、entry / target preparation が残る。HIR type mapping と aggregate field selector は分離済み。 |
| `nepl-core/src/monomorphize.rs` | 1454 | trait impl indexing、call specialization、runtime helper selection、unresolved trait call reporting が集中している。 |

これらの line limit は完成形ではなく、これ以上の責務増加を防ぐ凍結線である。新規機能や大きな修正が必要な場合は、まず下記の分割 stage に従って責務を切り出してから実装する。

## 分割原則

1. parser は AST surface syntax の構築だけを担当し、type / effect / Resource IR の判断を持たない。
2. backend は checked HIR / prepared backend input を lower するだけを担当し、diagnostic 分類や Resource IR safety check を再実装しない。
3. monomorphize は specialization と unresolved trait call の構造化結果を返すだけにし、panic や文字列 sentinel で未解決状態を表さない。
4. target-specific な layout / helper / instruction emission は、WASM と LLVM それぞれの backend file に混在させず、共有できる設計情報は専用 module に分ける。
5. diagnostic は `DiagnosticCode` enum を生成時点で確定し、parser/backend 固有の文字列後付け分類を作らない。

## Parser split stages

### P0: policy freeze

- `parser.rs` の重複 module doc を除去する。
- `nodesrc/test_parser_backend_responsibility_policy.js` で baseline line limit、重複 doc 禁止、責務分割計画の存在を監視する。

### P1: token stream / recovery

候補 module:

- `parser/token_stream.rs`: `peek` / `advance` / indentation navigation。
- `parser/recovery.rs`: no-progress guard、recursion depth、error recovery。
- `parser/diagnostics.rs`: `ParserDiagnosticCode` を受け取る parser-local helper。

### P2: declarations and directives

候補 module:

- `parser/directive.rs`: `#target`、raw body directive、import directive。
- `parser/declaration.rs`: function / struct / enum / trait / impl declaration。
- `parser/signature.rs`: function signature、generic parameter、capability syntax。

### P3: expression and block syntax

候補 module:

- `parser/block.rs`: indentation block、single-line block。
- `parser/expr.rs`: prefix expression、call item、literal expression。
- `parser/match_expr.rs`: match arm、pattern、payload binding。

### P4: type expression

候補 module:

- `parser/type_expr.rs`: type constructor、tuple/function type、type argument。
- `parser/type_recovery.rs`: malformed type expression の復旧。

## Backend split stages

### B1: shared backend model

候補 module:

- `backend/layout.rs`: layout query wrapper と storage alignment/size boundary。
- `backend/runtime_helper.rs`: runtime helper selection と import/export planning。
- `backend/raw_body.rs`: raw WASM / LLVM body handling policy。

### B2: WASM backend

候補 module:

- `wasm/module.rs`: section assembly。
- `wasm/function.rs`: function lowering orchestration。
- `wasm/instruction.rs`: expression / statement instruction emission。
- `wasm/aggregate.rs`: struct / tuple / enum payload lowering。
- `wasm/call.rs`: direct / indirect / intrinsic call lowering。

進捗:

- `codegen_wasm/string_data.rs`: string literal の static data segment、heap base、minimum memory page 算出を root から分離した。
- `codegen_wasm/aggregate.rs`: tuple index / struct field name から field type と byte offset を得る selector layout 解決を root から分離した。
- `nodesrc/test_parser_backend_responsibility_policy.js`: root file の line freeze を 2525 行に下げ、上記 2 module の存在と責務上限を監視する。

### B3: LLVM backend

候補 module:

- `llvm/module.rs`: module text assembly。
- `llvm/function.rs`: function lowering orchestration。
- `llvm/value.rs`: SSA value and local mapping。
- `llvm/aggregate.rs`: struct / tuple / enum payload lowering。
- `llvm/raw_body.rs`: source raw LLVM body bridge。

進捗:

- `codegen_llvm/type_map.rs`: HIR `TypeId` から LLVM scalar/value type への写像を root から分離した。
- `codegen_llvm/aggregate.rs`: tuple index / struct field name から field type と byte offset を得る selector layout 解決を root から分離した。
- `nodesrc/test_parser_backend_responsibility_policy.js`: root file の line freeze を 4188 行に下げ、上記 2 module の存在と責務上限を監視する。

## Monomorphize split stages

### M1: trait impl index

- `monomorphize/impl_index.rs`: impl entry collection と lookup。
- `monomorphize/trait_identity.rs`: monomorphize 内部の trait / method identity newtype。
- `monomorphize/trait_lookup.rs`: trait application key、trait method key、impl entry / resolution model。
- `monomorphize/runtime_helper.rs`: helper specialization key selection。

進捗:

- 2026-05-12: `ISS-20260512T175900768Z-MONOMORPHIZE-TRAIT-LOOKUP-MODEL-EXCE-55B52B7E` を追加し、`MonoTraitApplication` / `MonoTraitMethodKey` / `MonoTraitLookupKey` / `TraitImplEntry` / `TraitImplResolution` を `monomorphize/trait_lookup.rs` へ分離した。root `monomorphize.rs` の line limit は 1455 から 1425 へ下げ、新 module も 90 lines 上限で監視する。
- 2026-05-13: `ISS-20260512T183111826Z-MONOMORPHIZE-TRAIT-APPLICATION-STILL-835C27CF` で `MonoTraitId` / `MonoTraitMethodId` を `monomorphize/trait_identity.rs` へ分離した。`trait_lookup.rs` の line limit は上げず、identity module を 45 lines 上限で監視する。

### M2: specialization engine

- `monomorphize/specialize.rs`: type substitution と function cloning。
- `monomorphize/call.rs`: direct / trait / indirect call rewrite。

### M3: diagnostics boundary

- `monomorphize/unresolved.rs`: `UnresolvedTraitCall` collection。
- `monomorphize/result.rs`: public `MonomorphizeResult` contract。

## Source policy

`nodesrc/test_parser_backend_responsibility_policy.js` は次を監視する。

- この計画 document が存在し、P/B/M stage と関連 issue を含む。
- `parser.rs` の重複 module doc が戻らない。
- parser/backend/monomorphize の baseline line count を超えて責務を追加しない。
- `monomorphize/trait_identity.rs` が存在し、trait / method identity newtype が lookup key module へ戻らない。
- `monomorphize/trait_lookup.rs` が存在し、trait lookup model が root `monomorphize.rs` へ戻らない。
- `run_source_policy_regressions.js` から policy が実行される。

今後、実際の分割で module が追加されたら、line limit は新 module へ移し、root file の limit を段階的に下げる。limit を上げて問題を隠すことは禁止する。

## 完了条件

1. parser / backend / monomorphize の責務境界が source policy で監視される。
2. 新規の大きな修正が monolith へ直接積み増されない。
3. 各 split stage の module 化が進むたびに、root file の line limit が下がる。
4. diagnostic / Resource IR / type safety の責務が parser/backend 側へ逆流しない。

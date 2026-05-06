# Pipeline / diagnostics review

対象 commit: `f108cebd`

## pipeline

`compiler.rs` の codegen 準備は、おおむね次の順序で動く。

1. target precheck。
2. typecheck。
3. HIR 上で `passes::insert_drops`。
4. monomorphize。
5. unresolved trait call check。
6. `run_move_check`。
7. backend へ `PreparedProgram` を渡す。

`run_move_check` 内では次を実行する。

1. Resource shadow report。
2. 旧 `passes::move_check::run`。
3. Resource IR lowering。
4. lowering coverage gate。
5. initialized/raw cell gate。
6. borrow lifetime gate。
7. effect boundary gate。
8. owner obligation gate。

## 評価

Resource IR gate はかなり前進しており、直近 main では indirect effect、Result variant/value condition、owner summary refinement が入っている。これは型安全・メモリ安全を compiler-owned state へ寄せる方向として妥当である。

一方で、drop insertion が Resource IR check 前の HIR で走り、旧 move checker が先に authoritative である点は、最終設計としては未完である。これは `ISS-20260425T000000Z-RV-CORE-009-58589A3F` の主要残件である。

## diagnostic code

`diagnostic_codes.rs` は次の階層 enum を持つ。

- `Loader`
- `Lexer`
- `Parser`
- `Resolve`
- `Type`
- `Effect`
- `Resource`
- `Backend`

Resource diagnostic はさらに次へ分かれる。

- `Move`
- `Borrow`
- `Cell`
- `Owner`
- `Raw`
- `Lower`

各 enum は `as_str()` と `message()` を持ち、stable string は外部境界でだけ生成される。これは selfhost にコピーすべき設計である。

## 良い点

- `DiagnosticSpec` と `Diagnostic::error_with_code` により、diagnostic 作成時点で enum code を確定できる。
- `ALL_DIAGNOSTIC_CODES` と uniqueness test により serialized name の重複を防いでいる。
- Resource owner / borrow / cell diagnostic は raw bucket に潰さず、それぞれの enum code へ写像される。
- effect boundary では impure indirect call が `EffectDiagnosticCode::PureCallsImpure` へ写像される。

## 残る問題

- `UnsafeMemoryInPureFunction` は 2026-05-06 時点で `effect.pure.calls_impure` へ error 化済みであり、残る未完了点は raw-memory-boundary capability の stdlib migration 限定許可である。
- raw memory boundary file の除外は移行期として必要だが、最終的には public safe API と internal unsafe boundary を module/token で分けるべきである。
- `compiler.rs` に pipeline orchestration と Resource diagnostic conversion test が同居している。現状は許容範囲だが、selfhost では `pipeline`, `diagnostic_mapping`, `resource_gate` の分割を前提にする。

## 結論

diagnostic code の設計方向は妥当で、selfhost でも raw string 主体に戻してはいけない。pipeline は移行中であり、最終的には checked Resource IR と drop elaboration を一体化する必要がある。

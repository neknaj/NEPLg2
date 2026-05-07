# Rust コンパイラ overview レビュー

確認対象 commit: `3742a1a7 fix(cli): run Resource IR gates for check-only`

## 確認範囲

- `nepl-core/src/lib.rs`
- `nepl-core/src/compiler.rs`
- `nepl-core/src/{lexer,parser,ast,loader,module_graph,resolve}.rs`
- `nepl-core/src/{typecheck,resource,passes}.rs`
- `nepl-core/src/{monomorphize,layout,codegen_wasm,codegen_llvm,target_gate,target_precheck}.rs`
- `nepl-cli/src/main.rs`

## 全体構造

Rust compiler は、lexer/parser で AST を作り、loader/source map で multi-file source を flat module として保持し、typecheck で HIR と type context を作る。その後、monomorphize 済み source HIR を ResourceIR へ lowering し、cell/owner/borrow/effect/drop を検査してから plan-based drop insertion と backend codegen へ進む。

`nepl-core` は no_std 対応を意識しているが、loader/module graph など host filesystem に依存する領域も同じ crate にある。`SourceMap` は core 側へ分離済みで、typecheck / diagnostics / ResourceIR は host path ではなく file id / span を通じて source 情報を受ける。

## 進捗状況

| 領域 | 状態 | 判定 |
|---|---|---|
| compiler pipeline | compile preparation は ResourceIR gate と drop insertion を通る。`--check` も `3742a1a7` で同じ prepare phase を共有するよう修正された。 | 良い。artifact emission と safety check の責務分離を保ったまま meaning が揃った。 |
| typecheck / ResourceIR | module split と source policy がある。 | 良い。安全性 review の中心として成立している。 |
| parser / backend / monomorphize | 巨大 file のままで、typecheck/resource 相当の responsibility guard がない。 | 新規 issue 化済み。selfhost parity 前に分割設計が必要。 |
| diagnostics | code-first / mandatory diagnostic code。 | 良い。selfhost 側への拡張が必要。 |
| target/profile gates | `target_gate.rs` と `target_precheck.rs` に共通化されている。 | 良い。codegen と typecheck の active statement 集合を揃える方向。 |
| loader raw-memory boundary | exact stdlib path table で raw memory capability を付与する。 | 移行措置として機能。最終的には型/API 境界へ寄せる必要がある。 |

## 主要リスク

- parser/backend/monomorphize の肥大化は、将来の diagnostics、layout、match lowering、selfhost parity の監査性を落とす。
- public `monomorphize` API は unresolved trait call で panic し得る。compile pipeline は diagnostic-returning API を使うが、public API として残るのは不適切。
- `--check` は修正済みだが、今後再び typecheck-only convenience API へ戻さない source policy / regression の維持が必要。

## issue 連携

- `ISS-20260507T143850332Z-CLI-CHECK-DOES-NOT-RUN-RESOURCEIR-ME-D1F139FF` fixed
- `ISS-20260507T144627703Z-RUST-PARSER-AND-BACKEND-CODEGEN-LACK-11798587`
- `ISS-20260507T144641729Z-PUBLIC-MONOMORPHIZE-API-PANICS-ON-UN-4492668C`

## 次の確認

Rust compiler の残りは、lexer/parser、loader/resolve、drop/effect、codegen/layout/target、tests の個別文書で扱う。

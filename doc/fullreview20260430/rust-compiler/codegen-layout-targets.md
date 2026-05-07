# Rust コンパイラ codegen / layout / target レビュー

確認対象 commit: `3742a1a7 fix(cli): run Resource IR gates for check-only`

## 確認範囲

- `nepl-core/src/codegen_wasm.rs`
- `nepl-core/src/codegen_llvm.rs`
- `nepl-core/src/wasm_shared.rs`
- `nepl-core/src/llvm_ir.rs`
- `nepl-core/src/layout.rs`
- `nepl-core/src/runtime_helpers.rs`
- `nepl-core/src/target_gate.rs`
- `nepl-core/src/target_precheck.rs`
- `nepl-core/tests/{codegen_diagnostics,layout,llvmir,neplg2,check_pipeline}.rs`
- `tests/compiler/{codegen_diagnostics,llvm_target,reference_codegen,raw_body_precheck,ret_string_example}.n.md`

## 進捗状況

| 領域 | 状態 | 判定 |
|---|---|---|
| WASM backend | code-first diagnostics と precheck があり、production panic/expect は基本的に除去済み。 | 機能は進んでいるが file が巨大。 |
| LLVM backend | raw `#llvmir` と subset lowering を持ち、host clang 実行は CLI 側。 | 機能は進んでいるが file が巨大。 |
| layout | storage size/align/field offset が `layout.rs` に集約されている。 | 良い。backend 間 drift を減らす方向。 |
| target gates | target/profile boolean gate は `target_gate.rs` に共通化されている。 | 良い。invalid gate diagnostic も typed。 |
| codegen precheck | unsupported signature、missing return、LLVM unsupported intrinsic などを codegen 前に diagnostic 化する。 | 良い。panic 経路削減に効いている。 |
| runtime helpers | alloc/dealloc/realloc ABI helper lookup が `runtime_helpers.rs` に分離されている。 | 良い。public `core/mem` 名に依存しすぎない方向。 |

## 良い点

- `codegen_diagnostics.rs` は unsupported wasm signature、unknown variable、missing string literal、missing return などを diagnostic code で確認している。
- `layout.rs` が `storage_size_bytes` / `storage_align_bytes` / field layout を共有し、backend ごとの layout 計算重複を減らしている。
- `target_gate.rs` は target/profile gate を `GateDecision` enum で表し、unknown gate を silently inactive にしない。
- `wasm_shared.rs` が reachable function collection、wasm signature、raw body precheck を共有している。

## 問題

### backend file が巨大

`codegen_llvm.rs` は 4188 lines、`codegen_wasm.rs` は 2573 lines。diagnostic panic 経路は減ったが、match lowering、aggregate lowering、runtime helper lower、raw body、function table、string/data segment、instruction emission などが大きく混在している。これは `ISS-20260507T144627703Z-RUST-PARSER-AND-BACKEND-CODEGEN-LACK-11798587` で追跡する。

### public monomorphize panic API

backend preparation は diagnostic-returning monomorphize API を使っているが、public `monomorphize` は unresolved trait call で panic し得る。backend review の観点では、codegen 前段が panic で落ちる public path を残さない必要がある。これは `ISS-20260507T144641729Z-PUBLIC-MONOMORPHIZE-API-PANICS-ON-UN-4492668C` で追跡する。

## 次に確認すること

- source policy 追加時に、WASM/LLVM の match lowering と aggregate layout が backend 間で同じ意味を持つかを regression 化する。
- LLVM backend の subset lowering と raw `#llvmir` path が target gate / entry resolution / diagnostics で分離されているかをさらに見る。

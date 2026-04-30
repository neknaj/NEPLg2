# Codegen review

対象 commit: `f108cebd`

## 概要

WASM backend と LLVM backend は実用段階にあるが、両方とも巨大 file である。過去の issue で panic 経路や backend diagnostic はかなり改善されているが、selfhost では分割設計が必須である。

## WASM backend

`codegen_wasm.rs` は約 2500 行で、WASM emission、intrinsic、function value、enum/struct/tuple lowering、validation error mapping などを担う。

良い点:

- backend diagnostic code があり、validation failure も `backend.wasm.validation_failed` へ写像される。
- indirect signature や function value unknown などの diagnostic が enum 化されている。
- WASM shared helper が `wasm_shared.rs` に分かれている。

残る問題:

- file が大きく、section emitter / binary writer / intrinsic lowering / layout application の境界が薄い。
- selfhost WASM backend では、`codegen/wasm/binary`, `section`, `leb128`, `layout`, `intrinsic`, `runtime` に分けるべきである。

## LLVM backend

`codegen_llvm.rs` は約 4000 行で、LLVM IR text generation と tests が同居している。

良い点:

- backend diagnostic code があり、raw body mismatch / unknown variable / unsupported HIR などを enum 化している。
- LLVM raw body と #entry bridge の tests がある。

残る問題:

- file が大きく、text emitter、layout、intrinsic、raw body handling、tests を分ける余地が大きい。
- README / docs では LLVM setup と dual backend verification の説明があるが、selfhost 初期段階で LLVM parity を完了条件にしない方針は維持すべきである。

## backend parity

backend は compiler safety の最後の砦ではない。Resource IR / typecheck / target precheck が安全性を保証し、backend は checked IR を実装する立場にするべきである。

## selfhost への示唆

selfhost S5 は WASM backend から始める。raw pointer 直接操作を backend 内に広げず、`ByteBuilder` / binary section emitter / layout table を分ける。LLVM backend は text emitter として後続段階にし、初期 selfhost の blocker にしない。

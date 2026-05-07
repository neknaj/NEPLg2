# Rust コンパイラ typecheck レビュー

確認対象 commit: `3742a1a7 fix(cli): run Resource IR gates for check-only`

## 確認範囲

- `nepl-core/src/typecheck.rs`
- `nepl-core/src/typecheck/**`
- `nepl-core/src/types.rs`
- `nepl-core/src/hir.rs`
- `nepl-core/src/compiler.rs`
- `nodesrc/test_static_check_boundary_responsibility.js`

このレビューでは、typecheck が巨大単一 file へ戻っていないか、match / enum / effect / diagnostic が静的検査しやすい形か、ResourceIR へ渡す HIR の前提を正しく作っているかを確認した。

## 進捗状況

| 領域 | 状態 | 判定 |
|---|---|---|
| module split | `typecheck.rs` は facade になり、`driver`、`prefix_check`、`call_resolution`、`block_check`、`match_check` などへ分割済み。 | 良い。source policy が主要 file の再肥大化を監視している。 |
| overload / call resolution | call resolution、overload selection、selected call apply が分離されている。 | 良い。generic / trait / field 系の後続レビューで詳細確認が必要。 |
| match checking | enum match は重複 arm、unknown variant、payload bind、non-exhaustive を診断する。bool は true/false で網羅、i32/u8/char は wildcard を要求する。 | 方針に合う。match の網羅性検査を user code に効かせる重要な基盤。 |
| scalar char support | char literal arm は char/u8/i32 の型に応じて制約され、bool との混同を拒否する。 | 既存 char 改善と整合。char stdlib 連携は stdlib review で確認する。 |
| effect typing | raw body memory operation は pure context で拒否し、raw memory boundary は source map で限定する。 | 移行 boundary として妥当。最終的には stdlib API 側の型境界へ寄せるべき。 |
| diagnostic | type/effect diagnostics は typed `DiagnosticCode` へ分類される。 | 良い。code-less diagnostic へ戻さない policy がある。 |
| check pipeline | `check_module_with_source_map` は `3742a1a7` で compile preparation を共有し、typecheck 後に ResourceIR gate まで進む。 | fixed。typecheck 成功だけを静的安全性全体と誤認する経路は閉じられた。 |

## 良い点

- typecheck の責務分割は、現時点で source policy line limit の範囲に収まっている。
- `match_check.rs` は enum と scalar を明確に分け、wildcard の位置、重複 arm、非網羅、payload bind を診断する。
- char literal と integer literal の混同を拒否しており、`'a'` の導入後に数値 sentinel へ戻す方向ではない。
- raw memory intrinsic の許可が `raw_body_memory_operations_allowed` に集約されており、無条件 pure 扱いではない。

## 問題

### check-only API の typecheck-only 退行を防ぐ必要がある

typecheck 自体は Rust compiler の前段として整ってきており、`3742a1a7` で CLI の `--check` も typecheck 後に ResourceIR gate へ進むようになった。今後のリスクは、利便 API や stack overflow 回避を理由に check-only が再び typecheck-only へ戻ることである。typecheck の役割は HIR/effect/diagnostic の前提を作ることにとどめ、静的安全性全体の authority は ResourceIR gate と共有 prepare phase で保証する必要がある。

### typecheck は ResourceIR に必要な情報を作る責務に集中すべき

現在の安全性 authority は ResourceIR へ移っている。typecheck に raw owner / borrow / drop の special-case を戻すと、二重 authority になり、後方互換用の雑設計が残る。typecheck の役割は型、effect、HIR、span/source map、diagnostic code を正しく作り、ResourceIR が検査可能な情報を欠落させないことに限定するべきである。

## 次に確認すること

- parser / HIR / typecheck の field projection と aggregate destructuring が ResourceIR の per-field move semantics とずれていないか。
- selfhost typecheck 設計で、Rust 側の module split と enum-first diagnostics を移植できるか。
- `tests/compiler` の match/char/effect diagnostics が、Rust/selfhost 共通 `.n.md` 運用へ移せる形か。

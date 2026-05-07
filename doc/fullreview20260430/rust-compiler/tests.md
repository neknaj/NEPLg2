# Rust コンパイラ tests レビュー

確認対象 commit: `3742a1a7 fix(cli): run Resource IR gates for check-only`

## 確認範囲

- `nepl-core/tests/**`
- `tests/compiler/**`
- `nodesrc/test_static_check_boundary_responsibility.js`
- `nodesrc/test_resource_gate_order.js`
- `nodesrc/test_resource_checker_responsibility.js`
- `nodesrc/test_diagnostic_code_first_boundary.js`
- GitHub Actions run list from `gh`

## 進捗状況

| 領域 | 状態 | 判定 |
|---|---|---|
| Rust unit/integration tests | `nepl-core/tests` は ResourceIR、typecheck、codegen、resolve、char、effects など広い。 | 厚い。`resource_ir.rs` は非常に大きいが regression 集約として機能している。 |
| `.n.md` compiler tests | syntax / type / move/effect / codegen / LLVM / match / char などを持つ。 | 広い。Rust/selfhost 共通運用は別 plan の未完課題。 |
| source policy tests | typecheck/resource/diagnostic/selfhost/stdlib boundary を Node tests で監視する。 | 有効。parser/backend/monomorphize には不足。 |
| Actions status | 最新 `3742a1a7` run は review 時点で pending。直前 `d2ba8b8b` run は in_progress。 | green 判定は未確定。completed latest run を後続 checkpoint で確認する。 |

## 良い点

- `check_pipeline.rs` は deep prefix chain の stack overflow 回避、prepare codegen、monomorphize、ResourceIR static check を確認している。
- `3742a1a7` で check-only API と CLI `--check` が ResourceIR gate を通す regression が追加された。
- `resource_ir.rs` と `tests/stdlib/memory_safety.n.md` は、recent `region_ptr` / `region_ptr_at` non-owning provenance regression を compiler gate と doctest の両方で固定している。
- `test_diagnostic_code_first_boundary.js` は code-less diagnostic と wildcard diagnostic match を拒否し、診断 taxonomy の退行を防ぐ。
- `.n.md` 側は expected `diag_code` を使えるため、Rust/selfhost の diagnostic parity に向けた土台がある。

## 問題

### parser/backend source policy が不足

typecheck/resource には file split/line limit の source policy があるが、parser/backend/monomorphize には同等の guard がない。巨大 file の問題は review だけでなく regression として固定する必要がある。

### GitHub Actions は最新完了 run で判断する必要がある

main への連続 push で Actions は cancelled が多い。review の test 状況は local test ではなく `gh` で latest completed run を確認する方針なので、pending/in_progress の間は green 判定を保留する。

## 次に確認すること

- `quality/tests.md` で `.n.md` stdout/assert report 設計と Rust/selfhost 共通運用を再確認する。
- latest main run が completed になった時点で job conclusion を記録する。

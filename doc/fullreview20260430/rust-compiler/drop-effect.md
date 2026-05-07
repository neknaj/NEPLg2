# Rust コンパイラ drop / effect レビュー

確認対象 commit: `3742a1a7 fix(cli): run Resource IR gates for check-only`

## 確認範囲

- `nepl-core/src/passes/drop_insertion.rs`
- `nepl-core/src/resource/drop_*.rs`
- `nepl-core/src/resource/effect*.rs`
- `nepl-core/src/typecheck/effect_check.rs`
- `nepl-core/src/effects.rs`
- `nepl-core/src/target_precheck.rs`
- `nepl-core/tests/{drop,drop_overwrite,effects,check_pipeline}.rs`
- `tests/compiler/{drop,drop_overwrite,move_effect,raw_body_precheck,indirect_effect}.n.md`

## 進捗状況

| 領域 | 状態 | 判定 |
|---|---|---|
| drop plan | ResourceIR cell check の live auto-drop fact から `ResourceDropElaborationPlan` を作る。 | 良い。candidate plan ではなく checked fact を入力にしている。 |
| drop insertion | `insert_resource_drops` が plan を消費し、scope local / assignment overwrite を enum で分岐する。 | 良い。旧 VarState walker ではない。 |
| HIR bridge | plan と HIR の対応を bridge gate で検証してから drop insertion する。 | 良い。二重 authority を避ける方向。 |
| effect typing | typecheck は pure raw body memory operation を拒否し、ResourceIR effect gate は raw memory/external IO/nondet/unknown を分類する。 | 良い。raw boundary の最終設計は未完。 |
| target raw body precheck | `target_precheck.rs` が active raw body と target mismatch を共通診断する。 | 良い。WASM/LLVM 差分診断を減らす。 |

## 良い点

- `ResourceAutoDropKind`、`ResourceDropRequirement`、`ResourceEffectBoundaryDiagnostic` などが enum で分かれており、match による網羅性が効きやすい。
- ResourceIR gate は drop insertion より前に実行され、生成 Drop call が source violation を隠さない。
- raw body precheck と effect check は target/profile gate と同じ active statement 評価へ寄せている。

## 問題

### raw boundary capability はまだ path-based

drop/effect の中核は整理されたが、stdlib raw memory 実装の許可は loader source map の exact path table に依存する。これは段階移行としては必要だが、最終的には `OwnedRegion` / `MemPtr` / raw storage capability の型設計と ResourceIR summary で表すべきである。

### check-only と effect/resource diagnostics の断絶は修正済み

`3742a1a7` で `--check` が shared prepare phase を通るようになり、effect boundary と owner/cell diagnostics が CLI check でも見えるようになった。drop/effect review の観点では fixed だが、artifact emission へ入らず drop insertion bridge まで検査する現在の責務分離を維持する必要がある。

## 次に確認すること

- stdlib `core/mem` / collection / string の effect boundary が public API と internal raw API に分離されているか。
- selfhost ResourceIR/drop/effect 設計が Rust の old HIR drop logic ではなく ResourceIR plan を前提にしているか。

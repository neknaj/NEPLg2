# HIR / monomorphize / layout review

対象 commit: `f108cebd`

## 概要

HIR、monomorphize、layout は typecheck と backend の間をつなぐ。`monomorphize.rs` は約 1300 行で、trait call resolution、function instance、lowered signature と backend 入力に関わる。

## 現状

- `hir.rs` は比較的小さく、HIR expression kind と match arm / function metadata を保持する。
- `monomorphize.rs` は generic instance 化と unresolved trait call の検出を担う。
- `layout.rs` は backend 前の type layout 計算を担う。
- `compiler.rs` は monomorphize 後に unresolved trait call を backend diagnostic として処理する。

## 良い点

- unresolved trait call が backend 到達前に diagnostic になる。
- HIR は Resource IR lowering の入力として使われ、static check input の coverage を比較できる。
- char / match / function value などの typecheck result が HIR に残り、backend と Resource IR に渡る。

## 残る問題

- drop insertion が monomorphize 前の HIR に対して実行されるため、checked Resource IR と drop elaboration の責務が分かれている。
- monomorphize と backend signature lowering の境界は selfhost で再設計が必要である。
- `monomorphize.rs` は大きく、selfhost S4/S5 では instance cache、name mangling、trait call resolution を分けるべきである。

## selfhost への示唆

selfhost では `hir/`, `resource/`, `mono/`, `codegen/` を明確に分ける。Resource IR check と drop elaboration が終わった checked IR だけを monomorphize/codegen へ渡す設計に寄せる。

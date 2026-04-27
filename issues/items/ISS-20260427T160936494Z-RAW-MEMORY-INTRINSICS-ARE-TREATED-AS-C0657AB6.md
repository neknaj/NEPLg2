---
id: ISS-20260427T160936494Z-RAW-MEMORY-INTRINSICS-ARE-TREATED-AS-C0657AB6
title: "raw memory intrinsics are treated as pure effects"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-27
updated: 2026-04-28
target: "nepl-core/src/effects.rs, nepl-core/src/typecheck.rs, tests/compiler/move_effect.n.md"
---

# ISS-20260427T160936494Z-RAW-MEMORY-INTRINSICS-ARE-TREATED-AS-C0657AB6: raw memory intrinsics are treated as pure effects

## 概要

`#intrinsic "load"` と `#intrinsic "store"` が `intrinsic_effect` 上は `Pure` として扱われており、user source が `core/mem` の audit boundary を通さずに raw memory を直接読み書きできる。

## 対象

- `nepl-core/src/effects.rs, nepl-core/src/typecheck.rs, tests/compiler/move_effect.n.md`

## 根拠

- `nepl-core/src/effects.rs` の `intrinsic_effect` は WASI 系 marker だけを `Impure` とし、それ以外を `Pure` としていた。
- `nepl-core/src/typecheck.rs` の `PrefixItem::Intrinsic` effect check は `intrinsic_effect` の結果だけを見るため、`#intrinsic "load"` / `#intrinsic "store"` は pure 関数内でも拒否されなかった。
- `stdlib/core/mem.nepl` の generic `load<T>` / `store<T>` は移行中の compiler-owned memory boundary として raw intrinsic を使うため、全面禁止ではなく source boundary を明示する必要がある。

## 問題

pure user function から raw memory intrinsic を直接呼べるため、raw body memory instruction の検査を強化しても、別の surface escape hatch から同じ raw memory effect を作れてしまう。

## 影響

Pure functions can observe or mutate raw memory through intrinsic load/store, bypassing the effect checks added for raw bodies and weakening memory safety assumptions for borrow/move/drop analysis.

## 修正方針

Classify raw memory intrinsics as memory effects and reject them in pure user contexts. Keep the transitional stdlib/core/mem boundary explicit so existing memory primitives can be audited separately.

## 検証

Add compile_fail tests for pure #intrinsic load/store and keep stdlib core/mem wrapper behavior covered by existing compiler tests.

## 対応結果

- `nepl-core/src/effects.rs` に raw memory intrinsic effect marker を追加し、`load` / `store` を impure effect として分類した。
- `nepl-core/src/typecheck.rs` では pure context の raw memory intrinsic を `D3025` で拒否するようにした。
- 移行中の `stdlib/core/mem.nepl` だけは SourceMap path に基づく compiler-owned memory boundary として許可し、既存の audited wrapper を維持した。
- `tests/compiler/move_effect.n.md` と `nepl-core/tests/effects.rs` に direct `#intrinsic "load"` / `#intrinsic "store"` の compile_fail 回帰テストを追加した。

## 実施した検証

- `cargo fmt --check`: pass
- `cargo test -p nepl-core --test effects`: `12 passed`
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/raw-memory-intrinsic-effect.json -j 1`: `total=35`, `passed=35`

# Rust コンパイラ module / resolve / loader レビュー

確認対象 commit: `3742a1a7 fix(cli): run Resource IR gates for check-only`

## 確認範囲

- `nepl-core/src/loader.rs`
- `nepl-core/src/module_graph.rs`
- `nepl-core/src/resolve.rs`
- `nepl-core/src/source_map.rs`
- `nepl-core/tests/{import_clause,loader_cycle,resolve,debug_loader}.rs`
- `tests/compiler/{resolve,loader_cycle,debug_loader}.n.md`
- stdlib raw-memory-boundary source policy tests that inspect `loader.rs`

## 進捗状況

| 領域 | 状態 | 判定 |
|---|---|---|
| loader | `Loader` が SourceMap を作り、import/include を file id 付き flat AST へまとめる。 | 現行 pipeline として機能。host FS 依存は core safety pass から切り離されている。 |
| source map | raw memory boundary capability と source path/file id を保持する。 | ResourceIR/effect gate の移行に重要。 |
| raw-memory boundary | exact configured stdlib path table で capability を付与する。 | 移行措置として妥当。public facade へ広げない source policy がある。 |
| module graph | host-side ModuleGraph / export table がある。 | selfhost split pipeline の参考になるが main pipeline はまだ flat loader 中心。 |
| resolve | `ImportResolution` が qualified / unqualified visibility を扱う。 | 進捗あり。DefId を HIR snapshot として完全保持する段階は別 issue で継続。 |

## 良い点

- `SourceMap` が `loader.rs` から分離済みで、typecheck/ResourceIR は host filesystem path へ直接依存しない。
- raw-memory boundary は exact path table と source policy によって、root facade ではなく直接 raw intrinsic を持つ実装 module へ限定されている。
- `target_gate` / `target_precheck` と loader/source map が分かれており、条件付き compile の評価規則を loader に埋め込んでいない。
- selective import / facade re-export 周辺は recent issue で修正され、`resolve.rs` に import visibility helper が集約されている。

## 問題

### raw-memory-boundary table は最終設計ではない

exact path table は現行 stdlib 移行を安全側に保つ実用的な防壁だが、最終的な memory safety contract は path table ではなく stdlib API と ResourceIR の型付き capability で表すべきである。stdlib split のたびに loader table を更新する運用は、漏れた場合に `effect.pure.calls_impure` や owner/cell 診断のずれを起こす。

### main pipeline はまだ flat loader 表現

`ModuleGraph` と `resolve.rs` は進んでいるが、main compiler は file/module boundary を HIR の stable DefId として十分に保持しているわけではない。selfhost の module resolver では、この flat-loader 依存をそのまま移植しない方針が必要である。

## issue 連携

- `ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84`
- `ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF`
- `ISS-20260427T011048496Z-HIR-FUNCTION-REFERENCES-STILL-LACK-S-5EF51F12`

## 次に確認すること

- stdlib review で `RAW_MEMORY_BOUNDARY_STDLIB_PATHS` の exactness と実際の raw intrinsic 所有 module を照合する。
- selfhost module graph / loader が Rust flat loader の制約を引き継いでいないか確認する。

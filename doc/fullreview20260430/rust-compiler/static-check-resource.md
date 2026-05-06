# Static check / Resource IR review

対象 commit: `f108cebd`

## 概要

Resource IR は現行 compiler の最重要進捗である。`nepl-core/src/resource/` は 50 file 以上に分割され、lowering、coverage、cell、owner、borrow、effect、summary、alias、variant condition を扱う。

## 現状

Resource IR gate は compiler pipeline の authoritative static-check gate として実行される。

- lowering coverage: HIR と Resource IR の static-check input 欠落を hard error にする。
- initialized/raw cell: raw memory cell state の uninit / moved / dropped / conflict を検査する。
- borrow lifetime: return escape と borrow conflict を検査する。
- effect boundary: raw identity escape、impure call、unsafe memory operation、host effect in pure context を検査する。
- owner obligation: leak / maybe leak / double free / reserved / no free obligation を検査する。

直近 main では、typed indirect call effect、fallible owner effects、Result::Ok-gated owner consumption、checked MemPtr load variant refinement、owner variant value conditions が追加された。

## 良い点

- `resource/mod.rs` は public API を整理し、monolithic `check.rs` の再導入を source policy で禁止している。
- `nodesrc/test_resource_checker_responsibility.js` は file existence と line count limit により責務再集中を検出する。対象 Actions run では aggregate の source policy step は成功しているため、review 上の CI status と local 直接確認の結果は分けて扱う。
- `condition_fact.rs` などにより、variant/value condition を owner summary に渡す方向が始まっている。
- `EffectOp::IndirectCall { effect }` により、indirect call を unknown effect として落とす設計から前進している。
- Resource diagnostic は `Move` / `Borrow` / `Cell` / `Owner` / `Raw` / `Lower` に分かれている。

## 残る問題

- 旧 `passes::move_check::run` fallback は 2026-05-06 に削除済みである。
- Resource IR check は HIR `passes::insert_drops` より前に実行される。ただし、drop elaboration 自体はまだ HIR pass に残っている。
- `UnsafeMemoryInPureFunction` は 2026-05-06 時点で compiler error gate へ接続済みである。ただし `stdlib/core/mem.nepl` など raw-memory-boundary capability を持つ移行中 source は Stage 6 完了まで限定許可される。
- `MemPtr` / `RegionToken` が compiler-issued owner/provenance capability ではないため、owner checker は複雑な alias/variant condition を増やし続ける圧力がある。
- `tests/stdlib/memory_safety.n.md` の残失敗は、stdlib cleanup ではなく owner token / non-owning pointer 分離が必要な問題として残っている。
- `owner_summary_variant_paths.rs` の責務再集中は `ISS-20260430T135243330Z-RESOURCE-OWNER-VARIANT-PATH-BUILDER--87B356A8` で解消済みである。現在は path traversal orchestration、condition refinement、construct/match handling が分割され、source policy も通過している。

## 設計評価

Resource IR の方向は正しい。型安全・メモリ安全を必達にするなら、HIR direct traversal ではなく Resource IR で resource operation を明示化し、enum state と exhaustive match で検査すべきである。

ただし現状は「二重防壁」であり、最終設計ではない。これを完了と誤認すると、selfhost が旧 checker と Resource IR の両方をコピーする危険がある。

## 2026-05-06 追補

この review の対象 commit 時点では、`UnsafeMemoryInPureFunction` は shadow-only として扱われていた。その後の main では、Stage 5 の effect boundary が次の状態まで進んでいる。

- `RawMemoryOp`、`ExternalIoOp`、`NondetOp` は typed enum として Resource IR `EffectOp` と diagnostic に保持される。
- raw memory / host effect の operation-level count は専用 module に分離され、exhaustive `match` により operation 追加時の更新漏れを検出できる。
- direct host effect、nondet effect、unsafe memory operation は pure function 境界で Resource IR diagnostic から compiler error へ変換される。
- `UnsafeMemoryInPureFunction` は `effect.pure.calls_impure` として error 化済みであり、旧 HIR typecheck gate だけに依存しない。

残る未完了点は「unsafe memory gate が shadow-only かどうか」ではなく、次の点である。

- 旧 `passes::move_check::run` fallback は削除済みであり、Resource IR check は HIR drop insertion より前に実行される。ただし HIR `passes::insert_drops` 自体がまだ drop elaboration authority として残る。
- raw-memory-boundary capability が stdlib/core/mem 移行のために残っており、safe public API と internal raw implementation の Stage 6 分離が未完了である。
- `MemPtr = non-owning pointer`、`OwnedRegion/Storage = free obligation owner`、`InitializedCell/Resource IR = initialized/moved/drop state` の最終分離は完了していない。
- owner variant path builder の責務分割は完了済みであり、現在の blocker からは外す。

したがって、今後の優先順位は UnsafeMemory gate の再実装ではなく、HIR drop insertion の Resource IR drop elaboration への置換、owner/provenance capability、stdlib raw-memory-backed API の境界移行である。

## 2026-05-06 pre-drop gate 追補

`ISS-20260425T000000Z-RV-CORE-009-58589A3F` の Stage 4 進捗として、Resource IR check の入力を「drop 未挿入 source semantics を保持したまま monomorphize した reachable HIR」へ変更した。

typecheck 直後の未単相化 HIR 全体を Resource IR へ直接下げると、`#target std` で未使用 stdlib source まで検査対象になり、Resource IR の責務境界を越えて timeout する。一方で HIR を clone して二経路化すると deep prefix tree の再帰 clone が native stack overflow を起こす。そのため現時点の pipeline は、Resource IR 用 HIR と codegen 用 HIR を typecheck の再実行で分離する。

これは最終形ではない。最終的には HIR `passes::insert_drops` を Resource IR drop elaboration へ置き換え、Resource IR check と drop elaboration を同じ Resource IR 上で連続させる。その時点で二重 typecheck 経路は削除する。

## 2026-05-06 EndScope auto-drop 追補

Resource IR initialized/cell checker は `EndScope` で live non-Copy local を auto-drop state transition として扱うようになった。これにより、source Resource IR check は HIR `passes::insert_drops` が生成した `drop` 式に依存せず、scope exit の drop obligation を Resource IR 上で検査できる。

同名・同型 shadowing では inner scope の auto-drop が outer local の `CellState` を壊すため、Resource IR lowering は有効範囲内の同名 local を内部 place として固有化する。通常 local の表示名は維持し、shadowed local だけ `x#N` 形式の Resource IR local name を使う。

残る blocker は codegen 側である。現在も wasm 生成前の実 drop call 挿入は HIR `passes::insert_drops` に残っているため、次は Resource IR drop elaboration の結果から HIR/Wasm の drop 呼び出しを生成する構造へ移す必要がある。

## 次の確認対象

- `ISS-20260425T000000Z-RV-CORE-009-58589A3F`: Resource IR final authority。
- `ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF`: owner/provenance capability。
- `ISS-20260427T132406497Z-CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS-DC67DF04`: raw memory operation boundary。
- `ISS-20260429T040748194Z-RUST-COMPILER-DIAGNOSTICS-ARE-NOT-AL-1617747D`: diagnostic parity。
- `ISS-20260430T135243330Z-RESOURCE-OWNER-VARIANT-PATH-BUILDER--87B356A8`: owner variant path builder の責務分割。

## selfhost への示唆

selfhost Resource checker は旧 HIR move checker をコピーしない。Rust 側の Resource IR model、diagnostic enum、coverage gate、state merge、variant/value condition を参考に、最初から Resource IR を正規 checker として設計する。

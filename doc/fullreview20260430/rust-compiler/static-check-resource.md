# Static check / Resource IR review

対象 commit: `f108cebd`

## 概要

Resource IR は現行 compiler の最重要進捗である。`nepl-core/src/resource/` は 50 file 以上に分割され、lowering、coverage、cell、owner、borrow、effect、summary、alias、variant condition を扱う。

## 現状

Resource IR gate は `run_move_check` から呼ばれる。

- lowering coverage: HIR と Resource IR の static-check input 欠落を hard error にする。
- initialized/raw cell: raw memory cell state の uninit / moved / dropped / conflict を検査する。
- borrow lifetime: return escape と borrow conflict を検査する。
- effect boundary: raw identity escape と impure call in pure context を検査する。
- owner obligation: leak / maybe leak / double free / reserved / no free obligation を検査する。

直近 main では、typed indirect call effect、fallible owner effects、Result::Ok-gated owner consumption、checked MemPtr load variant refinement、owner variant value conditions が追加された。

## 良い点

- `resource/mod.rs` は public API を整理し、monolithic `check.rs` の再導入を source policy で禁止している。
- `nodesrc/test_resource_checker_responsibility.js` は file existence と line count limit により責務再集中を検出する。対象 Actions run では aggregate の source policy step は成功しているため、review 上の CI status と local 直接確認の結果は分けて扱う。
- `condition_fact.rs` などにより、variant/value condition を owner summary に渡す方向が始まっている。
- `EffectOp::IndirectCall { effect }` により、indirect call を unknown effect として落とす設計から前進している。
- Resource diagnostic は `Move` / `Borrow` / `Cell` / `Owner` / `Raw` / `Lower` に分かれている。

## 残る問題

- 旧 `passes::move_check::run` が Resource IR gate より先に authoritative として残る。
- `passes::insert_drops` は Resource IR check 前に HIR 上で drop を入れる。
- `UnsafeMemoryInPureFunction` は shadow-only のままである。
- `MemPtr` / `RegionToken` が compiler-issued owner/provenance capability ではないため、owner checker は複雑な alias/variant condition を増やし続ける圧力がある。
- `tests/stdlib/memory_safety.n.md` の残失敗は、stdlib cleanup ではなく owner token / non-owning pointer 分離が必要な問題として残っている。
- `owner_summary_variant_paths.rs` は 637 行規模であり、Result owner variant path enumeration、condition propagation、call effect reservation、returned owner path collection が集中している。対象 Actions run の `Source policy regressions` step は成功しているが、local 直接確認では responsibility split policy が赤くなる。この review では Actions を test 状況の根拠としつつ、責務再集中そのものは `ISS-20260430T135243330Z-RESOURCE-OWNER-VARIANT-PATH-BUILDER--87B356A8` として追跡する。

## 設計評価

Resource IR の方向は正しい。型安全・メモリ安全を必達にするなら、HIR direct traversal ではなく Resource IR で resource operation を明示化し、enum state と exhaustive match で検査すべきである。

ただし現状は「二重防壁」であり、最終設計ではない。これを完了と誤認すると、selfhost が旧 checker と Resource IR の両方をコピーする危険がある。

## 次の確認対象

- `ISS-20260425T000000Z-RV-CORE-009-58589A3F`: Resource IR final authority。
- `ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF`: owner/provenance capability。
- `ISS-20260427T132406497Z-CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS-DC67DF04`: raw memory operation boundary。
- `ISS-20260429T040748194Z-RUST-COMPILER-DIAGNOSTICS-ARE-NOT-AL-1617747D`: diagnostic parity。
- `ISS-20260430T135243330Z-RESOURCE-OWNER-VARIANT-PATH-BUILDER--87B356A8`: owner variant path builder の責務分割。

## selfhost への示唆

selfhost Resource checker は旧 HIR move checker をコピーしない。Rust 側の Resource IR model、diagnostic enum、coverage gate、state merge、variant/value condition を参考に、最初から Resource IR を正規 checker として設計する。

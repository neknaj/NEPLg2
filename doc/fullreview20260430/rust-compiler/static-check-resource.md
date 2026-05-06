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

## 2026-05-06 drop plan 追補

EndScope auto-drop は `ResourceDropPlan` として明示データ化された。`compute_resource_drop_plan` は Resource IR の nested control-flow を含めて non-Copy scope local の auto-drop 候補を列挙し、initialized/cell checker も同じ候補列挙を使う。

これにより、次の codegen 移行で checker と codegen が別々に drop 対象を推定する危険は下がった。残る作業は、この plan を HIR/Wasm drop call 生成へ接続し、旧 HIR `passes::insert_drops` を削除することである。

## 2026-05-06 drop requirement classification 追補

`ResourceDropPlan` の各 auto-drop 候補は `ResourceDropRequirement` を持つようになった。分類は `StateOnly`、`WholeValue`、`DynamicEnumPayload`、`Structural` であり、checker / codegen 境界で direct Drop impl、structural field Drop、runtime tag 依存 enum payload Drop を enum-first に扱える。

これは Resource IR drop elaboration への移行で必要な中間段階である。候補列挙だけでは codegen が旧 HIR `passes::insert_drops` 相当の型走査を別途持つことになり、checker と codegen の drop 対象推定が分岐する。分類済み plan により、次はこの requirement を消費して実 drop call を生成し、HIR drop insertion を削除する段階へ進める。

## 2026-05-06 insert_drops requirement consumer 追補

HIR `passes::insert_drops` はまだ残るが、内部の drop-needed 判定は `ResourceDropRequirement` を消費する形へ寄せた。旧 `structural_drop_fields`、`structural_enum_field_drop_lines`、`type_needs_structural_drop` は削除され、direct Drop / structural field Drop / dynamic enum payload Drop は `match ResourceDropRequirement` で生成される。

この状態は最終形ではないが、checker と codegen が別々に type graph を走査して drop 対象を推測する技術的負債は減った。次に確認すべき点は、HIR pass の scope walker が保持する local move state を Resource IR `CellState` / drop plan へ置き換えられるかである。

## 2026-05-06 drop point grouping 追補

`ResourceDropFunctionPlan` は `drop_points` を持つようになった。`ResourceDropPoint` は EndScope span と、その scope end で処理する auto-drop 候補群を保持する。既存の `auto_drops` は `drop_points` から flatten した view として残している。

flat list だけでは、codegen 側が nested control-flow のどの EndScope に drop を挿入するかを再推定する必要がある。drop point grouping により、次の移行では HIR scope walker の位置推定ではなく Resource IR lowering が生成した EndScope を正にできる。

## 2026-05-06 drop point typed path 追補

`ResourceDropPoint` は `ResourceDropPointPath` を持つようになった。path は block id と `ResourceDropPointStep` enum の列であり、`Op`、`BranchThen`、`BranchElse`、`LoopCondition`、`LoopBody`、`MatchArm` を区別する。

span は診断表示には必要だが、drop elaboration の挿入位置としては不十分である。同一 span の nested lowering や複数 EndScope を扱うため、Resource IR 構造上の位置を typed data として保持する。

## 2026-05-06 drop point resolver 追補

`ResourceDropPointPath` は `resolve_resource_drop_point_path` / `resolve_resource_drop_point_end_scope` で実 Resource IR op へ解決できるようになった。resolver は `ResourceDropPointResolutionError` enum を返し、block 不在、op index 範囲外、container step と実 op の不一致、match arm 範囲外、EndScope 以外の選択を区別する。

これにより、drop point path は span 補助情報ではなく、codegen が消費前に検証できる typed insertion anchor になった。残る作業は、この resolver の EndScope 結果を実 drop call 生成へ渡し、HIR `passes::insert_drops` の scope traversal を削除することである。

## 2026-05-06 live drop facts 追補

`ResourceFunctionCheck` は `auto_drop_points` を持つようになった。これは `ResourceDropPlan` の型ベース候補ではなく、initialized-state traversal が EndScope 到達時点で実際に `Initialized` と判定して `Dropped` へ遷移させた live drop fact である。

また Resource IR lowering は non-Copy function parameter の EndScope anchor を terminator return 前に生成する。これにより HIR `insert_drops` が outer scope で扱っていた parameter drop obligation も Resource IR 上で追跡できる。次は candidate plan ではなくこの checked live fact を実 drop call 生成へ接続する必要がある。

## 2026-05-06 drop elaboration plan gate 追補

checked live fact を codegen 境界へ渡す `ResourceDropElaborationPlan` を追加した。入力は `ResourceFunctionCheck::auto_drop_points` に限定し、`ResourceDropPlan` の candidate は使わない。

plan 構築時には、Resource IR function と check report の対応、`ResourceDropPointPath` が実際に EndScope へ解決できること、各 auto-drop place がその EndScope locals に含まれることを検証する。失敗は `ResourceDropElaborationPlanError` enum で分類し、compiler pipeline では Resource IR cell gate 直後に `resource.lower.incomplete` の hard error へ写像する。

この変更で実 drop call 生成そのものはまだ HIR `passes::insert_drops` に残る。ただし次の置換作業では、candidate plan や HIR scope traversal ではなく、この checked live drop elaboration plan を唯一の入力として消費できる。

## 2026-05-06 drop elaboration source binding 追補

drop elaboration plan は checked `Place` だけでなく、backend/HIR が参照する source binding 名も保持するようになった。Resource IR lowering は `DeclareLocal` に `source_name` を記録し、shadowing で内部 place を `x#...` に固有化しても、実 drop call 生成時に参照すべき source 名 `x` を失わない。

`ResourceDropElaborationDrop` は place、source_name、drop requirement、span を持つ。binding metadata は parameter、`DeclareLocal`、match arm binding から収集し、見つからない場合は `MissingDropBinding` enum error で hard gate する。これにより、次の HIR `passes::insert_drops` 置換で source 名復元のために HIR scope walker へ戻る必要をなくす。

## 2026-05-06 drop elaboration function origin 追補

Resource IR drop elaboration plan は function 単位で `origin_name` を持つようになった。`HirFunction` は typecheck で source-level name を `origin_name` として保持し、monomorphize は specialized `name` だけを変更して origin を維持する。Resource IR lowering と drop elaboration plan はこの metadata をそのまま伝搬する。

これは HIR `passes::insert_drops` 削除へ向けた codegen 境界の補強である。monomorphized function name を parsing して source HIR function を推測する設計は、generic specialization と overload/mangle に依存する技術的負債になる。今後の drop call 生成は `name`、`origin_name`、source binding、checked drop point path を合わせて使い、Resource IR の checked live fact から source/backend の挿入位置を決める。

## 2026-05-06 prepared drop elaboration plan 追補

`run_resource_static_check` は checked `ResourceDropElaborationPlan` を返すようになり、`PreparedProgram` は `resource_drop_elaboration_plan` を保持する。これにより checked live drop facts は compiler gate の一時値ではなく、codegen bridge が消費できる pipeline artifact になった。

まだ HIR `passes::insert_drops` は残っているが、次の置換作業は Resource IR plan を再計算せずに `PreparedProgram` から受け取れる。残る blocker は、この prepared plan を実 drop call 生成へ渡し、HIR scope walker の drop 対象推定を削除することである。

## 2026-05-06 HIR bridge gate 追補

checked `ResourceDropElaborationPlan` が source HIR へ戻せることを compiler gate で検証するようになった。bridge validator は HIR の function parameter、block-local `let`、match arm binding を scope span ごとに収集し、plan の `origin_name` / `source_name` / span と照合する。

この gate は実 drop call 挿入そのものではない。ただし、bridge 不可能な plan を `ResourceDropElaborationHirBridgeError` enum で早期に止めるため、次の置換作業で HIR scope walker や文字列 fallback を復活させる必要がなくなる。残る blocker は bridge 済み plan を drop call 生成へ渡すことである。

## 2026-05-06 ResourceDropElaborationPlan consumer 追補

実 drop call 生成は `passes::insert_resource_drops` が checked `ResourceDropElaborationPlan` を消費する形へ移った。旧 `passes::insert_drops`、`VarState`、`var_stacks` による HIR scope walker は削除済みである。

新しい consumer は `ResourceAutoDropKind::ScopeLocal` と `AssignmentOverwrite` を enum で分け、`ResourceDropRequirement` の `StateOnly` / `WholeValue` / `DynamicEnumPayload` / `Structural` を exhaustive match して HIR drop call を生成する。これにより、Resource IR initialized-state traversal が記録した live drop fact と、codegen が挿入する実 drop call の authority が一致した。

compiler pipeline も、Resource IR check 用 HIR と legacy drop insertion 用 HIR を二重 typecheck する構造をやめた。drop 未挿入の reachable monomorphized HIR を Resource IR check し、同じ HIR に checked plan から drop call を挿入し、最後に monomorphize を再実行して生成 Drop trait call を concrete user call へ解決する。

この後挿入では、最初の monomorphize 時点で未到達だった Drop impl method body が output functions から落ちる危険がある。そのため `monomorphize_internal` は `HirModule.impls` に保持されている impl method function を function table へ再登録する。これにより final monomorphize が生成 Drop call を解決したとき、call target body も final HIR に残り、wasm codegen の unknown function にならない。今後の確認対象は、この新 authority の partial field move / nested control-flow / LLVM parity を full review で固定することである。

## 次の確認対象

- `ISS-20260425T000000Z-RV-CORE-009-58589A3F`: Resource IR final authority。
- `ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF`: owner/provenance capability。
- `ISS-20260427T132406497Z-CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS-DC67DF04`: raw memory operation boundary。
- `ISS-20260429T040748194Z-RUST-COMPILER-DIAGNOSTICS-ARE-NOT-AL-1617747D`: diagnostic parity。
- `ISS-20260430T135243330Z-RESOURCE-OWNER-VARIANT-PATH-BUILDER--87B356A8`: owner variant path builder の責務分割。

## selfhost への示唆

selfhost Resource checker は旧 HIR move checker をコピーしない。Rust 側の Resource IR model、diagnostic enum、coverage gate、state merge、variant/value condition を参考に、最初から Resource IR を正規 checker として設計する。

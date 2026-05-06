# NEPLg2 静的検査設計確認 2026-04-30

## 目的

NEPLg2 の静的検査が、型安全・メモリ安全を「実装上も設計上も必達」にできる構造になっているかを確認する。

この確認は、既存の検査を弱めるためではない。現行 Rust compiler、self-host 計画、stdlib memory model の間で設計がずれている箇所を明示し、Resource IR / enum / `match` / owner token による検査へ収束させるための監査である。

## 確認対象

- `doc/neplg2/static_check_complexity_reduction_plan.md`
- `doc/neplg2/compiler_diagnostics_redesign_plan.md`
- `doc/neplg2/self_host_plan.md`
- `doc/neplg2/stdlib_collection_mem_string_static_safety_design.md`
- `nepl-core/src/resource/**`
- `nepl-core/src/compiler.rs`
- `nepl-core/src/diagnostic_codes.rs`
- `stdlib/core/mem.nepl`
- `stdlib/alloc/string.nepl`
- `stdlib/alloc/io.nepl`
- `stdlib/alloc/collections/**`
- `stdlib/neplg2/core/infra/diag.nepl`

## 進捗状況

| 領域 | 現状 | 判定 | 次の必須作業 |
|---|---|---|---|
| Rust typecheck | module 分割と typed diagnostic 化が進み、`match` 検査や trait capability は既存 pass で動く。 | 方向は正しい。 | type / effect / resource の責務境界をさらに固定し、Resource IR に渡す入力を欠落させない。 |
| Resource IR model | `ResourceId`、`StorageId`、`Place`、`CellState`、`OwnerState`、`BorrowState`、`StorageOrigin`、`EffectOp` が存在する。 | 中核 model は妥当。 | owner/cell/drop/effect を最終 authority にする前に、旧 HIR checker との二重状態を解消する。 |
| Resource IR lowering gate | HIR と Resource IR の coverage comparison が compiler gate になっている。 | 必須の安全網として妥当。 | coverage 数だけでなく、semantic place / owner / borrow summary の欠落を継続して regression 化する。 |
| CellState gate | raw load/store/dealloc/realloc/fill/bulk の cell violation が `resource.cell.*` として compiler error 化されている。 | 良い。 | raw bucket へ戻さず、self-host 側でも同じ cell diagnostic taxonomy を持つ。 |
| OwnerState gate | owner leak / maybe leak / unavailable が `resource.owner.*` として compiler error 化されている。 | 良いが未完。 | owner diagnostic 分離は維持し、`MemPtr` owner 兼用の stdlib 設計を解消する。 |
| BorrowState gate | return escape と通常 borrow conflict が compiler error 化されている。 | 良い。 | self-host 側も同じ borrow diagnostic taxonomy を持つ。 |
| Effect gate | raw identity escape は `resource.raw.identity_escape` へ分離済み。`UnsafeMemoryInPureFunction` は 2026-05-06 時点で `effect.pure.calls_impure` へ error 化済み。 | 改善済みだが migration allowance は未完。 | raw-memory-boundary capability は Stage 6 の stdlib migration 完了まで限定許可とし、最終状態では internal unsafe boundary と public safe API を分離する。 |
| drop elaboration | `passes::insert_drops` が Resource IR check より前に HIR 上で動く。 | 設計未完。 | drop は Resource IR 上の checked state から挿入する。旧 `drop_insertion` は移行対象。 |
| Diagnostic code | Rust core は `Move` / `Borrow` / `Cell` / `Owner` / `Raw` / `Lower` の階層 enum と stable string 境界へ移行済み。 | 方向は正しい。 | `Cell` / `Owner` 分離を維持し、self-host も同じ category contract へ揃える。 |
| self-host diagnostic | 現在の S1/S2 用 code は enum 化済み。 | 現段階では妥当。 | S3 以降で Type / Effect / Resource / Backend category を Rust と同じ contract で追加する。 |
| stdlib mem/string/io | ByteBuf / builders は `Option<MemPtr<u8>>` 化で空/所有 storage を型に出した。 | 過渡として妥当。 | `OwnedRegion` / `OwnedBytes` / compiler-issued token へ移行する。 |
| stdlib collections | `HashMap` / `HashSet` は enum bucket state、Queue / Deque / RingBuffer / Stack / BinaryHeap / BTreeMap / BTreeSet は typed slot storage へ移行済み。`Vec`、raw byte/numeric collection、List node chain は raw storage owner が残る。 | 改善中だが self-host 中核にはまだ制限が必要。 | `OwnedBuffer`、`StorageState`、owner-preserving fallible result、collection-wide drop traversal へ再設計する。 |

## 確認結果

現行設計の方向性は正しい。Resource IR の data model、typed diagnostic、coverage gate、cell / owner / borrow gate は、型安全・メモリ安全を compiler が検査する方向へ進んでいる。

ただし、現在の実装を「最終設計として適切」とは判定しない。理由は次の通りである。

1. `passes::move_check::run` がまだ先に authoritative で走り、その後に Resource IR gate が追加で走る。
2. `passes::insert_drops` が Resource IR check より前に HIR 上で drop を挿入している。
3. Resource IR の `CellState` / `OwnerState` diagnostic は `resource.cell.*` / `resource.owner.*` へ分離されたが、旧 `passes::move_check::run` との二重 authority はまだ残っている。
4. raw-memory-boundary capability が file/source 単位の移行中許可として残っている。
5. `MemPtr<T>` / `RegionToken<T>` が stdlib struct として forge 可能で、owner token と non-owning pointer の分離が完了していない。
6. collections が safety-critical state を enum ではなく null pointer / numeric status / raw header layout で表している。

これらは表面的な bug ではなく、静的検査の authority をどこに置くかという設計問題である。したがって、今後も旧 checker の special-case を増やす方向は不可とする。

## 正しい最終設計

### pass 順序

最終的な検査順序は次にする。

| 順序 | pass | 責務 |
|---:|---|---|
| 1 | resolve / type inference | name、scope、型、trait capability、match exhaustiveness を確定する。 |
| 2 | effect inference | surface effect と internal effect を分離し、fold 可否を決める。 |
| 3 | Resource IR lowering | HIR の resource-relevant な情報を `Place` / `EffectOp` / `RawMemoryOp` へ落とす。 |
| 4 | lowering coverage gate | Resource IR が検査入力を失っていないことを compiler error として保証する。 |
| 5 | resource check | cell、owner、borrow、raw provenance、effect boundary を同じ Resource IR state で検査する。 |
| 6 | drop elaboration | checked Resource IR 上で auto drop を挿入する。 |
| 7 | backend lowering | checked / drop-elaborated IR から WASM / LLVM へ下げる。 |

`drop_insertion` と旧 HIR `move_check` は、この順序へ移行するまでの防壁であり、最終設計ではない。

### Resource state

Resource IR の state は safety-critical enum として保持する。

- `CellState`: `Uninit` / `Initialized(T)` / `Moved` / `Dropped` / `MaybeMoved`
- `OwnerState`: `NoFreeObligation` / `Live(StorageId)` / `Moved` / `Freed` / `MaybeFreed`
- `BorrowState`: `Unborrowed` / `Shared` / `Unique` / `Released`
- `StorageOrigin`: `Owned` / `Unmanaged` / `Internal`
- `EffectOp`: `Pure` / `InternalAlloc` / `UnsafeMemory` / `ExternalIo` / `Nondet` / `Unknown`

分岐は wildcard で握り潰さず、意味のある `match` arm を明示する。新しい state を追加したときに Rust / self-host の exhaustive match が壊れる構造を保つ。

### diagnostic taxonomy

現行 Rust compiler の `ResourceDiagnosticCode` は `Move` / `Borrow` / `Cell` / `Owner` / `Raw` / `Lower` に分かれている。`ResourceCheckDiagnostic::CellUnavailable` と `ResourceOwnerDiagnostic::*` は、compiler boundary で `resource.cell.*` / `resource.owner.*` へ写像されており、旧 D3100 相当の raw ownership bucket へ戻してはならない。

現在維持すべき taxonomy は少なくとも次である。

- `resource.cell.uninit`
- `resource.cell.moved`
- `resource.cell.dropped`
- `resource.cell.possibly_moved`
- `resource.owner.no_free_obligation`
- `resource.owner.leaked`
- `resource.owner.maybe_leaked`
- `resource.owner.double_free`
- `resource.raw.identity_escape`
- `resource.raw.provenance_unknown`
- `resource.borrow.return_escape`
- `resource.lower.incomplete`

raw memory 上で cell / owner violation が起きた場合も、原因が cell state なのか owner obligation なのかは失わない。`resource.raw.*` は raw identity escape、unsafe boundary、provenance そのものの問題に限定する。

残件は、Rust 側で `Cell` / `Owner` category を追加することではない。残件は、旧 HIR `move_check` と HIR `insert_drops` を Resource IR の checked state へ統合し、self-host 側の diagnostic enum / stable string も同じ taxonomy へ揃えることである。

### memory model

`MemPtr<T>` は non-owning pointer projection として固定する。free obligation は持たせない。

所有権は compiler-issued token で表す。

- `OwnedRegion<T>`: allocator が発行した region owner。
- `OwnedBytes`: byte buffer owner。
- `OwnedBuffer<T>`: initialized prefix と capacity を持つ collection storage owner。
- `StorageState<T>`: `Empty` / `Allocated(OwnedRegion<T>)`。
- `BucketState<K,V>`: `Empty` / `Full(Bucket<K,V>)` / `Tombstone`。

`RegionToken<T>` のような token は、stdlib code から任意に作れる struct のままでは compiler capability として不十分である。`region_new(MemPtr<T>, i32)` で forge できる経路は廃止対象にする。

### collection API

non-Copy payload を扱う collection は、API が owner discipline を表す必要がある。

| 操作 | API 方針 |
|---|---|
| read | `T: Copy` の copy read と borrow read を分ける。 |
| move-out | collection owner と item owner を同時に返す result enum にする。 |
| replace | 旧 item を返すか drop 済みであることを型で表す。 |
| free | initialized prefix を drop/consume してから storage-only free する。 |
| fallible update | `Err` で collection と item owner を返す。 |

`Result<Vec<T>, E>` だけでは失敗時に owner がどうなるか表現できない。self-host の typecheck / Resource IR / diagnostic 集約では、owner を失う fallible collection API を標準にしない。

## self-host 開始可否

S1/S2 の lexer / parser / module loader は開始可能である。ただし、次の制限を置く。

- token id、span id、small enum など Copy payload だけに既存 `Vec` を使う。
- diagnostic は現在の `SelfhostDiagnosticCode` enum 方針を維持し、自由文字列 code に戻さない。
- typecheck / Resource IR / symbol table の中核には raw header collection や `MemPtr` owner field を持ち込まない。
- S3 以降では Type / Effect / Resource / Backend diagnostic category を Rust 側と同じ taxonomy で追加する。
- non-Copy AST / HIR / diagnostic payload の大量 collection が必要になる前に、`OwnedBuffer` / owner-preserving collection API を実装する。

## 既存 issue との対応

今回の確認で、静的検査設計本体の新規 root issue は追加しない。確認した設計残件は既存 P1 issue に対応している。別途、doc と source policy の監視漏れは個別 issue として追加した。

| 残件 | 既存 issue |
|---|---|
| Resource IR を最終 authority にし、旧 `move_check` / `drop_insertion` を統合削除する | `ISS-20260425T000000Z-RV-CORE-009-58589A3F` |
| raw memory effect / unsafe memory boundary を public surface から閉じる | `ISS-20260427T132406497Z-CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS-DC67DF04` |
| `MemPtr` / `RegionToken` を owner token と pointer projection に分離する | `ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF` |
| Resource diagnostic の Rust 側 taxonomy を self-host と note/help contract へ揃える | `ISS-20260429T040748194Z-RUST-COMPILER-DIAGNOSTICS-ARE-NOT-AL-1617747D` |
| stdlib raw-memory-backed API を safe public discipline へ移す | `ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84` |
| collection の null/numeric storage state を enum owner state へ移す | `ISS-20260429T155343006Z-COLLECTION-STORAGE-STATES-USE-NUMERI-E4B3A749` |
| static-check 設計 doc の obsolete diagnostic taxonomy 記述を修正する | `ISS-20260430T063316041Z-STATIC-CHECK-DESIGN-DOCS-KEEP-STALE--D0874958` |
| Resource checker module 分割 policy に `initialized_summary_variant_build.rs` を含める | `ISS-20260430T062912063Z-RESOURCE-CHECKER-RESPONSIBILITY-POLI-CC55287A` |

## 検証方針

静的検査設計の回帰監視は、次の種類を必須にする。

- Resource IR lowering coverage が `Place::Unknown` を compiler error にする regression。
- `CellState` の raw load/store/dealloc/realloc/fill/bulk operation regression。
- `OwnerState` の leak / maybe leak / double free / no free obligation regression。
- `BorrowState` の return escape / shared / unique conflict regression。
- raw identity escape と ordinary impure call が別 diagnostic code になる regression。
- diagnostic code の `as_str()` / `message()` が wildcard なしの exhaustive match であること。
- self-host diagnostic code が自由文字列へ戻らない source policy。
- stdlib collection が numeric bucket state / null owning pointer / raw header owner を増やさない source policy。

## 判定

NEPLg2 の静的検査設計は、Resource IR と enum-first diagnostic へ向かう方針として妥当である。しかし、現在の実装はまだ移行途中であり、最終設計として承認できる状態ではない。

短期の方針は、既存 Resource IR gate を弱めず、`resource.cell.*` / `resource.owner.*` の分離を raw bucket へ戻さず、旧 checker と stdlib の曖昧な所有権表現を増やさないこと。中期の必須作業は、drop elaboration と owner/cell state authority を Resource IR の第一級設計へ移し、stdlib collection / mem / string を owner token と enum state に揃えることである。

pass 順序と authority の詳細な再確認は [静的検査 soundness review 2026-04-30](./static_check_soundness_review_20260430.md) にまとめる。

# NEPLg2 静的検査 soundness review 2026-04-30

## 目的

静的検査が型安全・メモリ安全・effect safety を保証できる設計になっているかを、pass 順序と authority の観点から確認する。

この文書は、既存実装を最終設計として正当化するためのものではない。現在の Rust compiler にある防壁、Resource IR へ移行済みの検査、旧 HIR checker に残る責務、self-host 実装でコピーしてはいけない暫定構造を分ける。

## 判定

現行実装は、self-host S1/S2 と stdlib 改善を進めるための防壁としては有効である。ただし、最終的な静的検査設計としては未完である。

完了済みとして扱えるもの:

- Diagnostic code は Rust 側で階層 enum 化され、Resource diagnostic も `Move` / `Borrow` / `Cell` / `Owner` / `Raw` / `Lower` に分離済みである。
- `match` は enum / bool / i32 / u8 / char scrutinee を型で分け、enum と bool では網羅性検査が働く。i32 / u8 / char は wildcard がない限り非網羅として扱う。
- Resource IR lowering coverage は missing function / count mismatch / unknown place を compiler error にする。
- Resource IR の raw cell / owner / borrow / raw identity escape gate は active compiler error として動いている。
- static-check / self-host safety source policy は aggregate runner に追加し、標準の source-policy run で監視するようにした。

最終設計として未完のもの:

- 旧 `passes::move_check::run` が Resource IR gate より前に authoritative として残っている。
- `passes::insert_drops` は checked Resource IR ではなく HIR 上で drop を挿入している。
- `ResourceCheckDiagnostic::CellUnavailable` は raw-memory cell operation だけ compiler error へ写像され、通常 read/move/drop/call argument などは旧 move checker の防壁に依存している。
- `UnsafeMemoryInPureFunction` は 2026-05-06 時点で `effect.pure.calls_impure` へ error 化済みである。ただし raw-memory-boundary capability は stdlib migration の限定許可として残る。
- `HirExprKind::CallIndirect` は Resource IR lowering で `EffectOp::Unknown` になり、Resource IR effect gate は unknown effect を count するだけで conservative error にしない。
- `MemPtr<T>` / `RegionToken<T>` は compiler-issued capability ではなく stdlib struct として forge 可能であり、owner token と non-owning pointer projection の分離が未完である。

## pass-by-pass matrix

| stage | 現行の authority | 現在の保証 | 最終設計との差 |
|---|---|---|---|
| loader / SourceMap | Rust loader | `stdlib/core/mem.nepl` だけに raw memory boundary capability を付ける。 | capability が file 単位で粗い。最終的には internal raw API / safe public API の module boundary と compiler-issued token に寄せる。 |
| lexer / parser | Rust lexer/parser | 構文エラーを typed diagnostic code で出す。char literal も token として扱う。 | self-host lexer/parser は Rust の diagnostic taxonomy と token contract に追従する必要がある。 |
| resolve / typecheck | Rust `typecheck` | name、type、trait capability、direct call effect、indirect call effect、match exhaustiveness を検査する。 | typecheck は必要だが、resource safety の final authority にはしない。 |
| old move check | `passes::move_check::run` | non-Copy move/use/drop/borrow の広い範囲を拒否する。 | 移行中の防壁であり、self-host に同じ HIR special-case をコピーしてはいけない。 |
| Resource IR lowering | `resource::lower_hir_module` | HIR の resource-relevant operation を `ResourceOp` / `Place` / `EffectOp` へ落とす。 | indirect call effect が `Unknown` になるなど、semantic 情報の欠落がまだ残る。 |
| lowering coverage gate | `compare_hir_resource_lowering_typed` | HIR と Resource IR の operation coverage、deref projection、unknown place を compiler error にする。 | count coverage は必要条件であり、function effect / owner summary の semantic completeness は別途必要。 |
| cell gate | `check_resource_initialized_moves` + compiler gate | raw load/store/dealloc/realloc/fill/bulk に対する uninit/moved/dropped/maybe-moved を `resource.cell.*` として拒否する。 | 通常 read/move/drop/call argument の cell violation はまだ Resource IR gate の final authority ではない。 |
| owner gate | `check_resource_owner_obligations` + compiler gate | leak / maybe leak / no free obligation / double free / use-after-move を `resource.owner.*` として拒否する。 | raw memory boundary file は一部除外される。stdlib owner token 移行後に除外範囲を縮める。 |
| borrow gate | `check_resource_borrow_lifetimes` + compiler gate | return escape、shared/unique conflict、drop/move/assign 中の borrow conflict を `resource.borrow.*` として拒否する。 | old move check との二重 authority を解消し、Resource IR state merge を最終状態にする必要がある。 |
| effect gate | `check_resource_effect_boundaries` + compiler gate | raw address identity escape を `resource.raw.identity_escape` として拒否する。 | unsafe memory in pure は shadow-only。unknown effect は count のみ。final では conservative error または typed summary が必要。 |
| drop insertion | `passes::insert_drops` | codegen 前に HIR 上で auto drop を挿入する。 | checked Resource IR 上の owner/cell/drop state から挿入する設計へ移す。 |
| backend lowering | wasm/llvm backend | typed diagnostic code を backend error へ写像する。 | backend は checked/drop-elaborated IR だけを受け取る形にする。 |

## soundness invariants

### Type safety

型安全の現在の中核は typecheck である。特に次は維持する。

- `DiagnosticCode` と self-host diagnostic code は enum を内部主識別子にする。
- enum / bool の `match` は網羅性検査を行う。
- scalar `match` の i32 / u8 / char は wildcard がない限り非網羅にする。
- wildcard arm は最後だけ許可する。
- function value / indirect call も pure context から impure function value を呼ぶと `effect.pure.calls_impure` で拒否する。

self-host では、diagnostic code や token kind を raw string / number で分岐させない。Rust と同様に enum と exhaustive `match` を使う。

### Memory safety

メモリ安全の現在の防壁は、old move checker と Resource IR gate の二重構造である。

- raw memory cell の initialized / moved / dropped / maybe-moved は Resource IR が active error にする。
- owner obligation は Resource IR が active error にする。
- borrow conflict は Resource IR が active error にする。
- non-raw owner/move/drop の広い範囲はまだ old move checker が防いでいる。

したがって、final design では `ResourceCheckOperation` のうち raw-memory cell だけを compiler gate に写す形を完了条件にしてはいけない。通常 read / move / drop / call argument / return value も Resource IR の cell state authority に移す。

### Effect safety

effect safety は現時点で三層に分かれている。

- typecheck: direct / indirect call の pure/impure mismatch を拒否する。
- Resource IR: raw identity escape を拒否する。
- migration allowance: `UnsafeMemoryInPureFunction` は error 化済みだが、raw-memory-boundary source は Stage 6 完了まで限定許可される。`EffectOp::Unknown` は最終 error になっていない。

final design では、function value / callback / indirect call の effect を Resource IR に typed summary として渡す。`EffectOp::Unknown` は通常の lowering 成功状態として許可せず、coverage failure か明示的な unsafe/internal boundary に限定する。

### Diagnostic safety

現在の Rust diagnostic taxonomy は正しい方向である。

- cell / owner violation は raw bucket へ潰さない。
- raw category は raw identity / provenance / unsafe boundary の問題に限定する。
- self-host diagnostic は Rust 側の category contract に合わせ、string serialization は外部境界だけに置く。

今後の追加 diagnostic は、先に enum variant と stable string contract を定め、source から発生する regression では `diag_code` を固定する。

## self-host 実装でコピーしてよいもの

- typed diagnostic enum と stable string boundary。
- enum state と exhaustive `match` による token / AST / Resource state 管理。
- Resource IR の `Place` / `CellState` / `OwnerState` / `BorrowState` / `EffectOp` という責務分割。
- lowering coverage を hard error にする方針。
- self-host S1/S2 の lexer/parser/module loader を safe subset の stdlib で進める方針。

## self-host 実装でコピーしてはいけないもの

- 旧 HIR `move_check` の ad-hoc summary / alias map を self-host の final checker として再実装すること。
- raw string / numeric code を diagnostic や state の主表現にすること。
- `EffectOp::Unknown` を non-error の通常 indirect call representation として扱うこと。
- `MemPtr` を owner token と non-owning pointer projection の両方に使うこと。
- raw memory boundary file を増やして unsafe memory を隠すこと。
- drop insertion を checked Resource IR state から切り離して HIR に後付けすること。

## issue 対応

| 未完了点 | issue |
|---|---|
| Resource IR を final authority にし、旧 `move_check` / HIR `insert_drops` を統合削除する | `ISS-20260425T000000Z-RV-CORE-009-58589A3F` |
| `MemPtr` / `RegionToken` を owner token と pointer projection に分離する | `ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF` |
| raw memory API を safe public discipline へ移す | `ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84` |
| Result::Ok-gated owner summary を Resource IR へ入れる | `ISS-20260430T060552075Z-RESOURCE-IR-LACKS-RESULT-OK-GATED-OW-5C2C877E` |
| checked MemPtr load の variant requirement refinement | `ISS-20260430T060600668Z-CHECKED-MEMPTR-LOAD-VARIANT-REQUIREM-1A1ADF53` |
| indirect call effect を `EffectOp::Unknown` ではなく typed effect summary として下げる | `ISS-20260430T064827021Z-RESOURCE-IR-INDIRECT-CALLS-KEEP-UNKN-E9B29774` |
| static-check / self-host source policy を aggregate runner に入れる | `ISS-20260430T064057030Z-STATIC-CHECK-SOURCE-POLICY-RUNNER-MI-812E7A30` |

## 次の実装順

1. Resource IR indirect call effect を typed summary 化する。
2. `UnsafeMemoryInPureFunction` は hard gate 化済みなので、残る raw-memory-boundary capability を stdlib/core/mem の internal/public boundary と compiler-issued token へ狭める。
3. Resource IR cell gate を raw-memory cell operation だけでなく通常 read/move/drop/call/return へ広げ、old move checker との差分を regression 化する。
4. drop insertion を checked Resource IR 上の owner/cell state から行う設計へ移す。
5. self-host S3 ではこの順序を前提にし、旧 HIR checker の special-case を再実装しない。

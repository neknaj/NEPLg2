# 横断レビュー: stdlib と selfhost readiness

対象 commit: `f108cebd`

参照 Actions: `25157230630`

## 結論

stdlib は selfhost に必要な部品を増やしており、string、byte buffer、hash、diagnostic、module graph、CLI I/O の基礎は進んでいる。しかし、selfhost compiler 全体を安全に実装できる状態ではまだない。理由は、`core/mem`、`Vec`、stdio/fs、collection drop contract が strict Resource IR と完全に噛み合っていないためである。

短期的には、S1/S2 の lexer / parser / module loader / diagnostic / CLI 境界は進められる。S3 以降の typecheck、Resource IR、arena、owned collection を多用する compiler core は、`OwnedBuffer` / owner token / initialized prefix / enum storage state の設計が入るまで固定しすぎない。

## 進捗状況

| 領域 | 状況 | selfhost readiness |
|---|---|---|
| `stdlib/alloc/string.nepl` | 実装中 | `StringBuilder` は改善済みで短期利用可。ただし巨大で、`from_f64_result` の current failure が残る。 |
| `stdlib/alloc/io.nepl` | 実装中 | `ByteBuf` / `ByteBuilder` は `Option<MemPtr<u8>>` 化で前進。最終的には `OwnedBytes` が必要。 |
| `stdlib/core/mem.nepl` | 過渡 | raw allocator と typed wrapper が同居。selfhost core の設計基盤にはまだ直接置けない。 |
| `stdlib/alloc/collections/vec.nepl` | 再設計対象 | `MemPtr<T>` が owner field であり、空状態は null pointer discipline。selfhost arena 基盤としては未完成。 |
| `HashMap` / `HashSet` | 改善中 | `BucketState` enum と typed bucket storage は良い方向。non-Copy payload と Vec 依存は残る。 |
| stack / queue / deque / heap | 改善中 | raw header から `Vec<Option<T>>` 系へ移った点は正しい。基礎 `Vec` の最終化待ち。 |
| bitset / bloom / matrix | 過渡 | payload は byte / i32 で扱いやすいが、owner field として裸 `MemPtr` が残る。 |
| `stdlib/alloc/hash` | 使用可能 | FNV / hash32 は symbol table の短期候補。hash dispatch を enum branch の代替に使ってはいけない。 |
| `stdlib/std/fs` / `stdio` | 実装中 | WASI 境界として必要だが owner failure が多い。Result contract と write buffer ownership の整理が必要。 |
| `stdlib/std/test` | 移行中 | stdout assertion report / exit code policy へ移行が必要。Rust/selfhost 共通 `.n.md` の前提。 |
| `stdlib/neplg2` | 初期実装 | S1/S2 は進められる。S3 以降は Rust Resource IR final design に合わせる。 |

## memory model の評価

`doc/neplg2/stdlib_collection_mem_string_static_safety_design.md` の設計判断は妥当である。`MemPtr<T>` は non-owning pointer / projection に固定し、free obligation は `OwnedRegion<T>` / `OwnedBytes` / `Storage<T>` のような owner token 側に移すべきである。

現状の `core/mem` は、過渡期の安全強化としては意味がある。Resource IR が raw allocation、cell initialization、owner obligation を検査し始めているため、以前より危険な経路は可視化されている。一方で、`MemPtr` と `RegionToken` をさらに拡張して owner を表す方向は不適切である。役割分割なしに special-case を増やすと、selfhost compiler の memory model が不安定になる。

理想形:

- `MemPtr<T>`: コピー可能な non-owning view。free obligation を持たない。
- `OwnedRegion<T>` / `OwnedBytes`: allocator が発行した owner token。stdlib から forge できない。
- initialized cell: Resource IR の `CellState` と stdlib wrapper が共有する状態。
- collection storage: `StorageState<T>` のような enum で empty/live/tombstone/uninit を表す。
- fallible mutation: 失敗時に owner を返すか、cleanup 済みであることを型に出す。

## string / byte buffer

`StringBuilder`、`ByteBuf`、`ByteBuilder` の `Option<MemPtr<u8>>` 化は、空 storage と owning storage を型に出す方向として正しい。selfhost の lexer token text、diagnostic message、byte output には短期利用してよい。

ただし完成形ではない。`Option<MemPtr<u8>>` は、型名としては owner obligation を表していない。将来は `OwnedBytes` や `OwnedStringRegion` だけが `str` へ確定できる構造にするべきである。

Actions 上の `from_f64_result` failure は、string の numeric formatter が Resource IR の moved-cell failure を再発させていることを示す。これは `ISS-20260430T140641137Z-FROM-F64-RESULT-SCRATCH-BUFFER-REINT-1D9324F1` で追跡済みであり、formatter の scratch buffer 所有権設計を直す必要がある。

## collections

derived collection が raw header を廃止し、`Vec<Option<T>>` や `BucketState` enum に寄ったのは正しい。特に HashMap / HashSet の bucket state は、数値 sentinel から enum state へ移す方向として selfhost に合っている。

残る根は `Vec<T>` である。`Vec<T> { len, cap, data: MemPtr<T> }` は、initialized prefix、free obligation、move-out、drop obligation を型だけで表現できない。selfhost の arena、symbol table、diagnostic list、module table は collection を多用するため、`Vec` の設計を固定しないまま上位設計を固めると、後で大きく作り直すことになる。

必要な設計:

- `Vec` は `OwnedBuffer<T>` と initialized prefix を持つ。
- read-only observer は owner を消費しない。
- `get_copy`、borrowed view、owned remove/pop を API と trait bound で分ける。
- `T: Copy` が必要な raw copy は typecheck と Resource IR の両方で検査する。
- container free/drop は element drop obligation を持つ。
- error path は owner を返すか cleanup するかを `Result` payload で明示する。

## std/fs/stdio

Actions artifact では、`stdio_write_fd_mem_result`、`fs_open_with_flags`、CLI arg out pointer 周辺の owner failure が目立つ。WASI 境界は selfhost CLI に必須だが、I/O helper は「外部 syscall の失敗」と「buffer owner の返却・消費」を同時に扱うため、Result contract が曖昧だと Resource IR が正しく検査できない。

今後の設計では、I/O API を次のように分けるべきである。

- syscall result は `Result` で表す。
- input buffer は borrowed view と owner-consuming API を分ける。
- output buffer を書き換える API は initialized range summary を持つ。
- file descriptor / path / byte buffer の lifetime と drop obligation を明示する。
- debug / ANSI / stdio convenience と raw fd operation を同じ関数群に押し込めない。

## selfhost readiness

開始してよい:

- source map / span / diagnostic enum / stable string boundary
- lexer token stream と Rust parity fixture
- parser AST subset と direct enum match dispatch
- in-memory module graph / stdlib map
- CLI args / file_io / reporter の shell
- string compare、byte scanner、hash、path helper の不足 API 整備

慎重に進める:

- type environment arena
- owned AST / HIR table
- diagnostic aggregation storage
- Resource IR lowering
- codegen output buffer

まだ固定すべきでない:

- `MemPtr` owner field を前提にした compiler arena
- raw memory helper を selfhost core public API にする設計
- 旧 move checker 型の special-case visitor
- numeric/string diagnostic ID
- hash 値による token / AST / state dispatch

## 既存 issue との対応

| issue | review 判断 |
|---|---|
| `ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF` | `MemPtr` と owner token 分離の中心。selfhost readiness の gate。 |
| `ISS-20260427T164432612Z-CORE-MEM-DEALLOC-APIS-DO-NOT-ENCODE--204F1F47` | dealloc API が initialized payload / drop obligation を表せない問題。 |
| `ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84` | raw-memory-backed stdlib API の段階移行親 issue。 |
| `ISS-20260429T155343006Z-COLLECTION-STORAGE-STATES-USE-NUMERI-E4B3A749` | collection storage state の enum 化。 |
| `ISS-20260430T140641137Z-FROM-F64-RESULT-SCRATCH-BUFFER-REINT-1D9324F1` | string formatter の current Resource IR failure。 |
| `ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD` | stdlib assert と `.n.md` output policy の移行。 |

今回の readiness review では、上記で追跡できない新規 issue は確認していない。

# stdlib overview review

対象 commit: `f108cebd`

参照 Actions: `25157230630`

## 概要

`stdlib` は 144 file あり、`core`、`alloc`、`std`、`nm`、`kp`、`platforms`、`neplg2` selfhost、旧 `neplg3` に分かれている。現状は、selfhost の S1/S2 に必要な string / byte buffer / CLI / module helper が増えている一方、strict Resource IR の下では stdlib doctest 全体がまだ green ではない。

Actions の `stdlib-test` artifact は `415 total / 232 passed / 173 failed / 10 errored`。失敗は local 実行ではなく `gh run download 25157230630 -n stdlib-tests` で取得した artifact から分類した。

## module map

| 領域 | 主な内容 | 判定 |
|---|---|---|
| `stdlib/core` | primitive helper、`mem`、`char`、`Option`、`Result`、traits | `char` / enum / trait は進んだが `mem` は owner token 未完 |
| `stdlib/alloc/string.nepl` | `str` layout、UTF-8、StringBuilder、numeric parser | selfhost に必要だが巨大で、`from_f64_result` に current Actions failure |
| `stdlib/alloc/io.nepl` | `ByteBuf` / `ByteBuilder` / I/O traits | `Option<MemPtr<u8>>` 化で改善、最終的には `OwnedBytes` が必要 |
| `stdlib/alloc/collections` | Vec / List / HashMap / HashSet / tree / heap / bitset | 改善中だが `Vec` と raw storage collection が未完 |
| `stdlib/alloc/diag` | `Diag` / `Diags` / `Outcome` | selfhost に有用だが string code ではなく typed diag へ寄せ続ける必要 |
| `stdlib/alloc/encoding/json` | typed `JsonValue` と serializer | enum 化は良い。owned payload drop/free は collection 設計に依存 |
| `stdlib/alloc/hash` | FNV / generic hash / SHA256 | selfhost symbol table の基盤候補。SHA256 は Vec<i32> 依存 |
| `stdlib/std` | fs / stdio / streamio / env / text / test | WASI と assertion report の中心。Actions failure が多い |
| `stdlib/nm` | gloss/nm parser と HTML generator | raw AST storage 回避は良いが parser/html はまだ巨大で if nest が残る |
| `stdlib/kp` / `features` / `platforms` | 競プロ helper、TUI | useful examples だが selfhost 中核の優先度は低い |
| `stdlib/neplg2` | selfhost compiler | 別章で詳細 review |

## 巨大ファイル

| file | lines | review |
|---|---:|---|
| `stdlib/core/math.nepl` | 4435 | primitive numeric helper が過密。stdlib file 分割 issue の中心 |
| `stdlib/alloc/string.nepl` | 3290 | string / UTF-8 / builder / numeric parser が集中。selfhost 必須なので分割設計が必要 |
| `stdlib/std/stdio.nepl` | 1741 | fd read/write、debug、ANSI、read_line が同居 |
| `stdlib/alloc/collections/vec.nepl` | 1660 | collection 基礎型。現状の raw storage owner model の根 |
| `stdlib/std/fs.nepl` | 1596 | WASI path/fs/read/write/dir が集中 |
| `stdlib/std/streamio.nepl` | 1572 | stream scanner/writer。ByteBuf/string/stdio に強く依存 |
| `stdlib/neplg2/core/syntax/lexer.nepl` | 1230 | selfhost lexer。別章対象 |
| `stdlib/core/mem.nepl` | 1121 | raw allocator と typed wrappers が同居 |
| `stdlib/alloc/collections/vec/sort.nepl` | 1021 | raw slice sort helper が多い |

## Actions stdlib-test 分類

Artifact `stdlib-tests.json` の失敗分類:

| 分類 | 失敗数 | 主な原因 |
|---|---:|---|
| collections | 73 | `Vec` / `List` owner leak、HashMap/HashSet が `from_f64_result` failure に隠れる |
| string | 10 | `from_f64_result` scratch buffer、StringBuilder / std/test 絡み |
| std/io/fs/stdio | 33 | `stdio_write_fd_mem_result`、`fs_open_with_flags`、`cliarg` out pointer / owner summary |
| selfhost | 19 | CLI / module graph timeout と owner gate |
| core | 16 | `Option` / `Result` / traits doctest が std/test owner issue を拾う |
| nm | 3 | parser/html generator と StringBuilder/JSON 依存 |
| kp | 7 | Vec / std/test 依存 |

失敗の上位 error は、`sb_build_result` owner may leak、`stdio_write_fd_mem_result` owner may leak、`from_f64_result` `resource.cell.possibly_moved`、`fs_open_with_flags` owner may leak、timeout である。

## 追加 issue

Actions artifact で `from_f64_result` の scratch buffer failure が current main に残っていることを確認したため、`ISS-20260430T140641137Z-FROM-F64-RESULT-SCRATCH-BUFFER-REINT-1D9324F1` を追加した。これは resolved 済み string constructor 系 issue の説明だけでは current Actions failure を追跡できないため、再発監視として必要である。

## 結論

stdlib の方向性は、raw sentinel から enum / Option / Result へ移す点では正しい。しかし、`core/mem`、`Vec`、`stdio/fs`、string numeric formatting はまだ strict Resource IR と完全には噛み合っていない。selfhost は lexer / parser / CLI shell などの限定領域なら進められるが、typecheck / ResourceIR / diagnostic aggregation の中核で owning collection を多用する前に、`OwnedBuffer`、owner-preserving failure result、stdout assertion report の整備が必要である。

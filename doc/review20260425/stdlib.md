# Stdlib レビュー

作成日: 2026-04-25

対象: `stdlib/**`

## レビュー範囲

| 区分 | 主なファイル |
|---|---|
| core | `stdlib/core/mem.nepl`, `stdlib/core/math.nepl`, `stdlib/core/result.nepl`, `stdlib/core/option.nepl`, `stdlib/core/traits/**` |
| alloc | `stdlib/alloc/string.nepl`, `stdlib/alloc/io.nepl`, `stdlib/alloc/collections/**`, `stdlib/alloc/diag/**`, `stdlib/alloc/encoding/json.nepl` |
| std | `stdlib/std/stdio.nepl`, `stdlib/std/streamio.nepl`, `stdlib/std/fs.nepl`, `stdlib/std/env/cliarg.nepl`, `stdlib/std/test.nepl` |
| platform / tools | `stdlib/platforms/wasix/tui.nepl`, `stdlib/nm/**`, `stdlib/kp/**` |
| self-host | `stdlib/neplg2/**` |

## 総評

stdlib は API 数が多く、コメントと doctest はかなり整備されています。一方で、低レベルメモリ管理と owning collection の所有権設計が現行 compiler の Resource IR 不足を補えていません。特に allocator と `Vec` / `Stack` の `Copy` 実装は、実行時のメモリ破壊に直結する可能性があります。

また、I/O と filesystem 周辺は skip test が多く、CLI runtime の WASI 実装不足と合わせて未検証領域が残っています。`stdlib/neplg2` は現時点では placeholder で、セルフホスト compiler と呼べる実装にはなっていません。

## RV-STDLIB-001: allocator がアドレス 0 のメタデータと最初のブロックを衝突させる

- 解決済: true
- 状態: verified
- 優先度: P0
- 種別: bug
- 対象: `stdlib/core/mem.nepl`

### 根拠

- `stdlib/core/mem.nepl:8`: 0..4 に heap_ptr、4..8 に free_list_head を置く設計コメント。
- `stdlib/core/mem.nepl:236`: `alloc_raw`。
- `stdlib/core/mem.nepl:304`: `let heap_ptr <i32> load_i32 0`。
- `stdlib/core/mem.nepl:305`: `let start <i32> align8 heap_ptr`。
- `stdlib/core/mem.nepl:314`: `store_i32 0 new_heap`。
- `stdlib/core/mem.nepl:315`: `store_i32 start total`。

### 問題

初回 allocation では `heap_ptr` が 0 のため `start` も 0 になります。その直後に `store_i32 0 new_heap` で heap pointer を書いた後、`store_i32 start total` が同じ address 0 に block header を書きます。結果として heap metadata と最初の allocation header が衝突します。

さらに、最初の block の header address は 0 になり、free list の null sentinel と区別できません。`dealloc_raw` で free list head に 0 を入れても「空」と同じ意味になるため、最初の block は再利用不能です。

### 影響

allocator の根本不整合です。小さな program では偶然動いても、allocation / deallocation が増えると heap pointer、free list、block header の意味が破壊されます。

### 修正方針

allocator 初期化を明示し、heap base を 8 以上に固定します。`alloc_raw` は heap pointer が 0 の場合に allocator metadata を初期化し、payload block は metadata 領域を避けます。0 sentinel と実 block address が衝突しないようにします。

### 対応結果

`alloc_raw` の bump allocation で `heap_ptr` が metadata 領域内の場合は `8` へ進めるようにしました。これにより最初の block header は address `8` から始まり、返却 data pointer は address `16` 以降になります。

### 検証

初回 `alloc_raw 4` の返却アドレスが metadata 領域外であり、`load_i32 0` / `load_i32 4` が allocator state として壊れていないことを確認する doctest を `stdlib/core/mem.nepl` に追加しました。

確認済み:

- `node nodesrc/run_doctest.js -i stdlib/core/mem.nepl -n 3`
- `trunk build`
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json` (`caseCount=13`, `passedCount=13`, `failedCount=0`)

## RV-STDLIB-002: free list 分割で余りブロックがリストへ戻らない

- 解決済: true
- 状態: verified
- 優先度: P0
- 種別: bug
- 対象: `stdlib/core/mem.nepl`

### 根拠

- `stdlib/core/mem.nepl:260`: free list から見つけた block を unlink。
- `stdlib/core/mem.nepl:264`: `remain` を計算。
- `stdlib/core/mem.nepl:269`: `new_blk` を作る。
- `stdlib/core/mem.nepl:270`: `new_blk` に remain size を書く。
- `stdlib/core/mem.nepl:272`: `new_blk_next_ptr` に old next を書く。
- しかし `prev` または free list head を `new_blk` に差し替える処理がない。

### 問題

free block を split したとき、余り block の header は作っていますが、free list head または previous node の next に接続していません。つまり余り領域が free list から失われます。

### 影響

free list reuse が進むほど断片化ではなく実質リークになります。長時間動く stdlib test や string/Vec-heavy code でメモリ消費が増え続けます。

### 修正方針

unlink 前に split 判定するか、split 後に `new_blk` を同じ位置へ再 link します。`prev == 0` なら free list head を `new_blk` に、そうでなければ `prev.next = new_blk` にします。

### 対応結果

free list から見つけた block を split する場合は、余り block `new_blk` を free list head または `prev.next` に接続するようにしました。split しない場合だけ従来どおり `next` へつなぎ替えます。

### 検証

大きい block を free し、小さい allocation で split した後、残りサイズに収まる allocation が free list から取られることを確認する doctest を `stdlib/core/mem.nepl` に追加しました。

確認済み:

- `node nodesrc/run_doctest.js -i stdlib/core/mem.nepl -n 4`
- `trunk build`
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json` (`caseCount=13`, `passedCount=13`, `failedCount=0`)

## RV-STDLIB-003: 所有権を持つ Vec/Stack が Copy/Clone になっている

- 解決済: true
- 状態: verified
- 優先度: P0
- 種別: bug
- 対象: `stdlib/alloc/collections/vec.nepl`, `stdlib/alloc/collections/stack.nepl`

### 根拠

- `stdlib/alloc/collections/vec.nepl:114`: `impl<.T> Copy for Vec<.T>`。
- `stdlib/alloc/collections/vec.nepl:118`: `impl<.T> Clone for Vec<.T>` が shallow copy。
- `stdlib/alloc/collections/vec.nepl:1519`: `free` が `data` 領域を解放。
- `stdlib/alloc/collections/stack.nepl:99`: `impl<.T> Copy for Stack<.T>`。
- `stdlib/alloc/collections/stack.nepl:548`: `free` が内部領域を解放。

### 問題

`Vec` と `Stack` は heap buffer の所有権を持つのに `Copy` / shallow `Clone` です。コピー後に片方を変更すると alias 先も同じ buffer を見ます。両方を `free` すると double free になります。

### 影響

compiler の move checker が `Copy` と判断すると、所有権移動を検出できません。メモリ破壊、use-after-free、double free が stdlib の通常 API で発生します。

### 修正方針

`Vec` / `Stack` の `Copy` impl と shallow `Clone` impl を削除しました。owning collection は値渡しで所有権が移動し、`free` 後の同じ buffer への別名所有を作れないようにします。

`Clone` は deep copy として設計する余地がありますが、要素 `T` の clone/drop 方針が未整理な段階で shallow clone を残すと危険なため、今回は削除に留めました。

### 検証

`let moved v; free v; free moved` と同等の double free pattern が compile_fail になる doctest を `Vec` / `Stack` に追加しました。`RV-CORE-013` の修正により、`len_ref &v` / `peek_ref &stk` のような borrow-based read API は読み取り後に元の collection を更新できます。

確認済み:

- `node nodesrc/tests.js -i stdlib/alloc/collections/stack.nepl --no-tree -o tmp/stack-rv-stdlib-003-final.json -j 1` (`total=12`, `passed=12`, `failed=0`)
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec.nepl --no-tree -o tmp/vec-rv-stdlib-003-final.json -j 1` (`total=37`, `passed=36`, `failed=1`)
- `vec.nepl::doctest#28` の失敗は `partition` 戻り値 `.Pair` から取り出した `Vec<i32>` の overload 解決問題で、`RV-CORE-014` として分離しました。
- `trunk build`
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json` (`caseCount=13`, `passedCount=13`, `failedCount=0`)

## RV-STDLIB-004: collection free が要素の Drop を呼ばない

- 解決済: false
- 状態: open
- 優先度: P1
- 種別: bug
- 対象: `stdlib/alloc/collections/**`

### 根拠

- `stdlib/alloc/collections/vec.nepl:1519`: `free` は `data` buffer を dealloc するだけ。
- `stdlib/alloc/collections/hashmap.nepl:457`: hashmap free も entries buffer と header を直接解放。
- `stdlib/alloc/collections/list.nepl:481`: list free は node を順に raw dealloc。
- `stdlib/core/traits/drop.nepl:6`: compiler 側が Drop trait から auto drop を構成する方針がコメントされている。

### 問題

`Vec<str>` や `Vec<Owned>` のような要素がさらに所有権を持つ場合、collection の `free` は要素ごとの drop を呼びません。逆に shallow copy と組み合わさると double free も起こり得ます。

### 影響

stdlib の collection を使うほど、要素所有権のリークまたは二重解放が増えます。Resource IR がない現行 compiler では型システムでも補足しきれません。

### 修正方針

collection は `Drop<T>` bound を持つ要素 cleanup と、単純 dealloc の fast path を分けます。最低限、owning element を含む collection は `free` の仕様で責務を明確にし、compiler drop pass と整合させます。

### 検証

drop counter を持つテスト用型を作り、`Vec<T>` / `List<T>` / `HashMap<K,V>` の free で要素 drop 回数が一致することを確認します。

## RV-STDLIB-005: stdio read_all が 4096 byte で切り捨てる

- 解決済: false
- 状態: open
- 優先度: P1
- 種別: bug
- 対象: `stdlib/std/stdio.nepl`, `stdlib/std/streamio.nepl`

### 根拠

- `stdlib/std/stdio.nepl:400`: `read_all` の説明に最大 4096 byte とある。
- `stdlib/std/stdio.nepl:407`: 4096 byte 超過は切り捨てると明記。
- `stdlib/std/stdio.nepl:430`: `let cap <i32> 4096`。
- `stdlib/std/streamio.nepl:10`: streamio も text read は 4096 byte 固定と説明。

### 問題

`read_all` という名前にもかかわらず、text input を全て読みません。4096 byte を超える入力は正常系として切り捨てられます。

### 影響

競技プログラミングや CLI tool の標準入力で誤答になります。バグとして発見しづらく、入力サイズに依存します。

### 修正方針

`stdio_read_all_bytes` を基礎にし、EOF まで growable buffer で読みます。固定長 helper が必要なら `read_chunk` など別名にします。

### 検証

4097 byte 以上の stdin を与え、`len read_all` が入力 byte 数と一致するテストを追加します。

## RV-STDLIB-006: fs/cliarg の主要テストが skip されている

- 解決済: false
- 状態: open
- 優先度: P1
- 種別: test
- 対象: `stdlib/std/fs.nepl`, `stdlib/std/env/cliarg.nepl`

### 根拠

- `stdlib/std/fs.nepl:174`, `244`, `435`, `469`, `510`: fs API の doctest が `neplg2:test[skip]`。
- `stdlib/std/env/cliarg.nepl:227`, `275`, `331`, `401`, `521`: cliarg API の doctest が `skip`。
- `nepl-cli/src/main.rs:842` 以降: args host functions は Rust CLI runtime 側で実装されているが、stdlib 側の主要 doctest は実行されない。

### 問題

filesystem と argv は runtime 依存が強いにもかかわらず、回帰テストが skip されています。CLI runtime と stdlib wrapper のズレを検出できません。

### 影響

path_open、args_get、fd_read の ABI 不一致や buffer size バグが残りやすくなります。

### 修正方針

test runner に fixture file と argv/stdin 指定を持たせ、fs/cliarg の最小成功・失敗ケースを実行可能にします。skip は runtime 未対応のケースだけに限定します。

### 検証

`fs_read_to_string` で fixture file を読む test、`cliarg_count` / `cliarg_get` の argv test を追加します。

## RV-STDLIB-007: str の UTF-8 保証が実装で守られていない

- 解決済: false
- 状態: open
- 優先度: P1
- 種別: bug
- 対象: `stdlib/alloc/string.nepl`, `stdlib/std/fs.nepl`, `stdlib/alloc/io.nepl`

### 根拠

- `plan.md`: 文字列は UTF-8 保証された immutable 値という方針。
- `stdlib/alloc/string.nepl:111`: `string_data_ptr` は raw layout を直接扱う。
- `stdlib/std/fs.nepl:48`: UTF-8 検証は行わず byte 列をそのまま str に格納すると説明。
- `stdlib/std/fs.nepl:443`: `fs_bytes_to_string` が `ByteBuf` を `str` へ変換する。

### 問題

`str` 型が UTF-8 保証を持つ仕様なのに、filesystem / byte buffer から検証なしで `str` を作る経路があります。これにより `str` として不正な byte列が入り得ます。

### 影響

文字列 API、HTML/NM generator、debug/serialize が UTF-8 前提で動く場合に壊れます。将来 `char` / Unicode 処理を追加するとさらに深刻になります。

### 修正方針

`ByteBuf -> str` は UTF-8 validate を必須にし、失敗時は `Result::Err` を返します。raw bytes が必要な API は `ByteBuf` のまま扱います。

### 検証

不正 UTF-8 byte を含む file/buffer で `fs_read_to_string` が Err になるテストを追加します。

## RV-STDLIB-008: self-host compiler がプレースホルダのまま

- 解決済: false
- 状態: open
- 優先度: P2
- 種別: architecture
- 対象: `stdlib/neplg2/**`

### 根拠

- `stdlib/neplg2/core/parser.nepl`: 17 行の skip doctest だけ。
- `stdlib/neplg2/core/typecheck.nepl`: 17 行の skip doctest だけ。
- `stdlib/neplg2/core/ast.nepl`: 17 行の skip doctest だけ。
- `stdlib/neplg2/cli/main.nepl`: 17 行の skip doctest だけ。
- `doc/self_host.md`: `stdlib/neplg2` を self-host compiler 本体として位置づけている。

### 問題

self-host compiler のディレクトリは存在しますが、実装はありません。`core/ast`, `parser`, `typecheck`, `diagnostic`, `span`, `cli/main` が同じ placeholder 形式です。

### 影響

セルフホスト計画の進捗が実装から判別できません。Rust compiler の分割計画と NEPL 側 module の対応表も未確立です。

### 修正方針

まず `span`, `diagnostic`, `ast token subset`, `lexer` の最小実装から milestone を切ります。placeholder file には TODO ではなく、実装段階と依存関係を明記します。

### 検証

`stdlib/neplg2/core/span.nepl` など最小 module ごとに doctest を実行可能にし、skip を外します。

## RV-STDLIB-009: 巨大 stdlib ファイルが分割されていない

- 解決済: false
- 状態: open
- 優先度: P2
- 種別: architecture
- 対象: `stdlib/core/math.nepl`, `stdlib/alloc/string.nepl`, `stdlib/std/stdio.nepl`, `stdlib/alloc/collections/vec.nepl`

### 根拠

- `stdlib/core/math.nepl`: 4435 行。
- `stdlib/alloc/string.nepl`: 2134 行。
- `stdlib/std/streamio.nepl`: 1525 行。
- `stdlib/alloc/collections/vec.nepl`: 1488 行。
- `stdlib/std/stdio.nepl`: 1403 行。

### 問題

stdlib の主要 file が肥大化しています。API、内部 helper、target-specific raw body、doctest が同居しており、修正範囲の把握が難しくなっています。

### 影響

コンパイル性能にも影響します。loader が import file を丸ごと clone / merge する現状では、巨大 stdlib file はそのまま型検査候補数と clone 量を増やします。

### 修正方針

`math/i32`, `math/i64`, `math/f32`, `math/f64`、`string/parse`, `string/format`, `stdio/read`, `stdio/write` のように用途別 module へ分割します。public facade は既存 path 互換を保ちます。

### 検証

分割前後で public import path の doctest が同じ結果になることを確認します。

## RV-STDLIB-010: Result/Option の unsafe helper が通常コードに広く残っている

- 解決済: false
- 状態: open
- 優先度: P2
- 種別: bug
- 対象: `stdlib/**`

### 根拠

- `stdlib/core/result.nepl:11`: `unwrap_ok` / `unwrap_err` は unsafe helper と説明。
- `stdlib/core/option.nepl:13`: `unwrap` は unsafe helper と説明。
- `stdlib/alloc/string.nepl`: `unwrap_ok` / `unwrap` を内部処理で多数使用。
- `stdlib/std/env/cliarg.nepl:174`: `unwrap<i32> load_u8 ...` のような経路。

### 問題

unsafe helper と明記されている `unwrap` 系が stdlib 内部の通常処理に残っています。境界チェック済みの箇所もありますが、前提が崩れると `unreachable` に落ち、Result として呼び出し側へ返せません。

### 影響

I/O、文字列、parser 系で入力依存の trap が起きやすくなります。エラーを値として扱う stdlib 方針と矛盾します。

### 修正方針

public API と input-dependent code では `unwrap` 系を禁止し、`match` で `Result` / `Option` を返します。内部不変条件だけに `unwrap_unchecked` 的な名前を付けて限定します。

### 検証

`rg "unwrap"` の許可リストを作り、stdlib test で unsafe helper の新規使用を検出します。

## RV-STDLIB-011: Clone と collection read API が by-value で非 Copy 所有型を扱えない

- 解決済: true
- 状態: verified
- 優先度: P0
- 種別: architecture
- 対象: `stdlib/core/traits/copy.nepl`, `stdlib/alloc/collections/vec.nepl`, `stdlib/alloc/collections/stack.nepl`

### 根拠

- `stdlib/core/traits/copy.nepl`: `Clone::clone` は `<(Self)->Self>` で、非 Copy 値を複製する前に元の値を move する形になっている。
- `stdlib/alloc/collections/vec.nepl`: `len`, `cap`, `get`, `data_len` などの read API が `Vec<.T>` を by-value で受け取る。
- `stdlib/alloc/collections/stack.nepl`: `len`, `is_empty`, `peek` などの read API が `Stack<.T>` を by-value で受け取る。
- `tests/compiler/move_effect.n.md`: 非 Copy 値は by-value で渡すと move 後再利用不可になることを検証している。

### 問題

`RV-STDLIB-003` で `Vec` / `Stack` から `Copy` を外すと、読み取り API を呼んだだけで所有権を消費します。さらに現在の `Clone` は `Self` を値渡しで受けるため、非 Copy 所有型を「元を残して複製」する能力としては使えません。

### 影響

`Vec` / `Stack` を正しく非 Copy にするための前提が stdlib API 側にありません。このまま `Copy` を削除すると、既存の正常な読み取りや `free` 前の確認処理が move error になり、逆に `Copy` を残すと double free / aliasing が残ります。

### 修正方針

`Clone` trait の `clone` を `(&Self)->Self` に変更し、標準の `Clone` impl を参照渡しへ移行しました。`Vec` には `len_ref` / `cap_ref` / `data_ptr_ref` / `data_mem_ptr_ref` / `data_len_ref` / `is_empty_ref` / `get_ref` を追加し、`Stack` には `len_ref` / `is_empty_ref` / `peek_ref` を追加しました。

`get_ref` / `peek_ref` は memory から値を読み出す API なので `.T: Copy` に限定し、所有権を持つ要素の浅い複製を避けます。`Vec` / `Stack` 自体の shallow `Copy` / `Clone` 削除は `RV-STDLIB-003` で継続します。

### 検証

`tests/stdlib/traits_text.n.md` で `Clone::clone &x` が generic bound 経由で動作することを確認しました。`stdlib/alloc/collections/vec.nepl` と `stdlib/alloc/collections/stack.nepl` の doctest で、`len_ref` / `get_ref` / `peek_ref` の後も元の collection を更新できることを確認しました。

## RV-STDLIB-012: HashKey/Hasher の clone/copy capability が標準 trait と不整合

- 解決済: false
- 状態: open
- 優先度: P1
- 種別: architecture
- 対象: `stdlib/core/traits/hash_key.nepl`, `stdlib/core/traits/hash.nepl`, `tests/stdlib/traits_hash.n.md`

### 根拠

- `stdlib/core/traits/hash_key.nepl`: `HashKey` が `#capability clone` / `#capability copy` と by-value `fn clone <(Self)->Self>` を独自に持っている。
- `stdlib/core/traits/hash.nepl`: `Hasher<.K>` が `#capability clone` / `#capability copy` を要求するが、標準 `Clone` / `Copy` の method とは trait 上で接続されていない。
- `RV-STDLIB-011` で標準 `Clone::clone` を `(&Self)->Self` に移行した後も、hash 系 trait だけ旧 by-value clone semantics が残る。
- `tests/stdlib/traits_hash.n.md`: custom key / hasher の doctest が hash 系 trait の capability を再実装している。

### 問題

hash collection 用の `HashKey` / `Hasher` が、標準 `Clone` / `Copy` と別の capability として clone/copy を表現しています。`HashKey` の by-value `clone` は非 Copy key では元の値を消費するため、標準 `Clone` と同じ意味になりません。`Hasher` は capability だけを掲げており、実際に `Clone` / `Copy` trait を満たすことと型システム上で同期しません。

### 影響

`Vec` / `Stack` の shallow `Copy` 削除と同じ方向で hash collection を安全化するとき、key / hasher が本当に copy 可能なのか、borrow して clone できるのかを trait bound から判断できません。所有権を持つ key を hash collection に入れる設計へ進むと、move checker と capability 判定の不整合が runtime aliasing や誤った compile 許可につながります。

### 修正方針

`HashKey` から独自 `clone` capability を外し、必要な箇所では標準 `Clone` / `Copy` trait を明示的に bound します。key の比較と hash 計算は `HashKey` に残し、複製・コピー可能性は `core/traits/copy.nepl` の trait に一本化します。`Hasher` も `#capability clone` / `#capability copy` ではなく、collection API が必要な場面で `.H: Clone` / `.H: Copy` を要求する形へ分離します。

### 検証

`tests/stdlib/traits_hash.n.md` の custom key / hasher doctest を標準 `Clone::clone &x` の形へ移行します。非 Copy key では by-value clone が使えないことを compile_fail で確認し、hash collection の key 取得・挿入・検索が所有権を破壊しないことを追加テストで確認します。

## RV-STDLIB-013: stdlib collection doctest 群が所有型 API 移行後の実装とずれている

- 解決済: false
- 状態: open
- 優先度: P1
- 種別: test
- 対象: `stdlib/alloc/collections/**`, `stdlib/tests/**`, `tests/stdlib/**`

### 根拠

- GitHub Actions run `24932659255` の `stdlib-test`: `total=388`, `passed=310`, `failed=78`, `errored=0`。
- 同 run: `stdlib/alloc/collections/btreemap.nepl::doctest#2` から `#7` が `error[D3004]: type annotation mismatch (expected BTreeMap_K_V_i32_i32, got unit)` で失敗。
- 同 run: `stdlib/alloc/collections/btreeset.nepl`、`queue.nepl`、`ringbuffer.nepl` でも同様に `new` / constructor 系 doctest が expected collection type に対して `unit` を返した扱いになっている。
- 同 run: `stdlib/alloc/collections/fenwick.nepl` など 14 件が `stdlib/alloc/collections/vec.nepl:938` の `error[D3016]: expression left extra values on the stack` で失敗。
- 同 run: collection diagnostic / hashmap / set 系で `RuntimeError: unreachable`、`RuntimeError: memory access out of bounds`、`error[D1206]: indentation is not aligned to #indent width` も出ている。
- GitHub Actions run `24940960078` の `stdlib-test`: `total=398`, `passed=340`, `failed=58`, `errored=0`。失敗は `btreemap.nepl` 6件、`btreeset.nepl` 5件、`fenwick.nepl` 5件、`ringbuffer.nepl` 5件、`queue.nepl` 4件、`sparse_set.nepl` 4件など collection 系に集中している。
- 同 run: 失敗種別は `D3004` が20件、`D3006` が16件、`D3016` が14件で、constructor / overload / stack discipline の問題が主です。

### 問題

collection 系 stdlib は、`RV-STDLIB-003` / `RV-STDLIB-011` で所有型と borrow-based read API へ寄せた後も、doctest と一部 helper 実装が旧 API・旧構文・旧 ownership 前提を含んだまま残っています。constructor の戻り値を `unwrap_ok ... new<T>` で受ける箇所が `unit` と推論されるケース、`Vec` の predicate callback helper が stack 形を崩すケース、runtime trap へ進む collection diagnostic ケースが同じ `stdlib-test` に混在しています。

`RV-CORE-017` の function value 問題に巻き込まれている callback 系 failure もありますが、BTreeMap / BTreeSet / Queue / RingBuffer の constructor doctest や runtime trap は stdlib 側の実装・ドキュメント・テスト fixture の差分として分離して追跡します。

### 影響

CI の `stdlib-test` が常に失敗し、標準 collection の利用可否を確認できません。tutorial や user code が collection を使う場合にも、型検査で落ちるもの、runtime trap するもの、test fixture の書き方だけが古いものを切り分けられず、stdlib の信頼性を評価できない状態になります。

### 修正方針

まず failure を原因別に棚卸しします。

- constructor / `new` 系 doctest は、現在の zero-argument call、effect marker、`unwrap_ok` の型引数指定に合わせて最小再現を作る。
- `vec_find_impl` / `vec_take_while_len_impl` など callback helper は `RV-CORE-017` 修正後に再実行し、core 側に残る failure と stdlib 側の stack discipline failure を分ける。
- runtime trap 系は `unwrap` による trap、allocator / free、collection invariant 破壊のどれかを JSON artifact と最小 fixture で特定する。
- doctest の indentation failure は実装修正と混ぜず、該当コメントブロックを formatter / parser 方針に合わせて直す。

### 検証

修正後は `node nodesrc/tests.js -i stdlib/alloc/collections -o tmp/collections-rv-stdlib-013.json -j 1` と `node nodesrc/tests.js -i tests/stdlib -o tmp/tests-stdlib-rv-stdlib-013.json -j 1` を通します。CI では `stdlib-test` artifact の summary が `failed=0`, `errored=0` になることを確認します。

2026-04-26 時点では run `24940960078` の artifact `stdlib-tests/stdlib-tests.json` で失敗が58件残っています。修正時はこの artifact の failure set を baseline として、constructor 系と `vec_find_impl` / `vec_take_while_len_impl` 系を別コミットで切り分けます。

## RV-STDLIB-014: Stack の 更新 API が by-value pop に偏り所有値の継続利用を阻害する

- 解決済: true
- 状態: verified
- 優先度: P1
- 種別: architecture
- 対象: `stdlib/alloc/collections/stack.nepl`, `stdlib/tests/stack.n.md`, `tests/stdlib/stack_collections.n.md`

### 根拠

- `stdlib/alloc/collections/stack.nepl:334`: `pop` は `Stack<.T>` を by-value で受け取り、呼び出し元の stack handle を move する。
- `stdlib/alloc/collections/stack.nepl:511`: `len_ref` は存在するが、末尾を取り出して length を更新する借用 API がなかった。
- `examples/rpn.nepl`: `pop` 後に同じ stack を表示・push・free しようとして move checker に止められる。
- `examples/bf.nepl`: bracket 対応表の作成で `pop` 後に `is_empty` / `free` しようとして move checker に止められる。

### 問題

`Stack` から `Copy` を外した後、読み取りは `len_ref` / `peek_ref` で対応できましたが、更新を伴う取り出しは by-value `pop` しかありませんでした。`pop` を使うと stack handle の所有権が移動するため、典型的な「pop してから同じ stack を使い続ける」処理が書けません。

### 影響

example が `core/mem` / `core/field` で `Stack` 内部 layout を直接読む方向へ流れ、stdlib の public API を使うという例として不適切になります。所有権上も、低レベルメモリ操作のほうが move checker を迂回しやすく、根本的な安全化に逆行します。

### 修正方針

`Stack` に `get_ref` と `pop_ref` を追加しました。どちらも `&Stack<.T>` を受け取り、`.T: Copy` に限定して memory から値を読み出します。`pop_ref` は header の length を更新しますが、stack handle の所有権は移動しないため、呼び出し後も `push` / `free` を継続できます。所有権を持つ要素については、別途 move/drop 設計を伴う API が必要なので対象外とします。

### 対応結果

`stdlib/alloc/collections/stack.nepl` に `get_ref` / `pop_ref` と doctest を追加し、`stdlib/tests/stack.n.md` と `tests/stdlib/stack_collections.n.md` に借用後も同じ stack を更新・解放できる回帰テストを追加しました。

### 検証

確認済み:

- `node nodesrc/tests.js -i stdlib/alloc/collections/stack.nepl -i stdlib/tests/stack.n.md -i tests/stdlib/stack_collections.n.md --no-tree -o tmp/stack-ref-tests.json -j 4` (`total=31`, `passed=31`, `failed=0`)

## RV-STDLIB-015: byte/Vec 操作の public API 不足により example が raw memory へ依存する

- 解決済: true
- 状態: verified
- 優先度: P1
- 種別: architecture
- 対象: `stdlib/alloc/collections/vec.nepl`, `stdlib/alloc/string.nepl`, `stdlib/std/stdio.nepl`

### 根拠

- `examples/bf.nepl`: Brainfuck の tape と jump table を `alloc_raw` / `load_u8` / `store_u8` / `load_i32` / `store_i32` で直接操作していた。
- `stdlib/alloc/collections/vec.nepl`: owning `Vec` を消費せずに要素を上書きする public API がなかった。
- `stdlib/alloc/string.nepl`: `str` の byte を範囲チェック付きで読む public API がなかった。
- `stdlib/std/stdio.nepl`: 1 byte を stdout に出す public API がなく、example 側が一時 `str` layout を raw memory で組み立てていた。

### 問題

byte VM のような実用的なサンプルを public stdlib API だけで書くための基本操作が不足していました。そのため example が collection や string の内部表現を直接扱い、現在推奨したい所有権・抽象化境界と矛盾していました。

### 影響

example が低レベル memory helper の利用例になってしまい、`Vec` / `str` / stdio の public API 境界が弱くなります。内部 layout の変更で example が壊れやすく、move checker の所有権規則も raw memory 操作で迂回されます。

### 修正方針

`Vec` には `.T: Copy` 限定の `replace_ref` を追加し、owning handle を消費せずに範囲内要素を上書きできるようにしました。`alloc/string` には `byte_at` を追加し、byte index の範囲チェックを `Option` で返すようにしました。`std/stdio` には `print_byte` を追加し、1 byte 出力を stdio 側へ集約しました。

### 対応結果

`examples/bf.nepl` は tape / jump table を `Vec<i32>`、命令読み取りを `s::byte_at`、byte 出力を `print_byte` で行えるようになりました。example 側から `core/mem` と raw allocation を除去しました。

### 検証

確認済み:

- `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec.nepl -n 38`
- `node nodesrc/run_doctest.js -i stdlib/alloc/string.nepl -n 5`
- `node nodesrc/run_doctest.js -i stdlib/std/stdio.nepl -n 1`
- `node nodesrc/run_doctest.js -i stdlib/tests/vec.n.md -n 1`
- `node nodesrc/tests.js -i stdlib/tests/string.n.md -i tests/stdlib/string.n.md -i tests/stdlib/stdout.n.md --no-tree -o tmp/string-stdout-api-tests.json -j 4` (`total=28`, `passed=28`, `failed=0`)

## RV-STDLIB-016: Stack push が所有権を消費し example で panic helper 回避を妨げる

- 解決済: true
- 状態: verified
- 優先度: P1
- 種別: architecture
- 対象: `stdlib/alloc/collections/stack.nepl`, `examples/rpn_legacy.nepl`

### 根拠

- `stdlib/alloc/collections/stack.nepl`: public な `push` は `Stack<.T>` を by-value で受け取り、`Result<Stack<.T>, Diag>` を返す。
- `examples/rpn_legacy.nepl`: stack へ数値や演算結果を積む箇所で、`push` の `Err` 側に入った時点で元の stack handle を保持できないため、`unwrap_ok` で panic するしかない構造になっていた。
- `stdlib/core/result.nepl`: `unwrap_ok` は unsafe helper として説明されており、通常コードは `match` / `unwrap_or` などを使う方針になっている。

### 問題

`Stack` の更新 API が consuming `push` だけだと、example 側は失敗時に stack を `free` しながら処理を継続できません。example から低レベル memory layout を直接触る方向へ戻すと所有権と抽象化を壊すため、stdlib 側に借用更新 API が必要でした。

### 影響

小さな REPL example でも allocation failure が panic path になり、`Result` を明示的に扱う現在の書き方の例として不適切になります。さらに、他の `Stack<i32>` 利用 example でも同じ制約が残り、`unwrap_ok` 除去が局所的な置換では進められません。

### 修正方針

`Stack` に `push_ref <.T: Copy> <(&Stack<.T>,.T)*>Result<(), Diag>>` を追加します。内部 layout と容量拡張は既存の `push` と同じですが、stack handle の所有権を移動させず、失敗時も呼び出し側が同じ handle を `free` できるようにします。対象を `.T: Copy` に限定し、失敗時の item 所有権回収を必要としない値だけを扱います。

### 対応結果

`stdlib/alloc/collections/stack.nepl` に `push_ref` と doctest を追加し、ファイル先頭コメントにも API の用途を追記しました。`rpn_legacy.nepl` はこの API を使って `push` failure を `match` で扱えるようにしました。

### 検証

確認済み:

- `trunk build`
- `node nodesrc/tests.js -i stdlib/alloc/collections/stack.nepl --no-tree -o tmp/stack-push-ref-tests.json -j 2` (`total=15`, `passed=15`, `failed=0`)
- `node nodesrc/tests.js -i examples/rpn_legacy.nepl --no-tree -o tmp/rpn-legacy-no-unwrap-tests.json -j 2` (`total=1`, `passed=1`, `failed=0`)
- `node nodesrc/tests.js -i examples --no-tree -o tmp/examples-rpn-legacy-push-ref.json -j 4` (`total=12`, `passed=12`, `failed=0`)

## RV-STDLIB-017: Vec に固定長初期化 API がなく example が panic helper に依存する

- 解決済: true
- 状態: verified
- 優先度: P1
- 種別: architecture
- 対象: `stdlib/alloc/collections/vec.nepl`, `examples/bf.nepl`

### 根拠

- `examples/bf.nepl`: tape と jump table を `with_capacity` + `push` loop で初期化し、`unwrap_ok<Vec<i32>, StdErrorKind>` で失敗を panic にしていた。
- `stdlib/alloc/collections/vec.nepl`: 固定長 buffer を同じ値で埋めて `Result<Vec<T>, StdErrorKind>` として返す public API がなかった。
- `Vec::push` は consuming API なので、example 側だけで `match` へ置き換えると `Err` 側で途中の Vec handle を扱いづらい。

### 問題

Brainfuck の tape / jump table のような固定長初期化は stdlib の代表的な用途ですが、既存 API だけでは「確保して n 個の値で埋める」処理を panic helper なしで素直に書けませんでした。example に低レベル memory 操作を戻すのではなく、stdlib 側に用途に合った API が必要でした。

### 影響

大きめの VM example が `unwrap_ok` 依存を残し、`Result` を明示的に扱う現在の書き方と矛盾します。同様の table / buffer 初期化を行う将来の example でも同じ問題を繰り返します。

### 修正方針

`Vec` に `.T: Copy` 限定の `filled <(i32,.T)->Result<Vec<.T>, StdErrorKind>>` を追加します。`n <= 0` は空 Vec を返し、`n > 0` は必要な領域を 1 回確保して同じ値を順に書き込み、`len = cap = n` の Vec として返します。

### 対応結果

`stdlib/alloc/collections/vec.nepl` に `filled` と doctest を追加しました。`bf.nepl` は `make_i32_vec` を `Result` returning helper にし、tape / jump table 初期化の失敗を `out of memory` として処理できるようにしました。

### 検証

確認済み:

- `trunk build`
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec.nepl --no-tree -o tmp/vec-filled-tests.json -j 2` (`total=39`, `passed=39`, `failed=0`)
- `node nodesrc/tests.js -i examples/bf.nepl --no-tree -o tmp/bf-filled-tests.json -j 2` (`total=2`, `passed=2`, `failed=0`)
- `node nodesrc/tests.js -i examples --no-tree -o tmp/examples-bf-filled-tests.json -j 4` (`total=12`, `passed=12`, `failed=0`)

## RV-STDLIB-018: streamio の WASI doctest が trait bound 不一致と出力破損で失敗する

- 解決済: true
- 状態: verified
- 優先度: P1
- 種別: bug
- 対象: `stdlib/std/streamio.nepl`, `tests/stdlib/streamio.n.md`, `nodesrc/run_test.js`

### 根拠

- GitHub Actions run `24940960078` の `wasi-test` / `nmd-doctest` artifact で、`tests/stdlib/streamio.n.md` が5件失敗している。
- `doctest#2` と `doctest#12` は `error[D3069]: type does not satisfy trait bound 'StreamWritable'` で `write bytes0` が解決できない。
- `doctest#5` は `writeln` が `D3006`、pipe が `D3013`、`close` が `D3016` で失敗している。
- `doctest#6` は expected `line1\nline2` に対して actual が `" \u0000\u0000\u0000\u000b\u0000\u0000\u0000\u0010\u0000\u0000"`、`doctest#7` は expected `text via read` に対して actual が `"\r\u0000\u0000\u0000\u0010\u0000\u0000\u0000 rea\u0000"` になり、文字列ではなく raw layout 由来の bytes が stdout に混入している。

### 問題

streamio は trait bound 解決と runtime I/O の両方で壊れています。compile failure は `StreamWritable` の impl / marker / import のいずれかが現在の型推論と噛み合っていないことを示し、runtime stdout mismatch は `str` / byte buffer / stream write の境界で内部 layout をそのまま出力している可能性を示します。

### 影響

標準 I/O wrapper が信頼できず、tutorial や競技プログラミング向け I/O の検証が止まります。`stdio.read_all` や UTF-8 保証の issue と関連する可能性がありますが、streamio 固有の trait 解決と出力破損を先に最小再現へ分ける必要があります。

### 修正方針

まず `tests/stdlib/streamio.n.md` の5失敗を compile と runtime に分けます。compile 側は `StreamWritable` impl の対象型、import clause、generic bound の正規化を確認します。runtime 側は `write` / `writeln` が `str` の data pointer と length を正しく取り出しているか、`ByteBuf` と `str` を混同していないかを確認します。修正は `unwrap` や raw memory で覆わず、stream abstraction の境界で型と layout を明確にします。

### 検証

- `node nodesrc/tests.js -i tests/stdlib/streamio.n.md --no-tree -o tmp/streamio-rv-stdlib-018.json -j 1`
- `node nodesrc/tests.js -i tests/stdlib -o tmp/tests-stdlib-streamio-after.json -j 4`
- `node nodesrc/tests.js -i tests -o tmp/tests-current-streamio-after.json -j 4`

## RV-STDLIB-019: collection doctest の値ブロック末尾セミコロンが戻り値を unit にしている

- 解決済: false
- 状態: open
- 優先度: P0
- 種別: test
- 対象: `stdlib/alloc/collections/btreemap.nepl`, `stdlib/alloc/collections/btreeset.nepl`, `stdlib/alloc/collections/queue.nepl`, `stdlib/alloc/collections/ringbuffer.nepl`

### 根拠

GitHub Actions run `24943799653` の `stdlib-test` で、`btreemap.nepl::doctest#2-#7` が `D3004 type annotation mismatch (expected BTreeMap<i32,i32>, got unit)` で失敗しました。ローカルでも `nodesrc/run_doctest.js -i stdlib/alloc/collections/btreemap.nepl -n 2` で再現します。

該当 doctest は `let hm <BTreeMap<i32,i32>>:` の値ブロック末尾に `|> ... |> uwok;` のようなセミコロンを置いています。NEPL のブロックでは末尾セミコロン付き式は値を返さないため、ブロック全体の値が `unit` になり、`let` の型注釈と衝突します。

### 問題

collection doctest の一部が「値を返すブロック」と「文として終えるブロック」を混同しています。これは compiler の型検査が正しく検出しているfixture誤りであり、`RV-CORE-029` の pipe 試験簡約バグとは別に修正する必要があります。

### 影響

`BTreeMap` / `BTreeSet` / `Queue` / `RingBuffer` の doctest がまとまって compile failure になり、stdlib-test が赤くなります。collection API の実行時問題と、単純なfixture構文ミスが混ざって見えるため、後続の runtime trap 調査を妨げます。

### 修正方針

値ブロックとして collection を初期化している doctest から、ブロック末尾のセミコロンを削除します。ブロックの途中式や後続で値を捨てる文のセミコロンは維持し、戻り値が必要な箇所だけを修正します。

### 対応結果

`BTreeMap` / `BTreeSet` / `Queue` / `RingBuffer` の doctest コメントに残っていた、値ブロック末尾の `;` を削除しました。単行 `let x <T> ...;` や、後続で値を使わない通常文のセミコロンは維持しています。

### 検証

確認済み:

- `node nodesrc/tests.js -i stdlib/alloc/collections/btreemap.nepl -i stdlib/alloc/collections/btreeset.nepl -i stdlib/alloc/collections/queue.nepl -i stdlib/alloc/collections/ringbuffer.nepl --no-tree -o tmp/collection-semicolon-rv-stdlib-019.json -j 1` (`total=34`, `passed=34`, `failed=0`)
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-rv-stdlib-019.json -j 4` (`total=379`, `passed=357`, `failed=22`, `errored=0`)

stdlib 全体の残り22件は、値ブロック末尾セミコロンとは別原因です。`RV-STDLIB-020` から `RV-STDLIB-024` へ分解して追跡します。

## RV-STDLIB-020: Fenwick/SegmentTree doctest が D3016 expression left extra values で失敗する

- 解決済: true
- 状態: verified
- 優先度: P0
- 種別: test
- 対象: `stdlib/alloc/collections/vec.nepl`, `stdlib/alloc/collections/fenwick.nepl`, `stdlib/alloc/collections/segment_tree.nepl`, `stdlib/tests/fenwick.n.md`, `stdlib/tests/segment_tree.n.md`

### 根拠

`RV-STDLIB-019` 修正後の `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-rv-stdlib-019.json -j 4` で、Fenwick 7件と SegmentTree 7件が `error[D3016]: expression left extra values on the stack` で失敗していました。

最小再現の error span は Fenwick / SegmentTree ではなく `stdlib/alloc/collections/vec.nepl` の `vec_find_impl` と `vec_take_while_len_impl` を指していました。該当箇所は `vec_find_impl<.T> data len add idx 1 p` のように、再帰呼び出しの第三引数 `add idx 1` と後続引数 `p` の境界が曖昧な形でした。

### 問題

`vec` の再帰 helper は、関数呼び出し引数に `add idx 1` を inline で渡していました。prefix call の境界上、後続の関数値引数が余剰 stack value として残り、`#target std` の compile 時に Fenwick / SegmentTree の doctest まで巻き込んで失敗していました。

### 修正方針

`vec_fold_impl` / `vec_reduce_impl` / `vec_find_impl` / `vec_take_while_len_impl` で、次 index を `let next_idx <i32> add idx 1` として一度束縛し、再帰呼び出しには `next_idx` を渡します。同型の潜在不具合を残さないため、error span に出た2箇所だけでなく同じ書き方の recursive helper をまとめて修正します。

### 対応結果

`stdlib/alloc/collections/vec.nepl` の recursive helper 4箇所を、`idx + 1` を明示束縛してから再帰呼び出しへ渡す形に変更しました。Fenwick / SegmentTree 側の API や doctest の意味は変更していません。

### 検証

確認済み:

- `node nodesrc/tests.js -i stdlib/alloc/collections/vec.nepl -i stdlib/alloc/collections/fenwick.nepl -i stdlib/alloc/collections/segment_tree.nepl -i stdlib/tests/fenwick.n.md -i stdlib/tests/segment_tree.n.md --no-tree -o tmp/fenwick-segtree-rv-stdlib-020.json -j 1` (`total=53`, `passed=53`, `failed=0`)
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-rv-stdlib-020.json -j 4` (`total=379`, `passed=371`, `failed=8`, `errored=0`)

stdlib 全体の残り8件は `RV-STDLIB-021` から `RV-STDLIB-024` の範囲に残しています。

## RV-STDLIB-021: vec sort doctest が overload 解決不一致で失敗する

- 解決済: true
- 状態: verified
- 優先度: P1
- 種別: test
- 対象: `stdlib/alloc/collections/vec/sort.nepl`

### 根拠

`RV-STDLIB-019` 修正後の stdlib 全体テストで、`stdlib/alloc/collections/vec/sort.nepl::doctest#1` と `#2` が `D3021 type arguments do not match any overload`、`#3` が `D3006 no matching overload found` で失敗しています。

### 問題

`Vec` の `Result` 化や sort API の generic signature 変更後に、sort doctest の型引数または comparator の渡し方が現行 API とずれています。fixture 側だけの誤りか、sort API の公開 signature が使いにくい状態になっているかを切り分ける必要があります。

### 修正方針

`sort.nepl` の public signature と doctest の explicit type arguments を照合し、必要であれば doctest を現行 API に同期します。API 側の型引数順や comparator bound が不自然な場合は、利用側が安全に書ける形へ signature を調整します。

### 対応結果

`Vec::new` / `Vec::push` が `Result<Vec<T>, StdErrorKind>` を返す現行 API に合わせ、doctest の `unwrap_ok` 型引数を `StdErrorKind` に更新しました。破壊的 sort の後に検証する例は、owner を返す `sort_quick_ret` / `sort_merge_ret` を使って `sort_is_sorted` で確認する形へ変更しました。`sort_merge` 自体の usage doctest は、現行 pipe style で Vec を構築してから `sort_merge` を呼ぶ compile/run smoke として維持しています。

### 検証

確認済み:

- `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec/sort.nepl -n 1 --dist dist` (`pass`, `return_value=1`)
- `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec/sort.nepl -n 2 --dist dist` (`pass`, `return_value=1`)
- `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec/sort.nepl -n 3 --dist dist` (`pass`, `return_value=0`)
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/sort.nepl --no-tree -o tmp/vec-sort-rv-stdlib-021.json -j 1` (`total=3`, `passed=3`, `failed=0`)
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-rv-stdlib-021.json -j 4` (`total=379`, `passed=375`, `failed=4`, `errored=0`)

## RV-STDLIB-022: HashMap doctest にインデント不整合が残っている

- 解決済: true
- 状態: verified
- 優先度: P1
- 種別: test
- 対象: `stdlib/alloc/collections/hashmap.nepl`

### 根拠

`RV-STDLIB-019` 修正後の stdlib 全体テストで、`stdlib/alloc/collections/hashmap.nepl::doctest#3` が `error[D1206]: indentation is not aligned to #indent width` で失敗しています。

### 問題

hashmap の public doctest が、現行 parser の indent 幅ルールに合っていません。これは runtime 以前の fixture 品質問題で、HashMap API の実行時問題と混ざると原因調査を誤ります。

### 修正方針

該当 doctest の生成ソースを抽出し、`#indent` と本文 indent の不一致だけを直します。API の意味や検証内容は変えず、parser が受理する形に揃えます。

### 対応結果

`get` の doctest で `if` の続きに置いた `match` block の indent を `#indent 4` に合わせました。indent 修正後に同じ `HashMap` owner へ `get` を2回呼んでいる move error が露出したため、missing check 用の空 map と existing value check 用の map を分け、by-value API の所有権規則にも合う fixture にしました。

### 検証

確認済み:

- `node nodesrc/run_doctest.js -i stdlib/alloc/collections/hashmap.nepl -n 3 --dist dist` (`pass`, `return_value=1`)
- `node nodesrc/tests.js -i stdlib/alloc/collections/hashmap.nepl --no-tree -o tmp/hashmap-indent-rv-stdlib-022.json -j 1` (`total=7`, `passed=7`, `failed=0`)
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-rv-stdlib-022.json -j 4` (`total=379`, `passed=372`, `failed=7`, `errored=0`)

## RV-STDLIB-023: HashMap/HashSet の文字列 key runtime test が memory OOB と return mismatch で失敗する

- 解決済: false
- 状態: open
- 優先度: P0
- 種別: bug
- 対象: `stdlib/tests/hashmap_str.n.md`, `stdlib/tests/hashset.n.md`, `stdlib/tests/hashset_str.n.md`, `stdlib/alloc/collections/hashmap.nepl`, `stdlib/alloc/collections/hashset.nepl`

### 根拠

`RV-STDLIB-019` 修正後の stdlib 全体テストで、`stdlib/tests/hashmap_str.n.md::doctest#1` と `stdlib/tests/hashset_str.n.md::doctest#1` が `RuntimeError: memory access out of bounds`、`stdlib/tests/hashset.n.md::doctest#1` が return value mismatch で失敗しています。

### 問題

HashMap / HashSet の string key 経路または hashing / equality / ownership 境界で runtime が壊れています。compile fixture ではなく実行時の memory safety 問題なので、`unwrap` で覆うのではなく、key の格納・参照・hash 計算・drop 所有権のどこで invalid address になるかを特定する必要があります。

### 修正方針

まず int key と string key の差分を最小化し、hash function 入力の `str` layout、bucket / entry array の pointer、key copy / move の扱いを追跡します。runtime trap と return mismatch を同じ原因で説明できない場合は、さらに issue を分割します。

### 検証

- `node nodesrc/tests.js -i stdlib/tests/hashmap_str.n.md -i stdlib/tests/hashset.n.md -i stdlib/tests/hashset_str.n.md --no-tree -o tmp/hash-collections-rv-stdlib-023.json -j 1`
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-rv-stdlib-023.json -j 4`

## RV-STDLIB-024: Deserialize doctest の match arm が Result と unit で不一致になる

- 解決済: false
- 状態: open
- 優先度: P1
- 種別: test
- 対象: `stdlib/core/traits/deserialize.nepl`

### 根拠

`RV-STDLIB-019` 修正後の stdlib 全体テストで、`stdlib/core/traits/deserialize.nepl::doctest#1` が `error[D3045]: match arms have incompatible types: Result_T_E_unit_str and unit` で失敗しています。

### 問題

deserialize doctest の `match` が、成功/失敗のどちらかの arm で `Result<(), str>` を返し、別 arm で `unit` を返しています。これは error path を検証する意図が型として明確になっていない状態です。

### 修正方針

doctest の期待結果を確認し、全 arm が同じ型を返すように揃えます。エラーを返すテストなら `Result` を明示し、単に失敗時に異常終了したいテストなら `panic` / `unreachable` 相当の既存テスト helper を適切に使います。

### 検証

- `node nodesrc/tests.js -i stdlib/core/traits/deserialize.nepl --no-tree -o tmp/deserialize-rv-stdlib-024.json -j 1`
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-rv-stdlib-024.json -j 4`

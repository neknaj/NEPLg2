# NEPLg2.1 メモリ管理仕様

最終更新: 2026-03-16

---

## 1. 設計目標

1. GC を使わず、コンパイラ管理のみでメモリ安全性を確保する。
2. heap / 線形メモリ操作を Pure として扱える論理モデルを確立する（内部効果 `InternalAlloc` として区別し、surface では `Pure` に畳み込む）。
3. 公開 API を `Result`/`Option` 前提の安全 API に統一する。
4. Wasm と LLVM はコンパイラの安全意味論を共有するが、物理レイアウトの共有は不要。

---

## 2. 値の三分類

NEPLg2.1 はすべての値を意味論上、次の 3 種類に分類する。

### A. Pure persistent value（純粋永続値）

例: `str`, `List .T`, immutable `Tree .T`, immutable struct（`Pair .A .B`, `Triple .A .B .C` 等）

- 共有してよい
- manual free を持たない
- 領域（region）単位でコンパイラが回収する

### B. Unique mutable work state（一意可変作業状態）

例: `ByteBuf`, `VecBuilder .T`, `StringBuilder`, mutable scratch buffer

- 一意所有でのみ更新できる
- pure 関数の内部実装に使ってよい（外に漏れない限り）

### C. Linear capability / owned external resource（線形 capability・外部資源）

例: `File`, `Socket`, `RegionToken`, `WriterToken`

- 必ず 1 回ずつ消費・返却・close/drop される

---

## 3. 二段構えの自動メモリ管理（GC なし）

### 3.1 Region Inference（領域推論）

対象: pure persistent value（`List .T`, immutable tree, `str` 内部表現, closure environment の pure 部分, map/filter/fold の一時 aggregate）

- コンパイラが region を推論し、region 単位で bulk free する
- ノード単位の free はしない
- ソースに region 構文は見せない
- プログラマは alloc/dealloc を書かない

### 3.2 Drop Elaboration（drop 展開）

対象: owned / linear resource（`File`, `Socket`, `OwnedBuf .T`, `VecBuilder .T`, `StringBuilder`, `RegionToken` など）

- scope exit / overwrite 時に自動 drop を挿入
- 初期化状態を dataflow で追い、条件付き drop を生成

---

## 4. 所有権・借用・線形性の規則

### 4.1 Ownership 規則

1. `Copy` 型は move でなく copy される。
2. `Drop` 型は scope exit / overwrite / early return の各点でコンパイラが drop 候補を生成する。
3. owned value は move 後に使えない。
4. owned value を共有 borrow 中に mutable access してはならない。
5. mutable borrow 中は他の access を禁止する。

### 4.2 Linear 規則

1. `RegionToken`, `BuilderToken`, `File`, `Socket` などは affine / linear resource。
2. close / drop / free は token を消費する。
3. 線形資源は複製できない。
4. 線形資源を返さずに終わる経路があれば compiler error（scope exit drop が自動挿入される場合を除く）。

---

## 5. `core/mem` の位置づけ

`core/mem` は safe surface ではなく **compiler / runtime 境界モジュール**。

- `MemPtr .T` や raw load/store は safe user code から見えない
- safe 側に公開するのは抽象型のみ: `str`, `ByteBuf`, `Slice .T`, `OwnedBuf .T`, `List .T`, `Vec .T`, `File`, `Socket`
- raw pointer・raw load/store は `unsafe` 層または compiler/runtime 層にのみ存在

---

## 6. 文字列仕様

### 6.1 `str` の意味論

`str` は immutable UTF-8 text。Pure persistent value であり、共有可能・manual free を持たない。物理表現は target ごとに異なってよいが、言語意味論として「UTF-8 妥当な不変文字列」で統一する。

### 6.2 文字列とバイト列の分離

| 型 | 意味 |
|----|------|
| `str` | UTF-8 保証された immutable text |
| `ByteBuf` | arbitrary bytes を持つ owned buffer |
| `StringBuilder` | text 構築用の unique mutable work state |

### 6.3 `StringBuilder`

`std::string_builder` モジュールに bare 名で提供する。

```nepl
let new %fn unit -> StringBuilder \ :
let push_str %fn StringBuilder str -> StringBuilder \ builder s :
let finish %fn StringBuilder -> str \ builder :
```

`finish` は builder を消費し、以後 builder は使えない。internal effect は `InternalAlloc` だが surface では `Pure`。

### 6.4 変換規則

```nepl
// ByteBuf → str（UTF-8 検査あり）
let to_str %fn ByteBuf -> Result str Utf8Error \ buf :

// str → ByteBuf（コピー）
let to_bytes %fn str -> ByteBuf \ s :
```

---

## 7. List 仕様

### 7.1 `List .T` は pure persistent list

- `cons`, `head`, `tail`, `map`, `fold`, `reverse` は pure
- `List .T` のノードは region-managed
- `free List .T` は公開 API から削除する
- manual node-by-node free は禁止

### 7.2 回収方法

`List .T` ノードは region inference により region 単位で解放する。ノードごとの free はしない。

### 7.3 builder

効率化が必要なら `ListBuilder .T` を別に置く。

- `ListBuilder .T` は unique mutable work state
- `push_front`, `push_back`, `finish` を持つ
- `finish` は builder を消費して immutable `List .T` を返す

---

## 8. IO 仕様

### 8.1 IO は必ず Impure

FS, STDIO, NETWORK, CLOCK, RANDOM, ENV, PROCESS はすべて `Impure`。

### 8.2 resource model

| 種別 | 例 | 扱い |
|------|-----|------|
| runtime-borrowed capability | `stdin`, `stdout`, `stderr` | close 不可、platform capability |
| owned external resource | `File`, `Socket` | owned linear resource、close で消費 |

### 8.3 推奨 API 形（consume-return handle）

```nepl
let Mode enum:
    Read
    Write
    Append

let open  %fn* Path Mode -> Result File IoError \ path mode :
let read  %fn* File -> Result Pair File str IoError \ file :
let write %fn* File str -> Result File IoError \ file text :
let flush %fn* File -> Result File IoError \ file :
let close %fn* File -> Result unit IoError \ file :
```

consume-return handle 方式は ownership の実装が単純で、linearity と整合しやすい。

**失敗時の所有権**:

- `read`/`write`/`flush` は `Err IoError` 側に `File` を返さない。I/O 失敗時、`File` ハンドルは**消費済みとなり再利用不可**。
- `close` も同様。`Err IoError` の場合、OS 側でのクローズは保証されないが、言語側の所有権は消費される（二重 close を防ぐため）。
- この設計により「エラー後に handle を使い続けてしまう」バグを型レベルで防ぐ。
- **リトライが必要な API** を作る場合は、`Err` 側にも handle を戻す別シグネチャを定義する:

```nepl
// リトライ可能な read（失敗時も File を返す）
let try_read %fn* File -> Result Pair File str Pair File IoError \ file :
```

標準 API は「失敗後の retry を想定しない」設計を基本とする。

---

## 9. Escape Analysis（逸出解析）

pure function の内部メモリ操作を pure と扱うために必要:

- raw pointer / internal handle が戻り値に含まれない
- global / outer scope へ書き込まれない
- closure capture によって外へ持ち出されない
- borrowed alias が内部 mutable state を指さない

---

## 10. unsafe 境界

safe NEPLg2 では次を禁止する:

- raw address の観測
- raw load / store
- unchecked cast による resource forgery
- manual free of persistent values

これらは `unsafe` 層または compiler / runtime 層にだけ存在してよい。

---

## 11. マルチターゲットでの安全意味論

コンパイラは Wasm であっても LLVM であっても**全く同じ安全意味論の検査**（型検査・Resource IR 解析・Region Inference・Drop Elaboration）を実行する。コンパイラ内部に「Wasm 固有の解析パス」や「LLVM 固有の検査規則」は配置しない。

| 項目 | Wasm 向け | LLVM 向け |
|------|-----------|-----------|
| pointer 表現 | linear memory offset（`i32` にラップ） | native pointer |
| allocator | `core/mem` 内の bump + free_list 実装 | libc `malloc`/`free` の FFI |
| `str` header | `[len:i32][data...]`（linear memory 上） | native `{ptr, len}` 構造体 |
| file/socket | WASI API と `fd`（`i32`） | POSIX / OS native API |

物理レイアウト差は標準ライブラリの `#if[target="..."]` で吸収する。

---

## 12. 将来拡張（Phase 8）への接続

値が型の中に入る依存型では、その値が「実行中に書き換えられない」ことが必須要件。Pure Persistent Value による不変性保証と GC レスのメモリ安全モデルは、形式証明との統合に必要な条件を構造的に満たしている（詳細は [phase8.md](./phase8.md)）。

将来的に以下を導入可能とするが、メモリ安全の最低保証を依存型に委ねてはならない:

```nepl
// Phase 8 例: 長さをコンパイル時に追跡する Vec
let Vec struct .T .n:
    ...

// 境界チェックなしアクセス（証明必須）
let get .T .len .idx %fn Vec .T .len .idx -> .T
    where %IsLess .idx .len
    \ vec index :
    ...
```

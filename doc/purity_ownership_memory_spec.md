# 純粋性・所有権・メモリ管理 統合仕様

最終更新: 2026-03-15

> この文書は `doc/memory_safety_compiler_design.md` と `doc/move_effect_spec.md` を包含・発展させた統合仕様である。
> 設計の出発点と理論背景は `doc/chat/dump/mem1.md` を参照。

## 1. 設計目標

### 1.1 基本理念

NEPLg2 は次の 4 原則を満たす。

1. 外部観測可能な副作用を持たない関数は **Purity** (純粋性) を持つ。
2. 資源の所有・移動・借用は静的に検査する。
3. GC は使わず、必要な alloc/free/drop はコンパイラが自動挿入する。
4. Wasm と LLVM は同じ安全意味論を共有するが、同じメモリレイアウトを共有する必要はない。

### 1.2 非目標

- **依存型**: 今回は導入しない。将来、長さ付き buffer や protocol state の証明として追加できるよう、依存型なしで sound であることを優先する。
- **GC**: 導入しない。
- **未定義動作**: 安全でない操作は `Result/Option` で表現する。
- **旧 API との後方互換**: 維持しない。

---

## 2. 用語と責務の分離

### 2.1 3 つの判断体系

改良後 NEPLg2 では、式に対する静的判断を 3 軸に分離する。

| 判断 | 記法 | 本質 |
|------|------|------|
| **Typing** | `Γ ⊢ e : τ` | 通常の型付け |
| **Effect** | `Γ ⊢ e ! ε` | 外界観測可能な副作用 |
| **Resource Usage** | `Γ ⊢ e ▷ σ` | move / borrow / drop / region に関わる資源使用 |

重要: **Purity** と **Ownership** と **Linearity** は同一視しない。

### 2.2 Ownership と Linearity の違い

- **Ownership** (所有権): 「誰がその資源の解放責任を持つか」「他に危険な別名が存在しないか」を扱う。
- **Linearity** (線形性): 「その値を何回使ってよいか」を扱う。

NEPLg2 での使い分け:

- 永続値の sharing → pure value semantics
- builder や mutable buffer の一意性 → ownership / uniqueness
- token や capability の「必ず 1 回使い切り」 → linearity

### 2.3 Purity と Ownership は別軸

pure function とは、外界に対して観測可能な副作用を持たない関数を指す。「内部で一切 mutation をしない」ことを意味しない。内部メモリ操作がある関数でも、その操作が外から観測できなければ pure とみなせる。

---

## 3. 値の 3 分類

NEPLg2 の値は意味論上、次の 3 種類に分類する。

### A. Pure persistent value (純粋永続値)

例: `str`, `List<T>`, immutable `Tree<T>`, immutable struct (`Pair<A,B>`, `Triple<A,B,C>` 等)

- 共有してよい
- manual free を持たない
- 領域 (region) 単位で compiler が回収する

### B. Unique mutable work state (一意 mutable 作業状態)

例: `ByteBufBuilder`, `VecBuilder<T>`, `StringBuilder`, mutable scratch buffer

- 一意所有でのみ更新できる
- pure 関数の内部実装に使ってよい (外に漏れない限り)

### C. Linear capability / owned external resource (線形 capability / 外部資源)

例: `File`, `Socket`, `RegionToken`, `WriterToken`

- 必ず 1 回ずつ消費・返却・close/drop される

---

## 4. 表面効果と内部効果

### 4.1 surface effect

表面言語の関数効果は当面 `Pure | Impure` の 2 値。

- `->`: Pure
- `*>`: Impure

### 4.2 compiler 内部効果

compiler 内部では少なくとも次の分類を持つ。

| 内部効果 | surface への畳み込み |
|----------|---------------------|
| `Pure` | → `Pure` |
| `InternalAlloc` | → `Pure` |
| `ExternalIO` | → `Impure` |
| `Nondet` | → `Impure` |
| `Unsafe` | → `Impure` |

これにより「内部 scratch memory を使う pure function」は許しつつ、FS/STDIO/NETWORK/clock/random は必ず impure にできる。

### 4.3 関数が Pure であるための条件

1. I/O、時刻、乱数、raw foreign call を行わないこと
2. 関数内で確保したメモリの raw address や backend 依存 handle が結果として外へ漏れないこと
3. 内部 mutation が uniqueness / ownership によって一意に管理され、外部 alias と競合しないこと
4. 関数終了時に内部 scratch memory が compiler によって完全に回収されること

---

## 5. 能力と型の関係

### 5.1 `Copy/Clone/Drop` は compiler-known capability

| 能力 | 意味 |
|------|------|
| `Copy` | read が move でなく copy になる |
| `Clone` | 明示複製が許される |
| `Drop` | scope exit / overwrite / early return で drop 候補になる |
| `Linear` | implicit copy も implicit discard も不可 |
| `Owned` | implicit discard は drop elaboration でのみ可能 |

### 5.2 種別ごとの既定

| 型 | resource usage |
|----|----------------|
| `i32`, `u8`, `bool`, `f32`, unit, label | `Unrestricted` |
| `str`, `List<T>`, immutable tree, immutable struct (`Pair`, `Triple` 等) | `Unrestricted` (region-managed) |
| `OwnedBuf<T>`, `VecBuilder<T>`, `File`, `Socket` | `Owned` |
| `RegionToken`, `BuilderToken`, `CloseToken` | `Linear` |

### 5.3 型の能力の合成則

複合型（Tuple, Struct, Enum など）の resource usage は、その構成要素の中で最も厳しい能力に引きずられる。

1. 要素に `Linear` 型が1つでも含まれる場合、複合型全体が `Linear` となる。
2. 要素に `Owned` 型が含まれ、かつ `Linear` が含まれない場合、複合型全体が `Owned` となる。
3. 全ての要素が `Unrestricted` である場合のみ、複合型全体が `Unrestricted` になる。

例:
- `(i32, str)` → `Unrestricted`
- `(File, i32)` → `Linear` （`File` 自体が `Linear` な外部資源であるため）
- `(OwnedBuf<u8>, str)` → `Owned`

制約の伝播順序: `Linear` > `Owned` > `Unrestricted`

---

## 6. 所有権・借用・線形性の規則

### 6.1 Ownership 規則

1. `Copy` 型は move でなく copy される。
2. `Drop` 型は scope exit, overwrite, early return の各点で compiler が drop 候補を生成する。
3. owned value は move 後に使えない。
4. owned value を共有 borrow 中に mutable access してはならない。
5. mutable borrow 中は他の access を禁止する。

### 6.2 Linear 規則

1. `RegionToken`, `BuilderToken`, `File`, `Socket` などは affine/linear resource。
2. close/drop/free は token を消費する。
3. 線形資源は複製できない。
4. 線形資源を返さずに終わる経路があれば compiler error (scope exit drop が自動挿入される場合を除く)。

### 6.3 `set` の新規則

現在の「局所なら pure」は廃止し、次の条件を満たすときのみ内部 mutation を pure とみなす:

- 更新対象が unique local state である
- その状態への参照が外へ escape しない
- 共有 borrow が存在しない
- 更新の結果が観測可能な raw identity を漏らさない

### 6.4 変数状態の追跡

move / borrow check は各変数について以下の状態を追跡する:

- `Live`: 初期化済み、使用可能
- `Moved`: move 済み、再使用不可
- `MaybeMoved`: 条件分岐により不定
- `Uninitialized`: 未初期化
- `BorrowedShared`: 共有 borrow 中（mutable access 禁止）
- `BorrowedUnique`: 一意 borrow 中（他の全 access 禁止）

分岐合流とループで状態を保守的にマージする。

診断:
- use-after-move (5001)
- borrow conflict: mutate while borrowed (5007)
- borrow conflict: access while uniquely borrowed (5008)
- linear value not consumed (5005)
- linear value may not be consumed on all paths (5006)

---

## 7. メモリ管理仕様

### 7.1 二段構えの自動メモリ管理

GC を用いず、コンパイラが次の 2 機構で alloc/free を自動挿入する。

#### A. Region Inference (領域推論)

対象: pure persistent value (`List<T>`, immutable tree, `str` 内部表現, closure environment の pure 部分, map/filter/fold の一時 aggregate)

- compiler が region を推論し、region 単位で bulk free する
- ノード単位の free はしない
- source に region 構文は見せない

#### B. Drop Elaboration (drop 展開)

対象: owned / linear resource (`File`, `Socket`, `OwnedBuf<T>`, `VecBuilder<T>`, `StringBuilder`, `RegionToken` など)

- scope exit / overwrite 時に自動 drop を挿入
- 初期化状態を dataflow で追い、条件付き drop を生成

### 7.2 `core/mem` の位置づけ

`core/mem` は safe surface ではなく **compiler/runtime 境界モジュール** とする。

- `MemPtr<T>` や `mem_ptr_addr` のような raw representation は safe user code からは見えない
- safe 側に公開するのは抽象型のみ: `str`, `ByteBuf`, `Slice<T>`, `OwnedBuf<T>`, `List<T>`, `Vec<T>`, `File`, `Socket`
- raw pointer や raw load/store は `unsafe` 層にのみ存在

### 7.3 Escape Analysis (逸出解析)

pure function の内部メモリ操作を pure と扱うために必要:

- raw pointer / internal handle が戻り値に含まれない
- global / outer scope へ書き込まれない
- closure capture によって外へ持ち出されない
- borrowed alias が内部 mutable state を指さない

---

## 8. 文字列仕様

### 8.1 `str` の意味論

`str` は immutable UTF-8 text。pure persistent value であり、共有可能、manual free を持たない。物理表現は target ごとに異なってよいが、言語意味論として「UTF-8 妥当な不変文字列」で統一する。

### 8.2 文字列とバイト列の分離

| 型 | 意味 |
|----|------|
| `str` | UTF-8 保証された immutable text |
| `ByteBuf` | arbitrary bytes を持つ owned buffer |
| `StringBuilder` | text 構築用の unique mutable work state |

### 8.3 `StringBuilder`

- `builder_new : () -> StringBuilder`
- `builder_push_str : (StringBuilder, str) -> StringBuilder`
- `builder_finish : (StringBuilder) -> str`

`builder_finish` は builder を消費し、以後 builder は使えない。internal effect は `InternalAlloc` だが surface では `Pure`。

### 8.4 変換規則

- `bytes_to_str : ByteBuf -> Result<str, Utf8Error>` は pure
- `str_to_bytes : str -> ByteBuf` は pure
- 内部 ByteBuf mutation を完全に内部に閉じ込めて `str` のみを返す関数は pure にしてよい

---

## 9. List 仕様

### 9.1 `List<T>` は pure persistent list

- `cons`, `head`, `tail`, `map`, `fold`, `reverse` は pure
- `List<T>` のノードは region-managed
- **`free(List<T>)` は公開 API から削除する**
- manual node-by-node free は禁止

### 9.2 回収方法

`List<T>` ノードは region inference により region 単位で解放する。ノードごとの free はしない。

### 9.3 builder

効率化が必要なら `ListBuilder<T>` を別に置く。

- `ListBuilder<T>` は unique mutable work state
- `push_front`, `push_back`, `finish` を持つ
- `finish` は builder を消費して immutable `List<T>` を返す

---

## 10. IO 仕様

### 10.1 IO は必ず Impure

FS, STDIO, NETWORK, CLOCK, RANDOM, ENV, PROCESS はすべて `Impure`。

### 10.2 `std/io` facade の継承

現行の facade 方式を維持し、bare 名 `read`, `write`, `flush`, `close` で統一する。

### 10.3 resource model

IO の資源は 2 種類:

| 種別 | 例 | 扱い |
|------|-----|------|
| runtime-borrowed capability | `stdin`, `stdout`, `stderr` | close 不可、platform capability |
| owned external resource | `File`, `Socket` | owned linear resource、close で消費 |

### 10.4 推奨 API 形 (consume-return handle)

```
open_read  : Path -> Result<File, IoError>
open_write : Path -> Result<File, IoError>
read_all_text : File -> Result<(File, str), IoError>
write_text : (File, str) -> Result<File, IoError>
flush : File -> Result<File, IoError>
close : File -> Result<(), IoError>
```

consume-return handle 方式は ownership の実装が単純で、linearity と整合しやすい。

### 10.5 effect の宣言的判定

「文字列マーカーで impure」方式から「primitive に `ExternalIO` を宣言して compiler が読む」方式へ移行する。

---

## 11. Compiler 実装仕様

### 11.1 Resource IR

typed HIR の後ろに **Resource IR** (資源 IR) を置く。CFG を持ち、以下を明示する:

- `move x -> y`
- `borrow_shared x -> b`
- `borrow_unique x -> b`
- `region_new ρ`
- `region_alloc ρ, n`
- `region_end ρ`
- `drop x`
- `io_open path`
- `io_write h, data`
- `io_close h`

この IR 上で次を診断する:

- use-after-move
- double free
- use-after-free
- borrow conflict
- leaked linear token
- unclosed external resource

### 11.2 解析パス順

1. surface typecheck
2. effect attribution
3. Resource IR 生成
4. ownership / borrow check
5. region inference
6. drop elaboration
7. target lowering

### 11.3 target lowering

| target | 方式 |
|--------|------|
| Wasm | linear memory ベース |
| LLVM | native pointer / native allocator ベース |

共通化すべきは「安全意味論」であって「レイアウト」ではない。

---

## 12. Wasm / LLVM でのプラットフォーム差異吸収

NEPLg2 コンパイラは、ターゲットが Wasm であっても LLVM であっても **全く同じ安全意味論の検査（型検査、Resource IR 解析、Region Inference、Drop Elaboration）を実行します。**
コンパイラ内部に「Wasm 固有の解析パス」や「LLVM 固有の検査規則」は配置しません。

### 12.1 コンパイラの責務（安全意味論の共通化）

コンパイラはターゲットに依存せず、以下の安全性を Resource IR 上で静的に証明します。

- moved value の再使用禁止
- borrowed place への不正 mutation 禁止
- freed resource の再使用禁止
- pure / impure の境界
- `str`, `List`, `OwnedBuf`, `File`, `Socket` の source semantics

各バックエンド（Wasm/LLVM）は、この完全に検証された IR を受け取り、純粋に **物理的な命令とレイアウトの生成（Target Lowering）** のみを行います。

### 12.2 標準ライブラリの責務（物理レイアウトの分岐）

プラットフォーム間の物理的な差異（ポインタ表現、アロケータ、ハンドル表現など）は、NEPLソースコード側（`core` / `std` などの標準ライブラリ）で `#if[target="..."]` を用いて吸収します。

| 項目 | Wasm 向け実装 (`#if[target="wasm"]`) | LLVM 向け実装 (`#if[target="llvm"]`) |
|------|--------------------------------------|--------------------------------------|
| pointer 表現 | linear memory offset (`i32` などにラップ) | native pointer |
| allocator | `core/mem` 内の bump + free_list 実装 | libc `malloc`/`free` の FFI 呼び出し |
| `str` header | `[len:i32][data...]` (linear memory 上) | native `{ptr, len}` 構造体 |
| file/socket | WASI API 呼び出しと `fd` (`i32`) | POSIX / OS ネイティブ API 呼び出し |

このように、コンパイラは単一の厳格なルールの下でコードを検査し、ターゲットごとの型の実態や関数呼び出しの差異は**ユーザーランド（標準ライブラリ）の条件付きコンパイル**によって解決します。

---

## 13. unsafe 境界

safe NEPLg2 では次を禁止する:

- raw address の観測
- raw load/store
- unchecked cast による resource forgery
- manual free of persistent values

これらは `unsafe` 層または compiler/runtime 層にだけ存在してよい。

---

## 14. 依存型への将来拡張

将来的に以下を導入可能とするが、メモリ安全の最低保証を依存型に委ねてはならない:

- `ByteBuf<n>`, `Vec<T, n>`, `Utf8(bytes)`, `Socket<State>`, `File<Mode>`

---

## 15. 実装優先順位

### Phase 1: 基盤修正

1. `alloc/dealloc/realloc` を surface `Pure` から外す
2. `MemPtr<T>` の raw address 露出を safe API から消す
3. `List<T>` の public `free` を削除して persistent list に固定
4. `std/io` の effect を宣言的にする
5. Resource IR と ownership pass を入れる

### Phase 2: 型・API 分離

1. `StringBuilder` / `ByteBuf` / `str` の分離
2. `File` / `Socket` の owned resource 化
3. `ListBuilder<T>` の導入
4. region inference の first version
5. Wasm/LLVM の表現分離

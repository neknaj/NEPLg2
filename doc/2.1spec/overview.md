# NEPLg2.1 言語概要

最終更新: 2026-03-16

---

## 1. 言語の三本柱

NEPLg2.1 は次の三つを同時に満たす言語基盤を目指す。

1. **どこでも動く（マルチプラットフォーム）**
   一つのプログラムを書けば、WebAssembly・WASI・ネイティブバイナリのいずれのターゲットでも等価な安全意味論で動作する。ターゲット差分は標準ライブラリ内の条件付きコンパイル（`#if[target="..."]`）で吸収し、コンパイラの検査規則はすべてのターゲットで共通。

2. **安全である（型安全・メモリ安全）**
   GC を使わず、コンパイラが所有権・借用・純粋性・資源解放を静的に証明する。実行時の未定義動作をゼロにし、失敗しうる操作はすべて `Result`/`Option` で型に反映する。

3. **高速である**
   GC オーバーヘッドなし。Region Inference と Drop Elaboration による決定論的なメモリ管理。純粋関数の内部 scratch memory は Pure のまま扱える。コンパイル時に証明可能な安全性はランタイム検査を省略できる。

---

## 2. 設計原則

### 2.1 前置記法・括弧なし

すべての関数呼び出しは前置 juxtaposition で書く。括弧によるグループ化は式文脈に存在しない。

```nepl
add 1 2           // add(1, 2) に相当
mul add 1 2 3     // mul(add(1, 2), 3)  — 型情報で境界を決定
```

型式も同じ原則に従う。型コンストラクタの kind が arity に相当し、kind-directed アルゴリズムが境界を決定する。

```nepl
Result i32 str    // Result<i32, str> に相当
Vec Option i32    // Vec<Option<i32>> に相当
```

### 2.2 式指向

`if`・`match`・`while`・`let`・`set`・ブロックはすべて式。`while` は Phase 0–7 では `unit` を返す（Phase 8 以降は「少なくとも 1 回実行される」という証明付きで本体の型 `T` を返せる）。

```nepl
let grade
    if ge score 90: "A"
    else if ge score 70: "B"
    else: "C"
```

### 2.3 オフサイドルール（インデントベースブロック）

ブロックは `:` の後のインデントで表現する。閉じ括弧はない。DSL やマルチシンタクスの埋め込みにも干渉しない。

### 2.4 パイプ演算子

中値演算子は `|>` のみ。関数型スタイルの連鎖を読みやすく書ける。

```nepl
input |> parse |> validate |> save
```

### 2.5 `let` への統一

すべての宣言（関数・struct・enum・trait・impl）は `let` キーワードで書く。

```nepl
let add %fn i32 i32 -> i32 \ a b :
    add_impl a b

let Point struct:
    x %i32
    y %i32
```

### 2.6 強力な静的検査

- 型検査・kind 検査・効果検査・所有権検査・借用検査・線形性検査をすべてコンパイル時に行う。
- 網羅性検査（match の全バリアントカバレッジ）もコンパイル時。
- 失敗しうる操作は `Result`/`Option` で型に現れる。実行時パニックは存在しない。

### 2.7 bare 名 API・モジュール分離

API 名には prefix/suffix を付けない（`get_opt`・`get_safe` のような命名は採用しない）。同じ bare 名を Core 層と Casual 層の別モジュールに置き、`use` の切り替えで使い分ける。

---

## 3. 値の三分類

NEPLg2.1 はすべての値を三種類に大別する。コンパイラはこの分類に基づいてメモリ管理・所有権検査を行う。

| 分類 | 代表例 | 特徴 |
|---|---|---|
| **Pure Persistent Value**（純粋永続値） | `str`, `List .T`, immutable struct | 共有可能・不変・Region Inference で自動回収 |
| **Unique Mutable Work State**（一意可変作業状態） | `StringBuilder`, `ByteBuf` | 一意所有で更新・純粋関数の内部実装に使用可 |
| **Linear Capability**（線形 capability） | `File`, `Socket`, `RegionToken` | 必ず 1 回消費または close/drop |

---

## 4. 副作用の二値分類

関数の副作用は表面言語では `Pure`/`Impure` の 2 値で表す。

| 記法 | 意味 |
|---|---|
| `%fn ... -> ...` | Pure（外部観測可能な副作用なし） |
| `%fn* ... -> ...` | Impure（I/O・ファイルシステム・乱数等を含む） |

内部メモリ操作（`InternalAlloc`）は raw address が外部に漏れない限り Pure に畳み込まれる。

---

## 5. GC なしのメモリ管理

| 機構 | 対象 | 動作 |
|---|---|---|
| **Region Inference** | Pure persistent value | compiler が生存域を推論し、スコープ出口で一括解放 |
| **Drop Elaboration** | Owned / Linear resource | スコープ出口・上書き時に compiler が自動 drop 挿入 |

プログラマが `alloc`/`dealloc` を書く必要はない。

---

## 6. マルチターゲット

| ターゲット | 特徴 |
|---|---|
| `wasm` | WASI import なしの純粋 WebAssembly |
| `wasi` | WASI syscall 付き WebAssembly |
| `llvm` | LLVM 経由のネイティブバイナリ |

コンパイラの安全意味論検査はすべてのターゲットで同一。物理レイアウト差（ポインタ長・アロケータ）は標準ライブラリの `#if[target="..."]` で吸収。

---

## 7. 将来拡張：依存型（Phase 8）

現フェーズ（Phase 0–7）では依存型を導入しない。メモリ安全・副作用分離・所有権の静的保証を先に確立する。Phase 8 では CTFE（コンパイル時関数評価）・停止性検査・命題型を導入し、形式証明と `Result`/`Option` ベースのカジュアルプログラミングを同一ライブラリ上で共存させる（詳細は [phase8.md](./phase8.md)）。

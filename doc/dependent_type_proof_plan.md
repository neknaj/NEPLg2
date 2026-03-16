# NEPLg2.1: 依存型・形式証明への拡張構想

最終更新: 2026-03-16

---

## 1. 目的

NEPLg2.1 は「どこでも動く」「安全である」「高速である」という三つの柱を掲げ、システム基盤からアプリ開発まであらゆる用途を対象とする言語基盤を目指している。この目標を完全に実現するには、型システムの最終形として**依存型（Dependent Types）による形式証明**を言語に統合する必要がある。

本ドキュメントは、現在進行中のメモリ安全モデル（`purity_ownership_memory_spec.md`）の実装を土台として、依存型への拡張経路を記述する青写真である。加えて、厳密な形式証明と `Result` を用いたカジュアルなエラーハンドリングが**同一の標準ライブラリ上で共存できる**設計方針を示す。

---

## 2. 依存型導入の三大要素

### 2.1. コンパイル時関数評価（CTFE）

依存型では型シグネチャの中に任意の値式が現れる（例: 長さ `add n 1` の配列を返す関数）。これを処理するため、コンパイラ内に `Pure` 関数を評価できるインタプリタ（CTFE エンジン）を設ける。

**型の位置に現れた式は、コンパイラが暗黙的に CTFE で評価する。** 専用の構文マーカー（`{...}` 等）は設けない。型文脈に式が現れた時点でコンパイラがコンパイル時評価を試み、評価不能な式（`Impure` または `Partial` な関数の呼び出しを含む場合）はコンパイルエラーとする。

NEPLg2.1 は中置演算子を持たず前置記法を原則とするため、型文脈の式も通常の前置記法でそのまま記述できる。kind-directed parsing により各引数の区切りが一意に決まるため、グルーピング構文も不要である。

**CTFE の土台:**
- NEPLg2 はすでに `Pure` / `Impure` を厳格に分離している。
- コンパイラは `Pure` な HIR に対して内部で直接評価（Reduction）を行う。
- Escape Analysis によって表面上 `Pure` と見なされた内部の `InternalAlloc` 操作も、コンパイラ内サンドボックスで安全に模倣実行し、最終的な `Pure Persistent` 値として確定させる。これにより、数学的な純粋性とアルゴリズムの実行効率（ミューテーション）を両立したまま型を評価できる。

### 2.2. 全域性・停止性チェック（Totality / Termination Checking）

型検査の過程で CTFE エンジンが無限ループに陥らないよう、「型文脈で実行されるコードは必ず停止する」ことを静的に保証するチェッカを導入する。

- `Pure` 関数に対して Totality Check パスを追加する。
- ループ内の変数が単調に停止条件へ向かっているか（Variant 定理）、再帰関数の引数が構造的に小さくなっているか（Structural Recursion）を静的に解析する。
- 停止が証明できない関数は `Partial` アノテーションで区別し、型文脈での評価（CTFE）を拒否する。`Partial` 関数は後述の Casual API 層で実行時に呼び出すことは可能である。

### 2.3. 命題型と証明オブジェクト（Proof Objects）

「インデックスが境界内にある」「リストが空でない」のような性質をプログラムの型として表現し、関数シグネチャで要求・提供するための**命題型**を導入する。

- `Eq .A .B`「型 `A` と `B` が等しい」、`IsLess .a .b`「`a < b`」のような命題を型として定義できる（Martin-Löf 等価性に基づく）。
- 証明オブジェクトは型の値であり、コンパイル時に検証可能な関数が構築して返す。
- API はこの証明オブジェクトを引数に要求することで、実行時チェックなしに安全性を保証する。

---

## 3. メモリ安全モデルとの接続

依存型は、値が型の中に入る以上、その値が「実行中に書き換えられない」ことが必須要件となる。NEPLg2.1 のメモリ安全モデルはこれを完璧に保証する。

**Pure Persistent による不変性の保証:**
型パラメータに使用できる値は `Pure Persistent`（共有可能・不変）に限定される。エイリアスから破壊的変更を受けるような値は型引数になれないため、「型レベルで評価した値が実行中に崩れる」問題が構造的に排除される。

**GC レスによる証明の安定性:**
GC による非決定的なメモリ解放は、システムの挙動予測を難しくし形式証明との相性が悪い。NEPLg2.1 の Region Inference（Pure 値のスコープ自動解放）と Drop Elaboration（所有権に基づく自動 Drop）により、コンパイラは「どこで確保しどこで解放されるか」の厳密な証明パスを内包する。証明対象のプログラムに不確実な GC 挙動を持ち込まない。

---

## 4. 厳密プログラミングとカジュアルプログラミングの融合

NEPLg2.1 はシステム基盤やコンパイラのような**厳密な用途**と、アプリ開発のような**カジュアルな用途**の両方を対象とする。依存型と `Result` によるエラーハンドリングは対立しない。**決定可能命題（Decidable Proposition）** パターンにより、同一のライブラリ実装を両者で共有できる。

### 4.1. 命名とモジュール分離の原則

NEPLg2 は **bare 名（prefix/suffix なし）でオーバーロードを揃える** 方針をとる。`get_opt` や `get_safe` のような接尾辞は使わない。

Core 層と Casual 層を **別モジュールとして分離し、`use` 対象の切り替えで解決する**。同じ `get` という名前が Core と Casual の両方に存在し、`use` するモジュールによって意味が変わる。

```nepl
// Core 層: 依存型・証明必須
use core::collections::vec as *
get vec proof   // IsLess idx len の証明が必要、境界チェックなし、戻り型 .T

// Casual 層: Result/Option ラッパー
use std::collections::vec as *
get vec 3       // 証明不要、実行時境界検査、戻り型 Option .T
```

モジュールパスを修飾すれば、両方を `use` している状態でも常に明示できる。

```nepl
core::vec::get vec proof    // Core 版を明示
std::vec::get vec 3         // Casual 版を明示
```

### 4.2. 標準ライブラリの二層構造

コレクション等の標準ライブラリは以下の二層で提供する。

```
┌──────────────────────────────────────────────────────────────────────┐
│ stdlib/collections/vec                                               │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │ core::collections::vec  （依存型・証明必須）                     │  │
│  │  get:  Vec T len idx → T    where IsLess idx len               │  │
│  │  push: Vec T n → T → Vec T (add n 1)                           │  │
│  │  → 境界チェックなし、ゼロオーバーヘッド                          │  │
│  └────────────────────────────┬────────────────────────────────────┘  │
│                               │ 証明を構築して呼ぶ                    │
│  ┌────────────────────────────▼────────────────────────────────────┐  │
│  │ std::collections::vec  （Result / Option ラッパー）              │  │
│  │  get:  Vec T → i32 → Option T                                  │  │
│  │  push: Vec T → T → Vec T   （長さをコンパイル時に追わない）      │  │
│  │  → 実行時に証明を構築、失敗なら None を返す                      │  │
│  └─────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────┘
```

実装は Core 層に一度だけ書く。Casual 層は薄いラッパーであり、ロジックの複製はない。

### 4.3. 決定可能命題パターン

`decide_less` のような**決定手続き**（Decidable Proposition）が融合の鍵となる。これは命題を実行時に検査し、成立すれば証明オブジェクト付きの `Yes`、失敗すれば `No` を返す `Pure Total` 関数である。

```nepl
// 命題: a < b であることの証明型
let IsLess enum .a .b:
    Yes %IsLess .a .b    // Yes の中に証明オブジェクトが入っている
    No

// 決定手続き: 実行時に命題を判定し証明を構築する
let decide_less %fn i32 i32 -> IsLess .a .b \ a b :
    ...
```

### 4.4. コードスケッチ

```nepl
// ── core::collections::vec ────────────────────────────────────────────
#module

// 型レベルで長さを追跡する Vector（len は Nat）
let Vec struct .T .len:
    ...

// push は必ず長さ add n 1 の Vec を返す（型レベルで追跡、CTFE 暗黙評価）
let push .T .n %fn Vec .T .n .T -> Vec .T add .n 1 \ vec item :
    ...

// 証明を持っていれば境界チェックなしでアクセスできる
let get .T .len .idx %fn Vec .T .len .idx -> .T \ vec index :
  where %IsLess .idx .len:
    ...  // 静的に安全が保証されているため無条件アクセス
```

```nepl
// ── std::collections::vec ─────────────────────────────────────────────
#module

use core::collections::vec    // Core 層を内部で呼ぶ
use core::decide as *         // IsLess と decide_less を使う

// 長さをコンパイル時に追わない簡易 Vec
let Vec struct .T:
    ...

// 実行時に境界を検査し、安全なら Core を呼ぶ
let get .T %fn Vec .T i32 -> Option .T \ vec index :
    match decide_less index len:
        IsLess::Yes proof:
            Option::Some core::vec::get vec index   // proof が揃ったので Core を呼べる
        IsLess::No:
            Option::None

let push .T %fn Vec .T .T -> Vec .T \ vec item :
    ...
```

### 4.5. 両モジュールを use したときのオーバーロード解決

同一スコープに `core::vec` と `std::vec` の両方の `get` が導入された場合、型推論が以下の順で解決する。

**段階 1: 制約フィルタリング**

`where` 節を充足できない候補を除外する。

```nepl
use core::collections::vec as *
use std::collections::vec as *

let x = get vec 3
// core::vec::get の where %IsLess 3 len → 証明が scope にない → 除外
// std::vec::get → 残る → Option .T に解決
```

```nepl
let x = get vec proof   // proof : IsLess idx len が scope にある
// core::vec::get の where を充足 → 残る
// std::vec::get → i32 を期待するが IsLess 型 → 型不一致 → 除外
// core::vec::get に解決
```

**段階 2: 期待型による絞り込み（双方向型推論）**

制約フィルタ後も複数候補が残った場合、呼び出し元の期待型で絞る。

```nepl
let x %Option .T = get vec idx     // 期待型 Option .T → std::vec::get に解決
let x %.T        = get vec idx     // 期待型 .T → core::vec::get に解決
```

**段階 3: 修飾名による明示**

フィルタ後に複数残り期待型でも絞れない場合は修飾名を要求する。

```nepl
let x = get vec idx
// ERROR: ambiguous — use `core::vec::get` or `std::vec::get`
```

### 4.6. 用途別の使い方

| 用途 | 使う方法 | コスト |
|---|---|---|
| システム・コンパイラ・形式検証 | `use core::collections::vec as *`、証明を静的に持ち込む | ゼロ |
| アプリ開発・一般ロジック | `use std::collections::vec as *`、戻り値 `Option` で分岐 | 境界チェック 1 回 |
| 両方使うファイル | 修飾名 `core::vec::get` / `std::vec::get` で明示 | 選択した方 |

### 4.7. CTFE による最適化

`decide_less` は `Pure Total` 関数であるため、コンパイラが index を定数として判断できる文脈では CTFE によって証明構築がコンパイル時に実行される。

- index がコンパイル時定数 → 境界チェックはコンパイル時に解決、実行時コードは Core 呼び出しのみ
- index が実行時の値 → `decide_less` が実行時に 1 回走る

使う側のコードがどちらを選んでも、ライブラリ本体（Core 層）は変わらない。

---

## 5. ロードマップ

**Phase 0–6（現在進行中）**
- `InternalAlloc` / `ExternalIO` 等の副作用分離、`MemPtr` 化
- Region Inference / Drop Elaboration による GC レスのメモリ安全モデル確立
- Resource IR・借用チェッカの実装

**Phase 7（基盤安定化）**
- パフォーマンス・チューニングと言語エコシステムの整備

**Phase 8（依存型への昇華）**
- **CTFE エンジン導入**: 型文脈の `Pure Total` 式をコンパイラが評価できるようにする
- **Totality Checker 実装**: ループ・再帰の停止性を静的保証
- **型の依存化**: 型引数として値式を渡せるよう型チェッカを拡張（暗黙 CTFE・前置記法）
- **命題型の導入**: `Eq`、`IsLess` 等の命題型とその決定手続きを標準ライブラリに追加

**Phase 9（標準ライブラリの刷新）**
- `Vec`、`List`、`str` 等のコレクションに Core 層と Casual 層を実装
- Core 層は静的証明必須・ランタイムパニックなしの究極の Safe API
- Casual 層は `Result` / `Option` による既存のプログラミングスタイルとの互換性を維持
- 両層とも同じ bare 名で揃え、`use` 対象の切り替えで使い分ける

---

## 6. まとめ

依存型と `Result` によるカジュアルなエラーハンドリングは対立しない。決定可能命題パターンにより、同一の実装を厳密な形式証明とカジュアルな `Option`/`Result` の両側から利用できる。

API 名は prefix/suffix を使わず bare 名で揃え、Core 層と Casual 層を別モジュールに配置することでオーバーロードの複雑化を避ける。両方を `use` した場合は、制約フィルタリング → 双方向型推論 → 修飾名 の順で解決する。

NEPLg2.1 の `Pure Persistent` による値の不変性保証と GC レスのメモリ安全モデルは、形式証明との統合に必要な条件を構造的に満たしている。現在のリブート計画は、依存型統合への自然な拡張経路として設計されている。

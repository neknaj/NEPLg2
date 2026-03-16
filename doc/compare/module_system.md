# NEPLg2.0 → NEPLg2.1 モジュールシステム変更

---

## 1. 廃止される構文

| 廃止対象 | 代替 |
|----------|------|
| `#import "path"` | `use modulepath::subpath`（依存宣言）＋ `merge "./path"`（ソース結合） |
| `#include "path"` | `merge "./path"` |
| `use path::*`（グロブ形式） | `use path as *` |
| `#use`（ディレクティブ形式） | `use`（構文として統合） |
| `#prelude` / `#no_prelude` | 将来仕様で再設計 |

---

## 2. 新規追加

### 2.1 ファイルヘッダ

```nepl
#module    // 独立モジュールの anchor ファイル（新規）
#entry     // エントリポイント（既存・文法を変更）
#part      // merge されるパートファイル（新規）
```

**`#entry` の文法変更**: NEPLg2.0 の `#entry main` のように名前を取る形式から、`#entry` 単独のヘッダ宣言に変更。エントリポイントの関数名はモジュール慣例または build 設定で指定する。

### 2.5 `module name:` ブロック（新規）

ファイル内の論理的なサブモジュールを `module name:` ブロックで定義できる（新規追加）。

```nepl
module parser:
    let parse ...:
        ...

module lexer:
    let lex ...:
        ...
```

- `module name:` は宣言キーワードではなく、ファイル内の論理分割ブロック。
- Canonical Module Path に `::name` セグメントとして追加される。
- 複数の `#part` ファイルに同名の `module` ブロックがあれば、同一論理モジュールの fragment として結合される。
- NEPLg2.0 には対応する構文は存在しなかった。

### 2.2 `use` の統一

```nepl
// モジュールをインポート
use std::streamio

// alias 付き
use std::streamio as io

// 全 item を現スコープへ
use std::streamio as *

// 特定 item をインポート
use core::math::gcd

// alias 付き item
use core::math::gcd as greatest_cd
```

**`use` パスの `::` セパレータ**: `use` パス内の `::` はモジュール階層の区切り文字。ファイル内の `module name:` ブロックも `::name` セグメントとして canonical path に追加される（例: `./editor.nepl` 内の `module parser:` → `editor::parser`）。バリアント参照の `EnumType::Variant` の `::` とは別の仕組みである（後者は型検査器が処理）。

### 2.3 `merge` 構文

```nepl
merge "./filepath/filename"   // 拡張子 .nepl は省略
```

### 2.4 `pub use` 再エクスポート

```nepl
pub use core::math::gcd    // gcd を自モジュールから再公開
```

---

## 3. 設計の変更点

### NEPLg2.0 の課題

- `#import` は物理的なファイル結合（AST 平坦化）を行っていた。
- パスはファイルシステム位置に直接解決していた。
- `#use` はパースされていたが名前解決で使われていなかった。
- `name_resolve.rs` はスタブで、型検査が字句的 Env 上でフラット化後に解決していた。

### NEPLg2.1 の解決策

- **物理層・構文層・論理層の 3 層分離**: ファイル（物理）とモジュール（論理）を直交させる。
- **`use` はモジュール依存の宣言**: overload なし。パスベースで一意解決。
- **`merge` はソース結合**: 独立モジュールにはならない `#part` ファイルのみ対象。
- **Canonical Module Path**: `anchor ファイルパス` + `nested module セグメント` で一意に識別。

---

## 4. 旧 `#import` パターンの移行方法

```nepl
// NEPLg2.0
#import "stdlib/std/streamio"

// NEPLg2.1 — モジュール依存として宣言
use std::streamio
```

```nepl
// NEPLg2.0 — ファイル分割（インライン結合）
#include "./editor_ops"

// NEPLg2.1 — 同一モジュール内のソース結合
#module

merge "./editor_ops"    // editor_ops.nepl は #part ヘッダを持つ
```

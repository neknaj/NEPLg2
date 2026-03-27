# NEPLg2.1 モジュールシステム仕様

最終更新: 2026-03-16

---

## 1. 基本概念

NEPLg2.1 のモジュールシステムは **ファイル** と **モジュール** を完全に直交させる。3 層に分離する。

| 層 | 内容 |
|---|---|
| **物理層（Physical）** | ディスク上のファイル。パス・ハッシュ・スパンのみを保持する。モジュール意味論を持ち込まない。 |
| **構文層（Syntax）** | パース後の AST。`module name:` ブロック・`merge` 文・`use` 文・トップレベル item を保持する。 |
| **論理層（Logical）** | モジュールツリー。名前解決・可視性・キャッシュの単位。 |

モジュール依存は `use`、ソース結合は `merge` のみを使う。

---

## 2. ファイルの種別

ファイルヘッダでファイルの役割を宣言する。

| ヘッダ | 意味 |
|---|---|
| `#module` | 独立モジュールの anchor ファイル。`use` の解決対象になる。 |
| `#entry` | エントリポイント。root モジュールの anchor として扱う。 |
| `#part` | 独立モジュールではない。`merge` されることで anchor モジュールの一部になる。 |

---

## 3. use 構文

`use` はモジュール依存を宣言し、スコープに名前を導入する。パスの区切り文字は `::` を使う。

### 構文一覧

```nepl
use modulepath::subpath                      // モジュールをインポート。alias = 最終セグメント名
use modulepath::subpath as aliasname         // モジュールを alias 付きでインポート
use modulepath::subpath as *                 // モジュールの公開 item を全て現スコープに導入
use modulepath::subpath::name                // 特定の item をインポート。alias = name
use modulepath::subpath::name as aliasname   // 特定の item を alias 付きでインポート
```

### 例

```nepl
use std::streamio                   // `streamio` として参照できる
use std::streamio as io             // `io` として参照できる
use std::streamio as *              // streamio の公開 item を全て現スコープに導入
use core::math::gcd                 // `gcd` として参照できる
use core::math::gcd as greatest_cd  // `greatest_cd` として参照できる
```

### 解決規則

- `use` はモジュールパスを純粋にパスベースで解決する（overload なし）。
- 一致が 1 件 → OK
- 一致が 0 件 → unresolved error
- 一致が 2 件以上 → ambiguous module error（コンパイルエラー）
- anchor でない `#part` ファイルのパスを `use` した場合 → コンパイルエラー（`#part` は直接 `use` 不可。anchor のパスを使うこと）

---

## 4. merge 構文

`merge` は現在のモジュールに別ファイルのソースを追加する（**同一モジュール内 Source Part 結合**）。単純な文字列置換ではない。

```nepl
merge "./filepath/filename"
```

- パスは `"..."` の文字列リテラル。**`.nepl` 拡張子は省略する。**
- `merge` された側のファイルは独立モジュールにはならない（`#part` ファイルに限る）。
- 結合後は declaration multiset として統合し、名前解決・型推論を行う。
- `private` は結合されたすべての part 間で共有される。
- 同じ part ファイルを複数の anchor が `merge` することは禁止（コンパイルエラー）。part は厳密に 1 つの anchor に属する。

**同名宣言の衝突**: 複数の merge 済み part ファイルが同一名の宣言を持つ場合、シャドウィング規則（[declarations.md §9](./declarations.md)）が適用される。シグネチャが同一であれば warning を発行して後者優先。シグネチャが異なる場合は両オーバーロードを保持する。`noshadow` が付いた宣言への上書きはコンパイルエラー。

```nepl
// editor.nepl  (#module)
#module
#indent 4

merge "./editor_ops"
merge "./editor_util"

pub let run \input:
    ...
```

```nepl
// editor_ops.nepl  (#part)
#part
#indent 4

let helper \ctx:
    ...
```

---

## 5. module ブロック（ファイル内の論理分割）

ファイル内で `module name:` を使って論理的なサブモジュールを定義できる。

```nepl
module parser:
    let parse \tokens:
        ...

module lexer:
    let lex \source:
        ...
```

- `parser` と `lexer` は sibling モジュール。
- 複数ファイル（merge 済みの part 群）に同じモジュール名ブロックがあれば、同じ論理モジュールの fragment として結合される。

---

## 6. Canonical Module Path

**Canonical Module Path** = `anchor ファイルパス` + `ファイル内 nested module セグメント`（`::` で連結）。

例:
- `./editor.nepl` が anchor、`module parser:` を含む → canonical path は `./editor::parser`
- `stdlib/core/math.nepl` が anchor、`module integer:` を含む → `core::math::integer`

### Anchor の決定規則

| 条件 | anchor |
|---|---|
| `#module` ファイル | そのファイル |
| `#entry` ファイル | そのファイル |
| `A merge B` の形 | A が anchor |

---

## 7. 可視性

| キーワード | スコープ |
|---|---|
| （なし） | 同一論理モジュール全体（`merge` された別ファイルも含む）に見える（暗黙 private） |
| `private` | 同上（明示的に書いた場合も挙動は同じ） |
| `pub` | `use` 越しに見える |
| `pub use` | 自モジュールの public surface として再エクスポートする |

`private` キーワードは省略しても同じ意味だが、意図を明示したい場合に書いてよい。`fileprivate` は導入しない（file / module 直交性を壊すため）。

### pub use による再エクスポート

`pub use path` はインポートした item を自モジュールから公開する。

```nepl
// mymod/mod.nepl
use core::math::gcd
pub use core::math::gcd    // gcd を mymod::gcd として再公開

// 利用側
use mymod::gcd             // core::math::gcd が mymod::gcd として解決される
```

### pub use の循環検出

`pub use` の再エクスポートチェーンに循環がある場合はコンパイルエラー。

```nepl
// a.nepl
pub use b::foo    // b::foo を a::foo として再公開

// b.nepl
pub use a::foo    // a::foo を b::foo として再公開 → 循環！
// ERROR: circular pub use: a::foo → b::foo → a::foo
```

コンパイラは `pub use` チェーンをグラフとして構築し、DFS でサイクルを検出する。検出後は最初の循環エッジを起点として診断を発行し、以降の解決は失敗として扱う。

---

## 8. 名前解決の 2 段階

1. **Module Resolution** — パスベースで `use` を解決。overload なし。一意でなければコンパイルエラー。
2. **Item Resolution** — モジュールが確定した後、その中の item を解決。overload 可。

---

## 9. キャッシュ設計

| 種別 | 単位 | key |
|---|---|---|
| parse cache | ファイル | ファイルハッシュ |
| semantic cache | 論理モジュール | canonical module path + anchor + part 集合の AST hash + import した module の interface hash |

interface hash と body hash を分離し、public surface が変わらない場合は downstream の再型検査を省略できる。

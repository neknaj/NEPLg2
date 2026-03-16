# NEPLg2.1 エラー・診断モデル

最終更新: 2026-03-16

---

## 1. 基本型

NEPLg2.1 はエラー処理を次の 3 つの型を中心に標準化する。

### 1.1 `Result .T .E`

軽量な成功/失敗制御。診断情報を伴わないか、伴っても呼び出し側が不要と判断できる API に用いる。既定の `.E` は `StdErrorKind`。

```nepl
let Result enum .T .E:
    Ok %.T
    Err %.E
```

### 1.2 `Diag`（診断値）

構造的に豊かな診断値。コンパイラ・ツール・DSL 実装系で共通利用できる診断基盤。

`Diag` は少なくとも次の情報を保持する:

| フィールド | 意味 |
|-----------|------|
| `kind` | 診断の種別（Log / Info / Warn / Error）+ 詳細 kind |
| `message` | 診断メッセージ |
| `span` | ソースコード上の位置（省略可） |
| `notes` | 補足説明・原因候補（複数可） |
| `help` | 修正案・推奨 API・参考情報（複数可） |
| `source` | 下位原因・内包した失敗（失敗の連鎖を表現） |

`Diag` 自体は表示処理を内包しない。表示は `Stringify` / `Debug` / `Serialize` と renderer の組み合わせで行う。

### 1.3 `Outcome .T .E`

`Result .T .E` と診断群 `Diags` を組み合わせて運ぶ構造体。概念的には `result` と `diags` の 2 フィールドを持つ。

```nepl
// 概念的な構造
let Outcome struct .T .E:
    result %Result .T .E
    diags  %Option Diags
```

---

## 2. 診断 kind の二層構造

`kind` は NEPLg2 全体で共通に扱える標準分類と、各ライブラリ・各言語実装が定義できる独自分類の二層構造にする。

**標準大分類:**
- `Log`
- `Info`
- `Warn`
- `Error`

その下に階層化された kind を持てるようにする（例: `Error/Compile/Type`・`Error/IO/NotFound`）。

独自分類は標準分類を置き換えるのではなく、その下位詳細として共存させる。

---

## 3. StdErrorKind

`Result .T .E` の既定の `.E` として使う軽量な標準エラー分類。`Diag.kind` のうち `Error` 系の標準分類と対応づける。`StdErrorKind` だけでは不足する詳細情報や複数診断は `Outcome .T .E` 側の `diags` で補う。

---

## 4. `Result`・`Outcome`・`Option` の使い分け

| 型 | 用途 |
|---|---|
| `Option .T` | 値がある/ない（診断なし。失敗理由を伴わない分岐） |
| `Result .T .E` | 単純な成功/失敗制御。軽量。診断は不要か呼び出し側が不要と判断できる場合 |
| `Outcome .T .E` | warning/info/log を失わずに返したい API。compiler・parser・json 変換など |

- `Result` は `Outcome` へ損失なく昇格できる。
- `Outcome` を使うかどうかは「診断群を持つべきか」に加えて、「source/span/help まで含めた rich reporting が本当に必要か」を基準に判断する。
- stdlib の通常ライブラリでは `Option` / `Result` を標準とし、`Outcome` は例外的に rich diagnostic が必要な API に限定して使う。

---

## 5. ソース位置情報

`callsite_span` は intrinsic であり、現在の呼び出し位置の `Span` を返す。ヘルパはこの span を `Diag` インスタンスに自動付与できる。

---

## 6. 診断のスコープ

`Diag` は stdlib エラーだけでなく、コンパイラ・ツール・DSL 診断にも使う ecosystem 全体の共通基盤。報告/フォーマット処理は `Diag` データ構造自体から分離し、`Stringify` / `Debug` / `Serialize` trait と renderer ツールが担当する。

---

## 7. メモリ管理

診断値はすべて明示的な値として管理される。隠れたグローバルエラー状態はない。`Diag` と `Outcome` 構造体は、用途に応じて Region Inference（pure persistent value として）または Drop Elaboration で自動管理される。プログラマが明示的にヒープ確保や解放を書く必要はない。

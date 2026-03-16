# NEPLg2.0 → NEPLg2.1 変更点まとめ

NEPLg2.0（現行実装）から NEPLg2.1（目標仕様）への主要な変更を記したドキュメント群。

---

## ドキュメント一覧

| ドキュメント | 内容 |
|---|---|
| [syntax.md](./syntax.md) | 宣言構文・型記法・引数リスト記法の変更 |
| [module_system.md](./module_system.md) | モジュールシステムの変更（import 廃止等） |
| [memory_model.md](./memory_model.md) | メモリモデルの変更（raw pointer 隔離・stdlib 移行計画） |

---

## 変更の全体像

### 削除されるもの

| 削除対象 | 理由 |
|---|---|
| `fn`・`struct`・`enum`・`trait`・`impl` 宣言キーワード | `let` に統一 |
| 型記法 `Name<A, B>`（angle bracket） | `Name A B`（juxtaposition）に変更 |
| 型記法 `(A, B) -> C`・`(A, B) *> C` | `fn A B -> C`・`fn* A B -> C` に変更 |
| 型注釈 `<TypeExpr>`（angle bracket） | `%TypeExpr` に変更 |
| unit 型 `()` | `unit` キーワードに変更 |
| 引数リスト `(a, b):` | `\ a b :` に変更 |
| Tuple 組み込み構文 | `Pair`・`Triple` の stdlib struct を使用 |
| `#import` ディレクティブ | `use`（モジュール依存）と `merge`（ソース結合）に分離 |
| `use path::*`（グロブ形式） | `use path as *` に変更 |
| `#entry main`（名前付き形式） | `#entry` 単独ヘッダに変更 |
| `if cond then ... else ...` 補助マーカー（`then`・`do`・`block`） | `:` + インデントに統一 |
| 括弧グループ `(expr)`（式・型文脈） | 型推論（arity/kind-directed）で境界確定 |
| 文区切りセミコロン `;` | 改行・インデントに統一 |
| alloc/dealloc の公開 API | `Result/Option` 前提の安全 API に統一 |
| raw pointer の公開面（`mem_ptr_addr` 等） | compiler/runtime 境界に隔離 |
| entry 関数の強制 Impure 特例 | 署名どおりの effect 判定に変更 |

### 追加されるもの

| 追加対象 | 内容 |
|---|---|
| `%TypeExpr` 型注釈記法 | 型注釈の開始を示す接頭辞 |
| kind-directed 型解析 | 括弧なしで型境界を決定 |
| `where` 節 | 複数制約の分離記述 |
| `noshadow let` | 同一シグネチャ再定義の保護 |
| `& <expr>` / `&mut <expr>` borrow 記法 | 式文法に組み込まれた borrow |
| `module name:` ブロック | ファイル内の論理サブモジュール分割（新規） |
| `EnumType::Variant` バリアント修飾形 | モジュール修飾と独立した型名修飾（型検査器が処理） |
| Region Inference | Pure persistent value の自動回収 |
| Drop Elaboration | owned/linear resource の自動 drop 挿入 |
| Resource IR | ownership/borrow/region/drop の解析中間表現 |
| `InternalAlloc` / `ExternalIO` 内部効果分類 | 効果判定の精密化 |
| `pub use` 再エクスポート・循環検出 | モジュール public surface の再構成 |
| Orphan Rule（グローバルCoherence） | クロスモジュール実装一意性の保証 |
| NLL（ライフタイム注釈なし） | borrow スコープはコンパイラが "last use" で推論（明示構文なし） |
| ジェネリクスの不変（invariant）意味論 | 変位バグを構造的に排除 |

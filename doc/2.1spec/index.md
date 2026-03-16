# NEPLg2.1 仕様書

NEPLg2.1 の完全な言語仕様。本ディレクトリのドキュメントが実装目標となる正の仕様。

---

## 言語仕様

| ドキュメント | 内容 |
|---|---|
| [overview.md](./overview.md) | 言語の理念・設計目標・三本柱 |
| [syntax.md](./syntax.md) | コア構文（前置記法・式・ブロック・パイプ） |
| [types.md](./types.md) | 型システム・型記法・kind-directed 解析 |
| [declarations.md](./declarations.md) | 宣言構文（let / struct / enum / trait / impl） |
| [patterns.md](./patterns.md) | パターン・match・let 分解・クロージャ |
| [effects.md](./effects.md) | 副作用システム（Pure / Impure・Move / Borrow） |
| [memory.md](./memory.md) | メモリ管理（値の三分類・Region Inference・Drop Elaboration） |
| [traits.md](./traits.md) | Trait システム・オーバーロード解決 |
| [modules.md](./modules.md) | モジュールシステム |
| [stdlib.md](./stdlib.md) | 標準ライブラリ設計 |
| [platform.md](./platform.md) | マルチプラットフォーム・ターゲット |
| [errors.md](./errors.md) | エラー・診断モデル |
| [phase8.md](./phase8.md) | Phase 8: 依存型・形式証明（将来仕様） |

## コンパイラ実装ガイド

| ドキュメント | 内容 |
|---|---|
| [compiler.md](./compiler.md) | コンパイラ内部設計・Resource IR・解析パス |

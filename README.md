# NEPLg2 - Neknaj Expression Prefix Language General-purpose 2

[![WebAssembly](https://img.shields.io/badge/Target-WebAssembly-654FF0?logo=webassembly&logoColor=white)](https://webassembly.org/)
[![WASI](https://img.shields.io/badge/Runtime-WASI%20Preview1-0A7EA4)](https://wasi.dev/)
[![Prefix](https://img.shields.io/badge/Syntax-Prefix-3B82F6)](#特徴)
[![Off--side](https://img.shields.io/badge/Layout-Off--side-10B981)](#特徴)

NEPLg2 は、**式指向**・**前置記法**・**オフサイドルール**を中核にした WebAssembly 向け言語です。
ブロックは `:` + インデントで表現し、`if` / `while` / `match` なども式として扱います。

現在の主作業は、現行 NEPLg2 を NEPLg2.1 表層構文へ切り替える移行です。NEPLg2.1 では `%` 型注釈、prefix 型式、`unit` unit 記法、`\` 関数リテラル、明示 generic postfix 撤廃を導入します。NEPLg3 はまだ未着手・未確定の将来設計であり、現在の実装や移行作業の正仕様ではありません。

## すぐ触る

- Web Playground: <https://neknaj.github.io/NEPLg2/>
- Getting Started Tutorial: <https://neknaj.github.io/NEPLg2/tutorials/getting_started/00_index.html>
- Standard Library: <https://neknaj.github.io/NEPLg2/doc/stdlib/index.html>

## 特徴

- ほぼすべてが式
- 前置記法とオフサイドルールで括弧依存を減らす
- WASM / WASI を主要ターゲットに据えたマルチプラットフォーム対応
- 依存関係を適切に切り分け、コンパイラなどの言語ツールも含めてWebブラウザ内で動作可能

## クイックサンプル

以下は NEPLg2.1 移行後の表層構文例です。

```neplg2
#indent 4

use core::math as *

fn classify %fn i32 str \score:
    if ge score 90
        "A"
        if ge score 70
            "B"
            "C"

fn main %impure fn unit i32 \unit:
    block:
        ; classify 85
        0
```

NEPLg2.1 の移行計画は [`doc/neplg2/neplg21_syntax_migration_plan.md`](doc/neplg2/neplg21_syntax_migration_plan.md) を参照してください。現行の Rust 実装、stdlib、tutorial には NEPLg2.0 の角括弧記法が残っており、この branch で移行します。

## チュートリアル

`tutorials/getting_started/` にチュートリアルが収録されています。

```
tutorials/
    getting_started/
        00_index.n.md
        01_hello_world.n.md
        02_variables.n.md
        ...（28 ファイル）
```

オンライン版: <https://neknaj.github.io/NEPLg2/tutorials/getting_started/00_index.html>

## 標準ライブラリ

`stdlib/` 配下にビルトイン関数をほぼ置かず、モジュール import を前提にした標準ライブラリが収録されています。

```
stdlib/
    core/        # 基本トレイト・演算・Option / Result
    std/         # stdio, streamio, fs, io
    alloc/       # コレクション: vec, hashmap, list など
    platforms/   # WASIX, TUI など
    neplg3/      # セルフホストコンパイラ（開発中）
```

よく使うモジュール:

| モジュール | 内容 |
|---|---|
| `core/math` | i32 算術・比較の基本 API |
| `std/stdio` | `print` / `println` / `print_i32` など |
| `std/streamio` | モダンなストリーム I/O（競技プログラミング向け高速 I/O 含む） |
| `std/test` | `assert` / `assert_eq_i32` / `assert_str_eq` など |
| `alloc/collections/vec` | 可変長配列 |

詳細なリファレンス: [Standard Library Documentation](https://neknaj.github.io/NEPLg2/doc/stdlib/index.html)

## ビルドとテスト

```bash
# Rust ビルド
cargo build --workspace --locked

# Rust ユニットテスト
cargo test --workspace --locked

# 統合テスト（trunk build が必要）
trunk build
NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-full.json --runner all --no-tree -j 2
```

CLI でのコンパイル・実行方法の詳細は [`doc/cli.md`](doc/cli.md) を参照してください。

## NEPLg2.1 移行と NEPLg3

現在進行中の変更は **NEPLg2.1** への表層構文移行です。これは現行 `nepl-core/`、`stdlib/`、`tests/`、`tutorials/` を同一ラインで更新する作業であり、`nepl-core-g3` の新規実装ではありません。

NEPLg3 は将来の次世代仕様として検討されていますが、まだ仕様も実装も確定していません。`doc/neplg3/` と `doc/migration/` は参考資料であり、NEPLg2.1 移行の正仕様として扱いません。

| 対象 | 現行 | 移行後 |
|---|---|---|
| コンパイラ | `nepl-core/` NEPLg2.0 表層構文 | `nepl-core/` NEPLg2.1 表層構文 |
| 標準ライブラリ | `stdlib/` 角括弧記法混在 | `stdlib/` NEPLg2.1 記法 |
| テスト | `tests/` 角括弧記法混在 | `tests/` NEPLg2.1 記法 |
| チュートリアル | `tutorials/` 角括弧記法混在 | `tutorials/` NEPLg2.1 記法 |
| selfhost | Rust 実装を踏まえて設計更新待ち | NEPLg2.1 frontend 実装後に設計更新 |

詳細:
- NEPLg2.1 表層構文移行: [`doc/neplg2/neplg21_syntax_migration_plan.md`](doc/neplg2/neplg21_syntax_migration_plan.md)
- NEPLg3 参考資料: [`doc/neplg3/`](doc/neplg3/README.md)

## 開発ドキュメント

| ドキュメント | 内容 |
|---|---|
| [`doc/cli.md`](doc/cli.md) | CLI コマンドリファレンス |
| [`doc/llvm_ir_setup.md`](doc/llvm_ir_setup.md) | LLVM IR セットアップ（clang 21.1.0） |
| [`doc/testing.md`](doc/testing.md) | テストワークフロー |
| [`doc/`](doc/README.md) | ドキュメント一覧 |

# NEPLg2 - Neknaj Expression Prefix Language General-purpose 2

[![WebAssembly](https://img.shields.io/badge/Target-WebAssembly-654FF0?logo=webassembly&logoColor=white)](https://webassembly.org/)
[![WASI](https://img.shields.io/badge/Runtime-WASI%20Preview1-0A7EA4)](https://wasi.dev/)
[![Prefix](https://img.shields.io/badge/Syntax-Prefix-3B82F6)](#特徴)
[![Off--side](https://img.shields.io/badge/Layout-Off--side-10B981)](#特徴)

NEPLg2 は、**式指向**・**前置記法**・**オフサイドルール**を中核にした WebAssembly 向け言語です。
ブロックは `:` + インデントで表現し、`if` / `while` / `match` なども式として扱います。

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

以下は Zenn #1 / #2 を正とした NEPLg3 コア構文の例です。

```neplg2
#indent 4

use core::math as *

let classify \score:
    if ge score 90
        "A"
        if ge score 70
            "B"
            "C"

let main \():
    block:
        ; classify 85
        ()
```

現行の Rust 実装とチュートリアルには NEPLg2.0 / 旧 2.1 案の記法が一部残っています。Zenn #1 / #2 を正とした設計文書は [`doc/neplg3/spec/`](doc/neplg3/spec/index.md) を参照してください。

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

## NEPLg3 への移行計画

現在 **NEPLg3** の設計・実装を並行して進めています。NEPLg3 は Zenn #1 / #2 を正とし、カリー化された関数型記法、`%` の式レベル型注釈、`let <name> <expr>`、`if cond a b` / `match pattern expr` / `block:` などのコア構文を含む次世代仕様です。

| 対象 | 現行 (NEPLg2.0) | 開発中 (NEPLg3) |
|---|---|---|
| コンパイラ | `nepl-core/` | `nepl-core-g3/`（未着手） |
| 標準ライブラリ | `stdlib/`（凍結） | `stdlib-g3/`（Stage 2 以降） |
| テスト | `tests/`（凍結） | `tests-g3/`（Stage 1 以降） |
| チュートリアル | `tutorials/`（凍結） | `tutorials-g3/`（先行作成可） |

`nepl-core-g3` の Stage 6 到達時に一括切り替えを行い、古いディレクトリは archive します。

詳細:
- 言語仕様: [`doc/neplg3/spec/`](doc/neplg3/spec/index.md)
- コンパイラ実装設計: [`doc/neplg3/impl/`](doc/neplg3/impl/index.md)
- stdlib / tests / tutorials 移行計画: [`doc/migration/index.md`](doc/migration/index.md)

## 開発ドキュメント

| ドキュメント | 内容 |
|---|---|
| [`doc/cli.md`](doc/cli.md) | CLI コマンドリファレンス |
| [`doc/llvm_ir_setup.md`](doc/llvm_ir_setup.md) | LLVM IR セットアップ（clang 21.1.0） |
| [`doc/testing.md`](doc/testing.md) | テストワークフロー |
| [`doc/`](doc/README.md) | ドキュメント一覧 |

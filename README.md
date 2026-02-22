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

## 特徴

- ほぼすべてが式
- 前置記法で処理を一定の読み順に統一
- オフサイドルールで括弧依存を減らす
- WASM / WASI を主要ターゲットに据えた設計

## クイックサンプル

```neplg2
#entry main
#indent 4
#target wasi

#import "std/math"
#use std::math::*
#import "std/stdio"
#use std::stdio::*

fn main <()*> ()> ():
    let mut x <i32> 0;
    while lt x 5:
        print "count = ";
        println_i32 x;
        set x add x 1;
```

## CLI でのコンパイル/実行

```bash
# wasm を直接実行（出力ファイルなし）
cargo run -p nepl-cli -- --input examples/counter.nepl --run

# 出力を書きつつ実行
cargo run -p nepl-cli -- --input examples/counter.nepl --output target/counter --run

# WASI ターゲットで実行
cargo run -p nepl-cli -- -i examples/rpn.nepl --run --target wasi

# ターゲット指定（wasm|wasi、既定は wasm）
cargo run -p nepl-cli -- --input examples/counter.nepl --target wasi --output target/counter

# 複数成果物の出力（wasm + wat + wat-min）
cargo run -p nepl-cli -- -i examples/counter.nepl -o target/counter --emit wasm,wat,wat-min

# プロファイル指定（debug|release）
cargo run -p nepl-cli -- -i examples/counter.nepl -o target/counter --profile debug
```

補足:

- `--output` は拡張子なしのベースパスとして扱われます。
- `--emit` は繰り返し指定またはカンマ区切り指定が可能です。
- `--emit all` は `wasm, wat, wat-min` に展開されます。
- `--profile` は `#if[profile=...]` の分岐に使われます。

## 外部 WASI ランタイムでの実行

```bash
# WASI 向け wasm を生成
cargo run -p nepl-cli -- -i examples/counter.nepl -o counter --target wasi

# wasmtime
wasmtime run counter.wasm

# wasmer
wasmer run counter.wasm

# stdin/stdout ありの例
cargo run -p nepl-cli -- -i examples/rpn.nepl -o rpn --target wasi
echo "3 5 +" | wasmtime run rpn.wasm
echo "3 5 +" | wasmer run rpn.wasm
```

`#entry` で指定した関数がエントリーポイントになります（WASI では `_start` として公開）。

## 標準ライブラリ（抜粋）

NEPLg2 にはビルトイン関数をほぼ置かず、モジュール import を前提にしています。

- `std/math`  
  i32 算術・比較の基本 API
- `std/stdio`  
  `print` / `println` / `print_i32` / `println_i32`、ANSI ヘルパ
- `std/test`  
  `assert` / `assert_eq_i32` / `assert_str_eq` など
- `std/diag`  
  診断出力用 API（debug 用含む）
- `kp/kpread`, `kp/kpwrite`  
  競技プログラミング向け高速 I/O

## テスト

```bash
cargo test --workspace --locked
```

stdlib doctest なども含めた検証は、`nodesrc/tests.js` を使用します。

## 開発ドキュメント

- CLI 出力仕様: `doc/cli.md`
- LLVM IR セットアップ（clang 21.1.0）: `doc/llvm_ir_setup.md`

## Web Playground（ローカル）

`web/` では、ブラウザ上でコンパイル・実行・WAT 確認が可能です。

```bash
trunk serve
```

起動後に <http://127.0.0.1:8080/> を開いてください。  
`web/vendor/editorsample` が無い場合はフォールバックの textarea エディタが使われます。

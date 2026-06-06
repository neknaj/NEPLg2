# NEPLg2.1 - Neknaj Expression Prefix Language General-purpose 2

[![WebAssembly](https://img.shields.io/badge/Target-WebAssembly-654FF0?logo=webassembly&logoColor=white)](https://webassembly.org/)
[![WASI](https://img.shields.io/badge/Runtime-WASI%20Preview1-0A7EA4)](https://wasi.dev/)
[![Prefix](https://img.shields.io/badge/Syntax-Prefix-3B82F6)](#neplg21-%E3%81%AE%E6%A7%8B%E6%96%87)
[![Off--side](https://img.shields.io/badge/Layout-Off--side-10B981)](#neplg21-%E3%81%AE%E6%A7%8B%E6%96%87)

現在の NEPL は **NEPLg2.1** です。

NEPLg2.1 は、式指向、前置記法、オフサイドルールを中核にした WebAssembly / WASI / LLVM 向け言語です。ブロックは `:` とインデントで表し、値式、型式、関数型、関数リテラル、制御構文を同じ前置記法の考え方で扱います。

NEPLg3 は完全に検討段階で未着手です。`doc/neplg3/` や `stdlib/neplg3/` に検討資料や置き場があっても、現行仕様、現行実装、または進行中の self-host 実装として扱いません。

## すぐ触る

- Web Playground: <https://neknaj.github.io/NEPLg2/>
- Getting Started Tutorial: <https://neknaj.github.io/NEPLg2/tutorials/getting_started/00_index.html>
- Standard Library: <https://neknaj.github.io/NEPLg2/doc/stdlib/index.html>

## NEPLg2.1 の構文

NEPLg2.1 では、構文を「値だけが前置」ではなく、型や関数境界まで含めて前置記法に揃えます。

- 値式は `add a b`、`println text`、`grade score` のように、関数名を先頭に置きます。
- 型注釈は `%T expr` です。`let n %i32 40` は「`40` を `i32` として検査する」という期待型境界であり、実行時の値を増やしません。
- 型式も前置です。`Vec i32`、`Result i32 str`、`fn i32 fn i32 i32`、`impure fn void unit` のように書きます。
- `unit` は unit 型かつ unit 値です。0 引数関数の marker は `void` で、`fn void unit` と `\void` にだけ使います。
- 関数型はカリー化に似た見た目ですが、NEPLg2.1 は部分適用を導入しません。`add 1` は暗黙の関数値にならず、必要な引数が揃わない呼び出しとして扱われます。
- `if`、`match`、`block` なども式です。ブロックは最後の式を値として返し、不要な値は `;` で明示的に捨てます。
- 正規構文では呼び出し側の explicit generic postfix を使いません。必要な型情報は `%` 型注釈、引数、戻り値期待型、trait / generic 解決から得ます。

簡単な例:

```neplg2
#entry main
#indent 4
#target std

#import "core/math" as *
#import "std/stdio" as *

fn grade %fn i32 str \score:
    if ge score 90:
        "A"
        else:
            if ge score 70:
                "B"
                else:
                    "C"

fn main %impure fn void unit \void:
    let score %i32 85
    let label %str grade score
    println label
```

詳しくは [NEPLg2.1 surface syntax migration plan](doc/neplg2/neplg21_syntax_migration_plan.md) と [zero-argument function marker `void`](doc/neplg2/zero_arg_void_marker_spec.md) を参照してください。

## 現在の実装

このリポジトリの実体は、NEPLg2.1 を基準にした Rust 製コンパイラ、CLI、Web Playground、標準ライブラリ、エディタ向け解析 API、GUI/TUI substrate、NEPLg2.1 self-host 実装です。

```
nepl-core/        Rust 製コンパイラ core。lexer / parser / typecheck / Resource IR / Wasm / LLVM。
nepl-cli/         CLI。check、run、emit、stdlib root、test mode などを提供する。
nepl-web/         wasm-bindgen 向け compiler API。Web Playground から使う。
nepl-language/    エディタ / LSP 向け共通解析 API。
nepl-lsp/         LSP server。diagnostics、hover、definition、semantic tokens など。
nepl-gui-native/  native GUI smoke runner。minifb は optional feature。
web/              Web Playground frontend。
nodesrc/          doctest、HTML生成、source policy、Playground検証ツール。
stdlib/           NEPLg2.1 標準ライブラリと self-host compiler source。
tests/            compiler / stdlib regression。
examples/         CLI、GUI、TUI寄りの実行サンプル。
```

## 標準ライブラリ

`stdlib/` は現行 NEPLg2.1 の標準ライブラリです。依存方向は `core`、`alloc`、`std`、`platforms` を基本に分けています。

```
stdlib/
    core/        allocation や host API に依存しない基本型、trait、math、Option / Result、GUI core。
    alloc/       Vec、String、collections、GUI app / layout / widget など allocation を使う層。
    std/         stdio、streamio、fs、env、timer、GUI host など host 依存の標準 API。
    platforms/   WASI / WASIX / GUI web / GUI terminal など platform backend。
    features/    feature facade。
    nm/          Neknaj Markdown 関連。
    kp/          競技プログラミング向け構造と helper。
    neplg2/      NEPLg2.1 self-host compiler source。
    tests/       stdlib doctest / regression。
```

よく使う module:

| module | 内容 |
|---|---|
| `core/math` | i32 算術・比較の基本 API |
| `core/option` / `core/result` | `Option` / `Result` と `match` 前提の失敗表現 |
| `std/stdio` | `print` / `println` / `println_i32` など |
| `std/streamio` | 高速入力や writer を含む stream I/O |
| `std/test` | doctest / regression 用 assertion |
| `alloc/collections/vec` | 可変長配列 |
| `core/gui` / `alloc/gui` / `std/gui` | GUI/TUI 共通 substrate |
| `platforms/gui/web` | Web Playground GUI backend |

## GUI / TUI

GUI と TUI は別々の巨大 framework ではなく、共通の UI substrate として実装しています。

基本方針:

```text
State + Event -> State + Effects
State -> ViewTree
ViewTree + LayoutContext -> LayoutTree
LayoutTree + RenderContext -> DrawCommand stream
DrawCommand stream -> RenderTarget / DrawTarget / Host backend
```

現在の主な実装:

- `stdlib/core/gui`: geometry、color、event、capability、error、draw / render command の基礎。
- `stdlib/alloc/gui`: app model、view tree、layout、widget、theme、routing、focus、diff。
- `stdlib/std/gui`: runtime、host、window、timer、IME、text measurement、error display。
- `stdlib/platforms/gui/web`: Web Playground 向け stdout frame protocol と input bridge。
- `stdlib/platforms/gui/terminal`: TUI を GUI substrate の terminal backend として再設計する入口。
- `examples/gui_*.nepl`: Counter、Life、Mandelbrot、calculator、scientific calculator、paint、breakout。
- `nepl-gui-native`: native 側の最小 runner と platform behavior 検証。

仕様と実装計画は [GUI/TUI 標準ライブラリ仕様](doc/neplg2/gui_standard_library_spec.md) と [GUI/TUI 実装計画](doc/neplg2/gui_tui_implementation_plan.md) を参照してください。

## Web Playground とエディタ解析

Web Playground は `web/` にあり、Trunk と TypeScript で構成されています。compiler は `nepl-web` を wasm-bindgen で呼び出します。

現在の editor / language support は、TypeScript の推測だけではなく Rust 側の解析結果を使います。

- `nepl-language` は native editor / LSP 向けに lex、parse、name resolution、semantic analysis を提供します。
- `nepl-web` は Web Playground 向けに同等の analysis / compile API を wasm から公開します。
- `nepl-lsp` は diagnostics、hover、definition、semantic tokens、inlay hints を返します。
- syntax highlighting は compiler-provided token classification、prefix expression range、`%T` type range、path namespace分類を使います。

関連文書:

- [Web Playground](doc/web_playground.md)
- [LSP 向け解析 API](doc/lsp_api.md)

## Self-Host

現在進めている self-host は **NEPLg2.1 self-host** です。場所は `stdlib/neplg2/` で、正規の設計入口は [NEPLg2.1 セルフホストコンパイラ設計](doc/neplg2/self_host_neplg21_compiler_design.md) です。

この self-host 実装は、NEPLg2.1 の `%` 型注釈、prefix 型式、`\` 関数リテラル、`void` marker、`#test`、Resource IR 静的検査、compiler artifact、compile-time performance 改良を基準にします。

NEPLg3 self-host は未着手です。NEPLg3 の資料は現行 NEPLg2.1 実装の authority ではありません。

## ビルドとテスト

Rust workspace:

```bash
cargo build --workspace --locked
cargo test --workspace --locked
```

CLI:

```bash
cargo run -p nepl-cli -- --check --input examples/helloworld.nepl --target std
cargo run -p nepl-cli -- --run --input examples/helloworld.nepl --target std
cargo run -p nepl-cli -- --input examples/helloworld.nepl --output tmp/helloworld --emit wasm,wat-min --target std
```

Web / Playground:

```bash
npm --prefix web run build:ts
trunk build
node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json
```

Doctest / source policy:

```bash
node nodesrc/tests.js -i examples --no-tree -o tmp/examples-tests.json -j 4
node nodesrc/tests.js -i tests/stdlib -i stdlib/tests --no-tree -o tmp/stdlib-tests.json -j 4
node nodesrc/run_source_policy_regressions.js --warn-only
node nodesrc/issues.js check --dir issues
git diff --check
```

詳しくは [CLI](doc/cli.md) と [Testing and doctest workflow](doc/testing.md) を参照してください。なお、一部の古い文書には過去または将来検討用の前提説明が残っている場合があります。現行 NEPLg2.1 の構文 authority は `doc/neplg2/` の NEPLg2.1 文書です。

## ドキュメント

| ドキュメント | 内容 |
|---|---|
| [doc/README.md](doc/README.md) | ドキュメント全体の入口 |
| [doc/neplg2/neplg21_syntax_migration_plan.md](doc/neplg2/neplg21_syntax_migration_plan.md) | NEPLg2.1 構文の authority |
| [doc/neplg2/zero_arg_void_marker_spec.md](doc/neplg2/zero_arg_void_marker_spec.md) | `void` marker と `unit` の分離 |
| [doc/neplg2/self_host_neplg21_compiler_design.md](doc/neplg2/self_host_neplg21_compiler_design.md) | NEPLg2.1 self-host compiler 設計 |
| [doc/neplg2/gui_standard_library_spec.md](doc/neplg2/gui_standard_library_spec.md) | GUI/TUI 標準ライブラリ仕様 |
| [doc/neplg2/gui_tui_implementation_plan.md](doc/neplg2/gui_tui_implementation_plan.md) | GUI/TUI 実装計画 |
| [doc/web_playground.md](doc/web_playground.md) | Web Playground |
| [doc/lsp_api.md](doc/lsp_api.md) | エディタ / LSP 解析 API |
| [doc/testing.md](doc/testing.md) | テストワークフロー |
| [doc/cli.md](doc/cli.md) | CLI |

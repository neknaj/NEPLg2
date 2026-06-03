# Editor Extensions

> **対象実装**: このドキュメントは現行 Bootstrap 実装（`nepl-core` / `nepl-web` / `nepl-language`）のエディタ連携方針を記述する。NEPLg3 文書は未確定 draft として扱い、現行 NEPLg2.1 の正仕様とは区別する。

## 方針

- `nepl-web` は Web Playground 向けの wasm API として維持する。
- editor extension 向けには別 Rust lib `nepl-language` を正とする。
- Zed / VSCode / 将来の WASIp1 Language Server は `nepl-language` を共通利用する。
- editor 固有の薄い層だけを extension 側へ置き、解析本体は compiler 実装を直接再利用する。
- 将来的に extension 実装を NEPLg2 へ置き換える場合も、この薄い境界だけを置換すればよい形にする。

## 現在の構成

- `nepl-core`
  - lexer / parser / typecheck / loader など、言語処理系の本体。
- `nepl-language`
  - editor extension 専用の共通解析 lib。
  - token / diagnostic / name resolution / semantic token / hover 向け情報を Rust struct で返す。
  - `LoadResult` を受ける API を持ち、複数ファイル解析でも path 付き範囲を返す。
- `nepl-web`
  - Web 向けの JS / wasm-bindgen API。
  - editor extension からは直接依存しない。

## `nepl-language` が返すもの

- 字句解析結果
  - token kind / token value / source range / diagnostic
- 名前解決結果
  - definitions / references / shadow diagnostics / by-name index
- semantic 解析結果
  - expression range と推論型
  - token 単位の inferred type / argument range
  - prefix call argument range と互換 alias を含む semantic token range
  - `%T expr`、関数 signature、struct field、enum payload、trait / impl type argument などの syntax range
  - syntax range、lexer marker、name-resolution trace に基づく token classification。字句だけでは判定できない lower-case type 名も、型式範囲内では `type` として扱う
  - type parameter を取る型式の constructor token は `type-constructor` として扱う。`Result unit GuiError` の `Result`、`%fn` の `fn`、`impure fn` の `impure` / `fn` は濃い緑で表示し、引数を取らない通常の型名は `type` として薄い緑で表示する。関数定義の先頭 `fn` は型式範囲外なので `keyword` のままにする
  - `void` は型ではなく 0 引数 marker として `literal-void` / `zero_arg_void_marker` に分類する。`unit` は型式範囲内でも値式でも表示 category は `literal-unit` とし、syntax range の role により `function_type_result` などの文脈を保持する
  - 関数名、定数名、変数名、parameter 名は name-resolution trace を使って `function` / `constant` / `variable` に分類する
  - `group1::group2::name` のような path は、左側の group / namespace を `namespace`、右端 member を解決結果または `constant` として分類する
  - editor の色付けは `token_classifications` を権威として扱う。`token_resolution` は hover / definition jump / 互換 fallback 用であり、Rust 側が返した分類を上書きしない
  - hover / 定義ジャンプ用の resolved definition
  - 複数ファイル時の file path 付き range

## Zed の実装方針

### 第1段階

- Zed extension は最小限の shell と language registration のみを持つ。
- semantic highlight / diagnostics / hover / definition は `nepl-language` を使う別 Rust 実装へ委譲する。
- tree-sitter grammar は syntax highlight の土台として別管理にする。

### 第2段階

- `nepl-language` の上に WASIp1 Language Server を追加する。
- Zed / VSCode は同じ server binary を利用する。
- semantic tokens / hover / goto definition / inlay hints を LSP で共通化する。

### 第3段階

- extension 側の薄い制御層を NEPLg2 実装へ段階置換する。
- compiler 再利用の境界は維持し、解析本体の二重実装は行わない。

## 未完了

- Zed extension package の build 検証
- 現行環境では `zed_extension_api` が `edition2024` を要求するため、Zed shell の compile 検証にはより新しい Rust/Cargo か互換 crate 世代の固定が必要
- tree-sitter grammar
- WASIp1 Language Server binary
- VSCode extension shell

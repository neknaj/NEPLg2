# LSP 向け解析 API（暫定）

> **注意**: 本ファイルは現行 Bootstrap 実装（`nepl-core` Rust 製コンパイラ）の公開 API を記述している。
> `kind: "fn"` などの定義種別は NEPLg2.0 の宣言キーワード体系に基づいており、
> NEPLg3 仕様（宣言は `let` に統一）とは対応が異なる。
> NEPLg3 仕様については [neplg3/spec/declarations.md](./neplg3/spec/declarations.md) を参照。

`nepl-web` の wasm 公開 API として、lexer/parser の結果を JSON 取得できる関数を追加した。

## analyze_lex

- シグネチャ: `analyze_lex(source: string): object`
- 返却内容:
  - `stage`: `"lex"`
  - `ok`: エラー診断がなければ `true`
  - `indent_width`
  - `tokens[]`
    - `kind`
    - `value`（一部 token のみ）
    - `debug`
    - `span`
      - `file_id`
      - `start`, `end`（byte offset）
      - `start_line`, `start_col`, `end_line`, `end_col`
  - `diagnostics[]`

## analyze_parse

- シグネチャ: `analyze_parse(source: string): object`
- 返却内容:
  - `stage`: `"parse"`
  - `ok`
  - `tokens[]`
  - `lex_diagnostics[]`
  - `diagnostics[]`
  - `module`
    - `indent_width`
    - `directives_count`
    - `root`（Block/Stmt/Expr/PrefixItem の木）
    - `debug`（AST の pretty 文字列）

## analyze_name_resolution

- シグネチャ: `analyze_name_resolution(source: string): object`
- 返却内容:
  - `stage`: `"name_resolution"`
  - `ok`
  - `diagnostics[]`
  - `definitions[]`
    - `id`
    - `name`
    - `kind`（`fn` / `let_hoisted` / `param` / `fn_alias` など）
    - `scope_depth`
    - `span`
  - `references[]`
    - `name`
    - `scope_depth`
    - `span`
    - `resolved_def_id`
    - `candidate_def_ids[]`
  - `by_name`
    - 識別子名ごとの `definitions[]` / `references[]` の index

## Node での利用

`nodesrc/analyze_source.js` から呼び出せる。

```bash
node nodesrc/analyze_source.js --stage lex -i tests/functions.n.md -o /tmp/functions-lex.json
node nodesrc/analyze_source.js --stage parse -i tests/functions.n.md -o /tmp/functions-parse.json
node nodesrc/analyze_source.js --stage resolve -i tests/functions.n.md -o /tmp/functions-resolve.json
```

## 今後

- typecheck 後の token ごとの型情報
- 定義ジャンプ情報（import 先を含む）
- Inlay Hint 向けの式範囲・引数範囲

を順次追加する。

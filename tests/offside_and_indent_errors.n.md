# offside とインデントの追加テスト

このファイルは、既存の doctest 群で不足していた「インデント不正」「オフサイドルールの境界条件」「単行ブロックの制約」などを追加で検証します。
plan.md では、`block:` の直下は 1 段深いインデントが必要であり、不正なインデントはエラーになると明記されています。
また、単行ブロックは行末までが範囲で、単行ブロック内に複行ブロックを置けないことも明記されています。
パイプ演算子 `|>` についても、改行は許可するがインデントは増やさない仕様です。

## indent_block_requires_deeper_indent

`block:` の次行が同じインデントだと、ブロックの本文として認識できません。
plan.md の「改行した次の行ではインデントが 1 つ増える」「インデントの不正はエラー」より、コンパイルエラーを期待します。

neplg2:test[compile_fail]
```neplg2
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    block:
    1
    0
```

## indent_block_requires_exact_step

`#indent 4` の設定下で、ブロック本文が 2 スペースだけ増える（= 1 段のインデントになっていない）ケースを検出できるかを確認します。
インデント段数の不整合はエラーになるべきです。

neplg2:test[compile_fail]
```neplg2
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    block:
      1
    0
```

## indent_args_must_align

引数オフサイドルールでは「並べる引数は同じインデントレベルで書く必要がある」とあります。
2 個目の引数だけ深いインデントになっているため、エラーを期待します。

neplg2:test[compile_fail]
```neplg2
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let x add:
        1
            2
    x
```

## block_colon_after_must_be_comment_only

plan.md では `block:` の後ろには「空白とコメントしか使えない」と明記されています。
同一行に式を続けた場合はエラーを期待します（単行ブロックは `block <...>` の形式で書くべきです）。

neplg2:test[compile_fail]
```neplg2
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    block: add 1 2
```

## block_statement_boundary_requires_semicolon_or_newline

plan.md の例では、`block:` の中で 1 行に 2 文（= 2 つの式）を書いた場合はエラーとされています。
このケースでは `add 1 2` の直後に `add 3 4` が同一行に続いており、明示的な区切り（改行 or `;`）がないためエラーを期待します。

neplg2:test[compile_fail]
```neplg2
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    <()> block:
        add 1 2 add 3 4
        ()
    0
```

## pipe_newline_must_not_increase_indent

`|>` は改行できるものの、`|>` 行でインデントを増やしてはいけません。
`|>` を 1 段深くしているためエラーを期待します。

neplg2:test[compile_fail]
```neplg2
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let result:
        add 1 2
            |> add 3
    result
```

## single_line_block_cannot_contain_multiline_block

単行ブロックは「行末までが範囲」であり、その中に複行ブロック（`while ...:` のような `:` 付き）を置けないとされています。
単行ブロックの中で複行ブロックを開始しているためエラーを期待します。

neplg2:test[compile_fail]
```neplg2
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    // 単行 block の中で while の複行ブロックを始めるのは不可
    block while lt 0 1:
        0
    0
```

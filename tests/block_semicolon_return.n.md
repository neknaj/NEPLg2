# block と `;` の値・型の追加テスト

既存のテストでは単行ブロック中心に「正しい結果が返る」ことは確認できていますが、
複行ブロック（`block:`）の「最後の文がブロックの値を決める」「最後の文が `;` 付きなら値は `()`」という仕様の実行検証が不足していました。
ここでは、返り値（`ret:`）や型エラー（`compile_fail`）で、その挙動を明確にテストします。

## block_colon_returns_last_expr_value

`block:` の最後の文が `;` なしの場合、その値がブロックの値になります。
`let a` / `let b` は途中結果を捨て、最後の `add a b` が `3` になることを確認します。

neplg2:test
ret: 3
```neplg2
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let x <i32> block:
        let a <i32> 1;
        let b <i32> 2;
        add a b
    x
```

## block_colon_last_semicolon_makes_unit_and_causes_type_error

最後の文が `;` 付きの場合、ブロックの値は `()` になります。
ここでは `let x <i32>` で i32 を要求しているため、型不一致でエラーを期待します。

neplg2:test[compile_fail]
```neplg2
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    let x <i32> block:
        1;
    x
```

## single_line_block_last_semicolon_makes_unit_and_causes_type_error

単行ブロックでも `;` はブロック直下の式をそこで終了させるために使えます。
`block 1;` は `()` になるはずなので、i32 を要求する `let x <i32>` では型エラーを期待します。

neplg2:test[compile_fail]
```neplg2
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    let x <i32> block 1;
    x
```

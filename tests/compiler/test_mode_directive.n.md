# test mode directive

`#test` は doctest / `nepl-cli test` 用の補助定義を同じ source の近くに置くための directive です。
通常 compile では直後の statement を無効化し、test mode compile では有効化します。

## test_directive_helper_is_visible_in_doctest_compile

neplg2:test
ret: 42
```neplg2
#entry main
#indent 4
#target wasm

#test
fn helper %fn void i32 \void:
    42

fn main %fn void i32 \void:
    helper
```

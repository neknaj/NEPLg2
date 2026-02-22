# llvm target doctest

`nodesrc/tests.js --runner llvm` から `nepl-cli --target llvm` を呼び出して検証する。

## llvm_raw_block_compile

neplg2:test[llvm_cli]
```neplg2
#target llvm
#entry main
#indent 4
#llvmir:
    define i32 @main() {
    entry:
        ret i32 7
    }
```

## llvm_parsed_subset_const_i32

neplg2:test[llvm_cli]
```neplg2
#target llvm
#indent 4
fn c <()->i32> ():
    123
```

## llvm_rejects_wasm_body

neplg2:test[llvm_cli, compile_fail]
```neplg2
#target llvm
#entry main
#indent 4

fn main <()->i32> ():
    #wasm:
        i32.const 1
```

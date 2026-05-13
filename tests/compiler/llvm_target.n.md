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
#entry c
#indent 4
fn c <()->i32> ():
    123
```

## llvm_rejects_wasm_body

neplg2:test[llvm_cli, compile_fail]
diag_code: effect.raw_body.target_mismatch
```neplg2
#target llvm
#entry main
#indent 4

fn main <()->i32> ():
    #wasm:
        i32.const 1
```

## llvm_math_add_from_stdlib

neplg2:test[llvm_cli]
```neplg2
#target llvm
#entry main
#indent 4
#import "core/math" as *

#llvmir:
    define i32 @main() {
    entry:
        %x = call i32 @add(i32 20, i32 22)
        ret i32 %x
    }
```

## llvm_mem_alloc_store_load

neplg2:test[llvm_cli]
```neplg2
#target llvm
#entry main
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *
#import "core/math" as *

#llvmir:
    define i32 @main() {
    entry:
        %p = add i32 16, 0
        call void @store_i32(i32 %p, i32 77)
        %v = call i32 @load_i32(i32 %p)
        ret i32 %v
    }
```

## llvm_precheck_rejects_wasm_only_intrinsic

neplg2:test[llvm_cli, compile_fail]
diag_code: type.intrinsic.unknown
```neplg2
#target llvm
#entry main
#indent 4

fn main <()->i32> ():
    #intrinsic "i32_add" <> (1, 2)
```

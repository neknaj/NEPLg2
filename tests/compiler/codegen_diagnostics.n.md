# codegen diagnostic regression

backend の未対応経路がプロセス panic ではなく compile_fail diagnostic になることを確認します。

## unsupported_intrinsic_is_compile_fail

neplg2:test[compile_fail]
diag_id: 3012
```neplg2
#target core
#entry main
#indent 4

fn main <()->i32> ():
    #intrinsic "rv_core_007_unknown" <> ()
    0
```

## unknown_field_selector_is_compile_fail

neplg2:test[compile_fail]
diag_id: 3011
```neplg2
#target core
#entry main
#indent 4

struct Pair:
    x <i32>
    y <i32>

fn main <()->i32> ():
    let p Pair 1 2;
    #intrinsic "get_field" <> (p,"z")
```

## invalid_raw_wasm_is_compile_fail

neplg2:test[compile_fail]
diag_id: 4004
```neplg2
#target core
#entry main
#indent 4

fn main <()->i32> ():
    #wasm:
        i32.rv_core_007_invalid
```

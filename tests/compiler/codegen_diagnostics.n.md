# codegen diagnostic regression

backend の未対応経路がプロセス panic ではなく compile_fail diagnostic になることを確認します。

## unsupported_intrinsic_is_compile_fail

neplg2:test[compile_fail]
diag_code: type.intrinsic.unknown
```neplg2
#target core
#entry main
#indent 4

fn main %fn () i32 \():
    #intrinsic "rv_core_007_unknown" <> ()
    0
```

## unknown_field_selector_is_compile_fail

neplg2:test[compile_fail]
diag_code: type.field.invalid_access
```neplg2
#target core
#entry main
#indent 4

struct Pair:
    x %i32
    y %i32

fn main %fn () i32 \():
    let p Pair 1 2;
    #intrinsic "get_field" <> (p,"z")
```

## invalid_raw_wasm_is_compile_fail

neplg2:test[compile_fail, skip_llvm]
diag_code: backend.wasm.raw_line_parse_error
```neplg2
#target core
#entry main
#indent 4

fn main %fn () i32 \():
    #wasm:
        i32.rv_core_007_invalid
```

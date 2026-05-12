# str/i32 boundaries

## str_annotation_rejects_raw_i32

neplg2:test[compile_fail]
diag_codes: type.annotation.mismatch
```neplg2
#entry main
#indent 4
#target core

fn main <()->i32> ():
    let s <str> 0
    0
```

## raw_i32_annotation_rejects_string_literal

neplg2:test[compile_fail]
diag_codes: type.annotation.mismatch, type.return.mismatch
```neplg2
#entry main
#indent 4
#target core

fn main <()->i32> ():
    let p <i32> "not a pointer"
    p
```

## string_literal_remains_str

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/string" as *

fn main <()*>i32> ():
    if str_eq "ok" "ok" 1 0
```

# block_if_semantics.rs 由来の doctest

このファイルは Rust テスト `block_if_semantics.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## epilogue_drop_preserves_return_value

neplg2:test
ret: 1
```neplg2

#entry main
#indent 4

fn main <()->i32> ():
    let x <i32> 1;
    x
```

## match_arm_local_drop_preserves_return

neplg2:test
ret: 5
```neplg2

#entry main
#indent 4
#import "core/option" as *

fn main <()->i32> ():
    match some<i32> 5:
        Some v:
            let y v;
            v
        None:
            0
```

## trailing_semicolon_makes_block_unit_and_errors_for_return

neplg2:test[compile_fail]
diag_code: type.return.mismatch
```neplg2

#entry main
#indent 4
#import "core/math" as *

fn main <()->i32> ():
    add 1 2;
```

## no_semicolons_on_line_allowed

neplg2:test
ret: 11
```neplg2

#entry main
#indent 4
#import "core/math" as *

fn main <()->i32> ():
    add 1 2
    add 3 4
    add 5 6
```

## multiple_semicolons_on_line_allowed

neplg2:test
ret: 11
```neplg2

#entry main
#indent 4
#import "core/math" as *

fn main <()->i32> ():
    add 1 2;;
    add 3 4;;;
    add 5 6
```

# string_numeric_overflow.n.md

## string_to_u128_accepts_max_decimal

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/string" as *

fn main %impure fn unit i32 \unit:
    match to_u128 "340282366920938463463374607431768211455":
        Result::Ok v:
            if str_eq from_u128 v "340282366920938463463374607431768211455" 1 0
        Result::Err _:
            0
```

## string_to_u128_rejects_max_plus_one_decimal

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/string" as *

fn main %impure fn unit i32 \unit:
    match to_u128 "340282366920938463463374607431768211456":
        Result::Ok _:
            0
        Result::Err _:
            1
```

## string_to_u128_radix_rejects_33_hex_digits

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/string" as *

fn main %impure fn unit i32 \unit:
    match to_u128_radix "100000000000000000000000000000000" 16:
        Result::Ok _:
            0
        Result::Err _:
            1
```

## string_to_i128_accepts_signed_edges

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/string" as *
#import "core/math" as *

fn main %impure fn unit i32 \unit:
    let ok_max %bool match to_i128 "170141183460469231731687303715884105727":
        Result::Ok v:
            str_eq from_i128 v "170141183460469231731687303715884105727"
        Result::Err _:
            false
    let ok_min %bool match to_i128 "-170141183460469231731687303715884105728":
        Result::Ok v:
            str_eq from_i128 v "-170141183460469231731687303715884105728"
        Result::Err _:
            false
    if and ok_max ok_min 1 0
```

## string_to_i128_rejects_signed_overflow_edges

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/string" as *
#import "core/math" as *

fn main %impure fn unit i32 \unit:
    let ok_high %bool match to_i128 "170141183460469231731687303715884105728":
        Result::Ok _:
            false
        Result::Err _:
            true
    let ok_low %bool match to_i128 "-170141183460469231731687303715884105729":
        Result::Ok _:
            false
        Result::Err _:
            true
    if and ok_high ok_low 1 0
```

## string_to_i64_accepts_signed_edges

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/string" as *
#import "core/math" as *

fn main %impure fn unit i32 \unit:
    let ok_max %bool match to_i64 "9223372036854775807":
        Result::Ok v:
            str_eq from_i64 v "9223372036854775807"
        Result::Err _:
            false
    let ok_min %bool match to_i64 "-9223372036854775808":
        Result::Ok v:
            str_eq from_i64 v "-9223372036854775808"
        Result::Err _:
            false
    if and ok_max ok_min 1 0
```

## string_to_i64_rejects_signed_overflow_edges

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/string" as *
#import "core/math" as *

fn main %impure fn unit i32 \unit:
    let ok_high %bool match to_i64 "9223372036854775808":
        Result::Ok _:
            false
        Result::Err _:
            true
    let ok_low %bool match to_i64 "-9223372036854775809":
        Result::Ok _:
            false
        Result::Err _:
            true
    if and ok_high ok_low 1 0
```

## string_to_i32_rejects_wrapped_huge_input

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/string" as *
#import "core/math" as *

fn main %impure fn unit i32 \unit:
    let ok_high %bool match to_i32 "2147483648":
        Result::Ok _:
            false
        Result::Err _:
            true
    let ok_huge %bool match to_i32 "340282366920938463463374607431768211456":
        Result::Ok _:
            false
        Result::Err _:
            true
    if and ok_high ok_huge 1 0
```

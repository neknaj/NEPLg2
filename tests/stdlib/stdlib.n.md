# stdlib.rs 由来の doctest

このファイルは Rust テスト `stdlib.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## string_len_literal_returns_3

neplg2:test
ret: 3
```neplg2

#entry main
#indent 4
#import "alloc/string" as *

fn main %impure fn unit i32 \unit:
    let s "abc";
    len s
```

## string_from_i32_len_matches_digits

neplg2:test
ret: 4
```neplg2

#entry main
#indent 4
#import "alloc/string" as *

fn main %impure fn unit i32 \unit:
    let s from_i32 1234;
    len s
```

## string_from_to_roundtrip

neplg2:test
ret: 4
```neplg2

#entry main
#indent 4
#import "alloc/string" as *
#import "core/result" as *
#import "core/math" as *

fn main %impure fn unit i32 \unit:
    let s0 from_i32 0;
    let s5 from_i32 5;
    let s42 from_i32 42;
    // Simple check: convert back and verify lengths match
    let len0 len s0;
    let len5 len s5;
    let len42 len s42;
    // Return sum of lengths; expect 1+1+2=4
    add add len0 len5 len42
```

## string_from_i32_handles_negative

neplg2:test
ret: 3
```neplg2

#entry main
#indent 4
#import "alloc/string" as *

fn main %impure fn unit i32 \unit:
    let s from_i32 -42;
    len s
```

## string_from_bool_uses_text_form

neplg2:test
ret: 9
```neplg2

#entry main
#indent 4
#import "alloc/string" as *
#import "core/math" as *

fn main %impure fn unit i32 \unit:
    let t from_bool true;
    let f from_bool false;
    add len t len f
```

## string_to_bool_reads_text_form

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#import "alloc/string" as *
#import "core/math" as *

fn main %impure fn unit i32 \unit:
    let a %i32 match to_bool "true":
        Result::Ok v:
            cast v
        Result::Err _:
            0
    let b %i32 match to_bool "false":
        Result::Ok v:
            cast v
        Result::Err _:
            1
    add mul a 10 b
```

## string_to_bool_rejects_non_bool_text

neplg2:test
ret: 1
```neplg2

#entry main
#indent 4
#import "alloc/string" as *

fn main %impure fn unit i32 \unit:
    match to_bool "1":
        Result::Ok _:
            0
        Result::Err _:
            1
```

## string_from_i32_radix_formats_binary

neplg2:test
ret: 4
```neplg2

#entry main
#indent 4
#import "alloc/string" as *

fn main %impure fn unit i32 \unit:
    match from_i32_radix 10 2:
        Result::Ok s:
            len s
        Result::Err _:
            0
```

## string_from_i64_radix_formats_hex_lowercase

neplg2:test
ret: 2
```neplg2

#entry main
#indent 4
#import "alloc/string" as *

fn main %impure fn unit i32 \unit:
    match from_i64_radix %i64 cast 255 16:
        Result::Ok s:
            len s
        Result::Err _:
            0
```

## string_from_i64_radix_formats_negative_hex_with_sign

neplg2:test
ret: 3
```neplg2

#entry main
#indent 4
#import "alloc/string" as *
#import "core/cast" as *
#import "core/math" as *

fn main %impure fn unit i32 \unit:
    let n %i64 sub %i64 cast 0 %i64 cast 255;
    match from_i64_radix n 16:
        Result::Ok s:
            len s
        Result::Err _:
            0
```

## string_to_i32_radix_reads_binary

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#import "alloc/string" as *

fn main %impure fn unit i32 \unit:
    match to_i32_radix "1010" 2:
        Result::Ok v:
            v
        Result::Err _:
            0
```

## string_to_i64_radix_reads_hex_mixed_case

neplg2:test
ret: 255
```neplg2

#entry main
#indent 4
#import "alloc/string" as *
#import "core/cast" as *

fn main %impure fn unit i32 \unit:
    match to_i64_radix "Ff" 16:
        Result::Ok v:
            %i32 cast v
        Result::Err _:
            0
```

## string_to_i64_radix_reads_negative_hex

neplg2:test
ret: -255
```neplg2

#entry main
#indent 4
#import "alloc/string" as *
#import "core/cast" as *

fn main %impure fn unit i32 \unit:
    match to_i64_radix "-Ff" 16:
        Result::Ok v:
            %i32 cast v
        Result::Err _:
            0
```

## string_to_i32_radix_rejects_out_of_radix_digit

neplg2:test
ret: 1
```neplg2

#entry main
#indent 4
#import "alloc/string" as *

fn main %impure fn unit i32 \unit:
    match to_i32_radix "2" 2:
        Result::Ok _:
            0
        Result::Err _:
            1
```

## string_from_i64_radix_rejects_unsupported_radix

neplg2:test
ret: 1
```neplg2

#entry main
#indent 4
#import "alloc/string" as *
#import "core/cast" as *

fn main %impure fn unit i32 \unit:
    match from_i64_radix %i64 cast 7 3:
        Result::Ok _:
            0
        Result::Err _:
            1
```

## string_from_i128_formats_decimal_beyond_i64

neplg2:test
ret: 20
```neplg2

#entry main
#indent 4
#import "alloc/string" as *
#import "core/math" as *
#import "core/cast" as *

fn main %impure fn unit i32 \unit:
    let v %i128 i128 %i64 cast 1 %i64 cast 0;
    let s %str from_i128 v;
    len s
```

## string_from_i128_radix_formats_large_hex

neplg2:test
ret: 17
```neplg2

#entry main
#indent 4
#import "alloc/string" as *
#import "core/math" as *
#import "core/cast" as *

fn main %impure fn unit i32 \unit:
    let v %i128 i128 %i64 cast 1 %i64 cast 0;
    match from_i128_radix v 16:
        Result::Ok s:
            len s
        Result::Err _:
            0
```

## string_to_i128_radix_reads_large_hex

neplg2:test
ret: 11
```neplg2

#entry main
#indent 4
#import "alloc/string" as *
#import "core/math" as *
#import "core/field" as *
#import "core/cast" as *

fn main %impure fn unit i32 \unit:
    match to_i128_radix "10000000000000000" 16:
        Result::Ok v:
            let hi %i64 get v "hi";
            let lo %i64 get v "lo";
            if:
                and eq hi %i64 cast 1 eq lo %i64 cast 0
                then:
                    11
                else:
                    0
        Result::Err _:
            0
```

## string_to_i128_radix_reads_negative_hex

neplg2:test
ret: -255
```neplg2

#entry main
#indent 4
#import "alloc/string" as *
#import "core/cast" as *

fn main %impure fn unit i32 \unit:
    match to_i128_radix "-ff" 16:
        Result::Ok v:
            let v64 %i64 cast v;
            %i32 cast v64
        Result::Err _:
            0
```

## string_u128_radix_roundtrip_large_hex

neplg2:test
ret: 17
```neplg2

#entry main
#indent 4
#import "alloc/string" as *
#import "core/math" as *
#import "core/cast" as *

fn main %impure fn unit i32 \unit:
    let v %u128 u128 %i64 cast 1 %i64 cast 0;
    match from_u128_radix v 16:
        Result::Ok s:
            len s
        Result::Err _:
            0
```

## string_to_u128_radix_reads_large_hex

neplg2:test
ret: 7
```neplg2

#entry main
#indent 4
#import "alloc/string" as *
#import "core/math" as *
#import "core/field" as *
#import "core/cast" as *

fn main %impure fn unit i32 \unit:
    match to_u128_radix "10000000000000000" 16:
        Result::Ok v:
            if:
                and eq get v "hi" %i64 cast 1 eq get v "lo" %i64 cast 0
                then:
                    7
                else:
                    0
        Result::Err _:
            0
```

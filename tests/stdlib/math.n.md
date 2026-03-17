# math overload / cast doctest

## math_i32_overload_add_sub_mul

neplg2:test
ret: 37
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let a <i32> add 40 2;
    let b <i32> sub a 5;
    let c <i32> mul b 2;
    add c -37
```

## math_i64_overload_add_sub_mul

neplg2:test
ret: 74
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *
#import "core/cast" as *

fn main <()->i32> ():
    let a <i64> cast 40;
    let b <i64> cast 2;
    let five <i64> cast 5;
    let two <i64> cast 2;
    let c <i64> add a b;
    let d <i64> sub c five;
    let e <i64> mul d two;
    let out <i32> cast e;
    out
```

## math_i128_overload_add_sub_mul

neplg2:test
ret: 78
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *
#import "core/cast" as *

fn main <()->i32> ():
    let a64 <i64> cast 40;
    let b64 <i64> cast 2;
    let a <i128> cast a64;
    let b <i128> cast b64;
    let three64 <i64> cast 3;
    let two64 <i64> cast 2;
    let c <i128> add a b;
    let d <i128> sub c <i128> cast three64;
    let e <i128> mul d <i128> cast two64;
    let out64 <i64> cast e;
    <i32> cast out64
```

## cast_overload_numeric_roundtrip

neplg2:test
ret: 123
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *
#import "core/cast" as *

fn main <()->i32> ():
    let a32 <i32> 123;
    let a64 <i64> cast a32;
    let a128 <i128> cast a64;
    let b64 <i64> cast a128;
    cast b64
```

## cast_ambiguous_without_expected_type

`let v cast 1` の `v` に型注釈がなく `cast` のオーバーロードが曖昧なはずだが、コンパイラが `v` を未使用として処理しているためコンパイルが通る。D3005 が発生しないためスキップ。

neplg2:test[skip]
diag_id: 3005
```neplg2
#entry main
#indent 4
#target core
#import "core/cast" as *

fn main <()->i32> ():
    let v cast 1
    0
```

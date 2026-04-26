# move_check.rs 由来の doctest

このファイルは Rust テスト `move_check.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## move_simple_ok

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target core
struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let t <LocalToken> LocalToken @token_id
    let u <LocalToken> t
    0
```

## move_use_after_move

neplg2:test[compile_fail]
diag_id: 3053
```neplg2
#entry main
#indent 4
#target core
struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let t <LocalToken> LocalToken @token_id
    let u <LocalToken> t
    let v <LocalToken> t
    0
```

## move_in_branch

neplg2:test[compile_fail]
diag_id: 3054
```neplg2
#entry main
#indent 4
#target core
struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn consume <(LocalToken)->i32> (_t):
    1

fn main <()->i32> ():
    let t <LocalToken> LocalToken @token_id
    if true:
        then:
            consume t
        else:
            0
    consume t
```

## move_in_loop

neplg2:test[compile_fail]
diag_id: 3065
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn consume <(LocalToken)->()> (_t):
    ()

fn main <()->i32> ():
    let t <LocalToken> LocalToken @token_id
    let mut c <bool> true
    while c:
        do:
            consume t
            set c false
    consume t
    0
```

## move_reassign_non_copy

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let mut x <LocalToken> LocalToken @token_id
    let y <LocalToken> x
    set x LocalToken @token_id
    let z <LocalToken> x
    0
```

## move_reassign_copy

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target core

fn main <()->i32> ():
    let mut x <i32> 1
    let y <i32> x
    set x 2
    let z <i32> x
    0
```

## move_reference_ok

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let x <LocalToken> LocalToken @token_id
    let r <&LocalToken> &x
    let y <LocalToken> x
    0
```

## move_live_reference_blocks_move

neplg2:test[compile_fail]
diag_id: 3051
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let x <LocalToken> LocalToken @token_id
    let r <&LocalToken> &x
    let y <LocalToken> x
    let z <&LocalToken> r
    0
```

## move_reference_call_arg_is_temporary_borrow

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn observe <(&LocalToken)->i32> (_x):
    1

fn consume <(LocalToken)->i32> (_x):
    0

fn main <()->i32> ():
    let x <LocalToken> LocalToken @token_id
    observe &x
    consume x
```

## move_borrow_after_move_err

neplg2:test[compile_fail]
diag_id: 3063
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()->()> ():
    let x <LocalToken> LocalToken @token_id
    let y <LocalToken> x
    let r <&LocalToken> &x
```

## move_pass_to_function_err

neplg2:test[compile_fail]
diag_id: 3053
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn consume <(LocalToken)->i32> (_w):
    0

fn main <()->()> ():
    let x <LocalToken> LocalToken @token_id
    consume x
    let y <LocalToken> x
```

## move_struct_field_err

neplg2:test[compile_fail]
diag_id: 3053
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

struct S:
    f <LocalToken>

fn main <()->()> ():
    let s <S> S LocalToken @token_id
    let a <LocalToken> s.f
    let b <LocalToken> s.f
```

## move_branch_reinit_mixed

neplg2:test[compile_fail]
diag_id: 3054
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()->()> ():
    let mut x <LocalToken> LocalToken @token_id
    let cnd <bool> true
    if cnd:
        then:
            let y <LocalToken> x
        else:
            set x LocalToken @token_id
    let z <LocalToken> x
```

## move_nested_match_potentially_moved

neplg2:test[compile_fail]
diag_id: 3054
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw <(i32)->i32>
fn token_id <(i32)->i32> (x):
    x
enum BoolWrap:
    True
    False

fn main <()->()> ():
    let x <LocalToken> LocalToken @token_id
    let a <BoolWrap> BoolWrap::True
    match a:
        BoolWrap::True:
            match a:
                BoolWrap::True:
                    let y <LocalToken> x
                    ()
                BoolWrap::False:
                    ()
        BoolWrap::False:
            ()
    let z <LocalToken> x
```

## move_in_match_arms

neplg2:test[compile_fail]
diag_id: 3054
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw <(i32)->i32>
fn token_id <(i32)->i32> (x):
    x
enum BoolWrap:
    True
    False

fn main <()->()> ():
    let x <LocalToken> LocalToken @token_id
    let v <BoolWrap> BoolWrap::True
    match v:
        BoolWrap::True:
            let y <LocalToken> x
            ()
        BoolWrap::False:
            ()
    let z <LocalToken> x
```

## move_return_local_reference_err

neplg2:test[compile_fail]
diag_id: 3099
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn leak <()->&LocalToken> ():
    let t <LocalToken> LocalToken @token_id
    &t

fn main <()->i32> ():
    let r <&LocalToken> leak
    0
```

## move_block_local_reference_escape_err

neplg2:test[compile_fail]
diag_id: 3099
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let r <&LocalToken> block:
        let t <LocalToken> LocalToken @token_id
        &t
    0
```

## move_set_outer_reference_to_inner_local_err

neplg2:test[compile_fail]
diag_id: 3099
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let outer <LocalToken> LocalToken @token_id
    let mut r <&LocalToken> &outer
    block:
        let inner <LocalToken> LocalToken @token_id
        set r &inner
    0
```

## move_return_local_reference_inside_struct_err

neplg2:test[compile_fail]
diag_id: 3099
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw <(i32)->i32>

struct RefBox:
    inner <&LocalToken>

fn token_id <(i32)->i32> (x):
    x

fn leak <()->RefBox> ():
    let t <LocalToken> LocalToken @token_id
    let b <RefBox> RefBox &t
    b

fn main <()->i32> ():
    let b <RefBox> leak
    0
```

## move_block_local_reference_inside_struct_escape_err

neplg2:test[compile_fail]
diag_id: 3099
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw <(i32)->i32>

struct RefBox:
    inner <&LocalToken>

fn token_id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let b <RefBox> block:
        let t <LocalToken> LocalToken @token_id
        let local <RefBox> RefBox &t
        local
    0
```

## move_set_outer_struct_reference_to_inner_local_err

neplg2:test[compile_fail]
diag_id: 3099
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw <(i32)->i32>

struct RefBox:
    inner <&LocalToken>

fn token_id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let outer <LocalToken> LocalToken @token_id
    let mut b <RefBox> RefBox &outer
    block:
        let inner <LocalToken> LocalToken @token_id
        let local <RefBox> RefBox &inner
        set b local
    0
```

## move_call_return_reference_to_block_local_err

neplg2:test[compile_fail]
diag_id: 3099
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn id_ref <(&LocalToken)->&LocalToken> (x):
    x

fn main <()->i32> ():
    let r <&LocalToken> block:
        let t <LocalToken> LocalToken @token_id
        id_ref &t
    0
```

## move_call_return_struct_reference_to_block_local_err

neplg2:test[compile_fail]
diag_id: 3099
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw <(i32)->i32>

struct RefBox:
    inner <&LocalToken>

fn token_id <(i32)->i32> (x):
    x

fn box_ref <(&LocalToken)->RefBox> (x):
    RefBox x

fn main <()->i32> ():
    let b <RefBox> block:
        let t <LocalToken> LocalToken @token_id
        box_ref &t
    0
```

## move_loop_owned_accumulator_reassigned_after_result_ok

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target core
#import "core/result" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn step <(LocalToken)->Result<LocalToken, i32>> (token):
    Result<LocalToken, i32>::Ok token

fn main <()->i32> ():
    let mut cur <LocalToken> LocalToken @token_id
    let mut i <i32> 0
    while lt i 3:
        match step cur:
            Result::Ok next:
                set cur next
                set i add i 1
            Result::Err _e:
                #intrinsic "unreachable" <> ()
    let out <LocalToken> cur
    0
```

## move_loop_owned_accumulator_err_continue_without_reinit_rejected

neplg2:test[compile_fail]
diag_id: 3065
```neplg2
#entry main
#indent 4
#target core
#import "core/result" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn step <(LocalToken)->Result<LocalToken, i32>> (token):
    Result<LocalToken, i32>::Ok token

fn main <()->i32> ():
    let mut cur <LocalToken> LocalToken @token_id
    let mut i <i32> 0
    while lt i 3:
        match step cur:
            Result::Ok next:
                set cur next
                set i add i 1
            Result::Err _e:
                set i 3
    let out <LocalToken> cur
    0
```

## move_borrowed_field_projection_keeps_owner_until_reference_last_use

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target core
#import "core/field" as field

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

struct Pair:
    token <LocalToken>
    count <i32>

fn observe <(&LocalToken)->i32> (_w):
    1

fn consume <(Pair)->i32> (_p):
    0

fn main <()->i32> ():
    let p <Pair> Pair (LocalToken @token_id) 7
    let token_ref <&LocalToken> field::get_ref &p "token"
    let count <i32> *field::get_ref &p "count"
    observe token_ref
    consume p
```

## move_borrowed_field_projection_blocks_owner_move_while_live

neplg2:test[compile_fail]
diag_id: 3051
```neplg2
#entry main
#indent 4
#target core
#import "core/field" as field

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

struct Pair:
    token <LocalToken>
    count <i32>

fn observe <(&LocalToken)->i32> (_w):
    1

fn consume <(Pair)->i32> (_p):
    0

fn main <()->i32> ():
    let p <Pair> Pair (LocalToken @token_id) 7
    let token_ref <&LocalToken> field::get_ref &p "token"
    consume p
    observe token_ref
```

## move_borrowed_field_projection_escape_rejected

neplg2:test[compile_fail]
diag_id: 3099
```neplg2
#entry main
#indent 4
#target core
#import "core/field" as field

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

struct Pair:
    token <LocalToken>
    count <i32>

fn leak <()->&LocalToken> ():
    let p <Pair> Pair (LocalToken @token_id) 7
    field::get_ref &p "token"

fn main <()->i32> ():
    let r <&LocalToken> leak
    0
```

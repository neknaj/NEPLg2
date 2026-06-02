# move_check.rs 由来の doctest

このファイルは Rust テスト `move_check.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## move_simple_ok

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"move_simple_ok\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"single move consumes owner\" expected=\"0\" actual=\"0\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "std/test" as test

struct LocalToken:
    raw %fn i32 i32

fn token_id %fn i32 i32 \x:
    x

fn main %impure fn void i32 \void:
    let t %LocalToken LocalToken @token_id
    let u %LocalToken t
    let actual %i32 0
    let report:
        test::test_report_new "move_simple_ok"
        |> test::test_report_push test::assert_eq_i32 "single move consumes owner" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## move_use_after_move

neplg2:test[compile_fail]
diag_code: resource.cell.moved
```neplg2
#entry main
#indent 4
#target core
struct LocalToken:
    raw %fn i32 i32

fn token_id %fn i32 i32 \x:
    x

fn main %fn void i32 \void:
    let t %LocalToken LocalToken @token_id
    let u %LocalToken t
    let v %LocalToken t
    0
```

## move_in_branch

neplg2:test[compile_fail]
diag_code: resource.cell.possibly_moved
```neplg2
#entry main
#indent 4
#target core
struct LocalToken:
    raw %fn i32 i32

fn token_id %fn i32 i32 \x:
    x

fn consume %fn LocalToken i32 \_t:
    1

fn main %fn void i32 \void:
    let t %LocalToken LocalToken @token_id
    if true:
        then:
            consume t
        else:
            0
    consume t
```

## move_in_loop

neplg2:test[compile_fail]
diag_code: resource.cell.possibly_moved
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw %fn i32 i32

fn token_id %fn i32 i32 \x:
    x

fn consume %fn LocalToken unit \_t:
    unit

fn main %fn void i32 \void:
    let t %LocalToken LocalToken @token_id
    let mut c %bool true
    while c:
        do:
            consume t
            set c false
    consume t
    0
```

## move_reassign_non_copy

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"move_reassign_non_copy\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"non-copy local reassign after move\" expected=\"0\" actual=\"0\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "std/test" as test

struct LocalToken:
    raw %fn i32 i32

fn token_id %fn i32 i32 \x:
    x

fn main %impure fn void i32 \void:
    let mut x %LocalToken LocalToken @token_id
    let y %LocalToken x
    set x LocalToken @token_id
    let z %LocalToken x
    let actual %i32 0
    let report:
        test::test_report_new "move_reassign_non_copy"
        |> test::test_report_push test::assert_eq_i32 "non-copy local reassign after move" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## move_reassign_copy

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"move_reassign_copy\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"copy local reassign after copy\" expected=\"0\" actual=\"0\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "std/test" as test

fn main %impure fn void i32 \void:
    let mut x %i32 1
    let y %i32 x
    set x 2
    let z %i32 x
    let actual %i32 0
    let report:
        test::test_report_new "move_reassign_copy"
        |> test::test_report_push test::assert_eq_i32 "copy local reassign after copy" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## move_reference_ok

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"move_reference_ok\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"borrow last use permits later move\" expected=\"0\" actual=\"0\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "std/test" as test

struct LocalToken:
    raw %fn i32 i32

fn token_id %fn i32 i32 \x:
    x

fn main %impure fn void i32 \void:
    let x %LocalToken LocalToken @token_id
    let r %&LocalToken &x
    let y %LocalToken x
    let actual %i32 0
    let report:
        test::test_report_new "move_reference_ok"
        |> test::test_report_push test::assert_eq_i32 "borrow last use permits later move" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## move_live_reference_blocks_move

neplg2:test[compile_fail]
diag_code: resource.borrow.move_from_shared
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw %fn i32 i32

fn token_id %fn i32 i32 \x:
    x

fn main %fn void i32 \void:
    let x %LocalToken LocalToken @token_id
    let r %&LocalToken &x
    let y %LocalToken x
    let z %&LocalToken r
    0
```

## move_branch_reference_last_use_releases_at_join

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"move_branch_reference_last_use_releases_at_join\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"branch borrow last use releases at join\" expected=\"0\" actual=\"0\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "std/test" as test

struct LocalToken:
    raw %fn i32 i32

fn token_id %fn i32 i32 \x:
    x

fn main %impure fn void i32 \void:
    let x %LocalToken LocalToken @token_id
    let r %&LocalToken &x
    let cnd %bool true
    if cnd:
        then:
            let rr %&LocalToken r
            0
        else:
            0
    let y %LocalToken x
    let actual %i32 0
    let report:
        test::test_report_new "move_branch_reference_last_use_releases_at_join"
        |> test::test_report_push test::assert_eq_i32 "branch borrow last use releases at join" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## move_branch_retained_borrow_blocks_later_move

neplg2:test[compile_fail]
diag_code: resource.borrow.move_from_shared
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw %fn i32 i32

fn token_id %fn i32 i32 \x:
    x

fn main %fn void i32 \void:
    let x %LocalToken LocalToken @token_id
    let y %LocalToken LocalToken @token_id
    let mut r %&LocalToken &x
    let cnd %bool true
    if cnd:
        then:
            set r &y
            0
        else:
            0
    let moved %LocalToken y
    let still_live %&LocalToken r
    0
```

## move_reference_call_arg_is_temporary_borrow

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"move_reference_call_arg_is_temporary_borrow\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"shared call argument borrow is temporary\" expected=\"0\" actual=\"0\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "std/test" as test

struct LocalToken:
    raw %fn i32 i32

fn token_id %fn i32 i32 \x:
    x

fn observe %fn &LocalToken i32 \_x:
    1

fn consume %fn LocalToken i32 \_x:
    0

fn main %impure fn void i32 \void:
    let x %LocalToken LocalToken @token_id
    observe &x
    let actual %i32 consume x
    let report:
        test::test_report_new "move_reference_call_arg_is_temporary_borrow"
        |> test::test_report_push test::assert_eq_i32 "shared call argument borrow is temporary" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## move_mut_reference_call_arg_is_temporary_borrow

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"move_mut_reference_call_arg_is_temporary_borrow\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"unique call argument borrow is temporary\" expected=\"0\" actual=\"0\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "std/test" as test

struct LocalToken:
    raw %fn i32 i32

fn token_id %fn i32 i32 \x:
    x

fn observe_mut %fn &mut LocalToken i32 \_x:
    1

fn consume %fn LocalToken i32 \_x:
    0

fn main %impure fn void i32 \void:
    let x %LocalToken LocalToken @token_id
    observe_mut &mut x
    let actual %i32 consume x
    let report:
        test::test_report_new "move_mut_reference_call_arg_is_temporary_borrow"
        |> test::test_report_push test::assert_eq_i32 "unique call argument borrow is temporary" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## move_call_mut_and_shared_reference_args_overlap_rejected

neplg2:test[compile_fail]
diag_code: resource.borrow.borrow_during_unique
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw %fn i32 i32

fn token_id %fn i32 i32 \x:
    x

fn use_both %fn &mut LocalToken fn &LocalToken i32 \_a\_b:
    0

fn main %fn void i32 \void:
    let x %LocalToken LocalToken @token_id
    use_both &mut x &x
```

## move_call_shared_and_mut_reference_args_overlap_rejected

neplg2:test[compile_fail]
diag_code: resource.borrow.unique_during_shared
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw %fn i32 i32

fn token_id %fn i32 i32 \x:
    x

fn use_both %fn &LocalToken fn &mut LocalToken i32 \_a\_b:
    0

fn main %fn void i32 \void:
    let x %LocalToken LocalToken @token_id
    use_both &x &mut x
```

## move_struct_mut_and_shared_reference_fields_overlap_rejected

neplg2:test[compile_fail]
diag_code: resource.borrow.borrow_during_unique
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw %fn i32 i32

struct RefPair:
    a %&mut LocalToken
    b %&LocalToken

fn token_id %fn i32 i32 \x:
    x

fn main %fn void i32 \void:
    let x %LocalToken LocalToken @token_id
    let p %RefPair RefPair &mut x &x
    0
```

## move_tuple_mut_and_shared_reference_items_overlap_rejected

neplg2:test[compile_fail]
diag_code: resource.borrow.borrow_during_unique
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw %fn i32 i32

fn token_id %fn i32 i32 \x:
    x

fn main %fn void i32 \void:
    let x %LocalToken LocalToken @token_id
    let p Tuple:
        &mut x
        &x
    0
```

## move_unique_reference_blocks_owner_move_while_live

neplg2:test[compile_fail]
diag_code: resource.borrow.use_during_unique
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw %fn i32 i32

fn token_id %fn i32 i32 \x:
    x

fn main %fn void i32 \void:
    let x %LocalToken LocalToken @token_id
    let r %&mut LocalToken &mut x
    let y %LocalToken x
    let keep %&mut LocalToken r
    0
```

## move_unique_reference_last_use_releases_owner

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"move_unique_reference_last_use_releases_owner\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"unique borrow last use releases owner\" expected=\"0\" actual=\"0\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "std/test" as test

struct LocalToken:
    raw %fn i32 i32

fn token_id %fn i32 i32 \x:
    x

fn main %impure fn void i32 \void:
    let x %LocalToken LocalToken @token_id
    let r %&mut LocalToken &mut x
    let rr %&mut LocalToken r
    let y %LocalToken x
    let actual %i32 0
    let report:
        test::test_report_new "move_unique_reference_last_use_releases_owner"
        |> test::test_report_push test::assert_eq_i32 "unique borrow last use releases owner" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## move_mut_reference_is_not_copy

neplg2:test[compile_fail]
diag_code: resource.cell.moved
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw %fn i32 i32

fn token_id %fn i32 i32 \x:
    x

fn main %fn void i32 \void:
    let x %LocalToken LocalToken @token_id
    let r %&mut LocalToken &mut x
    let rr %&mut LocalToken r
    let again %&mut LocalToken r
    0
```

## move_shared_borrow_blocks_unique_borrow

neplg2:test[compile_fail]
diag_code: resource.borrow.unique_during_shared
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw %fn i32 i32

fn token_id %fn i32 i32 \x:
    x

fn main %fn void i32 \void:
    let x %LocalToken LocalToken @token_id
    let r %&LocalToken &x
    let u %&mut LocalToken &mut x
    let keep %&LocalToken r
    0
```

## move_unique_borrow_blocks_shared_borrow

neplg2:test[compile_fail]
diag_code: resource.borrow.borrow_during_unique
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw %fn i32 i32

fn token_id %fn i32 i32 \x:
    x

fn main %fn void i32 \void:
    let x %LocalToken LocalToken @token_id
    let r %&mut LocalToken &mut x
    let s %&LocalToken &x
    let keep %&mut LocalToken r
    0
```

## move_copy_unique_borrow_blocks_shared_borrow

neplg2:test[compile_fail]
diag_code: resource.borrow.borrow_during_unique
```neplg2
#entry main
#indent 4
#target core

fn main %fn void i32 \void:
    let x %i32 1
    let u %&mut i32 &mut x
    let s %&i32 &x
    let keep %&mut i32 u
    0
```

## move_copy_shared_borrow_blocks_unique_borrow

neplg2:test[compile_fail]
diag_code: resource.borrow.unique_during_shared
```neplg2
#entry main
#indent 4
#target core

fn main %fn void i32 \void:
    let x %i32 1
    let s %&i32 &x
    let u %&mut i32 &mut x
    let keep %&i32 s
    0
```

## move_copy_shared_borrow_allows_owner_copy_while_reference_live

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"move_copy_shared_borrow_allows_owner_copy_while_reference_live\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"copy owner may be copied while shared reference lives\" expected=\"2\" actual=\"2\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as test

fn main %impure fn void i32 \void:
    let x %i32 1
    let s %&i32 &x
    let y %i32 x
    let keep %&i32 s
    let actual %i32 add x y
    let report:
        test::test_report_new "move_copy_shared_borrow_allows_owner_copy_while_reference_live"
        |> test::test_report_push test::assert_eq_i32 "copy owner may be copied while shared reference lives" 2 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## move_borrow_after_move_err

neplg2:test[compile_fail]
diag_code: resource.cell.moved
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw %fn i32 i32

fn token_id %fn i32 i32 \x:
    x

fn main %fn void unit \void:
    let x %LocalToken LocalToken @token_id
    let y %LocalToken x
    let r %&LocalToken &x
```

## move_pass_to_function_err

neplg2:test[compile_fail]
diag_code: resource.cell.moved
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw %fn i32 i32

fn token_id %fn i32 i32 \x:
    x

fn consume %fn LocalToken i32 \_w:
    0

fn main %fn void unit \void:
    let x %LocalToken LocalToken @token_id
    consume x
    let y %LocalToken x
```

## move_struct_field_err

neplg2:test[compile_fail]
diag_code: resource.cell.moved
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw %fn i32 i32

fn token_id %fn i32 i32 \x:
    x

struct S:
    f %LocalToken

fn main %fn void unit \void:
    let s %S S LocalToken @token_id
    let a %LocalToken s.f
    let b %LocalToken s.f
```

## move_distinct_owned_struct_fields_once_ok

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"move_distinct_owned_struct_fields_once_ok\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"distinct owned struct fields move once\" expected=\"0\" actual=\"0\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "core/field" as field
#import "core/field" as *
#import "std/test" as test

struct LocalToken:
    raw %fn i32 i32

fn token_id %fn i32 i32 \x:
    x

struct Pair:
    left %LocalToken
    right %LocalToken

fn consume %fn LocalToken i32 \_w:
    0

fn main %impure fn void i32 \void:
    let p %Pair Pair (LocalToken @token_id) (LocalToken @token_id)
    let left %LocalToken field::get p "left"
    let right %LocalToken field::get p "right"
    consume left
    let actual %i32 consume right
    let report:
        test::test_report_new "move_distinct_owned_struct_fields_once_ok"
        |> test::test_report_push test::assert_eq_i32 "distinct owned struct fields move once" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## move_same_owned_struct_field_twice_rejected

neplg2:test[compile_fail]
diag_code: resource.cell.moved
```neplg2
#entry main
#indent 4
#target core
#import "core/field" as field
#import "core/field" as *

struct LocalToken:
    raw %fn i32 i32

fn token_id %fn i32 i32 \x:
    x

struct Pair:
    left %LocalToken
    right %LocalToken

fn main %fn void unit \void:
    let p %Pair Pair (LocalToken @token_id) (LocalToken @token_id)
    let left %LocalToken field::get p "left"
    let again %LocalToken field::get p "left"
```

## move_owner_after_partial_field_move_rejected

neplg2:test[compile_fail]
diag_code: resource.cell.moved
```neplg2
#entry main
#indent 4
#target core
#import "core/field" as field
#import "core/field" as *

struct LocalToken:
    raw %fn i32 i32

fn token_id %fn i32 i32 \x:
    x

struct Pair:
    left %LocalToken
    right %LocalToken

fn consume_pair %fn Pair unit \_p:
    unit

fn main %fn void unit \void:
    let p %Pair Pair (LocalToken @token_id) (LocalToken @token_id)
    let left %LocalToken field::get p "left"
    consume_pair p
```

## move_field_from_borrowed_owner_rejected

neplg2:test[compile_fail]
diag_code: resource.borrow.move_from_shared
```neplg2
#entry main
#indent 4
#target core
#import "core/field" as field
#import "core/field" as *

struct LocalToken:
    raw %fn i32 i32

fn token_id %fn i32 i32 \x:
    x

struct Pair:
    left %LocalToken
    right %LocalToken

fn observe %fn &Pair unit \_p:
    unit

fn main %fn void unit \void:
    let p %Pair Pair (LocalToken @token_id) (LocalToken @token_id)
    let borrowed %&Pair &p
    let left %LocalToken field::get p "left"
    observe borrowed
```

## move_deref_copy_reference_ok

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"move_deref_copy_reference_ok\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"copy value can be dereferenced and reused\" expected=\"0\" actual=\"0\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "std/test" as test

fn main %impure fn void i32 \void:
    let x %i32 7
    let y %i32 *&x
    let z %i32 x
    let actual %i32 0
    let report:
        test::test_report_new "move_deref_copy_reference_ok"
        |> test::test_report_push test::assert_eq_i32 "copy value can be dereferenced and reused" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## move_deref_non_copy_reference_rejected

neplg2:test[compile_fail]
diag_code: resource.borrow.move_from_shared
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw %fn i32 i32

fn token_id %fn i32 i32 \x:
    x

fn main %fn void unit \void:
    let x %LocalToken LocalToken @token_id
    let y %LocalToken *&x
```

## move_deref_non_copy_field_reference_rejected

neplg2:test[compile_fail]
diag_code: resource.borrow.move_from_shared
```neplg2
#entry main
#indent 4
#target core
#import "core/field" as field
#import "core/field" as *

struct LocalToken:
    raw %fn i32 i32

fn token_id %fn i32 i32 \x:
    x

struct Pair:
    token %LocalToken
    count %i32

fn main %fn void unit \void:
    let p %Pair Pair (LocalToken @token_id) 7
    let token %LocalToken *field::get_ref &p "token"
```

## move_branch_reinit_mixed

neplg2:test[compile_fail]
diag_code: resource.cell.possibly_moved
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw %fn i32 i32

fn token_id %fn i32 i32 \x:
    x

fn main %fn void unit \void:
    let mut x %LocalToken LocalToken @token_id
    let cnd %bool true
    if cnd:
        then:
            let y %LocalToken x
        else:
            set x LocalToken @token_id
    let z %LocalToken x
```

## move_nested_match_potentially_moved

neplg2:test[compile_fail]
diag_code: resource.cell.possibly_moved
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw %fn i32 i32
fn token_id %fn i32 i32 \x:
    x
enum BoolWrap:
    True
    False

fn main %fn void unit \void:
    let x %LocalToken LocalToken @token_id
    let a %BoolWrap BoolWrap::True
    match a:
        BoolWrap::True:
            match a:
                BoolWrap::True:
                    let y %LocalToken x
                    unit
                BoolWrap::False:
                    unit
        BoolWrap::False:
            unit
    let z %LocalToken x
```

## move_in_match_arms

neplg2:test[compile_fail]
diag_code: resource.cell.possibly_moved
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw %fn i32 i32
fn token_id %fn i32 i32 \x:
    x
enum BoolWrap:
    True
    False

fn main %fn void unit \void:
    let x %LocalToken LocalToken @token_id
    let v %BoolWrap BoolWrap::True
    match v:
        BoolWrap::True:
            let y %LocalToken x
            unit
        BoolWrap::False:
            unit
    let z %LocalToken x
```

## move_match_reference_payload_blocks_owner_move_while_live

neplg2:test[compile_fail]
diag_code: resource.borrow.move_from_shared
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw %fn i32 i32

fn token_id %fn i32 i32 \x:
    x

enum RefOpt:
    Some %&LocalToken
    None

fn main %fn void i32 \void:
    let x %LocalToken LocalToken @token_id
    let e %RefOpt RefOpt::Some &x
    match e:
        RefOpt::Some r:
            let y %LocalToken x
            let keep %&LocalToken r
            0
        RefOpt::None:
            0
```

## move_match_reference_payload_last_use_releases_owner

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"move_match_reference_payload_last_use_releases_owner\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"match payload borrow last use releases owner\" expected=\"0\" actual=\"0\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "std/test" as test

struct LocalToken:
    raw %fn i32 i32

fn token_id %fn i32 i32 \x:
    x

enum RefOpt:
    Some %&LocalToken
    None

fn main %impure fn void i32 \void:
    let x %LocalToken LocalToken @token_id
    let e %RefOpt RefOpt::Some &x
    let actual %i32 match e:
        RefOpt::Some r:
            let keep %&LocalToken r
            let y %LocalToken x
            0
        RefOpt::None:
            0
    let report:
        test::test_report_new "move_match_reference_payload_last_use_releases_owner"
        |> test::test_report_push test::assert_eq_i32 "match payload borrow last use releases owner" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## move_return_local_reference_err

neplg2:test[compile_fail]
diag_code: resource.borrow.return_escape
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw %fn i32 i32

fn token_id %fn i32 i32 \x:
    x

fn leak %fn void &LocalToken \void:
    let t %LocalToken LocalToken @token_id
    &t

fn main %fn void i32 \void:
    let r %&LocalToken leak
    0
```

## move_block_local_reference_escape_err

neplg2:test[compile_fail]
diag_code: resource.borrow.return_escape
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw %fn i32 i32

fn token_id %fn i32 i32 \x:
    x

fn main %fn void i32 \void:
    let r %&LocalToken block:
        let t %LocalToken LocalToken @token_id
        &t
    0
```

## move_set_outer_reference_to_inner_local_err

neplg2:test[compile_fail]
diag_code: resource.borrow.return_escape
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw %fn i32 i32

fn token_id %fn i32 i32 \x:
    x

fn main %fn void i32 \void:
    let outer %LocalToken LocalToken @token_id
    let mut r %&LocalToken &outer
    block:
        let inner %LocalToken LocalToken @token_id
        set r &inner
    0
```

## move_return_local_reference_inside_struct_err

neplg2:test[compile_fail]
diag_code: resource.borrow.return_escape
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw %fn i32 i32

struct RefBox:
    inner %&LocalToken

fn token_id %fn i32 i32 \x:
    x

fn leak %fn void RefBox \void:
    let t %LocalToken LocalToken @token_id
    let b %RefBox RefBox &t
    b

fn main %fn void i32 \void:
    let b %RefBox leak
    0
```

## move_block_local_reference_inside_struct_escape_err

neplg2:test[compile_fail]
diag_code: resource.borrow.return_escape
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw %fn i32 i32

struct RefBox:
    inner %&LocalToken

fn token_id %fn i32 i32 \x:
    x

fn main %fn void i32 \void:
    let b %RefBox block:
        let t %LocalToken LocalToken @token_id
        let local %RefBox RefBox &t
        local
    0
```

## move_set_outer_struct_reference_to_inner_local_err

neplg2:test[compile_fail]
diag_code: resource.borrow.return_escape
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw %fn i32 i32

struct RefBox:
    inner %&LocalToken

fn token_id %fn i32 i32 \x:
    x

fn main %fn void i32 \void:
    let outer %LocalToken LocalToken @token_id
    let mut b %RefBox RefBox &outer
    block:
        let inner %LocalToken LocalToken @token_id
        let local %RefBox RefBox &inner
        set b local
    0
```

## move_call_return_reference_to_block_local_err

neplg2:test[compile_fail]
diag_code: resource.borrow.return_escape
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw %fn i32 i32

fn token_id %fn i32 i32 \x:
    x

fn id_ref %fn &LocalToken &LocalToken \x:
    x

fn main %fn void i32 \void:
    let r %&LocalToken block:
        let t %LocalToken LocalToken @token_id
        id_ref &t
    0
```

## move_call_return_struct_reference_to_block_local_err

neplg2:test[compile_fail]
diag_code: resource.borrow.return_escape
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw %fn i32 i32

struct RefBox:
    inner %&LocalToken

fn token_id %fn i32 i32 \x:
    x

fn box_ref %fn &LocalToken RefBox \x:
    RefBox x

fn main %fn void i32 \void:
    let b %RefBox block:
        let t %LocalToken LocalToken @token_id
        box_ref &t
    0
```

## move_loop_owned_accumulator_reassigned_after_result_ok

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"move_loop_owned_accumulator_reassigned_after_result_ok\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"loop owned accumulator reinitialized each iteration\" expected=\"0\" actual=\"0\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "core/result" as *
#import "core/math" as *
#import "std/test" as test

struct LocalToken:
    raw %fn i32 i32

fn token_id %fn i32 i32 \x:
    x

fn step %fn LocalToken Result LocalToken i32 \token:
    Result::Ok token

fn main %impure fn void i32 \void:
    let mut cur %LocalToken LocalToken @token_id
    let mut i %i32 0
    while lt i 3:
        match step cur:
            Result::Ok next:
                set cur next
                set i add i 1
            Result::Err _e:
                #intrinsic "unreachable" <> ()
    let out %LocalToken cur
    let actual %i32 0
    let report:
        test::test_report_new "move_loop_owned_accumulator_reassigned_after_result_ok"
        |> test::test_report_push test::assert_eq_i32 "loop owned accumulator reinitialized each iteration" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## move_loop_owned_accumulator_err_continue_without_reinit_rejected

neplg2:test[compile_fail]
diag_code: resource.cell.possibly_moved
```neplg2
#entry main
#indent 4
#target core
#import "core/result" as *
#import "core/math" as *

struct LocalToken:
    raw %fn i32 i32

fn token_id %fn i32 i32 \x:
    x

fn step %fn LocalToken Result LocalToken i32 \token:
    Result::Ok token

fn main %fn void i32 \void:
    let mut cur %LocalToken LocalToken @token_id
    let mut i %i32 0
    while lt i 3:
        match step cur:
            Result::Ok next:
                set cur next
                set i add i 1
            Result::Err _e:
                set i 3
    let out %LocalToken cur
    0
```

## move_borrowed_field_projection_keeps_owner_until_reference_last_use

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"move_borrowed_field_projection_keeps_owner_until_reference_last_use\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"borrowed field projection releases before owner move\" expected=\"0\" actual=\"0\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "core/field" as field
#import "core/field" as *
#import "std/test" as test

struct LocalToken:
    raw %fn i32 i32

fn token_id %fn i32 i32 \x:
    x

struct Pair:
    token %LocalToken
    count %i32

fn observe %fn &LocalToken i32 \_w:
    1

fn consume %fn Pair i32 \_p:
    0

fn main %impure fn void i32 \void:
    let p %Pair Pair (LocalToken @token_id) 7
    let token_ref %&LocalToken field::get_ref &p "token"
    let count %i32 *field::get_ref &p "count"
    observe token_ref
    let actual %i32 consume p
    let report:
        test::test_report_new "move_borrowed_field_projection_keeps_owner_until_reference_last_use"
        |> test::test_report_push test::assert_eq_i32 "borrowed field projection releases before owner move" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## move_borrowed_field_projection_blocks_owner_move_while_live

neplg2:test[compile_fail]
diag_code: resource.borrow.move_from_shared
```neplg2
#entry main
#indent 4
#target core
#import "core/field" as field
#import "core/field" as *

struct LocalToken:
    raw %fn i32 i32

fn token_id %fn i32 i32 \x:
    x

struct Pair:
    token %LocalToken
    count %i32

fn observe %fn &LocalToken i32 \_w:
    1

fn consume %fn Pair i32 \_p:
    0

fn main %fn void i32 \void:
    let p %Pair Pair (LocalToken @token_id) 7
    let token_ref %&LocalToken field::get_ref &p "token"
    consume p
    observe token_ref
```

## move_borrowed_field_projection_escape_rejected

neplg2:test[compile_fail]
diag_code: resource.borrow.return_escape
```neplg2
#entry main
#indent 4
#target core
#import "core/field" as field
#import "core/field" as *

struct LocalToken:
    raw %fn i32 i32

fn token_id %fn i32 i32 \x:
    x

struct Pair:
    token %LocalToken
    count %i32

fn leak %fn void &LocalToken \void:
    let p %Pair Pair (LocalToken @token_id) 7
    field::get_ref &p "token"

fn main %fn void i32 \void:
    let r %&LocalToken leak
    0
```

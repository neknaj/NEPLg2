# reference codegen tests

## scalar addr-of then deref returns the scalar value

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"scalar_addr_of_then_deref_returns_the_scalar_value\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"addr-of then deref scalar\" expected=\"6\" actual=\"6\" message=\"\"\n"
```neplg2
#entry main
#target std

#import "std/test" as *

fn deref_i32 %fn &i32 i32 \x:
    *x

fn main %impure fn unit i32 \unit:
    let a %i32 6
    let actual %i32 deref_i32 &a
    let report:
        test_report_new "scalar_addr_of_then_deref_returns_the_scalar_value"
        |> test_report_push assert_eq_i32 "addr-of then deref scalar" 6 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## stdlib clone of i32 through a reference returns the scalar value

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"stdlib_clone_of_i32_through_a_reference_returns_the_scalar_value\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"clone i32 through reference\" expected=\"6\" actual=\"6\" message=\"\"\n"
```neplg2
#entry main
#target std

#import "core/traits/copy" as *
#import "std/test" as *

fn clone_i32 %fn i32 i32 \x:
    Clone::clone &x

fn main %impure fn unit i32 \unit:
    let actual %i32 clone_i32 6
    let report:
        test_report_new "stdlib_clone_of_i32_through_a_reference_returns_the_scalar_value"
        |> test_report_push assert_eq_i32 "clone i32 through reference" 6 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## stdlib clone of generic MemPtr impl resolves before backend

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"stdlib_clone_of_generic_MemPtr_impl_resolves_before_backend\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"clone generic MemPtr address\" expected=\"32\" actual=\"32\" message=\"\"\n"
```neplg2
#entry main
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/traits/copy" as *
#import "std/test" as *

fn clone_ptr_addr %fn MemPtr u8 i32 \p:
    let q %MemPtr u8 Clone::clone &p
    mem_ptr_addr q

fn main %impure fn unit i32 \unit:
    let actual %i32 clone_ptr_addr mem_ptr_wrap<u8> 32
    let report:
        test_report_new "stdlib_clone_of_generic_MemPtr_impl_resolves_before_backend"
        |> test_report_push assert_eq_i32 "clone generic MemPtr address" 32 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## borrowed enum match binds scalar payload by reference

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"borrowed_enum_match_binds_scalar_payload_by_reference\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"borrowed enum scalar payload\" expected=\"42\" actual=\"42\" message=\"\"\n"
```neplg2
#entry main
#target std
#indent 4

#import "std/test" as *

enum LocalBox:
    Empty
    Full %i32

fn read_box %fn &LocalBox i32 \box:
    match box:
        Empty:
            0
        Full value:
            *value

fn main %impure fn unit i32 \unit:
    let box %LocalBox LocalBox::Full 42
    let actual %i32 read_box &box
    let report:
        test_report_new "borrowed_enum_match_binds_scalar_payload_by_reference"
        |> test_report_push assert_eq_i32 "borrowed enum scalar payload" 42 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## owned enum match preserves reference payload value

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"owned_enum_match_preserves_reference_payload_value\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"owned enum reference payload\" expected=\"57\" actual=\"57\" message=\"\"\n"
```neplg2
#entry main
#target std
#indent 4

#import "std/test" as *

enum RefOpt:
    None
    Some %&i32

fn read_ref_opt %fn RefOpt i32 \opt:
    match opt:
        RefOpt::None:
            0
        RefOpt::Some r:
            *r

fn main %impure fn unit i32 \unit:
    let x %i32 57
    let opt %RefOpt RefOpt::Some &x
    let actual %i32 read_ref_opt opt
    let report:
        test_report_new "owned_enum_match_preserves_reference_payload_value"
        |> test_report_push assert_eq_i32 "owned enum reference payload" 57 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## borrowed enum match does not move owner payload

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"borrowed_enum_match_does_not_move_owner_payload\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"borrowed enum owner payload remains consumable\" expected=\"0\" actual=\"0\" message=\"\"\n"
```neplg2
#entry main
#target std
#indent 4

#import "core/math" as *
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/result" as *
#import "std/test" as *

struct LocalToken:
    value %i32

enum TokenBox:
    Empty
    Owned %RegionToken LocalToken

fn token_box_addr %fn &TokenBox i32 \box:
    match box:
        Empty:
            0
        Owned token:
            mem_ptr_addr region_ptr token

fn run_case %fn unit i32 \unit:
    match alloc_region<LocalToken> 1:
        Result::Err _:
            1
        Result::Ok token:
            let box %TokenBox TokenBox::Owned token
            let addr %i32 token_box_addr &box
            match box:
                Empty:
                    1
                Owned owned_token:
                    match dealloc_region<LocalToken> owned_token:
                        Result::Err _:
                            1
                        Result::Ok _:
                            if gt addr 0 0 1

fn main %impure fn unit i32 \unit:
    let actual %i32 run_case
    let report:
        test_report_new "borrowed_enum_match_does_not_move_owner_payload"
        |> test_report_push assert_eq_i32 "borrowed enum owner payload remains consumable" 0 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

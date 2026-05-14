# reference codegen tests

## scalar addr-of then deref returns the scalar value

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"scalar_addr_of_then_deref_returns_the_scalar_value\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"addr-of then deref scalar\" expected=\"6\" actual=\"6\" message=\"\"\n"
```neplg2
#entry main
#target std

#import "std/test" as *

fn deref_i32 <(&i32)->i32> (x):
    *x

fn main <()*>i32> ():
    let a <i32> 6
    let actual <i32> deref_i32 &a
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

fn clone_i32 <(i32)->i32> (x):
    Clone::clone &x

fn main <()*>i32> ():
    let actual <i32> clone_i32 6
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

fn clone_ptr_addr <(MemPtr<u8>)->i32> (p):
    let q <MemPtr<u8>> Clone::clone &p
    mem_ptr_addr q

fn main <()*>i32> ():
    let actual <i32> clone_ptr_addr mem_ptr_wrap<u8> 32
    let report:
        test_report_new "stdlib_clone_of_generic_MemPtr_impl_resolves_before_backend"
        |> test_report_push assert_eq_i32 "clone generic MemPtr address" 32 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

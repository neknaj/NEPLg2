# stdlib/bitset.n.md

## bitset_insert_remove_and_len

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"bitset_insert_remove_and_len\" count=3 failed=0\nassertion index=0 status=ok kind=bool label=\"contains inserted bit\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"removed bit absent\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"bitset len\" expected=\"32\" actual=\"32\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/bitset" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/math" as *
#import "std/test" as *

fn main %impure fn () i32 \():
    let bs %BitSet:
        unwrap_ok<BitSet, Diag> new 32
        |> insert 1 |> uwok
        |> insert 7 |> uwok
        |> insert 15 |> uwok
        |> remove 7 |> uwok
    let ok0 %bool unwrap_ok<bool, Diag> contains &bs 1;
    let ok1 %bool not unwrap_ok<bool, Diag> contains &bs 7;
    let size %i32 len &bs;
    free bs
    let report:
        test_report_new "bitset_insert_remove_and_len"
        |> test_report_push assert "contains inserted bit" ok0
        |> test_report_push assert "removed bit absent" ok1
        |> test_report_push assert_eq_i32 "bitset len" 32 size
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## bitset_clear_and_fill

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"bitset_clear_and_fill\" count=2 failed=0\nassertion index=0 status=ok kind=bool label=\"clear removes inserted bit\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"fill sets highest bit\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/bitset" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/math" as *
#import "std/test" as *

fn main %impure fn () i32 \():
    let bs0 %BitSet:
        unwrap_ok<BitSet, Diag> new 10
        |> insert 2 |> uwok
        |> clear
    let ok0 %bool not unwrap_ok<bool, Diag> contains &bs0 2;
    free bs0
    let bs1 %BitSet fill unwrap_ok<BitSet, Diag> new 10;
    let ok1 %bool unwrap_ok<bool, Diag> contains &bs1 9;
    free bs1
    let report:
        test_report_new "bitset_clear_and_fill"
        |> test_report_push assert "clear removes inserted bit" ok0
        |> test_report_push assert "fill sets highest bit" ok1
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## bitset_update_error_returns_owner

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"bitset_update_error_returns_owner\" count=2 failed=0\nassertion index=0 status=ok kind=bool label=\"insert error returns owner\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"remove error returns owner\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/bitset" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/math" as *
#import "std/test" as *

fn main %impure fn () i32 \():
    let bs0 %BitSet unwrap_ok<BitSet, Diag> new 12;
    let ok0 %bool:
        match insert bs0 99:
            Result::Ok next:
                free next
                false
            Result::Err e:
                let recovered %BitSet bitset_update_error_owner e
                let ok %bool eq len &recovered 12
                free recovered
                ok
    let bs1 %BitSet unwrap_ok<BitSet, Diag> new 12;
    let ok1 %bool:
        match remove bs1 sub 0 1:
            Result::Ok next:
                free next
                false
            Result::Err e:
                let recovered %BitSet bitset_update_error_owner e
                let ok %bool eq len &recovered 12
                free recovered
                ok
    let report:
        test_report_new "bitset_update_error_returns_owner"
        |> test_report_push assert "insert error returns owner" ok0
        |> test_report_push assert "remove error returns owner" ok1
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

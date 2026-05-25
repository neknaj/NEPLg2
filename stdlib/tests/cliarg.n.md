# stdlib/cliarg.n.md

## cliarg_basic

neplg2:test[stdio, normalize_newlines]
argv: ["--flag", "value"]
exit_code: 0
stdout: "test_report name=\"cliarg_basic\" count=3 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"argc includes program and injected args\" expected=\"3\" actual=\"3\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"negative index rejected\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"end index rejected\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/env/cliarg" as *
#import "core/option" as *
#import "std/test" as *

fn main %impure fn () i32 \():
    let c %i32 cliarg_count;
    let neg_missing %bool is_none cliarg_get -1;
    let end_missing %bool is_none cliarg_get c;
    let report:
        test_report_new "cliarg_basic"
        |> test_report_push assert_eq_i32 "argc includes program and injected args" 3 c
        |> test_report_push assert "negative index rejected" neg_missing
        |> test_report_push assert "end index rejected" end_missing
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## cliarg_argv_stdout_count

neplg2:test[assert_io]
argv: ["--flag", "value"]
stdout: "3"
```neplg2
#entry main
#indent 4
#target std

#import "std/env/cliarg" as *
#import "std/stdio" as *

fn main %impure fn () () \():
    print_i32 cliarg_count;
```

## cliarg_get_reads_injected_argv_values

neplg2:test[assert_io]
argv: ["--flag", "value"]
stdout: "--flag:value"
```neplg2
#entry main
#indent 4
#target std

#import "std/env/cliarg" as *
#import "std/stdio" as *
#import "alloc/string" as *
#import "core/option" as *

fn print_arg %impure fn i32 () \idx:
    match cliarg_get idx:
        Option::Some arg:
            print arg
        Option::None:
            print "<none>"

fn main %impure fn () () \():
    print_arg 1;
    print ":";
    print_arg 2;
```

## cliarg_get_rejects_out_of_range

neplg2:test[stdio, normalize_newlines]
argv: ["--flag", "value"]
exit_code: 0
stdout: "test_report name=\"cliarg_get_rejects_out_of_range\" count=3 failed=0\nassertion index=0 status=ok kind=bool label=\"negative index rejected\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"raw negative index rejected\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"end index rejected\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/env/cliarg" as *
#import "std/env/cliarg/raw" as cli_raw
#import "core/option" as *
#import "std/test" as *

fn main %impure fn () i32 \():
    let c %i32 cliarg_count;
    let neg_missing %bool is_none cliarg_get -1;
    let raw_neg_missing %bool is_none cli_raw::cliarg_get_checked -1;
    let end_missing %bool is_none cliarg_get c;
    let report:
        test_report_new "cliarg_get_rejects_out_of_range"
        |> test_report_push assert "negative index rejected" neg_missing
        |> test_report_push assert "raw negative index rejected" raw_neg_missing
        |> test_report_push assert "end index rejected" end_missing
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## cliarg_cstr_bounded_conversion_reports

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"cliarg_cstr_bounded_conversion_reports\" count=3 failed=0\nassertion index=0 status=ok kind=bool label=\"bounded cstr length stops at nul\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"bounded cstr conversion validates utf8\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"missing nul in bound is rejected\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/env/cliarg/cstr" as *
#import "alloc/string/storage" as *
#import "alloc/string" as *
#import "core/result" as *
#import "core/math" as *
#import "std/test" as *

fn main %impure fn () i32 \():
    let p %MemPtr u8 string_data_ptr "nep\0tail";
    let no_nul %MemPtr u8 string_data_ptr "abc";
    let len_ok %bool match cstr_len_bounded_result p 4:
        Result::Ok n:
            eq n 3
        Result::Err _:
            false
    let str_ok %bool match cstr_to_str_bounded_result p 4:
        Result::Ok s:
            str_eq s "nep"
        Result::Err _:
            false
    let missing_ok %bool match cstr_len_bounded_result no_nul 3:
        Result::Ok _:
            false
        Result::Err _:
            true
    let report:
        test_report_new "cliarg_cstr_bounded_conversion_reports"
        |> test_report_push assert "bounded cstr length stops at nul" len_ok
        |> test_report_push assert "bounded cstr conversion validates utf8" str_ok
        |> test_report_push assert "missing nul in bound is rejected" missing_ok
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## cliarg_cstr_len_unbounded_is_not_public

neplg2:test[compile_fail]
diag_code: resolve.identifier.undefined
```neplg2
#entry main
#indent 4
#target std

#import "std/env/cliarg/cstr" as *
#import "alloc/string/storage" as *

fn main %impure fn () () \():
    let p %MemPtr u8 string_data_ptr "nep\0";
    let _n cstr_len p;
```

## cliarg_cstr_to_str_unbounded_is_not_public

neplg2:test[compile_fail]
diag_code: resolve.identifier.undefined
```neplg2
#entry main
#indent 4
#target std

#import "std/env/cliarg/cstr" as *
#import "alloc/string/storage" as *

fn main %impure fn () () \():
    let p %MemPtr u8 string_data_ptr "nep\0";
    let _s cstr_to_str p;
```

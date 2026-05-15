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

fn main <()*>i32> ():
    let c <i32> cliarg_count;
    let neg_missing <bool> is_none<str> cliarg_get -1;
    let end_missing <bool> is_none<str> cliarg_get c;
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

fn main <()*>()> ():
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

fn print_arg <(i32)*>()> (idx):
    match cliarg_get idx:
        Option::Some arg:
            print arg
        Option::None:
            print "<none>"

fn main <()*>()> ():
    print_arg 1;
    print ":";
    print_arg 2;
```

## cliarg_get_rejects_out_of_range

neplg2:test[stdio, normalize_newlines]
argv: ["--flag", "value"]
exit_code: 0
stdout: "test_report name=\"cliarg_get_rejects_out_of_range\" count=2 failed=0\nassertion index=0 status=ok kind=bool label=\"negative index rejected\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"end index rejected\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/env/cliarg" as *
#import "core/option" as *
#import "std/test" as *

fn main <()*>i32> ():
    let c <i32> cliarg_count;
    let neg_missing <bool> is_none<str> cliarg_get -1;
    let end_missing <bool> is_none<str> cliarg_get c;
    let report:
        test_report_new "cliarg_get_rejects_out_of_range"
        |> test_report_push assert "negative index rejected" neg_missing
        |> test_report_push assert "end index rejected" end_missing
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## cliarg_cstr_requires_mem_ptr

neplg2:test[compile_fail]
diag_code: type.overload.no_match
```neplg2
#entry main
#indent 4
#target std

#import "std/env/cliarg/cstr" as *

fn main <()*>()> ():
    let _n cstr_len 0;
```

## cliarg_cstr_to_str_requires_mem_ptr

neplg2:test[compile_fail]
diag_code: type.overload.no_match
```neplg2
#entry main
#indent 4
#target std

#import "std/env/cliarg/cstr" as *

fn main <()*>()> ():
    let _s cstr_to_str 0;
```

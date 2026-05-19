# stdlib/fs.n.md

## fs_main

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"fs_main\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"missing file returns error\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std

#import "std/fs" as *
#import "alloc/string" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as *

fn main <()*>i32> ():
    let report:
        match fs_read_to_string "__definitely_missing_file__.txt":
            Result::Ok s:
                let _drop <()> test_consume_str s;
                test_report_new "fs_main"
                |> test_report_push test_assertion_failed AssertionKind::Bool "missing file returns error" "true" "false" "fs_read_to_string unexpectedly succeeded"
            Result::Err e:
                test_report_new "fs_main"
                |> test_report_push assert "missing file returns error" true
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

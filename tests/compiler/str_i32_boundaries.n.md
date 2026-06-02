# str/i32 boundaries

## str_annotation_rejects_raw_i32

neplg2:test[compile_fail]
diag_codes: type.annotation.mismatch
```neplg2
#entry main
#indent 4
#target core

fn main %fn void i32 \void:
    let s %str 0
    0
```

## raw_i32_annotation_rejects_string_literal

neplg2:test[compile_fail]
diag_codes: type.annotation.mismatch, type.return.mismatch
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *

fn main %fn void i32 \void:
    let p %i32 "not a pointer"
    p
```

## string_literal_remains_str

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"string_literal_remains_str\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"string literal equality result\" expected=\"1\" actual=\"1\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/string" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let actual %i32 if str_eq "ok" "ok" 1 0
    let report:
        test_report_new "string_literal_remains_str"
        |> test_report_push assert_eq_i32 "string literal equality result" 1 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

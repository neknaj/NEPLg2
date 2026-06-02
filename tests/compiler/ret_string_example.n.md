# ret_string_example

`ret:` に JSON 文字列（"..."）を指定し、`main` の戻り値（i32 ポインタ）を NEPL の `str` 表現（[len:u32][bytes...]）として復号して比較します。

## return_str

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"return_str\" count=1 failed=0\nassertion index=0 status=ok kind=str_eq label=\"returned string value\" expected=\"hello\" actual=\"hello\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

fn main %impure fn void i32\void:
    let actual %str "hello"
    let report:
        test_report_new "return_str"
        |> test_report_push assert_str_eq "returned string value" "hello" actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

# GUI native font resource provider ABI fixture

F5nn native guest provider の snapshot session import と、host adapter 未設定時の fail-closed sentinel を実行確認する。filesystem adapter、resource-root configuration、success payload は後続sliceで追加する。

## default host imports fail closed

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_native_font_resource_provider_abi\" count=4 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"open\" expected=\"-1\" actual=\"-1\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"byte len\" expected=\"-1\" actual=\"-1\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"read bytes\" expected=\"-1\" actual=\"-1\" message=\"\"\nassertion index=3 status=ok kind=eq_i32 label=\"close\" expected=\"-1\" actual=\"-1\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "std/test" as test

#extern "nepl_gui_native" "font_resource_open" fn font_resource_open %impure fn i32 impure fn i32 impure fn i32 i32
#extern "nepl_gui_native" "font_resource_byte_len" fn font_resource_byte_len %impure fn i32 i32
#extern "nepl_gui_native" "font_resource_read_bytes" fn font_resource_read_bytes %impure fn i32 impure fn i32 impure fn i32 i32
#extern "nepl_gui_native" "font_resource_close" fn font_resource_close %impure fn i32 i32

fn main %impure fn void i32 \void:
    let report:
        test::test_report_new "gui_native_font_resource_provider_abi"
        |> test::test_report_push test::assert_eq_i32 "open" -1 font_resource_open 0 0 0
        |> test::test_report_push test::assert_eq_i32 "byte len" -1 font_resource_byte_len 1
        |> test::test_report_push test::assert_eq_i32 "read bytes" -1 font_resource_read_bytes 1 0 0
        |> test::test_report_push test::assert_eq_i32 "close" -1 font_resource_close 1
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

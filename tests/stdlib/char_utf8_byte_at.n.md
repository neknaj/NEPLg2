# char UTF-8 byte accessor

`char_utf8_byte_at` が、存在する UTF-8 byte だけを `Some` として返し、存在しない byte index を `None` として返すことを確認します。

## char_utf8_byte_at_reports_absent_bytes

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok,ok,ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
    ##: [3] ok
    ##: [4] ok
    ##: [5] ok
    ##: [6] ok
    ##: [7] ok
```neplg2
#entry main
#target std
#indent 4

#import "core/char" as *
#import "core/option" as *
#import "std/test" as *

fn expect_byte %fn Option i32 fn i32 TestAssertion \got\expected:
    match got:
        Option::Some byte:
            assert_eq_i32 expected byte
        Option::None:
            assert false

fn main %impure fn void i32 \void:
    let ascii %char 'A'
    let cent %char '\u{00A2}'
    let hira %char '\u{3042}'
    let mark %char '\u{1F4AF}'
    let checks:
        checks_new
        |> checks_push expect_byte char_utf8_byte_at ascii 0 65
        |> checks_push assert is_none char_utf8_byte_at ascii 1
        |> checks_push expect_byte char_utf8_byte_at cent 0 194
        |> checks_push expect_byte char_utf8_byte_at cent 1 162
        |> checks_push expect_byte char_utf8_byte_at hira 2 130
        |> checks_push assert is_none char_utf8_byte_at hira 3
        |> checks_push expect_byte char_utf8_byte_at mark 3 175
        |> checks_push assert is_none char_utf8_byte_at mark -1
    let shown checks_print_report checks
    checks_exit_code shown
```

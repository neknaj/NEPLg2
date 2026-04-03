# lexer diagnostics 回帰

## lexer_invalid_escape_reports_diag_id

neplg2:test[compile_fail]
diag_id: 1208
```neplg2
#entry main
#indent 4

fn main <()->i32> ():
    let s "bad\q";
    0
```

## lexer_unterminated_string_reports_diag_id

neplg2:test[compile_fail]
diag_id: 1209
```neplg2
#entry main
#indent 4

fn main <()->i32> ():
    let s "unterminated
    0
```

## lexer_invalid_pub_prefix_reports_diag_id

neplg2:test[compile_fail]
diag_id: 1205
```neplg2
#entry main
#indent 4

pub #target core

fn main <()->i32> ():
    0
```

# lexer diagnostics 回帰

## lexer_invalid_escape_reports_diag_code

neplg2:test[compile_fail]
diag_code: lexer.string.invalid_escape
```neplg2
#entry main
#indent 4

fn main %fn void i32 \void:
    let s "bad\q";
    0
```

## lexer_unterminated_string_reports_diag_code

neplg2:test[compile_fail]
diag_code: lexer.string.unterminated
```neplg2
#entry main
#indent 4

fn main %fn void i32 \void:
    let s "unterminated
    0
```

## lexer_invalid_pub_prefix_reports_diag_code

neplg2:test[compile_fail]
diag_code: lexer.pub_prefix.invalid
```neplg2
#entry main
#indent 4

pub #target core

fn main <()->i32> ():
    0
```

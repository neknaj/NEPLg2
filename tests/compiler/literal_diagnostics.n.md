# literal_diagnostics

## integer_literal_out_of_i32_range_reports_diag_code

neplg2:test[compile_fail]
diag_code: type.literal.int_invalid
```neplg2
#entry main
#indent 4
#target core

fn main %fn () i32 \():
    999999999999999999999999999999
```

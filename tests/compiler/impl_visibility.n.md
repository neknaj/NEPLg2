# impl visibility diagnostics

## pub_impl_visibility_is_rejected

neplg2:test[compile_fail]
diag_code: parser.impl.visibility_invalid
```neplg2
#entry main
#indent 4

trait Show:
    fn show %fn Self i32 \x:
        x

pub impl Show for i32:
    fn show %fn i32 i32 \x:
        x

fn main %fn unit i32 \unit:
    Show::show 1
```

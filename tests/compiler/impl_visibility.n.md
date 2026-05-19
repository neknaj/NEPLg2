# impl visibility diagnostics

## pub_impl_visibility_is_rejected

neplg2:test[compile_fail]
diag_code: parser.impl.visibility_invalid
```neplg2
#entry main
#indent 4

trait Show:
    fn show <(Self)->i32> (x):
        x

pub impl Show for i32:
    fn show <(i32)->i32> (x):
        x

fn main <()->i32> ():
    Show::show 1
```

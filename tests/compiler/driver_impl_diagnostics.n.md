# driver_impl_diagnostics

## inherent_impl_reports_diag_code

neplg2:test[compile_fail]
diag_code: type.impl.inherent_unsupported
```neplg2
#entry main
#indent 4
#target core

impl i32:
    fn id %fn i32 i32 \x:
        x

fn main %fn () i32 \():
    0
```

## impl_duplicate_method_reports_diag_code

neplg2:test[compile_fail]
diag_code: type.impl.duplicate_method
```neplg2
#entry main
#indent 4
#target core

trait Show:
    fn show %fn Self i32 \x:
        x

impl Show for i32:
    fn show %fn i32 i32 \x:
        x
    fn show %fn i32 i32 \x:
        x

fn main %fn () i32 \():
    0
```

## impl_method_type_params_report_diag_code

neplg2:test[compile_fail]
diag_code: type.trait_method.type_params_unsupported
```neplg2
#entry main
#indent 4
#target core

trait Show:
    fn show %fn Self i32 \x:
        x

impl Show for i32:
    fn show <.T> %fn i32 i32 \x:
        x

fn main %fn () i32 \():
    0
```

## impl_method_not_in_trait_reports_diag_code

neplg2:test[compile_fail]
diag_code: type.impl.method_not_in_trait
```neplg2
#entry main
#indent 4
#target core

trait Show:
    fn show %fn Self i32 \x:
        x

impl Show for i32:
    fn show %fn i32 i32 \x:
        x
    fn extra %fn i32 i32 \x:
        x

fn main %fn () i32 \():
    0
```

## impl_method_signature_mismatch_reports_diag_code

neplg2:test[compile_fail]
diag_code: type.impl.method_signature_mismatch
```neplg2
#entry main
#indent 4
#target core

trait Show:
    fn show %fn Self i32 \x:
        x

impl Show for i32:
    fn show %fn i32 i64 \x:
        x

fn main %fn () i32 \():
    0
```

## impl_missing_trait_method_reports_diag_code

neplg2:test[compile_fail]
diag_code: type.impl.missing_trait_method
```neplg2
#entry main
#indent 4
#target core

trait Pair:
    fn a %fn Self i32 \x:
        x
    fn b %fn Self i32 \x:
        x

impl Pair for i32:
    fn a %fn i32 i32 \x:
        x

fn main %fn () i32 \():
    0
```

## unknown_trait_in_impl_reports_diag_code

neplg2:test[compile_fail]
diag_code: type.trait.unknown
```neplg2
#entry main
#indent 4
#target core

impl Missing for i32:
    fn f %fn i32 i32 \x:
        x

fn main %fn () i32 \():
    0
```

## impl_trait_type_arg_count_reports_diag_code

neplg2:test[compile_fail]
diag_code: type.trait.type_params_unsupported
```neplg2
#entry main
#indent 4
#target core
#import "core/field" as *

trait Boxy<.T>:
    fn get %fn Self .T \x:
        unreachable

impl Boxy<i32, i32> for i32:
    fn get %fn i32 i32 \x:
        x

fn main %fn () i32 \():
    0
```

## duplicate_impl_reports_diag_code

neplg2:test[compile_fail]
diag_code: type.impl.duplicate_for_trait_target
```neplg2
#entry main
#indent 4
#target core

trait Show:
    fn show %fn Self i32 \x:
        x

impl Show for i32:
    fn show %fn i32 i32 \x:
        x

impl Show for i32:
    fn show %fn i32 i32 \x:
        x

fn main %fn () i32 \():
    0
```

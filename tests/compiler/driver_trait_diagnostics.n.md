# driver_trait_diagnostics

## unknown_trait_capability_reports_diag_code

neplg2:test[compile_fail]
diag_code: type.trait_capability.unknown
```neplg2
#entry main
#indent 4
#target core

trait BadCap:
    #capability cpoy
    fn f %fn Self Self \x:
        x

fn main %fn void unit \void:
    unit
```

## trait_method_type_params_report_diag_code

neplg2:test[compile_fail]
diag_code: type.trait_method.type_params_unsupported
```neplg2
#entry main
#indent 4
#target core
#import "core/field" as *

trait Boxy:
    fn get <.T> %fn Self .T \x:
        x

fn main %fn void unit \void:
    unit
```

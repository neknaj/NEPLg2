# driver_declaration_diagnostics

## enum_type_param_bounds_report_diag_code

neplg2:test[compile_fail]
diag_code: type.enum.type_param_bounds_unsupported
```neplg2
#entry main
#indent 4
#target core

trait Marker:
    fn mark <(Self)->i32> (_self):
        0

enum Box<.T: Marker>:
    Item <.T>

fn main <()->()> ():
    ()
```

## struct_type_param_bounds_report_diag_code

neplg2:test[compile_fail]
diag_code: type.struct.type_param_bounds_unsupported
```neplg2
#entry main
#indent 4
#target core

trait Marker:
    fn mark <(Self)->i32> (_self):
        0

struct Box<.T: Marker>:
    value <.T>

fn main <()->()> ():
    ()
```

## duplicate_enum_name_reports_diag_code

neplg2:test[compile_fail]
diag_code: resolve.item.name_conflict
```neplg2
#entry main
#indent 4
#target core

enum Foo:
    A

enum Foo:
    B

fn main <()->()> ():
    ()
```

## duplicate_struct_name_reports_diag_code

neplg2:test[compile_fail]
diag_code: resolve.item.name_conflict
```neplg2
#entry main
#indent 4
#target core

struct Foo:
    value <i32>

struct Foo:
    other <i32>

fn main <()->()> ():
    ()
```

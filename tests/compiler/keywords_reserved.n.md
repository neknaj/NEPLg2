# 予約語は識別子として使えないことの確認

## reserved_cond_cannot_be_identifier

neplg2:test[compile_fail]
diag_codes: parser.identifier.reserved_keyword
```neplg2
#entry main
#indent 4
#target core

fn main %fn unit i32 \unit:
    let cond 1;
    cond
```

## reserved_then_cannot_be_identifier

neplg2:test[compile_fail]
diag_codes: parser.identifier.reserved_keyword
```neplg2
#entry main
#indent 4
#target core

fn main %fn unit i32 \unit:
    let then 1;
    then
```

## reserved_else_cannot_be_identifier

neplg2:test[compile_fail]
diag_codes: parser.identifier.reserved_keyword
```neplg2
#entry main
#indent 4
#target core

fn main %fn unit i32 \unit:
    let else 1;
    else
```

## reserved_do_cannot_be_identifier

neplg2:test[compile_fail]
diag_codes: parser.identifier.reserved_keyword
```neplg2
#entry main
#indent 4
#target core

fn main %fn unit i32 \unit:
    let do 1;
    do
```

## reserved_unit_cannot_be_identifier

neplg2:test[compile_fail]
diag_codes: parser.identifier.reserved_keyword
```neplg2
#entry main
#indent 4
#target core

fn main %fn unit i32 \unit:
    let unit 1;
    unit
```

## reserved_let_cannot_be_fn_name

neplg2:test[compile_fail]
diag_codes: parser.identifier.reserved_keyword, parser.token.unexpected, parser.identifier.expected
```neplg2
#entry main
#indent 4
#target core

fn let %fn unit i32 \unit:
    1

fn main %fn unit i32 \unit:
    let
```

## reserved_fn_cannot_be_parameter

neplg2:test[compile_fail]
diag_codes: parser.identifier.reserved_keyword, parser.token.unexpected
```neplg2
#entry main
#indent 4
#target core

fn id %fn i32 i32 \fn:
    fn

fn main %fn unit i32 \unit:
    id 1
```

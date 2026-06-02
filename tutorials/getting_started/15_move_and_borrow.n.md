# move と borrow

非 Copy の値を別名へ束縛すると、所有権は移動します。移動後の値を再利用するコードは静的検査で拒否されます。

neplg2:test[compile_fail]
diag_code: resource.cell.moved
```neplg2
#entry main
#indent 4
#target core

struct Token:
    raw %fn i32 i32

fn id %fn i32 i32 \x:
    x

fn main %fn void i32 \void:
    let a %Token Token @id
    let b %Token a
    let c %Token a
    0
```

`i32`、`bool`、`char`、`str` などの Copy 値は再利用できます。`Vec` や `ByteBuf` のような owner は Copy ではないため、関数へ渡した後に同じ owner を使い続けないようにします。

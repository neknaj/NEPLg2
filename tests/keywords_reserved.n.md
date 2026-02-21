# cond/then/else/do の予約語テスト

`cond` / `then` / `else` / `do` はレイアウト制御キーワードであり、識別子としては使えないことを固定します。

neplg2:test[compile_fail]
```neplg2
| #entry main
| #indent 4
| #target wasm
fn main <()->i32> ():
    let cond 1;
    cond
```

neplg2:test[compile_fail]
```neplg2
| #entry main
| #indent 4
| #target wasm
fn then <()->i32> ():
    1
fn main <()->i32> ():
    then
```

neplg2:test[compile_fail]
```neplg2
| #entry main
| #indent 4
| #target wasm
fn main <()->i32> ():
    let else 2;
    else
```

neplg2:test[compile_fail]
```neplg2
| #entry main
| #indent 4
| #target wasm
fn do <()->i32> ():
    1
fn main <()->i32> ():
    do
```

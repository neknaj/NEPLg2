# ret_f64_example

f64 の戻り値を `ret:` で検査できることを確認するための最小テストです（通過しなくてもよい）。

## return_f64

neplg2:test[compile_fail]
```neplg2
#entry main
#indent 4

fn main <()->f64>():
    1.25
```

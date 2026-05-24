# ret_f64_example

f64 の戻り値を `ret:` で検査できることを確認するための最小テストです（通過しなくてもよい）。

## return_f64

neplg2:test[compile_fail]
diag_codes: type.annotation.mismatch, type.return.mismatch
```neplg2
#entry main
#indent 4

fn main %fn () f64():
    1.25
```

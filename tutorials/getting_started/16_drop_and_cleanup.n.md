# cleanup と Drop 方針

現在の stdlib では、所有権を持つ collection や buffer は明示的に解放する API を持っています。将来の Drop elaboration と Resource IR では、compiler が安全な解放経路を広げていく計画です。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok]
    ##: [0] ok
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "alloc/collections/vec" as *
#import "core/result" as *
#import "std/test" as *

fn main <()*>i32> ():
    match new<i32>:
        Result::Err _e:
            let checks checks_push checks_new Result<(),str>::Err "vec.new failed"
            let shown checks_print_report checks
            checks_exit_code shown
        Result::Ok values:
            let checks:
                checks_new
                |> checks_push assert_eq_i32 0 len<i32> &values
            free<i32> values;
            let shown checks_print_report checks
            checks_exit_code shown
```

raw memory の確保や pointer 計算は通常の tutorial では扱いません。public API だけで書けない場合は、stdlib 側の抽象 API が不足している可能性があります。

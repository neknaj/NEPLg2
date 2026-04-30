# collection の読み取り

collection から値を読むときは、「owner を消費する操作」と「借用して Copy 値だけ読む操作」を分けます。`get_ref` は `&Vec<T>` を受け取り、`.T: Copy` の要素だけを返します。

neplg2:test
ret: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
    ##: [3] ok
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "alloc/collections/vec" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn has_at <(&Vec<i32>,i32,i32)->bool> (v, idx, expected):
    match get_ref<i32> v idx:
        Option::Some value:
            eq value expected
        Option::None:
            false

fn main <()*>i32> ():
    match filled<i32> 3 7:
        Result::Err _e:
            let checks checks_push checks_new Result<(),str>::Err "vec.filled failed"
            let shown checks_print_report checks
            checks_exit_code shown
        Result::Ok values:
            let n <i32> len_ref<i32> &values
            let has0 <bool> has_at &values 0 7
            let has2 <bool> has_at &values 2 7
            let has3 <bool> has_at &values 3 7
            let checks:
                checks_new
                |> checks_push assert_eq_i32 3 n
                |> checks_push assert has0
                |> checks_push assert has2
                |> checks_push assert not has3
            free<i32> values;
            let shown checks_print_report checks
            checks_exit_code shown
```

非 Copy の要素を collection から読みたい場合は、単なる複製ではなく所有権の移動や借用 lifetime を考えます。入門ではまず Copy 要素の `*_ref` API で読み取りを固定します。
